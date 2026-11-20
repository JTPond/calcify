use std::error;
use std::fmt;
use std::iter;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::SubAssign;

use rmp::decode::*;
use rmp::encode::*;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

#[cfg(feature = "rand")]
use rand::distr::{Distribution, Uniform};

use crate::utils;
use utils::errors::CalcifyError;
use utils::{Deserializable, Serializable};

/// Point, or Two Vector, depending on your perspective.
/// A plot is a Collection of Points
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Returns new Point
    ///
    /// # Arguments
    ///
    /// * `x` - prim@f64 Independent Variable
    /// * `y` - prim@f64 Dependent Variable
    ///
    pub fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    /// Returns a new Point from a slice
    ///
    /// # Arguments
    ///
    /// * `slice` - &[prim@f64]
    ///
    /// # Example
    /// ```
    /// use calcify::Point;
    /// let point: Point = Point::from(&[1.0,1.0]);
    /// ```
    ///
    /// # Panics
    ///
    /// * `slice` length < 2
    pub fn from(slice: &[f64]) -> Point {
        Point {
            x: slice[0],
            y: slice[1],
        }
    }

    /// Returns a new Point with two random f64 from rand::Uniform between -1 and 1
    ///
    /// # Arguments
    ///
    /// * `max` - prim@f64: The absolute maximum value of each individule componant of the constituent Point
    ///
    /// # Example
    /// ```
    /// use calcify::Point;
    /// let vec2 = Point::random(10.0);
    /// ```
    #[cfg(feature = "rand")]
    pub fn random(max: f64) -> Point {
        let between = Uniform::new_inclusive(-1.0_f64, 1.0_f64).unwrap();
        let mut rng = rand::rng();
        Point {
            x: between.sample(&mut rng) * max,
            y: between.sample(&mut rng) * max,
        }
    }

    /// Returns the length of the 2vector
    ///
    /// # Example
    /// ```
    /// use calcify::Point;
    /// let vec2 = Point::new(1.0,0.0);
    /// assert_eq!(vec2.r(),1.0);
    /// ```
    pub fn r(&self) -> f64 {
        (*self * *self).sqrt()
    }
}

impl Serialize for Point {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Point", 2)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Point {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        enum Field {
            X,
            Y,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Field, D::Error> {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        f.write_str("`x` or `y`")
                    }

                    fn visit_str<E: de::Error>(self, value: &str) -> Result<Field, E> {
                        match value {
                            "x" => Ok(Field::X),
                            "y" => Ok(Field::Y),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct PointVisitor;

        impl<'de> Visitor<'de> for PointVisitor {
            type Value = Point;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("struct Point")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Point, V::Error> {
                let mut x: Option<f64> = None;
                let mut y: Option<f64> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::X => {
                            if x.is_some() {
                                return Err(de::Error::duplicate_field("x"));
                            }
                            x = Some(map.next_value()?);
                        }
                        Field::Y => {
                            if y.is_some() {
                                return Err(de::Error::duplicate_field("y"));
                            }
                            y = Some(map.next_value()?);
                        }
                    }
                }
                let x = x.ok_or_else(|| de::Error::missing_field("x"))?;
                let y = y.ok_or_else(|| de::Error::missing_field("y"))?;
                Ok(Point { x, y })
            }
        }

        const FIELDS: &[&str] = &["x", "y"];
        deserializer.deserialize_struct("Point", FIELDS, PointVisitor)
    }
}

impl Serializable for Point {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::with_capacity(3);
        write_array_len(&mut buf, 2)?;
        write_f64(&mut buf, self.x)?;
        write_f64(&mut buf, self.y)?;
        Ok(buf)
    }
}

impl Deserializable for Point {
    fn from_json(s: &str) -> Result<Self, Box<dyn error::Error>> {
        Ok(serde_json::from_str(s)?)
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        if let Ok(2) = read_array_len(&mut bytes) {
            let mut x: [f64; 2] = [f64::NAN; 2];
            for item in &mut x {
                *item = read_f64(&mut bytes)?;
            }
            Ok((Point::from(&x), bytes))
        } else {
            Err(Box::new(CalcifyError::ParseError(String::from(
                "invalid MessagePack Point",
            ))))
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:.*}, {:.*}]", 5, self.x, 5, self.y)
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl iter::Sum for Point {
    fn sum<I>(iter: I) -> Point
    where I: Iterator<Item = Point> {
        iter.fold(Point { x: 0.0, y: 0.0 }, |a, b| a + b)
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, other: Point) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, other: Point) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Mul<f64> for Point {
    type Output = Point;

    fn mul(self, coef: f64) -> Point {
        Point {
            x: self.x * coef,
            y: self.y * coef,
        }
    }
}

impl Mul<Point> for f64 {
    type Output = Point;

