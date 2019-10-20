use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;
use std::ops::SubAssign;
use std::ops::Mul;
use std::ops::Neg;
use std::fmt;
use std::str::FromStr;
use std::num::ParseFloatError;

/// Three Vector Module
mod three_vec;
pub use three_vec::ThreeVec;
pub use three_vec::{radians_between, degrees_between};

use crate::utils;
use utils::Serializable;

extern crate rmp;
use rmp::encode::*;

/// Three Matrix
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ThreeMat {
    /// Three rows, each a calcify::ThreeVec
    r0: ThreeVec,
    r1: ThreeVec,
    r2: ThreeVec,
}

impl ThreeMat {
    /// Returns a new ThreeMat from three ThreeVecs
    ///
    /// # Arguments
    ///
    /// * `r0` - calcify::ThreeVec
    /// * `r1` - calcify::ThreeVec
    /// * `r2` - calcify::ThreeVec
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeVec;
    /// use calcify::ThreeMat;
    /// let mat3 = ThreeMat::new(
    ///               ThreeVec::new(1.0,2.0,3.0),
    ///               ThreeVec::new(4.0,5.0,6.0),
    ///               ThreeVec::new(7.0,8.0,9.0)
    ///            );
    /// ```
    pub fn new(r0: ThreeVec, r1: ThreeVec, r2: ThreeVec) -> ThreeMat {
        ThreeMat {
            r0,
            r1,
            r2,
        }
    }

    /// Returns a new ThreeMat with three random ThreeVecs using calcify::ThreeVec::random(max: f64)
    ///
    /// # Arguments
    ///
    /// * `max` - f64: The absolute maximum value of each individule componant of the constituent ThreeVec
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeMat;
    /// let mat3 = ThreeMat::random(10.0);
    /// ```
    pub fn random(max: f64) -> ThreeMat {
        ThreeMat {
            r0: ThreeVec::random(max),
            r1: ThreeVec::random(max),
            r2: ThreeVec::random(max),
        }
    }

    /// Returns a new ThreeMat identity matrix
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeMat;
    /// let mat3 = ThreeMat::eye();
    ///
    /// assert_eq!(*mat3.r1().x1(),1.0);
    /// ```
    pub fn eye() -> ThreeMat {
        ThreeMat {
            r0: ThreeVec::new(1.0,0.0,0.0),
            r1: ThreeVec::new(0.0,1.0,0.0),
            r2: ThreeVec::new(0.0,0.0,1.0),
        }
    }

    /// Returns a new ThreeMat zero matrix
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeMat;
    /// let mat3 = ThreeMat::zero();
    ///
    /// assert_eq!(*mat3.r1().x1(),0.0);
    /// ```
    pub fn zero() -> ThreeMat {
        ThreeMat {
            r0: ThreeVec::new(0.0,0.0,0.0),
            r1: ThreeVec::new(0.0,0.0,0.0),
            r2: ThreeVec::new(0.0,0.0,0.0),
        }
    }

    /// Returns a new ThreeMat one matrix
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeMat;
    /// let mat3 = ThreeMat::one();
    ///
    /// assert_eq!(*mat3.r1().x1(),1.0);
    /// ```
    pub fn one() -> ThreeMat {
        ThreeMat {
            r0: ThreeVec::new(1.0,1.0,1.0),
            r1: ThreeVec::new(1.0,1.0,1.0),
            r2: ThreeVec::new(1.0,1.0,1.0),
        }
    }

    /// Returns a reference to the first row of the matrix.
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeVec;
    /// use calcify::ThreeMat;
    /// let mat3 = ThreeMat::new(
    ///               ThreeVec::new(1.0,2.0,3.0),
    ///               ThreeVec::new(4.0,5.0,6.0),
    ///               ThreeVec::new(7.0,8.0,9.0)
    ///            );
    /// let row_zero: ThreeVec = *mat3.r0();
    /// let element_zero_zero: f64 = *mat3.r0().x0();
    /// assert_eq!(row_zero,ThreeVec::new(1.0,2.0,3.0));
    /// assert_eq!(element_zero_zero,1.0);
    /// ```
    pub fn r0(&self) -> &ThreeVec {
        &self.r0
    }


