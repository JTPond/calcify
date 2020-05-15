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

/// A histogram is a Collection of Bins
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Bin {
    pub in_edge: f64,
    pub ex_edge: f64,
    pub count: u64,
}

impl Bin {
    /// Returns new Bin
    ///
    /// # Arguments
    ///
    /// * `in_edge` - f64 Inclusive Edge
    /// * `ex_edge` - f64 Exclusive Edge
    /// * `count` - u64 Bin value
    ///
    pub fn new(in_edge: f64, ex_edge: f64, count: u64) -> Bin {
        Bin {
            in_edge,
            ex_edge,
            count,
        }
    }
}

impl AddAssign<u64> for Bin {
    /// Increment Bin count.
    ///
    /// # Example
    /// ```
    /// use calcify::Bin;
    /// let mut test_bin = Bin::new(0.0,1.0,10);
    /// test_bin += 1;
    ///
    /// assert_eq!(test_bin.count, 11);
    /// ```
    fn add_assign(&mut self, other: u64) {
        self.count += other;
    }
}

impl Serializable for Bin {
    fn to_json(&self) -> String {
        format!("{{\"count\":{},\"range\":[{},{}]}}",self.count,self.in_edge,self.ex_edge)
    }

    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::with_capacity(5);
        write_array_len(&mut buf, 2)?;
        write_uint(&mut buf, self.count)?;
        write_array_len(&mut buf, 2)?;
        write_f64(&mut buf, self.in_edge)?;
        write_f64(&mut buf, self.ex_edge)?;
        Ok(buf)
    }
}

impl Deserializable for Bin {

    fn from_json(s: &str) -> Result<Self, Box<dyn error::Error>> {
        let mut count: u64 = 0;
        let mut in_edge: f64 = f64::NAN;
        let mut ex_edge: f64 = f64::NAN;
        for (i,dim) in s.replace(":",",").replace("[",",").replace("]",",").trim_matches(|p| p == '{' || p == '}' ).split_terminator(",").enumerate() {
            match i {
                0 => (),
                1 => count = dim.parse::<f64>()? as u64,
                2 => (),
                3 => (),
                4 => in_edge = dim.parse::<f64>()?,
                5 => ex_edge = dim.parse::<f64>()?,
                _ => return Err(Box::new(CalcifyError::ParseError)),
            }
        }
        Ok(Bin{count,in_edge,ex_edge})
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self,&[u8]), Box<dyn error::Error>> {
        if let Ok(2) = read_array_len(&mut bytes){
            let count: u64 = read_int(&mut bytes)?;
            if let Ok(2) = read_array_len(&mut bytes){
                let in_edge: f64 = read_f64(&mut bytes)?;
                let ex_edge: f64 = read_f64(&mut bytes)?;
                return Ok((Bin{count,in_edge,ex_edge},bytes));
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
        let xx = Bin::new(0.0,1.0,0);
        let pp = xx.to_json();
        assert_eq!(Bin::from_json(&pp).unwrap(),xx);
    }

    #[test]
    fn test_msg_parse() {
        let xx = Bin::new(0.0,1.0,0);
        let pp = xx.to_msg().unwrap();
        let (oo,_) = Bin::from_msg(&pp).unwrap();
        assert_eq!(oo,xx);
    }
}
