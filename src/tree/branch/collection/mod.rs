use std::iter::FromIterator;
use std::iter::Extend;
use std::error;
use std::convert::From;

mod point;
mod bin;
mod point_bin;
mod stamped_sample;

pub use point::Point;
pub use bin::Bin;
pub use point_bin::PointBin;
pub use stamped_sample::StampedSample;

use crate::utils;

use utils::{Serializable, Deserializable};
use utils::errors::CalcifyError;
use utils::serializable::split_json_array;

use rmp::encode::*;
use rmp::decode::*;

/// A wrapper around the std::vec::vec
///
/// # Note
/// * Collection only implements some basic functionality of real Vecs.
///   The goal is not to supersede, but to add to.
///   So you should use Vec in most cases, and wrap it in a Collection if you need one of those functions.
#[derive(Debug, PartialEq, Clone)]
pub struct Collection<T: Serializable> {
    pub vec: Vec<T>,
}

impl<T: Serializable> Collection<T> {
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
    /// let mut col4V = Collection::from(
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
        F: FnMut(&T) -> Z {
            let vbuff: Vec<Z> = self.vec.iter().map(close).collect();
            Collection::from(vbuff)
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
             Collection::from(new_vec)
    }

    pub fn len(&self) -> usize{
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

}

impl<T: Serializable + Clone> From<&[T]> for Collection<T> {
    /// Returns new Collection from a &[T: Serializable]
    ///
    /// # Arguments
    ///
    /// * `vec` - &[T: Serializable]
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::Collection;
    ///
    /// let a4v = [FourVec::new(10.0,1.0,1.0,1.0)];
    /// let col4V = Collection::from(
    ///     &a4v[..]
    /// );
    /// ```
    fn from(vec: &[T]) -> Self {
        let ivec: Vec<T> = vec.to_vec();
        Collection {
            vec: ivec,
        }
    }
}

impl<T: Serializable> From<Vec<T>> for Collection<T> {
    /// Returns new Collection from a &[T: Serializable]
    ///
    /// # Arguments
    ///
    /// * `vec` - Vec<T: Serializable>
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::Collection;
    ///
    /// let col4V = Collection::from(
    ///     vec![FourVec::new(10.0,1.0,1.0,1.0)]
    /// );
    /// ```
    fn from(vec: Vec<T>) -> Self {
        Collection {
            vec,
        }
    }
}

impl<T: Serializable> Serializable for Collection<T> {
    fn to_json(&self) -> String {
        let str_vec: Vec<String> = self.vec.iter().map(|x| x.to_json()).collect();
        format!("[{}]",str_vec.join(","))
    }

    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::new();
        write_array_len(&mut buf, (self.vec.len()) as u32)?;
        for x in self.vec.iter() {
            buf.append(&mut x.to_msg()?);
        }
        Ok(buf)
    }
}

impl<T: Serializable + Deserializable> Deserializable for Collection<T> {
    fn from_json(s: &str) -> Result<Self, Box<dyn error::Error>> {
        let mut out: Self = Collection::empty();
        for elem in split_json_array(s)? {
            out.push(T::from_json(elem)?);
        }
        Ok(out)
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self,&[u8]), Box<dyn error::Error>> {
        let mut out: Self = Collection::empty();
        if let Ok(len) = read_array_len(&mut bytes){
            for _ in 0..len {
                if let Ok((ot,rest)) = T::from_msg(bytes) {
                    out.push(ot);
                    bytes = rest;
                } else {
                    return Err(Box::new(CalcifyError::ParseError));
                }
            }
            return Ok((out,bytes));
        }
        Err(Box::new(CalcifyError::ParseError))
    }
}


/// Collects an iterator into a Collection, i.e. provides collect().
///
/// # Example
/// ```
/// use calcify::FourVec;
/// use calcify::Collection;
///
/// let mut col4V: Collection<FourVec> = Collection::empty();
/// let mut colf6: Collection<f64> = Collection::empty();
/// for _i in 0..9999 {
///     col4V.push(FourVec::new(1.0,0.0,0.0,0.0));
///     colf6.push(1.0);
/// }
///
/// let tCol: Collection<f64> = col4V.into_iter().map(|x| x.s()).collect();
///
/// assert_eq!(colf6, tCol);
/// ```
impl<T: Serializable> FromIterator<T> for Collection<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let mut c = Collection::empty();
        for i in iter {
            c.push(i);
        }
        c
    }
}

