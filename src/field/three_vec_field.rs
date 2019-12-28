use std::f64;
use std::ops::Mul;
use std::ops::Neg;

use crate::three_mat;
use three_mat::ThreeVec;

/// Three dimensional vector field
#[derive(Clone,Copy)]
pub struct ThreeVecField<'a> {
    func: &'a dyn Fn(&f64, &f64, &f64) -> ThreeVec,
    multi: f64,
}

impl<'a> ThreeVecField<'a> {
    /// Returns a new ThreeVecField
    ///
    /// # Arguments
    ///
    /// * `func` - &'a dyn Fn(&f64, &f64, &f64) -> calcify::ThreeVec
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeVec;
    /// use calcify::ThreeVecField;
    /// 
    /// let g = ThreeVecField::new(&|i: &f64, j: &f64, k: &f64| ThreeVec::new(i*i,j*j,k*k));
    /// let g2: ThreeVecField = 2.0*g;
    ///
    /// assert_eq!(g.loc(2.0,2.0,2.0),ThreeVec::new(4.0,4.0,4.0));
    /// assert_eq!(g.at(ThreeVec::new(2.0,2.0,2.0)),ThreeVec::new(4.0,4.0,4.0));
    /// assert_eq!(g2.loc(2.0,2.0,2.0),ThreeVec::new(8.0,8.0,8.0));
    /// ```
    pub fn new(func: &'a dyn Fn(&f64, &f64, &f64) -> ThreeVec) -> ThreeVecField {
        ThreeVecField {
            func,
            multi: 1.0,
        }
    }

    /// Return value of field from three floats.
    pub fn loc(&self, i: f64, j:f64, k:f64) -> ThreeVec {
        self.multi*(self.func)(&i,&j,&k)
    }

    /// Return value of field from a ThreeVec
    pub fn at(&self, vec: ThreeVec) -> ThreeVec {
        self.multi*(self.func)(vec.x0(),vec.x1(),vec.x2())
    }
}

impl<'a> Mul<f64> for ThreeVecField<'a> {
    type Output = ThreeVecField<'a>;

    /// Scale a ThreeVecField by float.
    fn mul(self, coef: f64) -> ThreeVecField<'a> {
        ThreeVecField {
            func: self.func,
            multi: coef*self.multi,
        }
    }
}

impl<'a> Mul<ThreeVecField<'a>> for f64 {
    type Output = ThreeVecField<'a>;

    /// Scale a ThreeVecField by float from the other side.
    fn mul(self, other: ThreeVecField<'a>) -> ThreeVecField<'a> {
        ThreeVecField {
            func: other.func,
            multi: self*other.multi,
        }
    }
}

impl<'a> Neg for ThreeVecField<'a> {
    type Output = ThreeVecField<'a>;

    /// Negate a ThreeVecField.
    fn neg(self) -> ThreeVecField<'a> {
        ThreeVecField {
            func: self.func,
            multi: -self.multi,
        }
    }
}
