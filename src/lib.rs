#![crate_name = "calcify"]
//! # Calcify
//!
//!  A crate for 3-D and 4-D vector and matrix algebra, conceived for use in physics simulations. Builds out from a basic ThreeVec struct including most commonly used operations built in.
//!  Includes physics constants, 3 and 4-D vectors and matrices and many associated operations, collections, histograms, and output trees, which are serialized in json or MessagePack.
//!
//! ## CalcifyLab
//!
//! Beginning work on a new lib crate to make simulations (such as the example) more straightforward.
//! Look forward to that in the coming months.
//! 
//! ## ICalcify
//! 
//! Check it out [here!](https://github.com/JTPond/ICalcify "ICalcify GitHub")
//!

mod field;
mod tree;
mod four_mat;
mod three_mat;
mod utils;

pub use field::ThreeField;
pub use field::ThreeVecField;

pub use tree::Branch;
pub use tree::Tree;
pub use tree::Feed;
pub use tree::FeedTree;
pub use tree::Collection;
pub use tree::Bin;
pub use tree::Point;

pub use four_mat::Sinv;
pub use four_mat::beta;
pub use four_mat::gamma;
pub use four_mat::boost;
pub use four_mat::FourVec;
pub use four_mat::FourMat;
pub use four_mat::LightSpeedError;

pub use three_mat::ThreeMat;
pub use three_mat::ThreeVec;
pub use three_mat::{radians_between, degrees_between};

pub use utils::consts;
pub use utils::Serializable;
