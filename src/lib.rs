#![crate_name = "calcify"]
//! # Calcify
//!
//! A crate for 3-D vector and matrix algebra, conceived for use in physics simulations. Builds out from a basic ThreeVec struct including most commonly used operations built in
//!
//! ## todo: Four-Vector and Four-Momentum
mod three_mat;
pub use three_mat::ThreeMat;
pub use three_mat::ThreeVec;
pub use three_mat::{radians_between, degrees_between};
