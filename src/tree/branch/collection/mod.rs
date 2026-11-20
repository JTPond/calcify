use std::convert::From;
use std::error;
use std::iter::Extend;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice::SliceIndex;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

mod bin;
mod point;
mod point_bin;
mod stamped_sample;

pub use bin::Bin;
pub use point::Point;
pub use point_bin::PointBin;
pub use stamped_sample::StampedSample;

use crate::utils;

use utils::errors::CalcifyError;
use utils::serializable::json_array_elements;
use utils::{Deserializable, Serializable};

use rmp::decode::*;
use rmp::encode::*;

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

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Returns an immutable reference to the element or subslice at `index`, or `None` if out of bounds.
    pub fn get<I: SliceIndex<[T]>>(&self, index: I) -> Option<&I::Output> {
        self.vec.get(index)
    }

    /// Returns a mutable reference to the element or subslice at `index`, or `None` if out of bounds.
    pub fn get_mut<I: SliceIndex<[T]>>(&mut self, index: I) -> Option<&mut I::Output> {
        self.vec.get_mut(index)
    }

    /// Returns a shared slice of the underlying elements.
    pub fn as_slice(&self) -> &[T] {
        &self.vec
    }

    /// Returns a mutable slice of the underlying elements.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.vec
    }

    /// Returns an iterator over shared references to elements.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.vec.iter()
    }

    /// Returns an iterator over mutable references to elements.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.vec.iter_mut()
    }
}

impl<T: Serializable> Deref for Collection<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.vec
    }
}

impl<T: Serializable> DerefMut for Collection<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.vec
    }
}

impl<T: Serializable> AsRef<[T]> for Collection<T> {
    fn as_ref(&self) -> &[T] {
        &self.vec
    }
}

impl<T: Serializable, I: SliceIndex<[T]>> Index<I> for Collection<T> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.vec[index]
    }
}

impl<T: Serializable, I: SliceIndex<[T]>> IndexMut<I> for Collection<T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.vec[index]
    }
}

impl<'a, T: Serializable> IntoIterator for &'a Collection<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter()
    }
}

impl<'a, T: Serializable> IntoIterator for &'a mut Collection<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter_mut()
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

impl<T: Serializable + Serialize> Serialize for Collection<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.vec.serialize(serializer)
    }
}

impl<'de, T: Serializable + Deserialize<'de>> Deserialize<'de> for Collection<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<T>::deserialize(deserializer).map(Collection::from)
    }
}