/// Returns the internal Vec iterator
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
///
/// assert_eq!(FourVec::new(9999.0,0.0,0.0,0.0),
///             col4V.into_iter().fold(FourVec::new(0.0,0.0,0.0,0.0), |acc, x| acc + x));
/// ```
impl<T: Serializable> IntoIterator for Collection<T> {
    type Item = T;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

/// Extends a collection by the elements in the provided Iter
///
/// # Example
/// ```
/// use calcify::FourVec;
/// use calcify::Collection;
///
/// let mut col4V_1: Collection<FourVec> = Collection::empty();
/// let mut col4V_2: Collection<FourVec> = Collection::empty();
/// for _i in 0..5000 {
///     col4V_1.push(FourVec::new(1.0,0.0,0.0,0.0));
///     col4V_2.push(FourVec::new(1.0,0.0,0.0,0.0));
/// }
///
/// col4V_1.extend(col4V_2.into_iter());
/// assert_eq!(col4V_1.len(),10_000);
/// ```
impl<T: Serializable> Extend<T> for Collection<T> {

    fn extend<U: IntoIterator<Item=T>>(& mut self, iter: U) {
        self.vec.extend(iter);
    }
}

impl Collection<Point> {
    /// Return Collection<Point> plot
    ///
    /// # Arguments
    ///
    /// * `ind` - Independent variable: &[prim@f64]
    /// * `dep` - Dependent variable: &[prim@f64]
    ///
    /// # Example
    /// ```
    /// use calcify::Collection;
    /// use calcify::Point;
    ///
    /// let test_plot: Collection<Point> = Collection::plot(&vec![0.0,1.0,2.0],&vec![3.0,4.0,5.0]);
    /// ```
    pub fn plot(ind: &[f64], dep: &[f64]) -> Collection<Point> {
        let mut out: Collection<Point> = Collection::empty();
        for (x , y) in ind.iter().zip(dep.iter()) {
            out.push(Point::new(*x,*y));
        }
        out
    }

    /// Return Collection<PointBin> 2D histogram
    ///
    /// # Arguments
    ///
    /// * `num_bins_x` - Number of bins along the x axis: u64 (>= 2)
    /// * `num_bins_y` - Number of bins along the y axis: u64 (>= 2)
    ///
    /// # Panics
    ///
    /// * If either num_bins is less than 2
    ///
    pub fn hist(&self, num_bins_x: u64, num_bins_y: u64) -> Collection<PointBin> {
        if num_bins_x < 2 || num_bins_y < 2 {panic!("num_bins must be 2 or greater.");}
        let mut min_x = self.vec[0].x;
        let mut max_x = self.vec[0].x;
        let mut min_y = self.vec[0].y;
        let mut max_y = self.vec[0].y;
        for b in self.vec.iter(){
            if b.x > max_x {max_x = b.x;}
            if b.x < min_x {min_x = b.x;}
            if b.y > max_y {max_y = b.y;}
            if b.y < min_y {min_y = b.y;}
        }
        let width_x = (max_x + 0.01 - min_x)/(num_bins_x as f64);
        let width_y = (max_y + 0.01 - min_y)/(num_bins_y as f64);
        let mut outs: Vec<Vec<PointBin>> = Vec::new();
        for i in 0..num_bins_x {
            outs.push(Vec::new());
            let edg0x = min_x + width_x * (i as f64);
            let edg1x = min_x + width_x * ((i+1) as f64);
            for j in 0..num_bins_y {
                let edg0y = min_y + width_y * (j as f64);
                let edg1y = min_y + width_y * ((j+1) as f64);
                outs[i as usize].push(PointBin::new(edg0x,edg1x,edg0y,edg1y,0));
            }
        }
        for p in self.vec.iter() {
            let x_bin: usize = ((p.x - min_x)/width_x) as usize;
            let y_bin: usize = ((p.y - min_y)/width_y) as usize;
            outs[x_bin][y_bin] += 1;
        }
        let o_vec: Vec<PointBin> = outs.iter().flatten().cloned().collect();
        let out: Collection<PointBin> = Collection::from(o_vec);
        out
    }
}

impl Collection<f64> {
    /// Return Collection<Bin> histogram
    ///
    /// # Arguments
    ///
    /// * `num_bins` - Number of bins: u64 (>= 2)
    ///
    /// # Panics
    ///
    /// * If num_bins is less than 2
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
        if num_bins < 2 {panic!("num_bins must be 2 or greater.");}
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

impl<T: Serializable> Collection<StampedSample<T>> {
    /// Returns a `Collection<StampedSample<T>>` stream by zipping `timestamps` and `values`.
    ///
    /// Pairs elements up to the length of the shorter slice, matching `Collection::plot`.
    ///
    /// # Example
    /// ```
    /// use calcify::{Collection, StampedSample};
    ///
    /// let stream = Collection::stream(&[10, 20, 30], &[1.0, 2.0, 3.0]);
    /// assert_eq!(stream.len(), 3);
    /// assert_eq!(stream.timestamps(), Collection::from(vec![10, 20, 30]));
    /// ```
    pub fn stream(timestamps: &[i64], values: &[T]) -> Collection<StampedSample<T>>
    where
        T: Clone,
    {
        timestamps
            .iter()
            .zip(values.iter())
            .map(|(t, v)| StampedSample::new(*t, v.clone()))
            .collect()
    }

