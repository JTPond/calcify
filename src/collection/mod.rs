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

/// A wrapper around the std::vec::vec
///
/// # Note
/// * Collection only implements some basic functionality of real Vecs.
/// The goal is not to supersede, but to add to.
/// So you should use Vec in most cases, and wrap it in a Collection if you need one of those functions.
#[derive(Debug, PartialEq, Clone)]
pub struct Collection<T: Serializable> {
    vec: Vec<T>,
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
}

impl<T: Serializable> Serializable for Collection<T> {
    fn to_json(&self) -> String {
        let str_vec: Vec<String> = self.vec.iter().map(|x| x.to_json()).collect();
        format!("[{}]",str_vec.join(","))
    }
}

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use std::io::BufWriter;
    use std::fs::File;
    use super::*;

    #[test]
    fn test_json() {
        let f = File::create("test_out.json").unwrap();
        let mut wr = BufWriter::new(f);
        let mut col4V = Collection::empty();
        for _i in 0..9999 {
            col4V.push(ThreeVec::random(10.0));
        }
        wr.write(col4V.to_json().as_bytes()).unwrap();
    }
}
