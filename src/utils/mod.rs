/// consts module
pub mod consts;

pub mod serializable;
pub use serializable::Serializable;
pub use serializable::Deserializable;

/// Errors  module
pub mod errors;

/// ## File IO
///
/// * Even though json is supported for both reading and writing, it's not as efficiently implemented and may lead to slowdowns when reading large files. Consider only using it for debugging, so that you can read the results of tests, otherwise use msg.
///
/// * Feel free to use Serde when implementing the Serialization traits for your types
///
/// ### Trees
///
/// | Write      | Read |
/// | ----------- | ----------- |
/// | Supports all subtypes      | Internal types only, and not `Object`|
///
/// ### FeedTrees
///
/// | Write| Read |
/// | -----| -----|
/// | Supports all subtypes| Supports all subtypes|
pub mod io;