    /// Appends a timestamped sample to the stream.
    ///
    /// # Example
    /// ```
    /// use calcify::{Collection, StampedSample};
    ///
    /// let mut stream: Collection<StampedSample<f64>> = Collection::empty();
    /// stream.push_sample(100, 1.5);
    /// assert_eq!(*stream.at(0), StampedSample::new(100, 1.5));
    /// ```
    pub fn push_sample(&mut self, timestamp: i64, value: T) {
        self.vec.push(StampedSample::new(timestamp, value));
    }

    /// Returns a `Collection<i64>` containing the timestamp of each sample in order.
    pub fn timestamps(&self) -> Collection<i64> {
        self.vec.iter().map(|s| s.timestamp).collect()
    }

    /// Returns a `Collection<T>` containing clones of each sample's value in order.
    pub fn values(&self) -> Collection<T>
    where
        T: Clone,
    {
        self.vec.iter().map(|s| s.value.clone()).collect()
    }

    /// Maps each sample's value while preserving its timestamp.
    ///
    /// # Example
    /// ```
    /// use calcify::{Collection, StampedSample, ThreeVec};
    ///
    /// let stream = Collection::stream(&[1, 2], &[ThreeVec::new(3.0, 4.0, 0.0), ThreeVec::new(0.0, 5.0, 12.0)]);
    /// let mags = stream.map_values(ThreeVec::r);
    /// assert_eq!(mags, Collection::stream(&[1, 2], &[5.0, 13.0]));
    /// ```
    pub fn map_values<F, Z: Serializable>(&self, mut f: F) -> Collection<StampedSample<Z>>
    where
        F: FnMut(&T) -> Z,
    {
        self.vec
            .iter()
            .map(|s| StampedSample::new(s.timestamp, f(&s.value)))
            .collect()
    }

