#![crate_name = "calcify"]
//! # Calcify
//!
//!  A crate for 3-D and 4-D vector and matrix algebra, conceived for use in physics simulations. Builds out from a basic ThreeVec struct including most commonly used operations built in.
//!
//! ## Notes
//!
//! * All physics constants are exported in the top in SI units. To retrieve them in Planck or natural units call calcify::Consts::planck() or calcify::Consts::natural().
//!
//! * FourMat::lambda() has been replaced by fn boost(initial: FourVec, v: ThreeVec). The math has been changed.
//!
//! ## todo
//!
//! Four-Momentum

mod collection;

pub use collection::Collection;

pub use collection::Sinv;
pub use collection::beta;
pub use collection::gamma;
pub use collection::boost;
pub use collection::FourVec;
pub use collection::FourMat;

pub use collection::ThreeMat;
pub use collection::ThreeVec;
pub use collection::{radians_between, degrees_between};

pub use collection::consts;
