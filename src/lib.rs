#![crate_name = "calcify"]
//! # Calcify
//!
//!  A crate for 3-D and 4-D vector and matrix algebra, conceived for use in physics simulations. Builds out from a basic ThreeVec struct including most commonly used operations built in.
//!  Includes physics constants, 3 and 4-D vectors and matrices and many associated operations, collections, histograms, and output trees, which are serialized in json.
//!
//! ## Notes
//! * Branches can now be extracted from Trees, but this is not an ideal process. Trees should still be seen as containers for output only.
//!
//! * All physics constants are exported in the top in SI units. To retrieve them in Planck or natural units call calcify::Consts::planck() or calcify::Consts::natural().
//!
//! * FourMat::lambda() has been replaced by fn boost(initial: FourVec, v: ThreeVec). The math has been changed.
//!
//! ## todo
//! * Plotting tool
//! * to_bytes
//! * Four-Momentum

mod tree;

pub use tree::Branch;
pub use tree::Tree;
pub use tree::Collection;
pub use tree::Bin;

pub use tree::Sinv;
pub use tree::beta;
pub use tree::gamma;
pub use tree::boost;
pub use tree::FourVec;
pub use tree::FourMat;

pub use tree::ThreeMat;
pub use tree::ThreeVec;
pub use tree::{radians_between, degrees_between};

pub use tree::consts;
pub use tree::Serializable;
