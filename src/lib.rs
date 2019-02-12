#![crate_name = "calcify"]
//! # Calcify
//!
//! A crate for 3-D vector and matrix algebra, conceived for use in physics simulations. Builds out from a basic ThreeVec struct including most commonly used operations built in
//!
//! ## todo:
//! Four-Momentum

mod four_mat;

pub use four_mat::C_LIGHT;
pub use four_mat::K_BOLTZ;
pub use four_mat::E_CHARGE;
pub use four_mat::Q_ALPHA;
pub use four_mat::G_ALPHA;
pub use four_mat::H_BAR;
pub use four_mat::EP_NAUGHT;
pub use four_mat::MU_NAUGHT;
pub use four_mat::BIG_G;
pub use four_mat::M_PROTON;
pub use four_mat::M_ELECTRON;
pub use four_mat::Consts;
pub use four_mat::Sinv;
pub use four_mat::beta;
pub use four_mat::gamma;
pub use four_mat::boost;
pub use four_mat::FourVec;
pub use four_mat::FourMat;

pub use four_mat::ThreeMat;
pub use four_mat::ThreeVec;
pub use four_mat::{radians_between, degrees_between};
