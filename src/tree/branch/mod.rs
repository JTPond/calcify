use std::error;

mod collection;
pub use collection::Collection;
pub use collection::Bin;
pub use collection::Point;
pub use collection::PointBin;
pub use collection::StampedSample;
use crate::four_mat::FourVec;
use crate::four_mat::FourMat;

use crate::three_mat::ThreeMat;
use crate::three_mat::ThreeVec;

use crate::utils;
use utils::{Serializable, Deserializable};
use utils::errors::CalcifyError;
use utils::serializable::{split_json_object, split_msgpack_value};

use rmp::encode::*;
use rmp::decode::*;

pub(crate) const VALUE_SUBTYPES: [&str; 12] = [
    "f64",
    "i64",
    "u64",
    "String",
    "ThreeVec",
    "ThreeMat",
    "FourVec",
    "FourMat",
    "Bin",
    "Point",
    "PointBin",
    "Object",
];

pub(crate) const STAMPED_SAMPLE_SUBTYPE: &str = "StampedSample";


/// Branch
///
/// # Note
///
/// * **Not intended for direct use. Use the memebers of Tree instead.**
pub struct Branch {
    subtype: String,
    value_subtype: Option<String>,
    branch: Box<dyn Serializable>,
    buffer: Option<Vec<u8>>,
}

impl Branch {
    pub fn new(subtype: String, branch: Box<dyn Serializable>) -> Branch {
        let buffer: Option<Vec<u8>> = None;
        Branch {
            subtype,
            value_subtype: None,
            branch,
            buffer,
        }
    }

    pub fn new_stamped(value_subtype: String, branch: Box<dyn Serializable>) -> Branch {
        Branch {
            subtype: String::from(STAMPED_SAMPLE_SUBTYPE),
            value_subtype: Some(value_subtype),
            branch,
            buffer: None,
        }
    }

    pub fn subtype(&self) -> &str {
        &self.subtype
    }

    pub fn value_subtype(&self) -> Option<&str> {
        self.value_subtype.as_deref()
    }

    /// Returns a Collection of the specified subtype from the Branch
    ///
    pub fn extract<T: Serializable + Deserializable>(&mut self) -> Result<Collection<T>, Box<dyn error::Error>> {
        if self.buffer.is_none() {
            self.buffer = Some(self.branch.to_msg()?);
        }
        if let Ok((out, rest)) = Collection::<T>::from_msg(self.buffer.as_ref().unwrap())
            && rest.is_empty()
        {
            return Ok(out);
        }
        Err(Box::new(CalcifyError::ParseError))
    }
}

fn decode_collection_json<T: 'static + Serializable + Deserializable>(
    s: &str,
) -> Result<Box<dyn Serializable>, Box<dyn error::Error>> {
    Ok(Box::new(Collection::<T>::from_json(s)?))
}

fn decode_collection_msg<T: 'static + Serializable + Deserializable>(
    bytes: &[u8],
) -> Result<Box<dyn Serializable>, Box<dyn error::Error>> {
    let (ot, rest) = Collection::<T>::from_msg(bytes)?;
    if !rest.is_empty() {
        return Err(Box::new(CalcifyError::ParseError));
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
            _ => Err(Box::new(CalcifyError::ParseError)),
        }
    };
}

fn decode_plain_json(subtype: &str, s: &str) -> Result<Box<dyn Serializable>, Box<dyn error::Error>> {
    match_builtin_subtype!(subtype, |T| decode_collection_json::<T>(s))
}

fn decode_stamped_json(
    value_subtype: &str,
    s: &str,
) -> Result<Box<dyn Serializable>, Box<dyn error::Error>> {
    match_builtin_subtype!(value_subtype, |T| decode_collection_json::<StampedSample<T>>(s))
}

fn decode_plain_msg(
    subtype: &str,
    bytes: &[u8],
) -> Result<Box<dyn Serializable>, Box<dyn error::Error>> {
    match_builtin_subtype!(subtype, |T| decode_collection_msg::<T>(bytes))
}

