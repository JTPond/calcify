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

/// collection
#[derive(Debug, PartialEq, Clone)]
pub struct Collection<T> {
    vec: Vec<T>,
}

impl Collection<FourVec> {
    /// Returns new Collection from a Vec<FourVec>
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
    pub fn from_vec(vec: Vec<FourVec>) -> Collection<FourVec> {
        Collection {
            vec,
        }
    }

    /// Returns new Collection from a Vec<FourVec>
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::Collection;
    ///
    /// let col4V = Collection::empty();
    pub fn empty() -> Collection<FourVec> {
        Collection {
            vec: Vec::<FourVec>::new(),
        }
    }

    /// Returns a mutable reference to the FourVec at index i
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
    pub fn at(&mut self, i: usize) -> &mut FourVec {
        &mut self.vec[i]
    }

    /// Push new FourVec into Collection
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::Collection;
    ///
    /// let mut col4V = Collection::empty();
    /// col4V.push(FourVec::new(10.0,1.0,1.0,1.0));
    /// assert_eq!(*col4V.at(0),FourVec::new(10.0,1.0,1.0,1.0));
    pub fn push(&mut self, nn: FourVec) {
        self.vec.push(nn);
    }
}
