use std::ops::AddAssign;
use std::iter::FromIterator;
use std::str::FromStr;
use std::num::ParseFloatError;

mod fitting;

pub use fitting::gaussian;
pub use fitting::Fit;

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

/// A plot is a Collection of Points
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Returns new Point
    ///
    /// # Arguments
    ///
    /// * `x` - f64 Independent Variable
    /// * `y` - f64 Dependent Variable
    ///
    pub fn new(x: f64, y: f64) -> Point {
        Point {
            x,
            y,
        }
    }
}

impl Serializable for Point {
    fn to_json(&self) -> String {
        format!("{{\"x\":{},\"y\":{}}}", self.x, self.y)
    }
}

impl FromStr for Point {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut x: f64 = std::f64::NAN;
        let mut y: f64 = std::f64::NAN;
        for dim in s.trim_matches(|p| p == '{' || p == '}' ).split(',') {
            let n_v: Vec<&str> = dim.split(':').collect();
            match n_v[0] {
                "\"x\"" => x = n_v[1].parse::<f64>()?,
                "\"y\"" => y = n_v[1].parse::<f64>()?,
                x => panic!("Unexpected invalid token {:?}", x),
            }
        }
        Ok(Point{x,y})
    }
}


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
    /// ```
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
    /// ```
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
    /// ```
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
    /// ```
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
    /// ```
    pub fn map<F,Z: Serializable>(&self, close: F) -> Collection<Z> where
        F: FnMut(&T) -> Z{
            Collection::from_vec(self.vec.iter().map(close).collect())
    }

    /// Cuts/Filters a function and returns a new Collection<T>
    ///
    /// Implements Vec::iter::filter and Vec::iter::collect.
    ///
    /// # Note
    ///
    /// * This may behave differently than expected. Cut keeps the elements that *pass* the test, not fail it. 
    ///
    /// # Arguments
    ///
    /// * `close` - F: FnMut(&&T: Serializable) -> bool
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
    /// col4V.cut(|&&x| x.s() < 10.0);
    /// ```
    pub fn cut<F>(&self, close: F) -> Collection<T> where
	F: FnMut(&&T) -> bool, T: Clone {
             let new_vec: Vec<T> = self.vec.iter().filter(close).cloned().collect();
             Collection::from_vec(new_vec)
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

impl Collection<Point> {
    /// Return Collection<Point> plot
    ///
    /// # Arguments
    ///
    /// * `ind` - Independent variable Vec<64>
    /// * `dep` - Dependent variable Vec<64>
    ///
    /// # Example
    /// ```
    /// use calcify::Collection;
    /// use calcify::Point;
    ///
    /// let test_plot: Collection<Point> = Collection::plot(vec![0.0,1.0,2.0],vec![3.0,4.0,5.0]);
    /// ```
    pub fn plot(ind: Vec<f64>, dep: Vec<f64>) -> Collection<Point> {
        let mut out: Collection<Point> = Collection::empty();
        for (x , y) in ind.iter().zip(dep.iter()) {
            out.push(Point::new(*x,*y));
        }
        out
    }

    /// Return Fit of the Collection
    ///
    /// # Arguments
    ///
    /// * `func` - &'static Fn(f64, Vec<f64>) -> f64
    ///
    /// # Example
    /// ```
    /// use calcify::Collection;
    /// use calcify::Point;
    /// use calcify::gaussian;
    ///
    /// let test_plot: Collection<Point> = Collection::plot(vec![0.0,1.0,2.0],vec![3.0,4.0,5.0]);
    /// let test_fit = test_plot.fit(&gaussian);
    /// ```
    pub fn fit(&self, func: &'static Fn(f64, Vec<f64>) -> f64) -> Fit {
        let mut ind: Vec<f64> = vec![];
        let mut dep: Vec<f64> = vec![];
        for pp in self.vec.iter() {
            ind.push(pp.x);
            dep.push(pp.y);
        }
        Fit::new(ind,dep,func)
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
        let width = (st_vec[st_vec.len()-1] + 0.01 - st_vec[0])/(num_bins as f64);
        let mut out: Collection<Bin> = Collection::empty();
        for i in 0..(num_bins) {
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
                col_3v.push(ThreeVec::random(10000.0));
            }
        let len_col: Collection<f64> = col_3v.map(|tv| { *tv.x0()});
        wr.write(len_col.hist(50).to_json().as_bytes()).unwrap();
    }

    #[test]
    fn test_plot() {
        let f = File::create("test_plot.json").unwrap();
        let mut wr = BufWriter::new(f);
        let test_plot: Collection<Point> = Collection::plot(vec![0.0,1.0,2.0],vec![3.0,4.0,5.0]);
        wr.write(test_plot.to_json().as_bytes()).unwrap();
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
