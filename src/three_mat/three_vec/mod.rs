use std::error;
use std::fmt;
use std::iter;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::SubAssign;

#[cfg(feature = "rand")]
use rand::distr::{Distribution, Uniform};

use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

use crate::utils;
use utils::errors::CalcifyError;
use utils::{Deserializable, Serializable};

use rmp::decode::*;
use rmp::encode::*;

/// Three Vector
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ThreeVec {
    x0: f64,
    x1: f64,
    x2: f64,
}

impl ThreeVec {
    /// Returns a new ThreeVec from three f64s
    ///
    /// # Arguments
    ///
    /// * `x0` - f64
    /// * `x1` - f64
    /// * `x2` - f64
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeVec;
    /// let vec3 = ThreeVec::new(1.0,2.0,3.0);
    /// ```
    pub fn new(x0: f64, x1: f64, x2: f64) -> ThreeVec {
        ThreeVec {
            x0,
            x1,
            x2,
        }
    }

    /// Returns a new ThreeVec from a slice
    ///
    /// # Arguments
    ///
    /// * `slice` - &[prim@f64]
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeVec;
    /// let vec3: ThreeVec = ThreeVec::from(&[1.0,1.0,1.0]);
    /// ```
    ///
    /// # Panics
    ///
    /// * `slice` length < 3
    pub fn from(slice: &[f64]) -> ThreeVec {

        ThreeVec {
            x0: slice[0],
            x1: slice[1],
            x2: slice[2],
        }
    }

    /// Returns a new ThreeVec with three random f64 from rand::Uniform between -1 and 1
    ///
    /// # Arguments
    ///
    /// * `max` - f64: The absolute maximum value of each individule componant of the constituent ThreeVec
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeVec;
    /// let vec3 = ThreeVec::random(10.0);
    /// ```
    #[cfg(feature = "rand")]
    pub fn random(max: f64) -> ThreeVec {
        let between = Uniform::new_inclusive(-1.0_f64, 1.0_f64).unwrap();
        let mut rng = rand::rng();
        ThreeVec {
            x0: between.sample(&mut rng) * max,
            x1: between.sample(&mut rng) * max,
            x2: between.sample(&mut rng) * max,
        }
    }

    /// Returns a reference to the first element of the vector
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeVec;
    /// let vec3 = ThreeVec::new(1.0,2.0,3.0);
    /// let element_zero: f64 = *vec3.x0();
    /// assert_eq!(element_zero,1.0);
    /// ```
    pub fn x0(&self) -> &f64 {
        &self.x0
    }

    /// Returns a reference to the second element of the vector
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeVec;
    /// let vec3 = ThreeVec::new(1.0,2.0,3.0);
    /// let element_one: f64 = *vec3.x1();
    /// assert_eq!(element_one,2.0);
    /// ```
    pub fn x1(&self) -> &f64 {
        &self.x1
    }

    /// Returns a reference to the third element of the vector
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeVec;
    /// let vec3 = ThreeVec::new(1.0,2.0,3.0);
    /// let element_two: f64 = *vec3.x2();
    /// assert_eq!(element_two,3.0);
    /// ```
    pub fn x2(&self) -> &f64 {
        &self.x2
    }

    /// Returns the length of the vector
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeVec;
    /// let vec3 = ThreeVec::new(1.0,0.0,0.0);
    /// assert_eq!(vec3.r(),1.0);
    /// ```
    pub fn r(&self) -> f64 {
        (*self**self).sqrt()
    }
}

impl fmt::Display for ThreeVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:.*}, {:.*}, {:.*}]", 5, self.x0(), 5, self.x1(), 5, self.x2())
    }
}

impl Serialize for ThreeVec {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("ThreeVec", 3)?;
        state.serialize_field("x0", &self.x0)?;
        state.serialize_field("x1", &self.x1)?;
        state.serialize_field("x2", &self.x2)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ThreeVec {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        enum Field {
            X0,
            X1,
            X2,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Field, D::Error> {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        f.write_str("`x0`, `x1`, or `x2`")
                    }

                    fn visit_str<E: de::Error>(self, value: &str) -> Result<Field, E> {
                        match value {
                            "x0" => Ok(Field::X0),
                            "x1" => Ok(Field::X1),
                            "x2" => Ok(Field::X2),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ThreeVecVisitor;

        impl<'de> Visitor<'de> for ThreeVecVisitor {
            type Value = ThreeVec;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("struct ThreeVec")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<ThreeVec, V::Error> {
                let mut x0: Option<f64> = None;
                let mut x1: Option<f64> = None;
                let mut x2: Option<f64> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::X0 => {
                            if x0.is_some() {
                                return Err(de::Error::duplicate_field("x0"));
                            }
                            x0 = Some(map.next_value()?);
                        }
                        Field::X1 => {
                            if x1.is_some() {
                                return Err(de::Error::duplicate_field("x1"));
                            }
                            x1 = Some(map.next_value()?);
                        }
                        Field::X2 => {
                            if x2.is_some() {
                                return Err(de::Error::duplicate_field("x2"));
                            }
                            x2 = Some(map.next_value()?);
                        }
                    }
                }
                let x0 = x0.ok_or_else(|| de::Error::missing_field("x0"))?;
                let x1 = x1.ok_or_else(|| de::Error::missing_field("x1"))?;
                let x2 = x2.ok_or_else(|| de::Error::missing_field("x2"))?;
                Ok(ThreeVec { x0, x1, x2 })
            }
        }

        const FIELDS: &[&str] = &["x0", "x1", "x2"];
        deserializer.deserialize_struct("ThreeVec", FIELDS, ThreeVecVisitor)
    }
}

impl Serializable for ThreeVec {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::with_capacity(4);
        write_array_len(&mut buf, 3)?;
        write_f64(&mut buf, *self.x0())?;
        write_f64(&mut buf, *self.x1())?;
        write_f64(&mut buf, *self.x2())?;
        Ok(buf)
    }
}