    /// Returns a reference to the second row of the matrix.
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeVec;
    /// use calcify::ThreeMat;
    /// let mat3 = ThreeMat::new(
    ///               ThreeVec::new(1.0,2.0,3.0),
    ///               ThreeVec::new(4.0,5.0,6.0),
    ///               ThreeVec::new(7.0,8.0,9.0)
    ///            );
    /// let row_one: ThreeVec = *mat3.r1();
    /// let element_one_one: f64 = *mat3.r1().x1();
    /// assert_eq!(row_one,ThreeVec::new(4.0,5.0,6.0));
    /// assert_eq!(element_one_one,5.0);
    /// ```
    pub fn r1(&self) -> &ThreeVec {
        &self.r1
    }

    /// Returns a reference to the third row of the matrix.
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeVec;
    /// use calcify::ThreeMat;
    /// let mat3 = ThreeMat::new(
    ///               ThreeVec::new(1.0,2.0,3.0),
    ///               ThreeVec::new(4.0,5.0,6.0),
    ///               ThreeVec::new(7.0,8.0,9.0)
    ///            );
    /// let row_two: ThreeVec = *mat3.r2();
    /// let element_two_two: f64 = *mat3.r2().x2();
    /// assert_eq!(row_two,ThreeVec::new(7.0,8.0,9.0));
    /// assert_eq!(element_two_two,9.0);
    /// ```
    pub fn r2(&self) -> &ThreeVec {
        &self.r2
    }

    /// Returns a new memory ThreeVec of the first column of the matrix.
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeVec;
    /// use calcify::ThreeMat;
    /// let mat3 = ThreeMat::new(
    ///               ThreeVec::new(1.0,2.0,3.0),
    ///               ThreeVec::new(4.0,5.0,6.0),
    ///               ThreeVec::new(7.0,8.0,9.0)
    ///            );
    /// let col_one: ThreeVec = mat3.c0();
    /// let element_one_one: f64 = *mat3.c0().x0();
    /// assert_eq!(col_one,ThreeVec::new(1.0,4.0,7.0));
    /// assert_eq!(element_one_one,1.0);
    /// ```
    pub fn c0(&self) -> ThreeVec {
        ThreeVec::new(*self.r0.x0(),*self.r1.x0(),*self.r2.x0())
    }

    pub fn c1(&self) -> ThreeVec {
        ThreeVec::new(*self.r0.x1(),*self.r1.x1(),*self.r2.x1())
    }

    pub fn c2(&self) -> ThreeVec {
        ThreeVec::new(*self.r0.x2(),*self.r1.x2(),*self.r2.x2())
    }
}

impl fmt::Display for ThreeMat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{},\n{},\n{}]", self.r0(), self.r1(), self.r2())
    }
}

impl Serializable for ThreeMat {
    fn to_json(&self) -> String {
        format!("{{\"r0\":{},\"r1\":{},\"r2\":{}}}",
            self.r0().to_json(),
            self.r1().to_json(),
            self.r2().to_json()
        )
    }
    fn to_jsonc(&self) -> String {
        format!("[{},{},{}]",
            self.r0().to_jsonc(),
            self.r1().to_jsonc(),
            self.r2().to_jsonc()
        )
    }
    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> {
        let mut buf = Vec::new();
        write_array_len(&mut buf, 3)?;
        buf.append(&mut self.r0().to_msg()?);
        buf.append(&mut self.r1().to_msg()?);
        buf.append(&mut self.r2().to_msg()?);
        Ok(buf)
    }
}

