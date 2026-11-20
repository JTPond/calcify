use std::error;
use std::fmt;
use std::ops::AddAssign;

use rmp::decode::*;
use rmp::encode::*;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

use crate::utils;

use utils::errors::CalcifyError;
use utils::{Deserializable, Serializable};

/// A histogram is a Collection of Bins
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Bin {
    pub in_edge: f64,
    pub ex_edge: f64,
    pub count: u64,
}

impl Bin {
    /// Returns new Bin
    ///
    /// # Arguments
    ///
    /// * `in_edge` - f64 Inclusive Edge
    /// * `ex_edge` - f64 Exclusive Edge
    /// * `count` - u64 Bin value
    ///
    pub fn new(in_edge: f64, ex_edge: f64, count: u64) -> Bin {
        Bin {
            in_edge,
            ex_edge,
            count,
        }
    }
}

impl AddAssign<u64> for Bin {
    /// Increment Bin count.
    ///
    /// # Example
    /// ```
    /// use calcify::Bin;
    /// let mut test_bin = Bin::new(0.0,1.0,10);
    /// test_bin += 1;
    ///
    /// assert_eq!(test_bin.count, 11);
    /// ```
    fn add_assign(&mut self, other: u64) {
        self.count += other;
    }
}

impl Serialize for Bin {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Bin", 2)?;
        state.serialize_field("count", &self.count)?;
        state.serialize_field("range", &[self.in_edge, self.ex_edge])?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Bin {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        enum Field {
            Count,
            Range,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Field, D::Error> {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        f.write_str("`count` or `range`")
                    }

                    fn visit_str<E: de::Error>(self, value: &str) -> Result<Field, E> {
                        match value {
                            "count" => Ok(Field::Count),
                            "range" => Ok(Field::Range),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct BinVisitor;

        impl<'de> Visitor<'de> for BinVisitor {
            type Value = Bin;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("struct Bin")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Bin, V::Error> {
                let mut count: Option<u64> = None;
                let mut range: Option<[f64; 2]> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Count => {
                            if count.is_some() {
                                return Err(de::Error::duplicate_field("count"));
                            }
                            count = Some(map.next_value()?);
                        }
                        Field::Range => {
                            if range.is_some() {
                                return Err(de::Error::duplicate_field("range"));
                            }
                            range = Some(map.next_value()?);
                        }
                    }
                }
                let count = count.ok_or_else(|| de::Error::missing_field("count"))?;
                let [in_edge, ex_edge] =
                    range.ok_or_else(|| de::Error::missing_field("range"))?;
                Ok(Bin {
                    in_edge,
                    ex_edge,
                    count,
                })
            }
        }

        const FIELDS: &[&str] = &["count", "range"];
        deserializer.deserialize_struct("Bin", FIELDS, BinVisitor)
    }
}

impl Serializable for Bin {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::with_capacity(5);
        write_array_len(&mut buf, 2)?;
        write_uint(&mut buf, self.count)?;
        write_array_len(&mut buf, 2)?;
        write_f64(&mut buf, self.in_edge)?;
        write_f64(&mut buf, self.ex_edge)?;
        Ok(buf)
    }
}

impl Deserializable for Bin {
    fn from_json(s: &str) -> Result<Self, Box<dyn error::Error>> {
        Ok(serde_json::from_str(s)?)
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        if let Ok(2) = read_array_len(&mut bytes) {
            let count: u64 = read_int(&mut bytes)?;
            if let Ok(2) = read_array_len(&mut bytes) {
                let in_edge: f64 = read_f64(&mut bytes)?;
                let ex_edge: f64 = read_f64(&mut bytes)?;
                return Ok((
                    Bin {
                        count,
                        in_edge,
                        ex_edge,
                    },
                    bytes,
                ));
            }
        }
        Err(Box::new(CalcifyError::ParseError(String::from(
            "invalid MessagePack Bin",
        ))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let xx = Bin::new(0.0, 1.0, 0);
        let pp = xx.to_json();
        assert_eq!(Bin::from_json(&pp).unwrap(), xx);

        // Reordered keys, whitespace, and exact u64 precision above 2^53
        let large = Bin::new(-2.5, 3.75, (1u64 << 53) + 123);
        let reordered = format!(
            r#" {{ "range" : [ -2.5 , 3.75 ] , "count" : {} }} "#,
            (1u64 << 53) + 123
        );
        assert_eq!(Bin::from_json(&reordered).unwrap(), large);

        let rt: Bin = serde_json::from_str(&serde_json::to_string(&large).unwrap()).unwrap();
        assert_eq!(rt, large);

        assert!(Bin::from_json(r#"{"count":1}"#).is_err());
        assert!(Bin::from_json(r#"{"range":[0.0,1.0]}"#).is_err());
        assert!(Bin::from_json(r#"{"count":1,"range":[0.0,1.0],"extra":2}"#).is_err());
        assert!(Bin::from_json(r#"{"count":1,"count":2,"range":[0.0,1.0]}"#).is_err());
    }

    #[test]
    fn test_msg_parse() {
        let xx = Bin::new(0.0, 1.0, 0);
        let pp = xx.to_msg().unwrap();
        let (oo, _) = Bin::from_msg(&pp).unwrap();
        assert_eq!(oo, xx);
    }
}
