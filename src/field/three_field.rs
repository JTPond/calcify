use std::f64;
use std::ops::Mul;
use std::ops::Neg;

use crate::three_mat;
use three_mat::ThreeVec;

/// Three dimensional scalar field
#[derive(Clone,Copy)]
pub struct ThreeField<'a> {
    func: &'a dyn Fn(&f64, &f64, &f64) -> f64,
    multi: f64,
}

impl<'a> ThreeField<'a> {
    /// Returns a new ThreeField
    ///
    /// # Arguments
    ///
    /// * `func` - &'a dyn Fn(&f64, &f64, &f64) -> f64
    ///
    /// # Example
    /// ```
    /// use calcify::ThreeVec;
    /// use calcify::ThreeField;
    /// 
    /// let g = ThreeField::new(&|i: &f64, j: &f64, k: &f64| i*i+j*j+k*k);
    /// let g2: ThreeField = 2.0*g;
    ///
    /// assert_eq!(g.loc(2.0,2.0,2.0),12.0);
    /// assert_eq!(g.at(ThreeVec::new(2.0,2.0,2.0)),12.0);
    /// assert_eq!(g2.loc(2.0,2.0,2.0),24.0);
    /// ```
    pub fn new(func: &'a dyn Fn(&f64, &f64, &f64) -> f64) -> ThreeField {
        ThreeField {
            func,
            multi: 1.0,
        }
    }
    
    /// Return value of field from three floats.
    pub fn loc(&self, i: f64, j:f64, k:f64) -> f64 {
        self.multi*(self.func)(&i,&j,&k)
    }

    /// Return value of field at vector. 
    pub fn at(&self, vec: ThreeVec) -> f64 {
        self.multi*(self.func)(vec.x0(),vec.x1(),vec.x2())
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