impl<T: Serializable> Serializable for Collection<T> {
    fn to_json(&self) -> String {
        let str_vec: Vec<String> = self.vec.iter().map(|x| x.to_json()).collect();
        format!("[{}]", str_vec.join(","))
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
        for elem in json_array_elements(s)? {
            out.push(T::from_json(elem)?);
        }
        Ok(out)
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        let mut out: Self = Collection::empty();
        if let Ok(len) = read_array_len(&mut bytes) {
            for _ in 0..len {
                if let Ok((ot, rest)) = T::from_msg(bytes) {
                    out.push(ot);
                    bytes = rest;
                } else {
                    return Err(Box::new(CalcifyError::ParseError(String::from(
                        "failed to decode Collection element from MessagePack",
                    ))));
                }
            }
            return Ok((out, bytes));
        }
        Err(Box::new(CalcifyError::ParseError(String::from(
            "invalid MessagePack Collection array",
        ))))
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

fn expand_degenerate_range(min: f64, max: f64) -> (f64, f64) {
    if min < max {
        return (min, max);
    }
    let half = (min.abs() * 1e-6).max(0.5);
    let mut lo = min - half;
    let mut hi = max + half;
    if !(lo < hi && lo.is_finite() && hi.is_finite()) {
        if min > 0.0 {
            lo = min * (1.0 - 1e-6);
            hi = max;
        } else {
            lo = min;
            hi = max * (1.0 - 1e-6);
        }
    }
    (lo, hi)
}

fn bin_edges(min: f64, max: f64, num_bins: u64) -> (f64, Vec<(f64, f64)>) {
    let bins = num_bins as usize;
    let n_f64 = num_bins as f64;
    let span = max - min;
    let width = if span.is_finite() {
        span / n_f64
    } else {
        (max / n_f64) - (min / n_f64)
    };
    let step_hi = max / n_f64;
    let step_lo = min / n_f64;
    let edge_at = |idx: usize| -> f64 {
        if idx == 0 {
            min
        } else if idx == bins {
            max
        } else if span.is_finite() {
            min + width * (idx as f64)
        } else {
            (min - step_lo * (idx as f64)) + step_hi * (idx as f64)
        }
    };
    let mut edges = Vec::with_capacity(bins);
    for i in 0..bins {
        edges.push((edge_at(i), edge_at(i + 1)));
    }
    (width, edges)
}

fn compute_bin_index(x: f64, min: f64, max: f64, width: f64, bins: usize) -> usize {
    if x >= max {
        return bins - 1;
    }
    if x <= min {
        return 0;
    }
    let raw = if (x - min).is_finite() {
        ((x - min) / width).floor()
    } else {
        ((x / width) - (min / width)).floor()
    };
    if raw.is_nan() || raw <= 0.0 {
        0
    } else if !raw.is_finite() {
        bins - 1
    } else {
        let idx = raw as usize;
        if idx >= bins {
            bins - 1
        } else {
            idx
        }
    }
}

fn welford_stats(slice: &[f64]) -> Option<(f64, f64)> {
    if slice.is_empty() {
        return None;
    }
    let mut mean = 0.0_f64;
    let mut m2 = 0.0_f64;
    for (i, &x) in slice.iter().enumerate() {
        if !x.is_finite() {
            return None;
        }
        let n = (i + 1) as f64;
        let delta = x - mean;
        mean += delta / n;
        let delta2 = x - mean;
        m2 += delta * delta2;
    }
    Some((mean, m2))
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
        for (x, y) in ind.iter().zip(dep.iter()) {
            out.push(Point::new(*x, *y));
        }
        out
    }

    /// Return Collection<PointBin> 2D histogram over explicit `x_range` and `y_range` bounds.
    ///
    /// Points with non-finite coordinates or coordinates outside `[min, max]` are ignored.
    ///
    /// # Panics
    ///
    /// * If `num_bins_x < 1` or `num_bins_y < 1`
    /// * If either range is non-finite or `min >= max`
    pub fn hist_with_range(
        &self,
        num_bins_x: u64,
        num_bins_y: u64,
        x_range: (f64, f64),
        y_range: (f64, f64),
    ) -> Collection<PointBin> {
        assert!(
            num_bins_x >= 1 && num_bins_y >= 1,
            "num_bins must be 1 or greater."
        );
        let (min_x, max_x) = x_range;
        let (min_y, max_y) = y_range;
        assert!(
            min_x.is_finite() && max_x.is_finite() && min_x < max_x,
            "x_range must be finite with min < max."
        );
        assert!(
            min_y.is_finite() && max_y.is_finite() && min_y < max_y,
            "y_range must be finite with min < max."
        );

        let bins_x = num_bins_x as usize;
        let bins_y = num_bins_y as usize;
        let (width_x, x_edges) = bin_edges(min_x, max_x, num_bins_x);
        let (width_y, y_edges) = bin_edges(min_y, max_y, num_bins_y);

        let mut outs = Vec::with_capacity(bins_x * bins_y);
        for &(edg0x, edg1x) in &x_edges {
            for &(edg0y, edg1y) in &y_edges {
                outs.push(PointBin::new(edg0x, edg1x, edg0y, edg1y, 0));
            }
        }

        for p in &self.vec {
            if !p.x.is_finite()
                || !p.y.is_finite()
                || p.x < min_x
                || p.x > max_x
                || p.y < min_y
                || p.y > max_y
            {
                continue;
            }
            let mut ix = compute_bin_index(p.x, min_x, max_x, width_x, bins_x);
            while ix + 1 < bins_x && p.x >= x_edges[ix].1 {
                ix += 1;
            }
            while ix > 0 && p.x < x_edges[ix].0 {
                ix -= 1;
            }

            let mut iy = compute_bin_index(p.y, min_y, max_y, width_y, bins_y);
            while iy + 1 < bins_y && p.y >= y_edges[iy].1 {
                iy += 1;
            }
            while iy > 0 && p.y < y_edges[iy].0 {
                iy -= 1;
            }

            outs[ix * bins_y + iy] += 1;
        }

        Collection::from(outs)
    }

    /// Return Collection<PointBin> 2D histogram.
    ///
    /// Returns an empty `Collection` if the collection has no points with both `x` and `y` finite.
    ///
    /// # Arguments
    ///
    /// * `num_bins_x` - Number of bins along the x axis: u64 (>= 1)
    /// * `num_bins_y` - Number of bins along the y axis: u64 (>= 1)
    ///
    /// # Panics
    ///
    /// * If either `num_bins` is less than 1
    pub fn hist(&self, num_bins_x: u64, num_bins_y: u64) -> Collection<PointBin> {
        assert!(
            num_bins_x >= 1 && num_bins_y >= 1,
            "num_bins must be 1 or greater."
        );
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        let mut has_finite = false;

        for p in &self.vec {
            if p.x.is_finite() && p.y.is_finite() {
                has_finite = true;
                if p.x < min_x {
                    min_x = p.x;
                }
                if p.x > max_x {
                    max_x = p.x;
                }
                if p.y < min_y {
                    min_y = p.y;
                }
                if p.y > max_y {
                    max_y = p.y;
                }
            }
        }

        if !has_finite {
            return Collection::empty();
        }

        let x_range = expand_degenerate_range(min_x, max_x);
        let y_range = expand_degenerate_range(min_y, max_y);
        self.hist_with_range(num_bins_x, num_bins_y, x_range, y_range)
    }
}

impl Collection<f64> {
    /// Return Collection<Bin> histogram over an explicit `[min, max]` range.
    ///
    /// Non-finite values and values outside `[min, max]` are ignored.
    ///
    /// # Panics
    ///
    /// * If `num_bins < 1`
    /// * If `min` or `max` is non-finite or `min >= max`
    pub fn hist_with_range(&self, num_bins: u64, min: f64, max: f64) -> Collection<Bin> {
        assert!(num_bins >= 1, "num_bins must be 1 or greater.");
        assert!(
            min.is_finite() && max.is_finite() && min < max,
            "range must be finite with min < max."
        );

        let bins = num_bins as usize;
        let (width, edges) = bin_edges(min, max, num_bins);
        let mut out_vec: Vec<Bin> = edges
            .into_iter()
            .map(|(edg0, edg1)| Bin::new(edg0, edg1, 0))
            .collect();

        for &x in &self.vec {
            if !x.is_finite() || x < min || x > max {
                continue;
            }
            let mut idx = compute_bin_index(x, min, max, width, bins);
            while idx + 1 < bins && x >= out_vec[idx].ex_edge {
                idx += 1;
            }
            while idx > 0 && x < out_vec[idx].in_edge {
                idx -= 1;
            }
            out_vec[idx] += 1;
        }

        Collection::from(out_vec)
    }

