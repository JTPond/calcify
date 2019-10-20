use std::f64;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;
use std::ops::SubAssign;
use std::ops::Mul;
use std::ops::Neg;
use std::error;
use std::fmt;
use std::str::FromStr;
use std::num::ParseFloatError;

use crate::three_mat;
use crate::utils;

use three_mat::ThreeVec;

use utils::consts;
use utils::Serializable;

extern crate rmp;
use rmp::encode::*;

/// Cannot have a velocity greater than C_LIGHT
#[derive(Debug,Clone)]
pub struct LightSpeedError;

impl fmt::Display for LightSpeedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Cannot have a velocity greater than calcify::C_LIGHT.")
    }
}

impl error::Error for LightSpeedError {
    fn description(&self) -> &str {
         "Cannot have a velocity greater than calcify::C_LIGHT"
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}


/// Variants of S space-time invariant
#[derive(Debug, PartialEq)]
pub enum Sinv {
    TimeLike,
    SpaceLike,
    LightLike,
}

/// Beta factor, |v| over the speed pf light in a vacuum, in SI.
///
/// Returns a Result<f64,&'static str> which contains an Ok(f64), or an error string.
///
/// # Arguments
///
/// * `v` - f64, |v|
///
/// # Example
///
/// ```
/// use calcify::beta;
/// use calcify::LightSpeedError;
/// let v = 149_896_229.0;
/// assert_eq!(beta(v).unwrap(),0.5);
/// assert!(beta(10e10).is_err(),LightSpeedError);
/// ```
pub fn beta(v: f64) -> Result<f64,LightSpeedError> {
    let b1 = v/consts::C_LIGHT;
    match b1 <= 1.0 {
        true => Ok(b1),
        false => Err(LightSpeedError),
    }

}

/// Gamma, the lorentz factor, in SI.
///
/// # Arguments
///
/// * `beta` - f64, |v|/C, use calcify::beta(v)
///
/// # Formula
///
/// ```
/// // 1/sqrt(1 - beta^2)
/// ```
pub fn gamma(beta: f64) -> f64 {
    1.0/(1.0 - beta*beta).sqrt()
}

/// Four Vector
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct FourVec {
    m0: f64,
    m1: f64,
    m2: f64,
    m3: f64,
}

impl FourVec {
    /// Returns a new FourVec from four f64s
    ///
    /// # Arguments
    ///
    /// * `m0` - f64
    /// * `m1` - f64
    /// * `m2` - f64
    /// * `m3` - f64
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// let vec4 = FourVec::new(1.0,2.0,3.0,4.0);
    /// ```
    pub fn new(m0: f64, m1: f64, m2: f64, m3: f64) -> FourVec {
        FourVec {
            m0,
            m1,
            m2,
            m3,
        }
    }

    /// Returns a new FourVec from one f64 and a ThreeVec
    ///
    /// # Arguments
    ///
    /// * `t` - f64
    /// * `x` - calcify::ThreeVec
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::ThreeVec;
    ///
    /// let vec4 = FourVec::from_3vec(1.0,ThreeVec::new(2.0,3.0,4.0));
    /// ```
    pub fn from_3vec(t: f64, x: ThreeVec) -> FourVec {
        FourVec {
            m0: t,
            m1: *x.x0(),
            m2: *x.x1(),
            m3: *x.x2(),
        }
    }

    /// Returns a reference to the first element of the vector
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// let vec4 = FourVec::new(1.0,2.0,3.0,4.0);
    /// let element_zero: f64 = *vec4.m0();
    /// assert_eq!(element_zero,1.0);
    /// ```
    pub fn m0(&self) -> &f64 {
        &self.m0
    }

    /// Returns a reference to the second element of the vector
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// let vec4 = FourVec::new(1.0,2.0,3.0,4.0);
    /// let element_one: f64 = *vec4.m1();
    /// assert_eq!(element_one,2.0);
    /// ```
    pub fn m1(&self) -> &f64 {
        &self.m1
    }

    /// Returns a reference to the third element of the vector
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// let vec4 = FourVec::new(1.0,2.0,3.0,4.0);
    /// let element_two: f64 = *vec4.m2();
    /// assert_eq!(element_two,3.0);
    /// ```
    pub fn m2(&self) -> &f64 {
        &self.m2
    }

    /// Returns a reference to the forth element of the vector
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// let vec4 = FourVec::new(1.0,2.0,3.0,4.0);
    /// let element_three: f64 = *vec4.m3();
    /// assert_eq!(element_three,4.0);
    /// ```
    pub fn m3(&self) -> &f64 {
        &self.m3
    }

    /// Returns the covariant vector with metric [1,-1,-1,-1].
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// let vec4 = FourVec::new(1.0,2.0,3.0,4.0);
    /// let cov_vec4: FourVec = vec4.cov();
    /// assert_eq!(cov_vec4,FourVec::new(1.0,-2.0,-3.0,-4.0));
    ///
    /// assert_eq!(vec4.cov()*vec4, -28.0)
    /// ```
    pub fn cov(self) -> FourVec {
        FourVec {
            m0: self.m0,
            m1: -self.m1,
            m2: -self.m2,
            m3: -self.m3,
        }
    }

    /// Returns the space-time invariant *classification* S^2 of a space-time vector.
    /// Returns a variant of the calcify::Sinv enum
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::Sinv;
    /// let vec4 = FourVec::new(10.0,2.0,2.0,2.0);
    /// let ss: Sinv = vec4.s2();
    /// assert_eq!(ss,Sinv::TimeLike);
    /// ```
    pub fn s2(&self) -> Sinv {
        let ss: f64 = self.cov()**self;
        if ss == 0.0 {
            Sinv::LightLike
        } else if ss > 0.0 {
            Sinv::TimeLike
        } else {
            Sinv::SpaceLike
        }
    }

