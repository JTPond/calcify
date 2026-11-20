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

/// A 2D histogram is a Collection of PointBins
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct PointBin {
    pub in_edge_x: f64,
    pub ex_edge_x: f64,
    pub in_edge_y: f64,
    pub ex_edge_y: f64,
    pub count: u64,
}

impl PointBin {
    /// Returns new PointBin
    ///
    /// # Arguments
    ///
    /// * `in_edge_x` - f64 Inclusive Edge along the X axis
    /// * `ex_edge_x` - f64 Exclusive Edge along the X axis
    /// * `in_edge_y` - f64 Inclusive Edge along the Y axis
    /// * `ex_edge_y` - f64 Exclusive Edge along the Y axis
    /// * `count` - u64 PointBin value
    ///
    /// Expected to be created by calling [`Collection::hist`](crate::Collection::hist) on a `Collection<Point>`
    pub fn new(
        in_edge_x: f64,
        ex_edge_x: f64,
        in_edge_y: f64,
        ex_edge_y: f64,
        count: u64,
    ) -> PointBin {
        PointBin {
            in_edge_x,
            ex_edge_x,
            in_edge_y,
            ex_edge_y,
            count,
        }
    }
}

impl AddAssign<u64> for PointBin {
    /// Increment PointBin count.
    ///
    /// # Example
    /// ```
    /// use calcify::PointBin;
    /// let mut test_PointBin = PointBin::new(0.0,1.0,0.0,1.0,10);
    /// test_PointBin += 1;
    ///
    /// assert_eq!(test_PointBin.count, 11);
    /// ```
    fn add_assign(&mut self, other: u64) {
        self.count += other;
    }
}

impl Serialize for PointBin {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("PointBin", 2)?;
        state.serialize_field("count", &self.count)?;
        state.serialize_field(
            "range",
            &[self.in_edge_x, self.ex_edge_x, self.in_edge_y, self.ex_edge_y],
        )?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for PointBin {
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

        struct PointBinVisitor;

        impl<'de> Visitor<'de> for PointBinVisitor {
            type Value = PointBin;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("struct PointBin")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<PointBin, V::Error> {
                let mut count: Option<u64> = None;
                let mut range: Option<[f64; 4]> = None;
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
                let [in_edge_x, ex_edge_x, in_edge_y, ex_edge_y] =
                    range.ok_or_else(|| de::Error::missing_field("range"))?;
                Ok(PointBin {
                    in_edge_x,
                    ex_edge_x,
                    in_edge_y,
                    ex_edge_y,
                    count,
                })
            }
        }

        const FIELDS: &[&str] = &["count", "range"];
        deserializer.deserialize_struct("PointBin", FIELDS, PointBinVisitor)
    }
}

impl Serializable for PointBin {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::with_capacity(7);
        write_array_len(&mut buf, 2)?;
        write_uint(&mut buf, self.count)?;
        write_array_len(&mut buf, 4)?;
        write_f64(&mut buf, self.in_edge_x)?;
        write_f64(&mut buf, self.ex_edge_x)?;
        write_f64(&mut buf, self.in_edge_y)?;
        write_f64(&mut buf, self.ex_edge_y)?;
        Ok(buf)
    }
}

impl Deserializable for PointBin {
    fn from_json(s: &str) -> Result<Self, Box<dyn error::Error>> {
        Ok(serde_json::from_str(s)?)
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        if let Ok(2) = read_array_len(&mut bytes) {
            let count: u64 = read_int(&mut bytes)?;
            if let Ok(4) = read_array_len(&mut bytes) {
                let in_edge_x: f64 = read_f64(&mut bytes)?;
                let ex_edge_x: f64 = read_f64(&mut bytes)?;
                let in_edge_y: f64 = read_f64(&mut bytes)?;
                let ex_edge_y: f64 = read_f64(&mut bytes)?;
                return Ok((
                    PointBin {
                        count,
                        in_edge_x,
                        ex_edge_x,
                        in_edge_y,
                        ex_edge_y,
                    },
                    bytes,
                ));
            }
        }
        Err(Box::new(CalcifyError::ParseError(String::from(
            "invalid MessagePack PointBin",
        ))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let xx = PointBin::new(0.0, 1.0, 0.0, 1.0, 0);
        let pp = xx.to_json();
        assert_eq!(PointBin::from_json(&pp).unwrap(), xx);

        let large = PointBin::new(-1.0, 1.0, 2.0, 4.0, (1u64 << 53) + 77);
        let reordered = format!(
            r#" {{ "range" : [ -1.0 , 1.0 , 2.0 , 4.0 ] , "count" : {} }} "#,
            (1u64 << 53) + 77
        );
        assert_eq!(PointBin::from_json(&reordered).unwrap(), large);

        let rt: PointBin = serde_json::from_str(&serde_json::to_string(&large).unwrap()).unwrap();
        assert_eq!(rt, large);

        assert!(PointBin::from_json(r#"{"count":1}"#).is_err());
        assert!(PointBin::from_json(r#"{"range":[0.0,1.0,0.0,1.0]}"#).is_err());
        assert!(PointBin::from_json(r#"{"count":1,"range":[0.0,1.0]}"#).is_err());
        assert!(PointBin::from_json(r#"{"count":1,"range":[0.0,1.0,0.0,1.0],"extra":2}"#).is_err());
        assert!(
            PointBin::from_json(r#"{"count":1,"count":2,"range":[0.0,1.0,0.0,1.0]}"#).is_err()
        );
    }

    #[test]
    fn test_msg_parse() {
        let xx = PointBin::new(0.0, 1.0, 0.0, 1.0, 0);
        let pp = xx.to_msg().unwrap();
        let (oo, _) = PointBin::from_msg(&pp).unwrap();
        assert_eq!(oo, xx);
    }
}
