/// consts module
pub mod consts;

pub mod serializable;
pub use serializable::Serializable;
pub use serializable::Deserializable;

/// Errors  module
pub mod errors;

/// ## File IO
///
/// * JSON serialization and deserialization are powered by `serde_json` (`to_json` / `from_json`), while MessagePack (`to_msg` / `from_msg`) uses `rmp` for compact binary files.
/// * `ToFile` and `FromFile` accept any `impl AsRef<Path>` (`&str`, `&Path`, `PathBuf`).
/// * Feel free to use `serde` when implementing the Serialization traits for your custom `"Object"` types.
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
