use std::any::Any;
use std::error;

mod collection;
pub use collection::Bin;
pub use collection::Collection;
pub use collection::Point;
pub use collection::PointBin;
pub use collection::StampedSample;
use crate::four_mat::FourMat;
use crate::four_mat::FourVec;

use crate::three_mat::ThreeMat;
use crate::three_mat::ThreeVec;

use crate::utils;
use utils::errors::CalcifyError;
use utils::serializable::{json_object_entries, split_msgpack_value};
use utils::{Deserializable, Serializable};

use rmp::decode::*;
use rmp::encode::*;

pub(crate) const OBJECT_SUBTYPE: &str = "Object";
pub(crate) const STAMPED_SAMPLE_SUBTYPE: &str = "StampedSample";

mod private {
    pub trait Sealed {}
}

/// Sealed marker trait mapping built-in Calcify value types to their branch subtype tag.
///
/// Custom user types do not implement `CalcifyType`; store them via
/// [`Tree::add_object_branch`](crate::Tree::add_object_branch) or
/// [`FeedTree::new_object`](crate::FeedTree::new_object).
pub trait CalcifyType: private::Sealed + 'static + Serializable {
    const SUBTYPE: &'static str;
}

macro_rules! impl_calcify_type {
    ($($ty:ty => $name:expr),* $(,)?) => {
        $(
            impl private::Sealed for $ty {}
            impl CalcifyType for $ty {
                const SUBTYPE: &'static str = $name;
            }
        )*
    };
}

impl_calcify_type!(
    f64 => "f64",
    i64 => "i64",
    u64 => "u64",
    String => "String",
    ThreeVec => "ThreeVec",
    ThreeMat => "ThreeMat",
    FourVec => "FourVec",
    FourMat => "FourMat",
    Bin => "Bin",
    Point => "Point",
    PointBin => "PointBin",
);

pub(crate) trait BranchPayload: Serializable + Any {
    fn as_any(&self) -> &dyn Any;
    fn len(&self) -> usize;
}

impl<T: 'static + Serializable> BranchPayload for Collection<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn len(&self) -> usize {
        self.vec.len()
    }
}

/// Branch
///
/// # Note
///
/// * **Not intended for direct construction in most cases; use the members of `Tree` instead.**
pub struct Branch {
    subtype: String,
    value_subtype: Option<String>,
    branch: Box<dyn BranchPayload>,
}

impl Branch {
    /// Creates a new `Branch` holding a built-in `Collection<T>`.
    pub fn new<T: CalcifyType>(branch: Collection<T>) -> Branch {
        Branch {
            subtype: String::from(T::SUBTYPE),
            value_subtype: None,
            branch: Box::new(branch),
        }
    }

    /// Creates a new stamped `Branch` holding a `Collection<StampedSample<T>>`.
    pub fn new_stamped<T: CalcifyType>(branch: Collection<StampedSample<T>>) -> Branch {
        Branch {
            subtype: String::from(STAMPED_SAMPLE_SUBTYPE),
            value_subtype: Some(String::from(T::SUBTYPE)),
            branch: Box::new(branch),
        }
    }