    /// Returns the invariant of the FourVec.
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// let vec4 = FourVec::new(1.0,0.0,0.0,0.0);
    /// assert_eq!(vec4.s(),1.0);
    /// ```
    pub fn s(&self) -> f64 {
        (self.cov()**self).sqrt()
    }

}

impl fmt::Display for FourVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:.*}, {:.*}, {:.*}, {:.*}]", 5, self.m0(), 5, self.m1(), 5, self.m2(), 5, self.m3())
    }
}

impl Serializable for FourVec {
    fn to_json(&self) -> String {
        format!("{{\"m0\":{},\"m1\":{},\"m2\":{},\"m3\":{}}}",
            self.m0(), self.m1(), self.m2(), self.m3())
    }
    fn to_jsonc(&self) -> String {
        format!("[{},{},{},{}]", self.m0(), self.m1(), self.m2(), self.m3())
    }
    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> {
        let mut buf = Vec::with_capacity(5);
        write_array_len(&mut buf, 4)?;
        write_f64(&mut buf, *self.m0())?;
        write_f64(&mut buf, *self.m1())?;
        write_f64(&mut buf, *self.m2())?;
        write_f64(&mut buf, *self.m3())?;
        Ok(buf)
    }
}

impl FromStr for FourVec {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut m0: f64 = std::f64::NAN;
        let mut m1: f64 = std::f64::NAN;
        let mut m2: f64 = std::f64::NAN;
        let mut m3: f64 = std::f64::NAN;
        for dim in s.trim_matches(|p| p == '{' || p == '}' ).split(',') {
            let n_v: Vec<&str> = dim.split(':').collect();
            match n_v[0] {
                "\"m0\"" => m0 = n_v[1].parse::<f64>()?,
                "\"m1\"" => m1 = n_v[1].parse::<f64>()?,
                "\"m2\"" => m2 = n_v[1].parse::<f64>()?,
                "\"m3\"" => m3 = n_v[1].parse::<f64>()?,
                x => panic!("Unexpected invalid token {:?}", x),
            }
        }
        Ok(FourVec{m0,m1,m2,m3})
    }
}


impl Add for FourVec {
    type Output = FourVec;

    fn add(self, other: FourVec) -> FourVec {
        FourVec {
            m0: self.m0 + *other.m0(),
            m1: self.m1 + *other.m1(),
            m2: self.m2 + *other.m2(),
            m3: self.m3 + *other.m3(),
        }
    }
}

impl AddAssign for FourVec {
    fn add_assign(&mut self, other: FourVec) {
        self.m0 += *other.m0();
        self.m1 += *other.m1();
        self.m2 += *other.m2();
        self.m3 += *other.m3();
    }
}

impl Sub for FourVec {
    type Output = FourVec;

    fn sub(self, other: FourVec) -> FourVec {
        FourVec {
            m0: self.m0 - *other.m0(),
            m1: self.m1 - *other.m1(),
            m2: self.m2 - *other.m2(),
            m3: self.m3 - *other.m3(),
        }
    }
}

impl SubAssign for FourVec {
    fn sub_assign(&mut self, other: FourVec) {
        self.m0 -= *other.m0();
        self.m1 -= *other.m1();
        self.m2 -= *other.m2();
        self.m3 -= *other.m3();
    }
}

impl Mul<f64> for FourVec {
    type Output = FourVec;

    fn mul(self, coef: f64) -> FourVec {
        FourVec {
            m0: self.m0 * coef,
            m1: self.m1 * coef,
            m2: self.m2 * coef,
            m3: self.m3 * coef,
        }
    }
}

impl Mul<FourVec> for f64 {
    type Output = FourVec;

    fn mul(self, vec: FourVec) -> FourVec {
        FourVec {
            m0: *vec.m0() * self,
            m1: *vec.m1() * self,
            m2: *vec.m2() * self,
            m3: *vec.m3() * self,
        }
    }
}

impl Mul<FourVec> for FourVec {
    type Output = f64;
    /// _Standard_ scalar product,
    ///
    /// # Example
    ///
    /// ```
    /// use calcify::FourVec;
    /// let vec4 = FourVec::new(2.0,2.0,2.0,2.0);
    ///
    /// assert_eq!(
    ///    vec4*vec4,
    ///    16.0
    /// );
    /// ```
    fn mul(self, other: FourVec) -> f64 {
        self.m0 * *other.m0() + self.m1 * *other.m1() + self.m2 * *other.m2() + self.m3 * *other.m3()
    }
}

impl Neg for FourVec {
    type Output = FourVec;

    fn neg(self) -> FourVec {
        FourVec {
            m0: -self.m0,
            m1: -self.m1,
            m2: -self.m2,
            m3: -self.m3,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beta() {
        let v = 149_896_229.0;
        assert_eq!(beta(v).unwrap(),0.5);
        assert!(beta(10e10).is_err(),"Beta must be ltgt 1.0");
    }

    #[test]
    fn test_invariant() {
        let vec4 = FourVec::new(5.0,2.0,2.0,2.0);
        assert_eq!(vec4.cov()*vec4,13.0);
    }

    #[test]
    fn test_json() {
        let vec4 = FourVec::new(5.0,2.0,2.0,2.0);
        assert_eq!(vec4.to_json(),"{\"m0\":5,\"m1\":2,\"m2\":2,\"m3\":2}");
    }

    #[test]
    fn test_parse() {
        let xx = FourVec::new(5.0,2.0,2.0,2.0);
        let pp = xx.to_json();
        assert_eq!(FourVec::from_str(&pp).unwrap(),xx);
    }
}
