use std::str::FromStr;
use std::num::ParseFloatError;
use std::ops::AddAssign;

extern crate rmp;
use rmp::encode::*;

use crate::utils;

use utils::Serializable;

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
    fn to_jsonc(&self) -> String {
        format!("[{},[{},{}]]", self.count, self.in_edge, self.ex_edge)
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

impl FromStr for Bin {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut count: u64 = 0;
        let mut in_edge: f64 = 0.0;
        let mut ex_edge: f64 = 0.0;
        let mut counter = 0;
        for i in s.replace(":",",").replace("[",",").replace("]",",").trim_matches(|p| p == '{' || p == '}' ).split_terminator(",") {
            match counter {
                1 => count = i.parse::<f64>()? as u64,
                4 => in_edge = i.parse::<f64>()?,
                5 => ex_edge = i.parse::<f64>()?,
                _ => (),
            }
            counter += 1;
        }

        Ok(Bin{count,in_edge,ex_edge})
    }
}