impl Deserializable for ThreeVec {
    fn from_json(s: &str) -> Result<Self, Box<dyn error::Error>> {
        Ok(serde_json::from_str(s)?)
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        if let Ok(3) = read_array_len(&mut bytes) {
            let mut x: [f64; 3] = [f64::NAN; 3];
            for item in &mut x {
                *item = read_f64(&mut bytes)?;
            }
            Ok((ThreeVec::from(&x), bytes))
        } else {
            Err(Box::new(CalcifyError::ParseError(String::from(
                "invalid MessagePack ThreeVec",
            ))))
        }
    }
}

impl Add for ThreeVec {
    type Output = ThreeVec;

    fn add(self, other: ThreeVec) -> ThreeVec {
        ThreeVec {
            x0: self.x0 + *other.x0(),
            x1: self.x1 + *other.x1(),
            x2: self.x2 + *other.x2(),
        }
    }
}

impl iter::Sum for ThreeVec {
    fn sum<I>(iter: I) -> ThreeVec
    where I: Iterator<Item = ThreeVec> {
        iter.fold(ThreeVec { x0: 0.0, x1: 0.0, x2: 0.0 }, |a, b| a + b)
    }
}

impl AddAssign for ThreeVec {
    fn add_assign(&mut self, other: ThreeVec) {
        self.x0 += *other.x0();
        self.x1 += *other.x1();
        self.x2 += *other.x2();
    }
}

impl Sub for ThreeVec {
    type Output = ThreeVec;

    fn sub(self, other: ThreeVec) -> ThreeVec {
        ThreeVec {
            x0: self.x0 - *other.x0(),
            x1: self.x1 - *other.x1(),
            x2: self.x2 - *other.x2(),
        }
    }
}

impl SubAssign for ThreeVec {
    fn sub_assign(&mut self, other: ThreeVec) {
        self.x0 -= *other.x0();
        self.x1 -= *other.x1();
        self.x2 -= *other.x2();
    }
}

impl Mul<f64> for ThreeVec {
    type Output = ThreeVec;

    fn mul(self, coef: f64) -> ThreeVec {
        ThreeVec {
            x0: self.x0 * coef,
            x1: self.x1 * coef,
            x2: self.x2 * coef,
        }
    }
}

impl Mul<ThreeVec> for f64 {
    type Output = ThreeVec;

    fn mul(self, vec: ThreeVec) -> ThreeVec {
        ThreeVec {
            x0: *vec.x0() * self,
            x1: *vec.x1() * self,
            x2: *vec.x2() * self,
        }
    }
}

impl Mul<ThreeVec> for ThreeVec {
    type Output = f64;
    /// Dot product
    ///
    /// # Example
    ///
    /// ```
    /// use calcify::ThreeVec;
    /// let vec3 = ThreeVec::new(2.0,2.0,2.0);
    ///
    /// assert_eq!(
    ///    vec3*vec3,
    ///    12.0
    /// );
    /// ```
    fn mul(self, other: ThreeVec) -> f64 {
        self.x0 * *other.x0() + self.x1 * *other.x1() + self.x2 * *other.x2()
    }
}

impl Neg for ThreeVec {
    type Output = ThreeVec;

    fn neg(self) -> ThreeVec {
        ThreeVec {
            x0: -self.x0,
            x1: -self.x1,
            x2: -self.x2,
        }
    }
}

/// Return the angle between two vectors in radians
///
/// # Example
///
/// ```
/// use std::f64;
/// use calcify::ThreeVec;
/// use calcify::radians_between;
///
/// let vec1 = ThreeVec::new(1.0,0.0,0.0);
/// let vec2 = ThreeVec::new(0.0,1.0,0.0);
///
/// assert_eq!(radians_between(vec1,vec2),f64::consts::PI/2.0);
/// ```
pub fn radians_between(one: ThreeVec, other: ThreeVec) -> f64 {
    let dot = one*other;
    let r1 = (one*one).sqrt();
    let r2 = (other*other).sqrt();
    (dot/(r1*r2)).acos()
}