impl FromStr for ThreeMat {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut r0: ThreeVec = ThreeVec::new(std::f64::NAN,std::f64::NAN,std::f64::NAN);
        let mut r1: ThreeVec = ThreeVec::new(std::f64::NAN,std::f64::NAN,std::f64::NAN);
        let mut r2: ThreeVec = ThreeVec::new(std::f64::NAN,std::f64::NAN,std::f64::NAN);
        for dim in s.replace("}}","|}").replace("},","}|").replace(":{",":!{").trim_matches(|p| p == '{' || p == '}' ).split_terminator('|') {
            let n_v: Vec<&str> = dim.split(":!").collect();
            match n_v[0] {
                "\"r0\"" => r0 = ThreeVec::from_str(n_v[1])?,
                "\"r1\"" => r1 = ThreeVec::from_str(n_v[1])?,
                "\"r2\"" => r2 = ThreeVec::from_str(n_v[1])?,
                x => panic!("Unexpected invalid token {:?}", x),
            }
        }
        Ok(ThreeMat{r0,r1,r2})
    }
}

impl Add for ThreeMat {
    type Output = ThreeMat;

    fn add(self, other: ThreeMat) -> ThreeMat {
        ThreeMat {
            r0: self.r0 + *other.r0(),
            r1: self.r1 + *other.r1(),
            r2: self.r2 + *other.r2(),
        }
    }
}

impl AddAssign for ThreeMat {
    fn add_assign(&mut self, other: ThreeMat) {
        self.r0 +=*other.r0();
        self.r1 +=*other.r1();
        self.r2 +=*other.r2();
    }
}

impl Sub for ThreeMat {
    type Output = ThreeMat;

    fn sub(self, other: ThreeMat) -> ThreeMat {
        ThreeMat {
            r0: self.r0 -*other.r0(),
            r1: self.r1 -*other.r1(),
            r2: self.r2 -*other.r2(),
        }
    }
}

impl SubAssign for ThreeMat {
    fn sub_assign(&mut self, other: ThreeMat) {
        self.r0 -=*other.r0();
        self.r1 -=*other.r1();
        self.r2 -=*other.r2();
    }
}

impl Mul<f64> for ThreeMat {
    type Output = ThreeMat;

    fn mul(self, coef: f64) -> ThreeMat {
        ThreeMat {
            r0: self.r0 *coef,
            r1: self.r1 *coef,
            r2: self.r2 *coef,
        }
    }
}

impl Mul<ThreeMat> for f64 {
    type Output = ThreeMat;

    fn mul(self, vec: ThreeMat) -> ThreeMat {
        ThreeMat {
            r0: *vec.r0() * self,
            r1: *vec.r1() * self,
            r2: *vec.r2() * self,
        }
    }
}

impl Mul<ThreeMat> for ThreeMat {
    type Output = ThreeMat;
    /// Matrix multiplication
    ///
    /// # Example
    ///
    /// ```
    /// use calcify::ThreeMat;
    /// use calcify::ThreeVec;
    ///
    /// let mat3 = ThreeMat::new(ThreeVec::new(1.0,2.0,3.0),
    ///                             ThreeVec::new(4.0,5.0,6.0),
    ///                             ThreeVec::new(7.0,8.0,9.0));
    ///
    /// assert_eq!(
    ///     mat3*mat3,
    ///     ThreeMat::new(ThreeVec::new(30.0,36.0,42.0),
    ///                 ThreeVec::new(66.0,81.0,96.0),
    ///                 ThreeVec::new(102.0,126.0,150.0)));
    /// ```
    fn mul(self, other: ThreeMat) -> ThreeMat {
        let c0 = other.c0();
        let c1 = other.c1();
        let c2 = other.c2();
        ThreeMat {
            r0: ThreeVec::new(self.r0*c0, self.r0*c1, self.r0*c2),
            r1: ThreeVec::new(self.r1*c0, self.r1*c1, self.r1*c2),
            r2: ThreeVec::new(self.r2*c0, self.r2*c1, self.r2*c2),
        }
    }
}