    /// Creates a new `"Object"` `Branch` holding a `Collection<T>` for custom types.
    pub fn new_object<T: 'static + Serializable>(branch: Collection<T>) -> Branch {
        Branch {
            subtype: String::from(OBJECT_SUBTYPE),
            value_subtype: None,
            branch: Box::new(branch),
        }
    }

    /// Creates a new stamped `"Object"` `Branch` holding a `Collection<StampedSample<T>>`.
    pub fn new_stamped_object<T: 'static + Serializable>(
        branch: Collection<StampedSample<T>>,
    ) -> Branch {
        Branch {
            subtype: String::from(STAMPED_SAMPLE_SUBTYPE),
            value_subtype: Some(String::from(OBJECT_SUBTYPE)),
            branch: Box::new(branch),
        }
    }

    pub(crate) fn from_payload(
        subtype: String,
        value_subtype: Option<String>,
        branch: Box<dyn BranchPayload>,
    ) -> Branch {
        Branch {
            subtype,
            value_subtype,
            branch,
        }
    }

    /// Returns the branch subtype tag (e.g., `"f64"`, `"Bin"`, `"StampedSample"`, or `"Object"`).
    pub fn subtype(&self) -> &str {
        &self.subtype
    }

    /// Returns the inner value subtype tag for `"StampedSample"` branches, or `None` otherwise.
    pub fn value_subtype(&self) -> Option<&str> {
        self.value_subtype.as_deref()
    }

    /// Returns the number of elements in the branch collection.
    pub fn len(&self) -> usize {
        self.branch.len()
    }

    /// Returns `true` if the branch collection has no elements.
    pub fn is_empty(&self) -> bool {
        self.branch.len() == 0
    }

    pub(crate) fn type_label(&self) -> String {
        match &self.value_subtype {
            Some(vs) => format!("{}<{}>", self.subtype, vs),
            None => self.subtype.clone(),
        }
    }

    /// Borrows the underlying `Collection<T>` directly without copying or re-deserializing.
    pub fn as_collection<T: 'static + Serializable>(&self) -> Option<&Collection<T>> {
        self.branch.as_any().downcast_ref::<Collection<T>>()
    }

    /// Clones and returns the `Collection<T>` stored in this `Branch`.
    pub fn extract<T: 'static + Serializable + Clone>(&self) -> Result<Collection<T>, CalcifyError> {
        self.as_collection::<T>()
            .cloned()
            .ok_or_else(|| CalcifyError::SubtypeMismatch {
                expected: String::from(std::any::type_name::<T>()),
                found: self.type_label(),
            })
    }
}

fn decode_collection_json<T: 'static + Serializable + Deserializable>(
    s: &str,
) -> Result<Box<dyn BranchPayload>, Box<dyn error::Error>> {
    Ok(Box::new(Collection::<T>::from_json(s)?))
}

fn decode_collection_msg<T: 'static + Serializable + Deserializable>(
    bytes: &[u8],
) -> Result<Box<dyn BranchPayload>, Box<dyn error::Error>> {
    let (ot, rest) = Collection::<T>::from_msg(bytes)?;
    if !rest.is_empty() {
        return Err(Box::new(CalcifyError::ParseError(String::from(
            "trailing bytes in MessagePack branch collection",
        ))));
    }
    Ok(Box::new(ot))
}

macro_rules! match_builtin_subtype {
    ($subtype:expr, |$T:ident| $body:expr) => {
        match $subtype {
            "f64" => {
                type $T = f64;
                $body
            }
            "i64" => {
                type $T = i64;
                $body
            }
            "u64" => {
                type $T = u64;
                $body
            }
            "String" => {
                type $T = String;
                $body
            }
            "ThreeVec" => {
                type $T = ThreeVec;
                $body
            }
            "ThreeMat" => {
                type $T = ThreeMat;
                $body
            }
            "FourVec" => {
                type $T = FourVec;
                $body
            }
            "FourMat" => {
                type $T = FourMat;
                $body
            }
            "Bin" => {
                type $T = Bin;
                $body
            }
            "Point" => {
                type $T = Point;
                $body
            }
            "PointBin" => {
                type $T = PointBin;
                $body
            }
            "Object" => Err(Box::new(CalcifyError::ObjectBranchDeserializeError)),
            other => Err(Box::new(CalcifyError::InvalidSubtype(String::from(other)))),
        }
    };
}

fn decode_plain_json(
    subtype: &str,
    s: &str,
) -> Result<Box<dyn BranchPayload>, Box<dyn error::Error>> {
    match_builtin_subtype!(subtype, |T| decode_collection_json::<T>(s))
}

fn decode_stamped_json(
    value_subtype: &str,
    s: &str,
) -> Result<Box<dyn BranchPayload>, Box<dyn error::Error>> {
    match_builtin_subtype!(value_subtype, |T| decode_collection_json::<StampedSample<T>>(s))
}

fn decode_plain_msg(
    subtype: &str,
    bytes: &[u8],
) -> Result<Box<dyn BranchPayload>, Box<dyn error::Error>> {
    match_builtin_subtype!(subtype, |T| decode_collection_msg::<T>(bytes))
}

fn decode_stamped_msg(
    value_subtype: &str,
    bytes: &[u8],
) -> Result<Box<dyn BranchPayload>, Box<dyn error::Error>> {
    match_builtin_subtype!(value_subtype, |T| decode_collection_msg::<StampedSample<T>>(bytes))
}

