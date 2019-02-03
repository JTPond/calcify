use std::f64;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;
use std::ops::SubAssign;
use std::ops::Mul;
use std::ops::Neg;
use std::fmt;

mod three_mat;
pub use three_mat::ThreeMat;
pub use three_mat::ThreeVec;
pub use three_mat::{radians_between, degrees_between};

/// The speed of light in a vacuum
pub const C_LIGHT: f64 = 299_792_458.0;

/// Beta factor, |v| over the speed pf light in a vacuum.
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
/// let v = 149_896_229.0;
/// assert_eq!(beta(v).unwrap(),0.5);
/// assert!(beta(10e10).is_err(),"Beta must be ltet 1.0");
/// ```
pub fn beta(v: f64) -> Result<f64,&'static str> {
    let b1 = v/super::C_LIGHT;
    match b1 <= 1.0 {
        true => Ok(b1),
        false => Err("Beta must be ltet 1.0"),
    }

}

/// Gamma, the lorentz factor.
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
}

impl fmt::Display for FourVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:.*}, {:.*}, {:.*}, {:.*}]", 5, self.m0(), 5, self.m1(), 5, self.m2(), 5, self.m3())
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
    /// Invariant product,
    ///
    /// # Formula
    ///
    /// ```
    /// //m_0*m^0 - (m_1*m^1 + m_2*m^2 + m_3*m^3)
    /// ```
    /// # Example
    ///
    /// ```
    /// use calcify::FourVec;
    /// let vec4 = FourVec::new(5.0,2.0,2.0,2.0);
    ///
    /// assert_eq!(
    ///    vec4*vec4,
    ///    13.0
    /// );
    /// ```
    fn mul(self, other: FourVec) -> f64 {
        self.m0 * *other.m0() - (self.m1 * *other.m1() + self.m2 * *other.m2() + self.m3 * *other.m3())
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

}