    /// Return Collection<Bin> histogram.
    ///
    /// Returns an empty `Collection` if the collection has no finite values.
    ///
    /// # Arguments
    ///
    /// * `num_bins` - Number of bins: u64 (>= 1)
    ///
    /// # Panics
    ///
    /// * If `num_bins` is less than 1
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
        assert!(num_bins >= 1, "num_bins must be 1 or greater.");
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut has_finite = false;

        for &x in &self.vec {
            if x.is_finite() {
                has_finite = true;
                if x < min {
                    min = x;
                }
                if x > max {
                    max = x;
                }
            }
        }

        if !has_finite {
            return Collection::empty();
        }

        let (lo, hi) = expand_degenerate_range(min, max);
        self.hist_with_range(num_bins, lo, hi)
    }

    /// Returns the sum of all elements (`0.0` if empty).
    pub fn sum(&self) -> f64 {
        self.vec.iter().copied().sum()
    }

    /// Returns the minimum element, or `None` if empty or any element is `NaN`.
    pub fn min(&self) -> Option<f64> {
        let mut iter = self.vec.iter().copied();
        let mut best = iter.next()?;
        if best.is_nan() {
            return None;
        }
        for x in iter {
            if x.is_nan() {
                return None;
            }
            if x.total_cmp(&best).is_lt() {
                best = x;
            }
        }
        Some(best)
    }

