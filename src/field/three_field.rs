use std::f64;
use std::ops::Mul;
use std::ops::Neg;

use crate::three_mat;
use three_mat::ThreeVec;

/// Three dimensional scalar field
#[derive(Clone,Copy)]
pub struct ThreeField<'a> {
    func: &'a dyn Fn(&ThreeVec) -> f64,
    multi: f64,
}

impl<'a> ThreeField<'a> {
    /// Returns a new ThreeField
    ///
    /// # Arguments
    ///
    /// * `func` - &'a dyn Fn(&ThreeVec) -> f64
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeVec;
    /// use calcify::ThreeField;
    /// 
    /// let g = ThreeField::new(&|tv: &ThreeVec| tv.r());
    /// let g2: ThreeField = 2.0*g;
    ///
    /// let test_vec = ThreeVec::new(2.0,2.0,2.0);
    ///
    /// assert_eq!(g.loc(2.0,2.0,2.0),test_vec.r());
    /// assert_eq!(g.at(test_vec),test_vec.r());
    /// assert_eq!(g2.loc(2.0,2.0,2.0),2.0*test_vec.r());
    /// ```
    pub fn new(func: &'a dyn Fn(&ThreeVec) -> f64) -> ThreeField {
        ThreeField {
            func,
            multi: 1.0,
        }
    }
    
    /// Return value of field from three floats.
    pub fn loc(&self, i: f64, j:f64, k:f64) -> f64 {
        let buf_vec = ThreeVec::new(i,j,k);
        self.multi*(self.func)(&buf_vec)
    }

    /// Return value of field at vector. 
    pub fn at(&self, vec: ThreeVec) -> f64 {
        self.multi*(self.func)(&vec)
    }
}

impl<'a> Mul<f64> for ThreeField<'a> {
    type Output = ThreeField<'a>;
    
    /// Scale a ThreeField by float.
    fn mul(self, coef: f64) -> ThreeField<'a> {
        ThreeField {
            func: self.func,
            multi: coef*self.multi,
        }
    }
}

impl<'a> Mul<ThreeField<'a>> for f64 {
    type Output = ThreeField<'a>;
    
    /// Scale a ThreeField by a float from the other side.
    fn mul(self, other: ThreeField<'a>) -> ThreeField<'a> {
        ThreeField {
            func: other.func,
            multi: self*other.multi,
        }
    }
}

impl<'a> Neg for ThreeField<'a> {
    type Output = ThreeField<'a>;
    
    /// Negate a ThreeField.
    fn neg(self) -> ThreeField<'a> {
        ThreeField {
            func: self.func,
            multi: -self.multi,
        }
    }
}
