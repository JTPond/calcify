use std::convert::From;
use std::error;
use std::fmt;

use rmp::decode::*;
use rmp::encode::*;

use crate::utils;
use utils::errors::CalcifyError;
use utils::serializable::split_json_object;
use utils::{Deserializable, Serializable};

/// A timestamped data sample pairing an `i64` timestamp with any `Serializable` value.
///
/// A timestamped stream is represented as a `Collection<StampedSample<T>>`.
///
/// # Example
/// ```
/// use calcify::{StampedSample, ThreeVec};
///
/// let sample = StampedSample::new(1_700_000_000, ThreeVec::new(1.0, 2.0, 3.0));
/// assert_eq!(sample.timestamp(), 1_700_000_000);
/// assert_eq!(*sample.value(), ThreeVec::new(1.0, 2.0, 3.0));
/// ```
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct StampedSample<T: Serializable> {
    pub timestamp: i64,
    pub value: T,
}

impl<T: Serializable> StampedSample<T> {
    /// Returns a new `StampedSample<T>` with the given `timestamp` and `value`.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - `i64` timestamp
    /// * `value` - `T: Serializable` payload
    ///
    /// # Example
    /// ```
    /// use calcify::StampedSample;
    ///
    /// let sample = StampedSample::new(42, 3.5);
    /// assert_eq!(sample.timestamp, 42);
    /// assert_eq!(sample.value, 3.5);
    /// ```
    pub fn new(timestamp: i64, value: T) -> StampedSample<T> {
        StampedSample { timestamp, value }
    }

    /// Returns the sample's timestamp.
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Returns an immutable reference to the sample's value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns a mutable reference to the sample's value.
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Consumes the sample and returns `(timestamp, value)`.
    pub fn into_parts(self) -> (i64, T) {
        (self.timestamp, self.value)
    }

    /// Transforms the sample's value while preserving its timestamp.
    ///
    /// # Example
    /// ```
    /// use calcify::{StampedSample, ThreeVec};
    ///
    /// let sample = StampedSample::new(100, ThreeVec::new(3.0, 4.0, 0.0));
    /// let mag = sample.map_value(ThreeVec::r);
    /// assert_eq!(mag, StampedSample::new(100, 5.0));
    /// ```
    pub fn map_value<F, Z: Serializable>(&self, f: F) -> StampedSample<Z>
    where
        F: FnOnce(&T) -> Z,
    {
        StampedSample {
            timestamp: self.timestamp,
            value: f(&self.value),
        }
    }
}

impl<T: Serializable> From<(i64, T)> for StampedSample<T> {
    fn from((timestamp, value): (i64, T)) -> Self {
        StampedSample { timestamp, value }
    }
}

impl<T: Serializable + fmt::Display> fmt::Display for StampedSample<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.timestamp, self.value)
    }
}

impl<T: Serializable> Serializable for StampedSample<T> {
    fn to_json(&self) -> String {
        format!(
            "{{\"timestamp\":{},\"value\":{}}}",
            self.timestamp,
            self.value.to_json()
        )
    }

    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::new();
        write_array_len(&mut buf, 2)?;
        write_sint(&mut buf, self.timestamp)?;
        buf.append(&mut self.value.to_msg()?);
        Ok(buf)
    }
}