impl Serializable for Branch {
    fn to_json(&self) -> String {
        match &self.value_subtype {
            Some(vs) => format!(
                "{{\"subtype\":{},\"value_subtype\":{},\"branch\":{}}}",
                self.subtype.to_json(),
                vs.to_json(),
                self.branch.to_json()
            ),
            None => format!(
                "{{\"subtype\":{},\"branch\":{}}}",
                self.subtype.to_json(),
                self.branch.to_json()
            ),
        }
    }

    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::new();
        let map_len = if self.value_subtype.is_some() { 3 } else { 2 };
        write_map_len(&mut buf, map_len)?;
        write_str(&mut buf, "subtype")?;
        buf.append(&mut self.subtype.to_msg()?);
        if let Some(vs) = &self.value_subtype {
            write_str(&mut buf, "value_subtype")?;
            buf.append(&mut vs.to_msg()?);
        }
        write_str(&mut buf, "branch")?;
        buf.append(&mut self.branch.to_msg()?);
        Ok(buf)
    }
}

impl Deserializable for Branch {
    fn from_json(s: &str) -> Result<Self, Box<dyn error::Error>> {
        let mut subtype: Option<String> = None;
        let mut value_subtype: Option<String> = None;
        let mut branch_str: Option<&str> = None;

        for (key, val_str) in json_object_entries(s)? {
            match key.as_str() {
                "subtype" => subtype = Some(String::from_json(val_str)?),
                "value_subtype" => value_subtype = Some(String::from_json(val_str)?),
                "branch" => branch_str = Some(val_str),
                _ => {
                    return Err(Box::new(CalcifyError::ParseError(format!(
                        "unexpected Branch field '{}'",
                        key
                    ))));
                }
            }
        }

        let (Some(subtype), Some(branch_str)) = (subtype, branch_str) else {
            return Err(Box::new(CalcifyError::ParseError(String::from(
                "missing Branch subtype or payload",
            ))));
        };

        if subtype == STAMPED_SAMPLE_SUBTYPE {
            let Some(vs) = value_subtype else {
                return Err(Box::new(CalcifyError::ParseError(String::from(
                    "StampedSample branch missing value_subtype",
                ))));
            };
            let branch = decode_stamped_json(&vs, branch_str)?;
            Ok(Branch::from_payload(subtype, Some(vs), branch))
        } else {
            if value_subtype.is_some() {
                return Err(Box::new(CalcifyError::ParseError(String::from(
                    "unexpected value_subtype on non-StampedSample branch",
                ))));
            }
            let branch = decode_plain_json(&subtype, branch_str)?;
            Ok(Branch::from_payload(subtype, None, branch))
        }
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        let map_len = read_map_len(&mut bytes).map_err(|e| {
            Box::new(CalcifyError::ParseError(format!(
                "invalid MessagePack Branch map: {}",
                e
            ))) as Box<dyn error::Error>
        })?;
        let mut subtype: Option<String> = None;
        let mut value_subtype: Option<String> = None;
        let mut branch_bytes: Option<&[u8]> = None;

        for _ in 0..map_len {
            let (key, rest) = read_str_from_slice(bytes).map_err(|e| {
                Box::new(CalcifyError::ParseError(format!(
                    "invalid MessagePack Branch key: {}",
                    e
                ))) as Box<dyn error::Error>
            })?;
            match key {
                "subtype" => {
                    if subtype.is_some() {
                        return Err(Box::new(CalcifyError::ParseError(String::from(
                            "duplicate Branch subtype",
                        ))));
                    }
                    let (val, next) = String::from_msg(rest)?;
                    subtype = Some(val);
                    bytes = next;
                }
                "value_subtype" => {
                    if value_subtype.is_some() {
                        return Err(Box::new(CalcifyError::ParseError(String::from(
                            "duplicate Branch value_subtype",
                        ))));
                    }
                    let (val, next) = String::from_msg(rest)?;
                    value_subtype = Some(val);
                    bytes = next;
                }
                "branch" => {
                    if branch_bytes.is_some() {
                        return Err(Box::new(CalcifyError::ParseError(String::from(
                            "duplicate Branch payload",
                        ))));
                    }
                    let (val_slice, next) = split_msgpack_value(rest)?;
                    branch_bytes = Some(val_slice);
                    bytes = next;
                }
                other => {
                    return Err(Box::new(CalcifyError::ParseError(format!(
                        "unexpected MessagePack Branch key '{}'",
                        other
                    ))));
                }
            }
        }

        let (Some(subtype), Some(branch_bytes)) = (subtype, branch_bytes) else {
            return Err(Box::new(CalcifyError::ParseError(String::from(
                "missing Branch subtype or payload",
            ))));
        };

        if subtype == STAMPED_SAMPLE_SUBTYPE {
            let Some(vs) = value_subtype else {
                return Err(Box::new(CalcifyError::ParseError(String::from(
                    "StampedSample branch missing value_subtype",
                ))));
            };
            let branch = decode_stamped_msg(&vs, branch_bytes)?;
            Ok((Branch::from_payload(subtype, Some(vs), branch), bytes))
        } else {
            if value_subtype.is_some() {
                return Err(Box::new(CalcifyError::ParseError(String::from(
                    "unexpected value_subtype on non-StampedSample branch",
                ))));
            }
            let branch = decode_plain_msg(&subtype, branch_bytes)?;
            Ok((Branch::from_payload(subtype, None, branch), bytes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_branch_roundtrip_and_format() {
        let col = Collection::from(vec![1.0, 2.5]);
        let br = Branch::new(col.clone());
        assert_eq!(br.subtype(), "f64");
        assert_eq!(br.value_subtype(), None);
        assert_eq!(br.len(), 2);
        assert!(!br.is_empty());
        assert_eq!(br.as_collection::<f64>(), Some(&col));
        assert_eq!(br.extract::<f64>().unwrap(), col);
        assert!(matches!(
            br.extract::<i64>(),
            Err(CalcifyError::SubtypeMismatch { .. })
        ));

        let json = br.to_json();
        assert_eq!(json, r#"{"subtype":"f64","branch":[1.0,2.5]}"#);
        let dec_json = Branch::from_json(&json).unwrap();
        assert_eq!(dec_json.subtype(), "f64");
        assert_eq!(dec_json.value_subtype(), None);
        assert_eq!(dec_json.extract::<f64>().unwrap(), col);

        let msg = br.to_msg().unwrap();
        let (dec_msg, rest) = Branch::from_msg(&msg).unwrap();
        assert!(rest.is_empty());
        assert_eq!(dec_msg.subtype(), "f64");
        assert_eq!(dec_msg.value_subtype(), None);
        assert_eq!(dec_msg.extract::<f64>().unwrap(), col);
    }

    #[test]
    fn test_stamped_branch_roundtrip_and_key_order() {
        let stream = Collection::stream(
            &[-10, 0, 25],
            &[
                ThreeVec::new(1.0, 2.0, 3.0),
                ThreeVec::new(4.0, 5.0, 6.0),
                ThreeVec::new(7.0, 8.0, 9.0),
            ],
        );
        let br = Branch::new_stamped(stream.clone());
        assert_eq!(br.subtype(), "StampedSample");
        assert_eq!(br.value_subtype(), Some("ThreeVec"));
        assert_eq!(br.len(), 3);

        let json = br.to_json();
        let dec_json = Branch::from_json(&json).unwrap();
        assert_eq!(dec_json.subtype(), "StampedSample");
        assert_eq!(dec_json.value_subtype(), Some("ThreeVec"));
        assert_eq!(
            dec_json.extract::<StampedSample<ThreeVec>>().unwrap(),
            stream
        );

        let msg = br.to_msg().unwrap();
        let (dec_msg, rest) = Branch::from_msg(&msg).unwrap();
        assert!(rest.is_empty());
        assert_eq!(dec_msg.subtype(), "StampedSample");
        assert_eq!(dec_msg.value_subtype(), Some("ThreeVec"));
        assert_eq!(
            dec_msg.extract::<StampedSample<ThreeVec>>().unwrap(),
            stream
        );

        // Reordered keys in JSON
        let reordered_json = format!(
            r#"{{"branch":{},"value_subtype":"ThreeVec","subtype":"StampedSample"}}"#,
            stream.to_json()
        );
        let dec_reordered = Branch::from_json(&reordered_json).unwrap();
        assert_eq!(
            dec_reordered.extract::<StampedSample<ThreeVec>>().unwrap(),
            stream
        );

        // Reordered keys in MessagePack: branch -> value_subtype -> subtype
        let mut reordered_msg = Vec::new();
        write_map_len(&mut reordered_msg, 3).unwrap();
        write_str(&mut reordered_msg, "branch").unwrap();
        reordered_msg.append(&mut stream.to_msg().unwrap());
        write_str(&mut reordered_msg, "value_subtype").unwrap();
        write_str(&mut reordered_msg, "ThreeVec").unwrap();
        write_str(&mut reordered_msg, "subtype").unwrap();
        write_str(&mut reordered_msg, "StampedSample").unwrap();

        let (dec_reordered_msg, rest2) = Branch::from_msg(&reordered_msg).unwrap();
        assert!(rest2.is_empty());
        assert_eq!(dec_reordered_msg.subtype(), "StampedSample");
        assert_eq!(dec_reordered_msg.value_subtype(), Some("ThreeVec"));
        assert_eq!(
            dec_reordered_msg
                .extract::<StampedSample<ThreeVec>>()
                .unwrap(),
            stream
        );
    }

    #[test]
    fn test_object_branch_error_and_malformed_metadata() {
        let plain_obj = Branch::new_object(Collection::<f64>::empty());
        assert_eq!(plain_obj.subtype(), "Object");
        assert_eq!(plain_obj.value_subtype(), None);
        assert!(plain_obj.is_empty());
        assert_eq!(plain_obj.extract::<f64>().unwrap(), Collection::empty());

        let err_json = Branch::from_json(&plain_obj.to_json()).err().unwrap();
        assert_eq!(
            err_json.downcast_ref::<CalcifyError>(),
            Some(&CalcifyError::ObjectBranchDeserializeError)
        );
        let err_msg = Branch::from_msg(&plain_obj.to_msg().unwrap()).err().unwrap();
        assert_eq!(
            err_msg.downcast_ref::<CalcifyError>(),
            Some(&CalcifyError::ObjectBranchDeserializeError)
        );

        let stamped_obj = Branch::new_stamped_object(Collection::<StampedSample<f64>>::empty());
        assert_eq!(stamped_obj.subtype(), "StampedSample");
        assert_eq!(stamped_obj.value_subtype(), Some("Object"));
        let serr_json = Branch::from_json(&stamped_obj.to_json()).err().unwrap();
        assert_eq!(
            serr_json.downcast_ref::<CalcifyError>(),
            Some(&CalcifyError::ObjectBranchDeserializeError)
        );
        let serr_msg = Branch::from_msg(&stamped_obj.to_msg().unwrap())
            .err()
            .unwrap();
        assert_eq!(
            serr_msg.downcast_ref::<CalcifyError>(),
            Some(&CalcifyError::ObjectBranchDeserializeError)
        );

        // Unknown subtype
        let bad_sub = Branch::from_json(r#"{"subtype":"Bogus","branch":[]}"#)
            .err()
            .unwrap();
        assert_eq!(
            bad_sub.downcast_ref::<CalcifyError>(),
            Some(&CalcifyError::InvalidSubtype(String::from("Bogus")))
        );

        // Missing value_subtype for StampedSample
        assert!(Branch::from_json(r#"{"subtype":"StampedSample","branch":[]}"#).is_err());
        // Unexpected value_subtype for non-StampedSample
        assert!(
            Branch::from_json(r#"{"subtype":"f64","value_subtype":"f64","branch":[]}"#).is_err()
        );
        // Unknown key
        assert!(Branch::from_json(r#"{"subtype":"f64","extra":"x","branch":[]}"#).is_err());

        // MsgPack malformed combinations
        let mut missing_vs = Vec::new();
        write_map_len(&mut missing_vs, 2).unwrap();
        write_str(&mut missing_vs, "subtype").unwrap();
        write_str(&mut missing_vs, "StampedSample").unwrap();
        write_str(&mut missing_vs, "branch").unwrap();
        write_array_len(&mut missing_vs, 0).unwrap();
        assert!(Branch::from_msg(&missing_vs).is_err());

        let mut extra_vs = Vec::new();
        write_map_len(&mut extra_vs, 3).unwrap();
        write_str(&mut extra_vs, "subtype").unwrap();
        write_str(&mut extra_vs, "f64").unwrap();
        write_str(&mut extra_vs, "value_subtype").unwrap();
        write_str(&mut extra_vs, "f64").unwrap();
        write_str(&mut extra_vs, "branch").unwrap();
        write_array_len(&mut extra_vs, 0).unwrap();
        assert!(Branch::from_msg(&extra_vs).is_err());
    }
}
