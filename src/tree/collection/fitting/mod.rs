use std::f64::NAN;
use std::f64::consts::E;

/// Gaussian function
pub fn gaussian(x: f64, co: Vec<f64>) -> f64 {
    if co.len() != 3 {panic!("Argument, co, for Gaussian must be of length 3.");}
    co[0]*E.powf(-(x - co[1]).powf(2.0)/(2.0*co[2]))
}


/// Fitter
pub struct Fit {
    ind: Vec<f64>,
    dep: Vec<f64>,
    func: &'static Fn(f64, Vec<f64>) -> f64,
    pub coef: Vec<f64>,
    pub conf: f64,
}

impl Fit {
    /// Create a new fit
    ///
    /// # Arguments
    ///
    /// * `ind` - Vec<f64>
    /// * `dep` - Vec<f64>
    /// * `func` - &'static Fn(f64, Vec<f64>) -> f64
    pub fn new(ind: Vec<f64>, dep: Vec<f64>, func: &'static Fn(f64, Vec<f64>) -> f64) -> Fit {
        Fit {
            ind,
            dep,
            func,
            coef: vec![NAN,NAN,NAN],
            conf: NAN,
        }
    }


}