/// Return the angle between two vectors in degrees
///
/// # Example
///
/// ```
/// use std::f64;
/// use calcify::ThreeVec;
/// use calcify::degrees_between;
///
/// let vec1 = ThreeVec::new(1.0,0.0,0.0);
/// let vec2 = ThreeVec::new(0.0,1.0,0.0);
///
/// assert_eq!(degrees_between(vec1,vec2),90.0);
/// ```
pub fn degrees_between(one: ThreeVec, other: ThreeVec) -> f64 {
    let dot = one*other;
    let r1 = (one*one).sqrt();
    let r2 = (other*other).sqrt();
    (dot/(r1*r2)).acos()*(180.0/std::f64::consts::PI)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let _test_vec1 = ThreeVec::new(1.0,2.0,3.0);
        let _test_vec2 = ThreeVec::new(2.0,3.0,4.0);

        assert_eq!(
            _test_vec1+_test_vec2,
            ThreeVec::new(3.0,5.0,7.0)
        );
    }

    #[test]
    fn test_loop_add() {
        let mut _test_vec1 = ThreeVec::new(1.0,1.0,1.0);
        for _i in 0..9999{
            _test_vec1 += ThreeVec::new(1.0,1.0,1.0);
        }

        assert_eq!(
            _test_vec1,
            ThreeVec::new(10_000.0,10_000.0,10_000.0)
        );
    }

    #[test]
    fn test_sub() {
        let _test_vec1 = ThreeVec::new(3.0,5.0,7.0);
        let _test_vec2 = ThreeVec::new(2.0,3.0,4.0);

        assert_eq!(
            _test_vec1-_test_vec2,
            ThreeVec::new(1.0,2.0,3.0)
        );
    }

    #[test]
    fn test_loop_sub() {
        let mut _test_vec1 = ThreeVec::new(10_000.0,10_000.0,10_000.0);
        for _i in 0..9999{
            _test_vec1 -= ThreeVec::new(1.0,1.0,1.0);
        }

        assert_eq!(
            _test_vec1,
            ThreeVec::new(1.0,1.0,1.0)
        );
    }

    #[test]
    fn test_mul() {
        let _test_vec1 = ThreeVec::new(2.0,2.0,2.0);
        let _test_vec2 = ThreeVec::new(2.0,2.0,2.0);

        assert_eq!(
            _test_vec1*_test_vec2,
            12.0
        );
    }

    #[test]
    fn test_mul_coef() {
        let _test_vec1 = ThreeVec::new(2.0,2.0,2.0);

        assert_eq!(
            _test_vec1*2.0,
            ThreeVec::new(4.0,4.0,4.0)
        );
        assert_eq!(
            2.0*_test_vec1,
            ThreeVec::new(4.0,4.0,4.0)
        );
    }

    #[test]
    fn test_neg() {
        let _test_vec1 = ThreeVec::new(2.0,2.0,2.0);

        assert_eq!(
            -_test_vec1,
            ThreeVec::new(-2.0,-2.0,-2.0)
        );
    }

    #[test]
    fn test_copy() {
        let xx = ThreeVec::new(1.0,1.0,1.0);
        let yy = xx;
        assert_eq!(
            xx+yy,
            ThreeVec::new(2.0,2.0,2.0)
        );
    }

    #[test]
    fn test_parse() {
        let xx = ThreeVec::new(1.0, 2.0, -3.5);
        let pp = xx.to_json();
        assert_eq!(ThreeVec::from_json(&pp).unwrap(), xx);
        assert_eq!(
            ThreeVec::from_json(r#" { "x2" : -3.5 , "x0" : 1.0 , "x1" : 2.0 } "#).unwrap(),
            xx
        );
        let rt: ThreeVec = serde_json::from_str(&serde_json::to_string(&xx).unwrap()).unwrap();
        assert_eq!(rt, xx);

        assert!(ThreeVec::from_json(r#"{"x0"}"#).is_err());
        assert!(ThreeVec::from_json(r#"{"x0":1.0,"x1":2.0}"#).is_err());
        assert!(ThreeVec::from_json(r#"{"x0":1.0,"x1":2.0,"x2":3.0,"x3":4.0}"#).is_err());
        assert!(ThreeVec::from_json(r#"{"x0":1.0,"x0":2.0,"x1":3.0,"x2":4.0}"#).is_err());
    }

    #[test]
    fn test_msg_parse() {
        let xx = ThreeVec::new(1.0,2.0,3.0);
        let pp = xx.to_msg().unwrap();
        let (oo,_) = ThreeVec::from_msg(&pp).unwrap();
        assert_eq!(oo,xx);
    }
}