    fn mul(self, vec: Point) -> Point {
        Point {
            x: vec.x * self,
            y: vec.y * self,
        }
    }
}

impl Mul<Point> for Point {
    type Output = f64;
    /// Dot product
    ///
    /// # Example
    ///
    /// ```
    /// use calcify::Point;
    /// let point = Point::new(2.0,2.0);
    ///
    /// assert_eq!(
    ///    point*point,
    ///    8.0
    /// );
    /// ```
    fn mul(self, other: Point) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

impl Neg for Point {
    type Output = Point;

    fn neg(self) -> Point {
        Point {
            x: -self.x,
            y: -self.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let _test_vec1 = Point::new(1.0,2.0);
        let _test_vec2 = Point::new(2.0,3.0);

        assert_eq!(
            _test_vec1+_test_vec2,
            Point::new(3.0,5.0)
        );
    }

    #[test]
    fn test_loop_add() {
        let mut _test_vec1 = Point::new(1.0,1.0);
        for _i in 0..9999{
            _test_vec1 += Point::new(1.0,1.0);
        }

        assert_eq!(
            _test_vec1,
            Point::new(10_000.0,10_000.0)
        );
    }

    #[test]
    fn test_sub() {
        let _test_vec1 = Point::new(3.0,5.0);
        let _test_vec2 = Point::new(2.0,3.0);

        assert_eq!(
            _test_vec1-_test_vec2,
            Point::new(1.0,2.0)
        );
    }

    #[test]
    fn test_loop_sub() {
        let mut _test_vec1 = Point::new(10_000.0,10_000.0);
        for _i in 0..9999{
            _test_vec1 -= Point::new(1.0,1.0);
        }

        assert_eq!(
            _test_vec1,
            Point::new(1.0,1.0)
        );
    }

    #[test]
    fn test_mul() {
        let _test_vec1 = Point::new(2.0,2.0);
        let _test_vec2 = Point::new(2.0,2.0);

        assert_eq!(
            _test_vec1*_test_vec2,
            8.0
        );
    }

    #[test]
    fn test_mul_coef() {
        let _test_vec1 = Point::new(2.0,2.0);

        assert_eq!(
            _test_vec1*2.0,
            Point::new(4.0,4.0)
        );
        assert_eq!(
            2.0*_test_vec1,
            Point::new(4.0,4.0)
        );
    }

    #[test]
    fn test_neg() {
        let _test_vec1 = Point::new(2.0,2.0);

        assert_eq!(
            -_test_vec1,
            Point::new(-2.0,-2.0)
        );
    }

    #[test]
    fn test_copy() {
        let xx = Point::new(1.0,1.0);
        let yy = xx;
        assert_eq!(
            xx+yy,
            Point::new(2.0,2.0)
        );
    }

    #[test]
    fn test_parse() {
        let xx = Point::new(1.0, 2.5);
        let pp = xx.to_json();
        assert_eq!(Point::from_json(&pp).unwrap(), xx);
        assert_eq!(
            Point::from_json(r#" { "y" : 2.5 , "x" : 1.0 } "#).unwrap(),
            xx
        );
        let rt: Point = serde_json::from_str(&serde_json::to_string(&xx).unwrap()).unwrap();
        assert_eq!(rt, xx);

        assert_eq!(Point::new(f64::NAN, 1.0).to_json(), r#"{"x":null,"y":1.0}"#);
        assert_eq!(
            Point::new(f64::INFINITY, f64::NEG_INFINITY).to_json(),
            r#"{"x":null,"y":null}"#
        );

        assert!(Point::from_json(r#"{"x":1.0}"#).is_err());
        assert!(Point::from_json(r#"{"x":1.0,"y":2.0,"z":3.0}"#).is_err());
        assert!(Point::from_json(r#"{"x":1.0,"x":2.0,"y":3.0}"#).is_err());
        assert!(Point::from_json(r#"{"x":null,"y":1.0}"#).is_err());
    }

    #[test]
    fn test_msg_parse() {
        let xx = Point::new(1.0,1.0);
        let pp = xx.to_msg().unwrap();
        let (oo,_) = Point::from_msg(&pp).unwrap();
        assert_eq!(oo,xx);
    }
}