impl<T: Serializable + Deserializable> Deserializable for StampedSample<T> {
    fn from_json(s: &str) -> Result<Self, Box<dyn error::Error>> {
        let mut timestamp: Option<i64> = None;
        let mut value: Option<T> = None;
        for (key, val_str) in split_json_object(s)? {
            match key.as_str() {
                "timestamp" => {
                    if timestamp.is_some() {
                        return Err(Box::new(CalcifyError::ParseError));
                    }
                    timestamp = Some(i64::from_json(val_str)?);
                }
                "value" => {
                    if value.is_some() {
                        return Err(Box::new(CalcifyError::ParseError));
                    }
                    value = Some(T::from_json(val_str)?);
                }
                _ => return Err(Box::new(CalcifyError::ParseError)),
            }
        }
        match (timestamp, value) {
            (Some(timestamp), Some(value)) => Ok(StampedSample { timestamp, value }),
            _ => Err(Box::new(CalcifyError::ParseError)),
        }
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        if let Ok(2) = read_array_len(&mut bytes) {
            let timestamp: i64 = read_int(&mut bytes)?;
            let (value, rest) = T::from_msg(bytes)?;
            Ok((StampedSample { timestamp, value }, rest))
        } else {
            Err(Box::new(CalcifyError::ParseError))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bin, FourMat, FourVec, Point, PointBin, ThreeMat, ThreeVec};

    fn assert_roundtrip<T: Serializable + Deserializable + PartialEq + fmt::Debug>(
        sample: StampedSample<T>,
    ) {
        let json = sample.to_json();
        assert_eq!(StampedSample::<T>::from_json(&json).unwrap(), sample);

        let msg = sample.to_msg().unwrap();
        let (decoded, rest) = StampedSample::<T>::from_msg(&msg).unwrap();
        assert_eq!(decoded, sample);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_roundtrip_all_builtin_types() {
        assert_roundtrip(StampedSample::new(-42, 3.25f64));
        for ts in [0i64, -1, -32, -33, 127, 128, i64::MIN, i64::MAX] {
            assert_roundtrip(StampedSample::new(ts, ts.wrapping_mul(3)));
        }
        assert_roundtrip(StampedSample::new(1_700_000_000_000, 99u64));
        assert_roundtrip(StampedSample::new(
            10,
            String::from("hello, \"world\" {a:1} [2]\n\\end"),
        ));
        assert_roundtrip(StampedSample::new(
            101,
            ThreeVec::new(1.0, -2.5, 3.75),
        ));
        assert_roundtrip(StampedSample::new(
            102,
            ThreeMat::new(
                ThreeVec::new(1.0, 2.0, 3.0),
                ThreeVec::new(4.0, 5.0, 6.0),
                ThreeVec::new(7.0, 8.0, 9.0),
            ),
        ));
        assert_roundtrip(StampedSample::new(
            103,
            FourVec::new(10.0, 1.0, -2.0, 3.0),
        ));
        assert_roundtrip(StampedSample::new(
            104,
            FourMat::new(
                FourVec::new(1.0, 0.0, 0.0, 0.0),
                FourVec::new(0.0, 2.0, 0.0, 0.0),
                FourVec::new(0.0, 0.0, 3.0, 0.0),
                FourVec::new(0.0, 0.0, 0.0, 4.0),
            ),
        ));
        assert_roundtrip(StampedSample::new(105, Bin::new(0.0, 1.5, 7)));
        assert_roundtrip(StampedSample::new(106, Point::new(-1.5, 2.25)));
        assert_roundtrip(StampedSample::new(
            107,
            PointBin::new(0.0, 1.0, 2.0, 3.0, 12),
        ));
    }

    #[test]
    fn test_helpers_and_json_flexibility() {
        let mut s = StampedSample::from((55, ThreeVec::new(3.0, 4.0, 0.0)));
        assert_eq!(s.timestamp(), 55);
        assert_eq!(*s.value(), ThreeVec::new(3.0, 4.0, 0.0));
        *s.value_mut() = ThreeVec::new(0.0, 3.0, 4.0);
        assert_eq!(s.map_value(ThreeVec::r), StampedSample::new(55, 5.0));
        assert_eq!(format!("{}", StampedSample::new(5, 2.5)), "(5, 2.5)");
        assert_eq!(s.into_parts(), (55, ThreeVec::new(0.0, 3.0, 4.0)));

        // Reversed key order and whitespace in JSON
        let reordered = r#" { "value" : [1, 2] , "timestamp" : 77 } "#;
        let expected_point = StampedSample::new(77, Point::new(1.0, 2.0));
        let reordered_point = r#" { "value" : {"x":1,"y":2} , "timestamp" : 77 } "#;
        assert_eq!(
            StampedSample::<Point>::from_json(reordered_point).unwrap(),
            expected_point
        );
        assert!(StampedSample::<Point>::from_json(reordered).is_err());
    }

    #[test]
    fn test_malformed_inputs() {
        assert!(StampedSample::<f64>::from_json(r#"{"timestamp":1}"#).is_err());
        assert!(StampedSample::<f64>::from_json(r#"{"value":1.0}"#).is_err());
        assert!(StampedSample::<f64>::from_json(r#"{"timestamp":1,"value":2.0,"extra":3}"#).is_err());
        assert!(StampedSample::<f64>::from_json(r#"{"timestamp":1,"timestamp":2,"value":3.0}"#).is_err());
        assert!(StampedSample::<f64>::from_json(r#"{"timestamp":"bad","value":1.0}"#).is_err());

        // Wrong array length or type in msgpack
        let mut bad_len = Vec::new();
        write_array_len(&mut bad_len, 3).unwrap();
        write_sint(&mut bad_len, 1).unwrap();
        write_f64(&mut bad_len, 2.0).unwrap();
        write_f64(&mut bad_len, 3.0).unwrap();
        assert!(StampedSample::<f64>::from_msg(&bad_len).is_err());

        let mut bad_ts = Vec::new();
        write_array_len(&mut bad_ts, 2).unwrap();
        write_str(&mut bad_ts, "not_an_int").unwrap();
        write_f64(&mut bad_ts, 2.0).unwrap();
        assert!(StampedSample::<f64>::from_msg(&bad_ts).is_err());
    }
}