    /// Returns the maximum element, or `None` if empty or any element is `NaN`.
    pub fn max(&self) -> Option<f64> {
        let mut iter = self.vec.iter().copied();
        let mut best = iter.next()?;
        if best.is_nan() {
            return None;
        }
        for x in iter {
            if x.is_nan() {
                return None;
            }
            if x.total_cmp(&best).is_gt() {
                best = x;
            }
        }
        Some(best)
    }

    /// Returns the arithmetic mean, or `None` if empty or any element is non-finite.
    pub fn mean(&self) -> Option<f64> {
        welford_stats(&self.vec).map(|(mean, _)| mean)
    }

    /// Returns the population variance (`M2 / N`), or `None` if empty or any element is non-finite.
    pub fn variance(&self) -> Option<f64> {
        welford_stats(&self.vec).map(|(_, m2)| m2 / (self.vec.len() as f64))
    }

    /// Returns the sample variance (`M2 / (N - 1)`), or `None` if `len < 2` or any element is non-finite.
    pub fn sample_variance(&self) -> Option<f64> {
        if self.vec.len() < 2 {
            return None;
        }
        welford_stats(&self.vec).map(|(_, m2)| m2 / ((self.vec.len() - 1) as f64))
    }

    /// Returns the population standard deviation, or `None` if empty or any element is non-finite.
    pub fn std_dev(&self) -> Option<f64> {
        self.variance().map(f64::sqrt)
    }

    /// Returns the sample standard deviation, or `None` if `len < 2` or any element is non-finite.
    pub fn sample_std_dev(&self) -> Option<f64> {
        self.sample_variance().map(f64::sqrt)
    }

    /// Returns the `q`-th quantile in `[0.0, 1.0]` using linear interpolation,
    /// or `None` if empty, `q` is outside `[0.0, 1.0]`, or any element is `NaN`.
    pub fn quantile(&self, q: f64) -> Option<f64> {
        if self.vec.is_empty() || !q.is_finite() || !(0.0..=1.0).contains(&q) {
            return None;
        }
        if self.vec.iter().any(|x| x.is_nan()) {
            return None;
        }
        let mut sorted = self.vec.clone();
        sorted.sort_by(f64::total_cmp);
        let n = sorted.len();
        if n == 1 {
            return Some(sorted[0]);
        }
        let pos = q * ((n - 1) as f64);
        let lo = pos.floor() as usize;
        let hi = pos.ceil() as usize;
        let frac = pos - (lo as f64);
        if lo == hi || frac == 0.0 {
            Some(sorted[lo])
        } else if sorted[lo].is_finite() && sorted[hi].is_finite() {
            Some(sorted[lo] + frac * (sorted[hi] - sorted[lo]))
        } else {
            Some(sorted[lo] * (1.0 - frac) + sorted[hi] * frac)
        }
    }

    /// Returns the median (`quantile(0.5)`), or `None` if empty or any element is `NaN`.
    pub fn median(&self) -> Option<f64> {
        self.quantile(0.5)
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

    /// Computes a 1D `Collection<Bin>` histogram over the stream's values using `[min, max]`.
    pub fn hist_with_range(&self, num_bins: u64, min: f64, max: f64) -> Collection<Bin> {
        self.values().hist_with_range(num_bins, min, max)
    }
}

impl Collection<StampedSample<Point>> {
    /// Computes a 2D `Collection<PointBin>` histogram over the stream's `Point` values.
    pub fn hist(&self, num_bins_x: u64, num_bins_y: u64) -> Collection<PointBin> {
        self.values().hist(num_bins_x, num_bins_y)
    }

