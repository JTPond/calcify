use std::ops::AddAssign;
use std::error;
use std::u64;
use std::f64;

extern crate rmp;
use rmp::encode::*;
use rmp::decode::*;

use crate::utils;

use utils::{Serializable, Deserializable};
use utils::errors::CalcifyError;

/// A histogram is a Collection of PointBins
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct PointBin {
    pub in_edge_x: f64,
    pub ex_edge_x: f64,
    pub in_edge_y: f64,
    pub ex_edge_y: f64,
    pub count: u64,
}

impl PointBin {
    /// Returns new PointBin
    ///
    /// # Arguments
    ///
    /// * `in_edge_x` - f64 Inclusive Edge along the X axis
    /// * `ex_edge_x` - f64 Exclusive Edge along the X axis
    /// * `in_edge_y` - f64 Inclusive Edge along the Y axis
    /// * `ex_edge_y` - f64 Exclusive Edge along the Y axis
    /// * `count` - u64 PointBin value
    ///
    pub fn new(in_edge_x: f64, ex_edge_x: f64, in_edge_y: f64, ex_edge_y: f64, count: u64) -> PointBin {
        PointBin {
            in_edge_x,
            ex_edge_x,
            in_edge_y,
            ex_edge_y,
            count,
        }
    }
}

impl AddAssign<u64> for PointBin {
    /// Increment PointBin count.
    ///
    /// # Example
    /// ```
    /// use calcify::PointBin;
    /// let mut test_PointBin = PointBin::new(0.0,1.0,0.0,1.0,10);
    /// test_PointBin += 1;
    ///
    /// assert_eq!(test_PointBin.count, 11);
    /// ```
    fn add_assign(&mut self, other: u64) {
        self.count += other;
    }
}

impl Serializable for PointBin {
    fn to_json(&self) -> String {
        format!("{{\"count\":{},\"range\":[{},{},{},{}]}}",self.count,self.in_edge_x,self.ex_edge_x,self.in_edge_y,self.ex_edge_y)
    }

    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::with_capacity(7);
        write_array_len(&mut buf, 2)?;
        write_uint(&mut buf, self.count)?;
        write_array_len(&mut buf, 4)?;
        write_f64(&mut buf, self.in_edge_x)?;
        write_f64(&mut buf, self.ex_edge_x)?;
        write_f64(&mut buf, self.in_edge_y)?;
        write_f64(&mut buf, self.ex_edge_y)?;
        Ok(buf)
    }
}

impl Deserializable for PointBin {

    fn from_json(s: &str) -> Result<Self, Box<dyn error::Error>> {
        let mut count: u64 = 0;
        let mut in_edge_x: f64 = f64::NAN;
        let mut ex_edge_x: f64 = f64::NAN;
        let mut in_edge_y: f64 = f64::NAN;
        let mut ex_edge_y: f64 = f64::NAN;
        for (i,dim) in s.replace(":",",").replace("[",",").replace("]",",").trim_matches(|p| p == '{' || p == '}' ).split_terminator(",").enumerate() {
            match i {
                0 => (),
                1 => count = dim.parse::<f64>()? as u64,
                2 => (),
                3 => (),
                4 => in_edge_x = dim.parse::<f64>()?,
                5 => ex_edge_x = dim.parse::<f64>()?,
                6 => in_edge_y = dim.parse::<f64>()?,
                7 => ex_edge_y = dim.parse::<f64>()?,
                _ => return Err(Box::new(CalcifyError::ParseError)),
            }
        }
        Ok(PointBin{count,in_edge_x,ex_edge_x,in_edge_y,ex_edge_y})
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self,&[u8]), Box<dyn error::Error>> {
        if let Ok(2) = read_array_len(&mut bytes){
            let count: u64 = read_int(&mut bytes)?;
            if let Ok(4) = read_array_len(&mut bytes){
                let in_edge_x: f64 = read_f64(&mut bytes)?;
                let ex_edge_x: f64 = read_f64(&mut bytes)?;
                let in_edge_y: f64 = read_f64(&mut bytes)?;
                let ex_edge_y: f64 = read_f64(&mut bytes)?;
                return Ok((PointBin{count,in_edge_x,ex_edge_x,in_edge_y,ex_edge_y},bytes));
            }
        }
        Err(Box::new(CalcifyError::ParseError))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let xx = PointBin::new(0.0,1.0,0.0,1.0,0);
        let pp = xx.to_json();
        assert_eq!(PointBin::from_json(&pp).unwrap(),xx);
    }

    #[test]
    fn test_msg_parse() {
        let xx = PointBin::new(0.0,1.0,0.0,1.0,0);
        let pp = xx.to_msg().unwrap();
        let (oo,_) = PointBin::from_msg(&pp).unwrap();
        assert_eq!(oo,xx);
    }
}