    /// Returns a new stream containing samples whose timestamps fall in `[start, end]` inclusive.
    ///
    /// # Example
    /// ```
    /// use calcify::{Collection, StampedSample};
    ///
    /// let stream = Collection::stream(&[10, 20, 30, 40], &[1.0, 2.0, 3.0, 4.0]);
    /// let slice = stream.window(20, 30);
    /// assert_eq!(slice, Collection::stream(&[20, 30], &[2.0, 3.0]));
    /// ```
    pub fn window(&self, start: i64, end: i64) -> Collection<StampedSample<T>>
    where
        T: Clone,
    {
        self.vec
            .iter()
            .filter(|s| s.timestamp >= start && s.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Sorts the stream in place by `timestamp` using a stable sort.
    pub fn sort_by_timestamp(&mut self) {
        self.vec.sort_by_key(|s| s.timestamp);
    }

    /// Returns a new stream sorted by `timestamp` using a stable sort.
    pub fn sorted_by_timestamp(&self) -> Collection<StampedSample<T>>
    where
        T: Clone,
    {
        let mut out = self.clone();
        out.sort_by_timestamp();
        out
    }

    /// Returns `true` if sample timestamps are non-decreasing (`t[i] <= t[i+1]`).
    pub fn is_monotonic(&self) -> bool {
        self.vec.windows(2).all(|w| w[0].timestamp <= w[1].timestamp)
    }

    /// Returns `Some((min_timestamp, max_timestamp))` across all samples, or `None` if empty.
    pub fn time_bounds(&self) -> Option<(i64, i64)> {
        let first = self.vec.first()?.timestamp;
        let mut min_t = first;
        let mut max_t = first;
        for sample in &self.vec[1..] {
            if sample.timestamp < min_t {
                min_t = sample.timestamp;
            }
            if sample.timestamp > max_t {
                max_t = sample.timestamp;
            }
        }
        Some((min_t, max_t))
    }

    /// Returns a `Collection<i64>` of adjacent timestamp differences `t[i+1] - t[i]`.
    pub fn deltas(&self) -> Collection<i64> {
        self.vec
            .windows(2)
            .map(|w| w[1].timestamp - w[0].timestamp)
            .collect()
    }
}

impl Collection<StampedSample<f64>> {
    /// Converts the scalar stream into a `Collection<Point>` with `x = timestamp as f64`, `y = value`.
    ///
    /// # Example
    /// ```
    /// use calcify::{Collection, Point};
    ///
    /// let stream = Collection::stream(&[0, 1, 2], &[3.0, 4.0, 5.0]);
    /// assert_eq!(stream.to_points(), Collection::plot(&[0.0, 1.0, 2.0], &[3.0, 4.0, 5.0]));
    /// ```
    pub fn to_points(&self) -> Collection<Point> {
        self.vec
            .iter()
            .map(|s| Point::new(s.timestamp as f64, s.value))
            .collect()
    }

    /// Converts the scalar stream into a `Collection<Point>` with `x = (timestamp - origin) as f64`.
    ///
    /// Useful for preserving precision when timestamps are large epoch values.
    ///
    /// # Example
    /// ```
    /// use calcify::{Collection, Point};
    ///
    /// let stream = Collection::stream(&[1_700_000_000_000, 1_700_000_000_005], &[1.0, 2.0]);
    /// assert_eq!(
    ///     stream.to_points_since(1_700_000_000_000),
    ///     Collection::plot(&[0.0, 5.0], &[1.0, 2.0])
    /// );
    /// ```
    pub fn to_points_since(&self, origin: i64) -> Collection<Point> {
        self.vec
            .iter()
            .map(|s| Point::new((s.timestamp - origin) as f64, s.value))
            .collect()
    }

    /// Computes a 1D `Collection<Bin>` histogram over the stream's values.
    pub fn hist(&self, num_bins: u64) -> Collection<Bin> {
        self.values().hist(num_bins)
    }
}

impl Collection<StampedSample<Point>> {
    /// Computes a 2D `Collection<PointBin>` histogram over the stream's `Point` values.
    pub fn hist(&self, num_bins_x: u64, num_bins_y: u64) -> Collection<PointBin> {
        self.values().hist(num_bins_x, num_bins_y)
    }
}

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use std::io::BufWriter;
    use std::fs::File;
    use crate::{ThreeMat, ThreeVec};
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
        let f = File::create("./scratch/test_hist.json").unwrap();
        let mut wr = BufWriter::new(f);
        let mut col_3v = Collection::empty();
            for _i in 0..99999 {
                col_3v.push(ThreeVec::random(10000.0));
            }
        let len_col: Collection<f64> = col_3v.map(|tv| { *tv.x0()});
        wr.write_all(len_col.hist(50).to_json().as_bytes()).unwrap();
    }

    #[test]
    fn test_plot() {
        let f = File::create("./scratch/test_plot.json").unwrap();
        let mut wr = BufWriter::new(f);
        let test_plot: Collection<Point> = Collection::plot(&[0.0,1.0,2.0],&[3.0,4.0,5.0]);
        wr.write_all(test_plot.to_json().as_bytes()).unwrap();
    }

    #[test]
    fn test_json() {
        let f = File::create("./scratch/test_out.json").unwrap();
        let mut wr = BufWriter::new(f);
        let mut col_3v = Collection::empty();
        for _i in 0..9999 {
            col_3v.push(ThreeVec::random(10.0));
        }
        wr.write_all(col_3v.map(ThreeVec::r).to_json().as_bytes()).unwrap();
    }