    /// Computes a 2D `Collection<PointBin>` histogram over the stream's `Point` values using explicit axis ranges.
    pub fn hist_with_range(
        &self,
        num_bins_x: u64,
        num_bins_y: u64,
        x_range: (f64, f64),
        y_range: (f64, f64),
    ) -> Collection<PointBin> {
        self.values()
            .hist_with_range(num_bins_x, num_bins_y, x_range, y_range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ThreeMat, ThreeVec};
    use std::fs::{self, File};
    use std::io::prelude::*;
    use std::io::BufWriter;

    fn sample_3vecs(count: usize, scale: f64) -> Collection<ThreeVec> {
        let mut col = Collection::empty();
        for i in 0..count {
            let t = (i as f64) / (count.max(1) as f64);
            let v = (t * 2.0 - 1.0) * scale;
            col.push(ThreeVec::new(v, v * 0.5, -v * 0.25));
        }
        col
    }

    #[test]
    fn test_map() {
        let col_3v = sample_3vecs(1000, 10.0);
        let len_col: Collection<f64> = col_3v.map(ThreeVec::r);
        assert_eq!(len_col.len(), 1000);
    }

    #[test]
    fn test_hist() {
        fs::create_dir_all("./scratch").unwrap();
        let f = File::create("./scratch/test_hist.json").unwrap();
        let mut wr = BufWriter::new(f);
        let col_3v = sample_3vecs(1000, 10000.0);
        let len_col: Collection<f64> = col_3v.map(|tv| *tv.x0());
        let h = len_col.hist(50);
        assert_eq!(h.len(), 50);
        assert_eq!(h.iter().map(|b| b.count).sum::<u64>(), 1000);
        wr.write_all(h.to_json().as_bytes()).unwrap();
        wr.flush().unwrap();
    }

    #[test]
    fn test_plot() {
        fs::create_dir_all("./scratch").unwrap();
        let f = File::create("./scratch/test_plot.json").unwrap();
        let mut wr = BufWriter::new(f);
        let test_plot: Collection<Point> = Collection::plot(&[0.0, 1.0, 2.0], &[3.0, 4.0, 5.0]);
        wr.write_all(test_plot.to_json().as_bytes()).unwrap();
        wr.flush().unwrap();
    }

    #[test]
    fn test_json() {
        fs::create_dir_all("./scratch").unwrap();
        let f = File::create("./scratch/test_out.json").unwrap();
        let mut wr = BufWriter::new(f);
        let col_3v = sample_3vecs(1000, 10.0);
        wr.write_all(col_3v.map(ThreeVec::r).to_json().as_bytes())
            .unwrap();
        wr.flush().unwrap();
    }

    #[test]
    fn test_collection_slice_indexing_and_serde() {
        let mut col = Collection::from(vec![1.0, 2.0, 3.0]);
        assert_eq!(col.get(1), Some(&2.0));
        assert_eq!(col.get(10), None);
        assert_eq!(col.as_slice(), &[1.0, 2.0, 3.0]);
        assert_eq!(col[0], 1.0);
        assert_eq!(&col[1..3], &[2.0, 3.0]);
        assert_eq!(col.first(), Some(&1.0)); // via Deref<Target = [T]>

        col[1] = 20.0;
        *col.get_mut(2).unwrap() = 30.0;
        let sum: f64 = (&col).into_iter().copied().sum();
        assert_eq!(sum, 51.0);
        for x in &mut col {
            *x += 1.0;
        }
        assert_eq!(col.as_ref(), &[2.0, 21.0, 31.0]);

        let pts = Collection::plot(&[1.0, 2.0], &[3.0, 4.0]);
        let json = serde_json::to_string(&pts).unwrap();
        let rt: Collection<Point> = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, pts);
    }