fn decode_stamped_msg(
    value_subtype: &str,
    bytes: &[u8],
) -> Result<Box<dyn Serializable>, Box<dyn error::Error>> {
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

        for (key, val_str) in split_json_object(s)? {
            match key.as_str() {
                "subtype" => {
                    if subtype.is_some() {
                        return Err(Box::new(CalcifyError::ParseError));
                    }
                    subtype = Some(String::from_json(val_str)?);
                }
                "value_subtype" => {
                    if value_subtype.is_some() {
                        return Err(Box::new(CalcifyError::ParseError));
                    }
                    value_subtype = Some(String::from_json(val_str)?);
                }
                "branch" => {
                    if branch_str.is_some() {
                        return Err(Box::new(CalcifyError::ParseError));
                    }
                    branch_str = Some(val_str);
                }
                _ => return Err(Box::new(CalcifyError::ParseError)),
            }
        }

        let (Some(subtype), Some(branch_str)) = (subtype, branch_str) else {
            return Err(Box::new(CalcifyError::ParseError));
        };

        if subtype == STAMPED_SAMPLE_SUBTYPE {
            let Some(vs) = value_subtype else {
                return Err(Box::new(CalcifyError::ParseError));
            };
            let branch = decode_stamped_json(&vs, branch_str)?;
            Ok(Branch::new_stamped(vs, branch))
        } else {
            if value_subtype.is_some() {
                return Err(Box::new(CalcifyError::ParseError));
            }
            let branch = decode_plain_json(&subtype, branch_str)?;
            Ok(Branch::new(subtype, branch))
        }
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        let map_len = read_map_len(&mut bytes)
            .map_err(|_| Box::new(CalcifyError::ParseError) as Box<dyn error::Error>)?;
        let mut subtype: Option<String> = None;
        let mut value_subtype: Option<String> = None;
        let mut branch_bytes: Option<&[u8]> = None;

        for _ in 0..map_len {
            let (key, rest) = read_str_from_slice(bytes)
                .map_err(|_| Box::new(CalcifyError::ParseError) as Box<dyn error::Error>)?;
            match key {
                "subtype" => {
                    if subtype.is_some() {
                        return Err(Box::new(CalcifyError::ParseError));
                    }
                    let (val, next) = String::from_msg(rest)?;
                    subtype = Some(val);
                    bytes = next;
                }
                "value_subtype" => {
                    if value_subtype.is_some() {
                        return Err(Box::new(CalcifyError::ParseError));
                    }
                    let (val, next) = String::from_msg(rest)?;
                    value_subtype = Some(val);
                    bytes = next;
                }
                "branch" => {
                    if branch_bytes.is_some() {
                        return Err(Box::new(CalcifyError::ParseError));
                    }
                    let (val_slice, next) = split_msgpack_value(rest)?;
                    branch_bytes = Some(val_slice);
                    bytes = next;
                }
                _ => return Err(Box::new(CalcifyError::ParseError)),
            }
        }

        let (Some(subtype), Some(branch_bytes)) = (subtype, branch_bytes) else {
            return Err(Box::new(CalcifyError::ParseError));
        };

        if subtype == STAMPED_SAMPLE_SUBTYPE {
            let Some(vs) = value_subtype else {
                return Err(Box::new(CalcifyError::ParseError));
            };
            let branch = decode_stamped_msg(&vs, branch_bytes)?;
            Ok((Branch::new_stamped(vs, branch), bytes))
        } else {
            if value_subtype.is_some() {
                return Err(Box::new(CalcifyError::ParseError));
            }
            let branch = decode_plain_msg(&subtype, branch_bytes)?;
            Ok((Branch::new(subtype, branch), bytes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_branch_roundtrip_and_format() {
        let col = Collection::from(vec![1.0, 2.5]);
        let br = Branch::new(String::from("f64"), Box::new(col.clone()));
        assert_eq!(br.subtype(), "f64");
        assert_eq!(br.value_subtype(), None);

        let json = br.to_json();
        assert_eq!(json, r#"{"subtype":"f64","branch":[1,2.5]}"#);
        let mut dec_json = Branch::from_json(&json).unwrap();
        assert_eq!(dec_json.subtype(), "f64");
        assert_eq!(dec_json.value_subtype(), None);
        assert_eq!(dec_json.extract::<f64>().unwrap(), col);

        let msg = br.to_msg().unwrap();
        let (mut dec_msg, rest) = Branch::from_msg(&msg).unwrap();
        assert!(rest.is_empty());
        assert_eq!(dec_msg.subtype(), "f64");
        assert_eq!(dec_msg.value_subtype(), None);
        assert_eq!(dec_msg.extract::<f64>().unwrap(), col);
    }

    #[test]
    fn test_stamped_branch_roundtrip_and_key_order() {
        let stream = Collection::stream(&[-10, 0, 25], &[ThreeVec::new(1.0, 2.0, 3.0), ThreeVec::new(4.0, 5.0, 6.0), ThreeVec::new(7.0, 8.0, 9.0)]);
        let br = Branch::new_stamped(String::from("ThreeVec"), Box::new(stream.clone()));
        assert_eq!(br.subtype(), "StampedSample");
        assert_eq!(br.value_subtype(), Some("ThreeVec"));

        let json = br.to_json();
        let mut dec_json = Branch::from_json(&json).unwrap();
        assert_eq!(dec_json.subtype(), "StampedSample");
        assert_eq!(dec_json.value_subtype(), Some("ThreeVec"));
        assert_eq!(dec_json.extract::<StampedSample<ThreeVec>>().unwrap(), stream);

        let msg = br.to_msg().unwrap();
        let (mut dec_msg, rest) = Branch::from_msg(&msg).unwrap();
        assert!(rest.is_empty());
        assert_eq!(dec_msg.subtype(), "StampedSample");
        assert_eq!(dec_msg.value_subtype(), Some("ThreeVec"));
        assert_eq!(dec_msg.extract::<StampedSample<ThreeVec>>().unwrap(), stream);

        // Reordered keys in JSON
        let reordered_json = format!(
            r#"{{"branch":{},"value_subtype":"ThreeVec","subtype":"StampedSample"}}"#,
            stream.to_json()
        );
        let mut dec_reordered = Branch::from_json(&reordered_json).unwrap();
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

        let (mut dec_reordered_msg, rest2) = Branch::from_msg(&reordered_msg).unwrap();
        assert!(rest2.is_empty());
        assert_eq!(dec_reordered_msg.subtype(), "StampedSample");
        assert_eq!(dec_reordered_msg.value_subtype(), Some("ThreeVec"));
        assert_eq!(
            dec_reordered_msg.extract::<StampedSample<ThreeVec>>().unwrap(),
            stream
        );
    }

    #[test]
    fn test_object_branch_error_and_malformed_metadata() {
        let plain_obj = Branch::new(String::from("Object"), Box::new(Collection::<f64>::empty()));
        let err_json = Branch::from_json(&plain_obj.to_json()).err().unwrap();
        assert_eq!(
            err_json.downcast_ref::<CalcifyError>().map(|e| format!("{}", e)),
            Some(format!("{}", CalcifyError::ObjectBranchDeserializeError))
        );
        let err_msg = Branch::from_msg(&plain_obj.to_msg().unwrap()).err().unwrap();
        assert_eq!(
            err_msg.downcast_ref::<CalcifyError>().map(|e| format!("{}", e)),
            Some(format!("{}", CalcifyError::ObjectBranchDeserializeError))
        );

        let stamped_obj = Branch::new_stamped(
            String::from("Object"),
            Box::new(Collection::<StampedSample<f64>>::empty()),
        );
        let serr_json = Branch::from_json(&stamped_obj.to_json()).err().unwrap();
        assert_eq!(
            serr_json.downcast_ref::<CalcifyError>().map(|e| format!("{}", e)),
            Some(format!("{}", CalcifyError::ObjectBranchDeserializeError))
        );
        let serr_msg = Branch::from_msg(&stamped_obj.to_msg().unwrap()).err().unwrap();
        assert_eq!(
            serr_msg.downcast_ref::<CalcifyError>().map(|e| format!("{}", e)),
            Some(format!("{}", CalcifyError::ObjectBranchDeserializeError))
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