    #[test]
    fn test_stream_helpers() {
        // Empty stream
        let empty: Collection<StampedSample<f64>> = Collection::empty();
        assert!(empty.is_monotonic());
        assert_eq!(empty.time_bounds(), None);
        assert_eq!(empty.deltas(), Collection::<i64>::empty());
        assert_eq!(empty.window(-10, 10), Collection::empty());
        assert_eq!(empty.timestamps(), Collection::<i64>::empty());
        assert_eq!(empty.values(), Collection::<f64>::empty());

        // Single-element stream
        let mut single: Collection<StampedSample<f64>> = Collection::empty();
        single.push_sample(-5, 42.0);
        assert!(single.is_monotonic());
        assert_eq!(single.time_bounds(), Some((-5, -5)));
        assert_eq!(single.deltas(), Collection::<i64>::empty());

        // Unsorted with equal and negative timestamps
        let s = Collection::from(vec![
            StampedSample::new(20, "b".to_string()),
            StampedSample::new(-10, "neg".to_string()),
            StampedSample::new(10, "first_10".to_string()),
            StampedSample::new(10, "second_10".to_string()),
            StampedSample::new(30, "c".to_string()),
        ]);
        assert!(!s.is_monotonic());
        assert_eq!(s.time_bounds(), Some((-10, 30)));
        assert_eq!(s.deltas(), Collection::from(vec![-30, 20, 0, 20]));
        assert_eq!(
            s.window(10, 20),
            Collection::from(vec![
                StampedSample::new(20, "b".to_string()),
                StampedSample::new(10, "first_10".to_string()),
                StampedSample::new(10, "second_10".to_string()),
            ])
        );

        let sorted = s.sorted_by_timestamp();
        assert!(sorted.is_monotonic());
        assert_eq!(
            sorted.timestamps(),
            Collection::from(vec![-10, 10, 10, 20, 30])
        );
        // Stable sort preserves order of equal timestamps
        assert_eq!(
            sorted.values(),
            Collection::from(vec![
                "neg".to_string(),
                "first_10".to_string(),
                "second_10".to_string(),
                "b".to_string(),
                "c".to_string(),
            ])
        );
        assert_eq!(sorted.deltas(), Collection::from(vec![20, 0, 10, 10]));

        let mut in_place = s.clone();
        in_place.sort_by_timestamp();
        assert_eq!(in_place, sorted);

        // Scalar and Point stream helpers
        let scalars = Collection::stream(&[1_700_000_000_000, 1_700_000_000_005, 1_700_000_000_010], &[1.0, 2.0, 3.0]);
        assert_eq!(
            scalars.to_points_since(1_700_000_000_000),
            Collection::plot(&[0.0, 5.0, 10.0], &[1.0, 2.0, 3.0])
        );
        assert_eq!(
            Collection::stream(&[1, 2], &[4.0, 5.0]).to_points(),
            Collection::plot(&[1.0, 2.0], &[4.0, 5.0])
        );
        let h1 = scalars.hist(2);
        assert_eq!(h1.vec.iter().map(|b| b.count).sum::<u64>(), 3);

        let pts = Collection::stream(
            &[1, 2, 3],
            &[Point::new(0.0, 0.0), Point::new(1.0, 1.0), Point::new(2.0, 2.0)],
        );
        let h2 = pts.hist(2, 2);
        assert_eq!(h2.vec.iter().map(|b| b.count).sum::<u64>(), 3);

        // map_values with Vec length mismatch on stream()
        let clipped = Collection::stream(&[1, 2, 3], &[10.0, 20.0]);
        assert_eq!(clipped.len(), 2);
        let scaled = clipped.map_values(|v| v * 2.0);
        assert_eq!(scaled, Collection::stream(&[1, 2], &[20.0, 40.0]));
    }

    #[test]
    fn test_collection_stamped_roundtrip() {
        let empty: Collection<StampedSample<f64>> = Collection::empty();
        let ej = empty.to_json();
        assert_eq!(ej, "[]");
        assert_eq!(Collection::<StampedSample<f64>>::from_json(&ej).unwrap(), empty);
        let em = empty.to_msg().unwrap();
        let (edm, erest) = Collection::<StampedSample<f64>>::from_msg(&em).unwrap();
        assert_eq!(edm, empty);
        assert!(erest.is_empty());

        let mats = Collection::from(vec![
            StampedSample::new(
                -5,
                ThreeMat::new(
                    ThreeVec::new(1.0, 2.0, 3.0),
                    ThreeVec::new(4.0, 5.0, 6.0),
                    ThreeVec::new(7.0, 8.0, 9.0),
                ),
            ),
            StampedSample::new(
                10,
                ThreeMat::new(
                    ThreeVec::new(9.0, 8.0, 7.0),
                    ThreeVec::new(6.0, 5.0, 4.0),
                    ThreeVec::new(3.0, 2.0, 1.0),
                ),
            ),
        ]);
        let mj = mats.to_json();
        assert_eq!(Collection::<StampedSample<ThreeMat>>::from_json(&mj).unwrap(), mats);
        let mm = mats.to_msg().unwrap();
        let (mdm, mrest) = Collection::<StampedSample<ThreeMat>>::from_msg(&mm).unwrap();
        assert_eq!(mdm, mats);
        assert!(mrest.is_empty());

        let strs = Collection::stream(
            &[1, 2],
            &[String::from("a,b,{c}"), String::from("d\"e\\f")],
        );
        let sj = strs.to_json();
        assert_eq!(Collection::<StampedSample<String>>::from_json(&sj).unwrap(), strs);
        let sm = strs.to_msg().unwrap();
        let (sdm, srest) = Collection::<StampedSample<String>>::from_msg(&sm).unwrap();
        assert_eq!(sdm, strs);
        assert!(srest.is_empty());
    }
}