    #[test]
    fn test_hist_1d_edge_cases_and_fixed_range() {
        // Gapped data: [0.0, 0.0, 0.0, 100.0] over 10 bins
        let gapped = Collection::from(vec![0.0, 0.0, 0.0, 100.0]);
        let hb = gapped.hist(10);
        assert_eq!(hb.len(), 10);
        assert_eq!(hb[0].in_edge, 0.0);
        assert_eq!(hb[9].ex_edge, 100.0);
        assert_eq!(hb[0].count, 3);
        for i in 1..9 {
            assert_eq!(hb[i].count, 0);
        }
        assert_eq!(hb[9].count, 1);

        // Empty and all-non-finite
        assert!(Collection::<f64>::empty().hist(5).is_empty());
        assert!(Collection::from(vec![f64::NAN, f64::INFINITY, f64::NEG_INFINITY])
            .hist(5)
            .is_empty());

        // Mixed NaN / Inf + finite
        let mixed = Collection::from(vec![f64::NAN, 0.0, f64::INFINITY, 10.0, f64::NEG_INFINITY]);
        let hm = mixed.hist(2);
        assert_eq!(hm.len(), 2);
        assert_eq!(hm[0].count, 1);
        assert_eq!(hm[1].count, 1);

        // Constant values
        for val in [0.0, 5.0, 1e20] {
            let c = Collection::from(vec![val, val, val]);
            let hc = c.hist(4);
            assert_eq!(hc.len(), 4);
            assert_eq!(hc.iter().map(|b| b.count).sum::<u64>(), 3);
            assert!(hc[0].in_edge < val && val < hc[3].ex_edge);
        }

        // Small range (no +0.01 overhang)
        let small = Collection::from(vec![0.0, 0.0005, 0.001]);
        let hs = small.hist(2);
        assert_eq!(hs[0].in_edge, 0.0);
        assert_eq!(hs[1].ex_edge, 0.001);
        assert_eq!(hs.iter().map(|b| b.count).sum::<u64>(), 3);

        // Huge range (no OOB panic or precision loss)
        let huge = Collection::from(vec![0.0, 5e19, 1e20]);
        let hh = huge.hist(5);
        assert_eq!(hh.len(), 5);
        assert_eq!(hh.iter().map(|b| b.count).sum::<u64>(), 3);
        assert_eq!(hh[4].ex_edge, 1e20);

        // Fixed range: excludes out-of-range, keeps inclusive endpoints, handles empty
        let data = Collection::from(vec![-5.0, 0.0, 2.5, 10.0, 20.0, f64::NAN]);
        let hr = data.hist_with_range(4, 0.0, 10.0);
        assert_eq!(hr.len(), 4);
        assert_eq!(hr[0].in_edge, 0.0);
        assert_eq!(hr[3].ex_edge, 10.0);
        assert_eq!(hr.iter().map(|b| b.count).sum::<u64>(), 3);
        assert_eq!(hr[0].count, 1);
        assert_eq!(hr[1].count, 1);
        assert_eq!(hr[3].count, 1);

        let empty_hr = Collection::<f64>::empty().hist_with_range(3, 0.0, 3.0);
        assert_eq!(empty_hr.len(), 3);
        assert_eq!(empty_hr.iter().map(|b| b.count).sum::<u64>(), 0);
        assert_eq!(empty_hr[0].in_edge, 0.0);
        assert_eq!(empty_hr[2].ex_edge, 3.0);
    }

    #[test]
    fn test_hist_2d_edge_cases_and_fixed_range() {
        assert!(Collection::<Point>::empty().hist(2, 2).is_empty());
        assert!(Collection::from(vec![
            Point::new(f64::NAN, 1.0),
            Point::new(1.0, f64::INFINITY),
        ])
        .hist(2, 2)
        .is_empty());

        let pts = Collection::from(vec![
            Point::new(0.0, 0.0),
            Point::new(1e20, 1e20),
            Point::new(f64::NAN, 0.0),
        ]);
        let h2 = pts.hist(3, 4);
        assert_eq!(h2.len(), 12);
        assert_eq!(h2.iter().map(|b| b.count).sum::<u64>(), 2);
        assert_eq!(h2[0].count, 1);
        assert_eq!(h2[11].count, 1);

        // Constant points
        let cpts = Collection::from(vec![Point::new(2.0, -3.0), Point::new(2.0, -3.0)]);
        let hc2 = cpts.hist(2, 2);
        assert_eq!(hc2.len(), 4);
        assert_eq!(hc2.iter().map(|b| b.count).sum::<u64>(), 2);

        // Fixed range 2D
        let grid_pts = Collection::from(vec![
            Point::new(-1.0, 0.5), // out of x_range
            Point::new(0.0, 0.0),
            Point::new(10.0, 10.0),
        ]);
        let hr2 = grid_pts.hist_with_range(2, 2, (0.0, 10.0), (0.0, 10.0));
        assert_eq!(hr2.len(), 4);
        assert_eq!(hr2[0].count, 1); // [0..5, 0..5]
        assert_eq!(hr2[3].count, 1); // [5..10, 5..10]
        assert_eq!(hr2.iter().map(|b| b.count).sum::<u64>(), 2);

        let empty_hr2 =
            Collection::<Point>::empty().hist_with_range(2, 3, (0.0, 2.0), (0.0, 3.0));
        assert_eq!(empty_hr2.len(), 6);
        assert_eq!(empty_hr2.iter().map(|b| b.count).sum::<u64>(), 0);
    }

