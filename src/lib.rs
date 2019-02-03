#![crate_name = "calcify"]
//! # Calcify
//!
//! A crate for 3-D vector and matrix algebra, conceived for use in physics simulations. Builds out from a basic ThreeVec struct including most commonly used operations built in
//!
//! ## todo:
//! Four-Vector and Four-Momentum

mod four_mat;
pub use four_mat::C_LIGHT;
pub use four_mat::beta;
pub use four_mat::gamma;
pub use four_mat::FourVec;

pub use four_mat::ThreeMat;
pub use four_mat::ThreeVec;
pub use four_mat::{radians_between, degrees_between};
