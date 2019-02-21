use std::ops::AddAssign;
use std::iter::FromIterator;
use std::str::FromStr;
use std::num::ParseFloatError;

mod four_mat;

pub use four_mat::Sinv;
pub use four_mat::beta;
pub use four_mat::gamma;
pub use four_mat::boost;
pub use four_mat::FourVec;
pub use four_mat::FourMat;

pub use four_mat::ThreeMat;
pub use four_mat::ThreeVec;
pub use four_mat::{radians_between, degrees_between};

pub use four_mat::consts;
pub use four_mat::Serializable;

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

/// A wrapper around the std::vec::vec
///
/// # Note
/// * Collection only implements some basic functionality of real Vecs.
/// The goal is not to supersede, but to add to.
/// So you should use Vec in most cases, and wrap it in a Collection if you need one of those functions.
#[derive(Debug, PartialEq, Clone)]
pub struct Collection<T: Serializable> {
    pub vec: Vec<T>,
}

impl<T: Serializable> Collection<T> {
    /// Returns new Collection from a Vec<T: Serializable>
    ///
    /// # Arguments
    ///
    /// * `vec` - Vec<calcify::FourVec>
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::Collection;
    ///
    /// let col4V = Collection::from_vec(
    ///     vec![FourVec::new(10.0,1.0,1.0,1.0)]
    /// );
    pub fn from_vec(vec: Vec<T>) -> Collection<T> {
        Collection {
            vec,
        }
    }

    /// Returns new Collection from a Vec<T: Serializable>
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::Collection;
    ///
    /// let col4V: Collection<FourVec> = Collection::empty();
    pub fn empty() -> Collection<T> {
        Collection {
            vec: Vec::<T>::new(),
        }
    }

    /// Returns a mutable reference to the T: Serializable at index i
    ///
    /// # Arguments
    ///
    /// * `i` - usize
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::Collection;
    ///
    /// let mut col4V = Collection::from_vec(
    ///     vec![FourVec::new(10.0,1.0,1.0,1.0)]
    /// );
    /// assert_eq!(*col4V.at(0),FourVec::new(10.0,1.0,1.0,1.0));
    /// *col4V.at(0) += FourVec::new(10.0,1.0,1.0,1.0);
    /// assert_eq!(*col4V.at(0),FourVec::new(20.0,2.0,2.0,2.0));
    pub fn at(&mut self, i: usize) -> &mut T {
        &mut self.vec[i]
    }

    /// Push new T: Serializable into Collection
    ///
    /// # Arguments
    ///
    /// * `nn` - T: Serializable
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::Collection;
    ///
    /// let mut col4V = Collection::empty();
    /// col4V.push(FourVec::new(10.0,1.0,1.0,1.0));
    /// assert_eq!(*col4V.at(0),FourVec::new(10.0,1.0,1.0,1.0));
    pub fn push(&mut self, nn: T) {
        self.vec.push(nn);
    }

    /// Maps a function and returns a new Collection<T>
    ///
    /// Implements Vec::iter::map and Vec::iter::collect.
    ///
    /// # Arguments
    ///
    /// * `close` - F: FnMut(&T: Serializable) -> Z: Serializable
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::Collection;
    ///
    /// let mut col4V: Collection<FourVec> = Collection::empty();
    /// for _i in 0..9999 {
    ///     col4V.push(FourVec::new(1.0,0.0,0.0,0.0));
    /// }
    /// let mut mass_col4V: Collection<f64> = Collection::empty();
    /// for _i in 0..9999 {
    ///     mass_col4V.push(1.0);
    /// }
    /// assert_eq!(col4V.map(FourVec::s), mass_col4V);
    pub fn map<F,Z: Serializable>(&self, close: F) -> Collection<Z> where
        F: FnMut(&T) -> Z{
        Collection {
            vec: self.vec.iter().map(close).collect(),
        }
    }
}

impl<T: Serializable> Serializable for Collection<T> {
    fn to_json(&self) -> String {
        let str_vec: Vec<String> = self.vec.iter().map(|x| x.to_json()).collect();
        format!("[{}]",str_vec.join(","))
    }
}

impl<T: Serializable> FromIterator<T> for Collection<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let mut c = Collection::empty();
        for i in iter {
            c.push(i);
        }
        c
    }
}

impl Collection<f64> {
    /// Return Collection<Bin> histogram
    ///
    /// # Example
    /// ```
    /// use calcify::Collection;
    /// use calcify::Bin;
    /// use calcify::ThreeVec;
    ///
    /// let mut col_3v = Collection::empty();
    ///     for _i in 0..99999 {
    ///         col_3v.push(ThreeVec::random(10.0));
    ///     }
    /// let len_col: Collection<f64> = col_3v.map(ThreeVec::r);
    /// let histogram: Collection<Bin> = len_col.hist(50);
    /// ```
    pub fn hist(&self, num_bins: u64) -> Collection<Bin> {
        let mut st_vec = self.vec.clone();
        st_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
        if num_bins < 1 {panic!("num_bins must be 0 or greater.");}
        let width = (st_vec[st_vec.len()-1] - st_vec[0])/(num_bins as f64);
        let mut out: Collection<Bin> = Collection::empty();
        for i in 0..(num_bins+1) {
            let edg0 = st_vec[0] + width * (i as f64);
            let edg1 = st_vec[0] + width * ((i+1) as f64);
            out.push(Bin::new(edg0,edg1,0));
        }
        let mut c_bin = 0;
        for x in st_vec.iter() {
            if x >= &out.at(c_bin).in_edge && x < &out.at(c_bin).ex_edge {
                *out.at(c_bin) += 1;
            }
            else {
                c_bin += 1;
                *out.at(c_bin) += 1;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use std::io::BufWriter;
    use std::fs::File;
    use super::*;

    #[test]
    fn test_map() {
        let mut col_3v = Collection::empty();
            for _i in 0..99999 {
                col_3v.push(ThreeVec::random(10.0));
            }
        let _len_col: Collection<f64> = col_3v.map(ThreeVec::r);
    }

    #[test]
    fn test_hist() {
        let f = File::create("test_hist.json").unwrap();
        let mut wr = BufWriter::new(f);
        let mut col_3v = Collection::empty();
            for _i in 0..99999 {
                col_3v.push(ThreeVec::random(10.0));
            }
        let len_col: Collection<f64> = col_3v.map(ThreeVec::r);
        wr.write(len_col.hist(50).to_json().as_bytes()).unwrap();
    }

    #[test]
    fn test_json() {
        let f = File::create("test_out.json").unwrap();
        let mut wr = BufWriter::new(f);
        let mut col_3v = Collection::empty();
        for _i in 0..9999 {
            col_3v.push(ThreeVec::random(10.0));
        }
        wr.write(col_3v.map(ThreeVec::r).to_json().as_bytes()).unwrap();
    }
}