    #[test]
    fn test_descriptive_stats() {
        let empty = Collection::<f64>::empty();
        assert_eq!(empty.sum(), 0.0);
        assert_eq!(empty.min(), None);
        assert_eq!(empty.max(), None);
        assert_eq!(empty.mean(), None);
        assert_eq!(empty.variance(), None);
        assert_eq!(empty.sample_variance(), None);
        assert_eq!(empty.std_dev(), None);
        assert_eq!(empty.sample_std_dev(), None);
        assert_eq!(empty.quantile(0.5), None);
        assert_eq!(empty.median(), None);

        let single = Collection::from(vec![4.5]);
        assert_eq!(single.sum(), 4.5);
        assert_eq!(single.min(), Some(4.5));
        assert_eq!(single.max(), Some(4.5));
        assert_eq!(single.mean(), Some(4.5));
        assert_eq!(single.variance(), Some(0.0));
        assert_eq!(single.std_dev(), Some(0.0));
        assert_eq!(single.sample_variance(), None);
        assert_eq!(single.sample_std_dev(), None);
        assert_eq!(single.quantile(0.0), Some(4.5));
        assert_eq!(single.quantile(1.0), Some(4.5));
        assert_eq!(single.median(), Some(4.5));

        let data = Collection::from(vec![10.0, 2.0, 4.0, 6.0, 8.0]);
        assert_eq!(data.sum(), 30.0);
        assert_eq!(data.min(), Some(2.0));
        assert_eq!(data.max(), Some(10.0));
        assert!((data.mean().unwrap() - 6.0).abs() < 1e-12);
        assert!((data.variance().unwrap() - 8.0).abs() < 1e-12);
        assert!((data.sample_variance().unwrap() - 10.0).abs() < 1e-12);
        assert!((data.std_dev().unwrap() - 8.0_f64.sqrt()).abs() < 1e-12);
        assert!((data.sample_std_dev().unwrap() - 10.0_f64.sqrt()).abs() < 1e-12);
        assert_eq!(data.quantile(0.0), Some(2.0));
        assert_eq!(data.quantile(0.25), Some(4.0));
        assert_eq!(data.median(), Some(6.0));
        assert_eq!(data.quantile(0.75), Some(8.0));
        assert_eq!(data.quantile(1.0), Some(10.0));
        assert!((data.quantile(0.125).unwrap() - 3.0).abs() < 1e-12);
        assert_eq!(data.quantile(-0.1), None);
        assert_eq!(data.quantile(1.1), None);
        assert_eq!(data.quantile(f64::NAN), None);

        // Numerical stability check (Welford's algorithm)
        let shifted = Collection::from(vec![1e9 + 1.0, 1e9 + 2.0, 1e9 + 3.0]);
        assert!((shifted.sample_variance().unwrap() - 1.0).abs() < 1e-12);
        assert!((shifted.variance().unwrap() - (2.0 / 3.0)).abs() < 1e-12);

        // Non-finite handling
        let has_nan = Collection::from(vec![1.0, f64::NAN, 3.0]);
        assert_eq!(has_nan.min(), None);
        assert_eq!(has_nan.max(), None);
        assert_eq!(has_nan.mean(), None);
        assert_eq!(has_nan.variance(), None);
        assert_eq!(has_nan.quantile(0.5), None);

        let has_inf = Collection::from(vec![1.0, f64::INFINITY, 3.0]);
        assert_eq!(has_inf.min(), Some(1.0));
        assert_eq!(has_inf.max(), Some(f64::INFINITY));
        assert_eq!(has_inf.mean(), None);
        assert_eq!(has_inf.variance(), None);
        assert_eq!(has_inf.median(), Some(3.0));
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
