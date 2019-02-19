extern crate rand;

use std::f64;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;
use std::ops::SubAssign;
use std::ops::Mul;
use std::ops::Neg;
use std::fmt;
use std::str::FromStr;
use std::num::ParseFloatError;

use self::rand::thread_rng;
use self::rand::distributions::{Distribution, Uniform};

/// consts module
pub mod consts;

pub mod serializable;
pub use serializable::Serializable;

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
    pub fn random(max: f64) -> ThreeVec {
        let between = Uniform::new_inclusive(-1.0f64,1.0f64);
        let mut rng = thread_rng();
        ThreeVec {
            x0: between.sample(&mut rng)*max,
            x1: between.sample(&mut rng)*max,
            x2: between.sample(&mut rng)*max,
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

impl Serializable for ThreeVec {
    fn to_json(&self) -> String {
        format!("{{\"x0\":{:.*},\"x1\":{:.*},\"x2\":{:.*}}}", 5, self.x0(), 5, self.x1(), 5, self.x2())
    }
}

impl FromStr for ThreeVec {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut x0: f64 = std::f64::NAN;
        let mut x1: f64 = std::f64::NAN;
        let mut x2: f64 = std::f64::NAN;
        for dim in s.trim_matches(|p| p == '{' || p == '}' ).split(',') {
            let n_v: Vec<&str> = dim.split(':').collect();
            match n_v[0] {
                "\"x0\"" => x0 = n_v[1].parse::<f64>()?,
                "\"x1\"" => x1 = n_v[1].parse::<f64>()?,
                "\"x2\"" => x2 = n_v[1].parse::<f64>()?,
                x => panic!("Unexpected invalid token {:?}", x),
            }
        }
        Ok(ThreeVec{x0,x1,x2})
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
    (dot/(r1*r2)).acos()*(180.0/f64::consts::PI)
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
        let xx = ThreeVec::new(1.0,1.0,1.0);
        let pp = xx.to_json();
        assert_eq!(ThreeVec::from_str(&pp).unwrap(),xx);
    }
}