impl Mul<ThreeVec> for ThreeMat {
    type Output = ThreeVec;
    /// Matrix multiplication with vector
    ///
    /// # Note
    ///
    /// Only works in one direction ThreeMat*ThreeVec, implying ThreeVec as a column vector.
    ///
    /// # Example
    ///
    /// ```
    /// use calcify::ThreeMat;
    /// use calcify::ThreeVec;
    ///
    /// let mat3 = ThreeMat::new(ThreeVec::new(1.0,2.0,3.0),
    ///                             ThreeVec::new(1.0,2.0,3.0),
    ///                             ThreeVec::new(1.0,2.0,3.0));
    ///
    /// assert_eq!(
    ///     mat3*ThreeVec::new(2.0,2.0,2.0),
    ///     ThreeVec::new(12.0,12.0,12.0)
    /// );
    /// ```
    fn mul(self, other: ThreeVec) -> ThreeVec {
        ThreeVec::new(self.r0*other,self.r1*other,self.r2*other)
    }
}

impl Neg for ThreeMat {
    type Output = ThreeMat;

    fn neg(self) -> ThreeMat {
        ThreeMat {
            r0: -self.r0,
            r1: -self.r1,
            r2: -self.r2,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access() {
        let _test_mat = ThreeMat::new(ThreeVec::new(1.0,2.0,3.0),
                                    ThreeVec::new(4.0,5.0,6.0),
                                    ThreeVec::new(7.0,8.0,9.0));
        assert_eq!(*_test_mat.r2().x2(),9.0);
        assert_eq!(_test_mat.c2(),ThreeVec::new(3.0,6.0,9.0));
        assert_eq!(*_test_mat.r2().x2(),9.0);
    }

    #[test]
    fn test_add() {
        let _test_mat1 = ThreeMat::new(ThreeVec::new(1.0,2.0,3.0),
                                    ThreeVec::new(4.0,5.0,6.0),
                                    ThreeVec::new(7.0,8.0,9.0));
        let _test_mat2 = ThreeMat::new(ThreeVec::new(1.0,2.0,3.0),
                                    ThreeVec::new(4.0,5.0,6.0),
                                    ThreeVec::new(7.0,8.0,9.0));

        assert_eq!(
            _test_mat1+_test_mat2,
            ThreeMat::new(ThreeVec::new(2.0,4.0,6.0),
                        ThreeVec::new(8.0,10.0,12.0),
                        ThreeVec::new(14.0,16.0,18.0))
        );
        assert_eq!(*_test_mat1.r2().x2(),9.0);
    }

    #[test]
    fn test_loop_add() {
        let mut _test_mat1 = ThreeMat::new(ThreeVec::new(1.0,1.0,1.0),
                                    ThreeVec::new(1.0,1.0,1.0),
                                    ThreeVec::new(1.0,1.0,1.0));
        for _i in 0..9999{
            _test_mat1 += ThreeMat::new(ThreeVec::new(1.0,1.0,1.0),
                                        ThreeVec::new(1.0,1.0,1.0),
                                        ThreeVec::new(1.0,1.0,1.0));
        }

        assert_eq!(
            _test_mat1,
            ThreeMat::new(ThreeVec::new(10_000.0,10_000.0,10_000.0),
                        ThreeVec::new(10_000.0,10_000.0,10_000.0),
                        ThreeVec::new(10_000.0,10_000.0,10_000.0))
        );
    }

    #[test]
    fn test_sub() {
        let _test_mat1 = ThreeMat::new(ThreeVec::new(2.0,4.0,6.0),
                                    ThreeVec::new(8.0,10.0,12.0),
                                    ThreeVec::new(14.0,16.0,18.0));
        let _test_mat2 = ThreeMat::new(ThreeVec::new(1.0,2.0,3.0),
                                    ThreeVec::new(4.0,5.0,6.0),
                                    ThreeVec::new(7.0,8.0,9.0));

        assert_eq!(
            _test_mat1-_test_mat2,
            ThreeMat::new(ThreeVec::new(1.0,2.0,3.0),
                        ThreeVec::new(4.0,5.0,6.0),
                        ThreeVec::new(7.0,8.0,9.0))
        );
        assert_eq!(*_test_mat1.r2().x2(),18.0);
    }

    #[test]
    fn test_loop_sub() {
        let mut _test_mat1 = ThreeMat::new(ThreeVec::new(10_000.0,10_000.0,10_000.0),
                    ThreeVec::new(10_000.0,10_000.0,10_000.0),
                    ThreeVec::new(10_000.0,10_000.0,10_000.0));
        for _i in 0..9999{
            _test_mat1 -= ThreeMat::new(ThreeVec::new(1.0,1.0,1.0),
                                        ThreeVec::new(1.0,1.0,1.0),
                                        ThreeVec::new(1.0,1.0,1.0));
        }

        assert_eq!(
            _test_mat1,
            ThreeMat::new(ThreeVec::new(1.0,1.0,1.0),
                                        ThreeVec::new(1.0,1.0,1.0),
                                        ThreeVec::new(1.0,1.0,1.0))
        );
    }

    #[test]
    fn test_mul() {
        let _test_mat = ThreeMat::new(ThreeVec::new(1.0,2.0,3.0),
                                    ThreeVec::new(4.0,5.0,6.0),
                                    ThreeVec::new(7.0,8.0,9.0));

        assert_eq!(
            _test_mat*_test_mat,
            ThreeMat::new(ThreeVec::new(30.0,36.0,42.0),
                        ThreeVec::new(66.0,81.0,96.0),
                        ThreeVec::new(102.0,126.0,150.0))
        );
    }

    #[test]
    fn test_mul_vec() {
        let _test_mat = ThreeMat::new(ThreeVec::new(1.0,2.0,3.0),
                                    ThreeVec::new(1.0,2.0,3.0),
                                    ThreeVec::new(1.0,2.0,3.0));

        assert_eq!(
            _test_mat*ThreeVec::new(2.0,2.0,2.0),
            ThreeVec::new(12.0,12.0,12.0)
        );
    }

    #[test]
    fn test_mul_coef() {
        let _test_mat = ThreeMat::new(ThreeVec::new(1.0,1.0,1.0),
                                    ThreeVec::new(1.0,1.0,1.0),
                                    ThreeVec::new(1.0,1.0,1.0));

        assert_eq!(
            _test_mat*2.0,
            ThreeMat::new(ThreeVec::new(2.0,2.0,2.0),
                        ThreeVec::new(2.0,2.0,2.0),
                        ThreeVec::new(2.0,2.0,2.0))
        );
        assert_eq!(
            2.0*_test_mat,
            ThreeMat::new(ThreeVec::new(2.0,2.0,2.0),
                        ThreeVec::new(2.0,2.0,2.0),
                        ThreeVec::new(2.0,2.0,2.0))
        );
    }

    #[test]
    fn test_neg() {
        let _test_mat = ThreeMat::new(ThreeVec::new(1.0,1.0,1.0),
                                    ThreeVec::new(1.0,1.0,1.0),
                                    ThreeVec::new(1.0,1.0,1.0));

        assert_eq!(
            -_test_mat,
            ThreeMat::new(ThreeVec::new(-1.0,-1.0,-1.0),
                        ThreeVec::new(-1.0,-1.0,-1.0),
                        ThreeVec::new(-1.0,-1.0,-1.0))
        );
    }

    #[test]
    fn test_copy() {
        let xx = ThreeMat::new(ThreeVec::new(1.0,1.0,1.0),
                                    ThreeVec::new(1.0,1.0,1.0),
                                    ThreeVec::new(1.0,1.0,1.0));
        let yy = xx;
        assert_eq!(
            xx+yy,
            ThreeMat::new(ThreeVec::new(2.0,2.0,2.0),
                        ThreeVec::new(2.0,2.0,2.0),
                        ThreeVec::new(2.0,2.0,2.0))
        );
    }

    #[test]
    fn test_parse() {
        let xx = ThreeMat::new(ThreeVec::new(1.0,1.0,1.0),
                                    ThreeVec::new(1.0,1.0,1.0),
                                    ThreeVec::new(1.0,1.0,1.0));
        let pp = xx.to_json();
        assert_eq!(ThreeMat::from_str(&pp).unwrap(),xx);
    }
}
