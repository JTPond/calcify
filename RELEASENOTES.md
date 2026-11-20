# Release Notes

### 0.10.0 2026-11-19:
* **Breaking (`Tree` / `FeedTree` compile-time typing & object constructors):** Introduce the sealed `CalcifyType` trait for all built-in branch types (`f64`, `i64`, `u64`, `String`, `ThreeVec`, `ThreeMat`, `FourVec`, `FourMat`, `Bin`, `Point`, `PointBin`). `Tree::add_branch` and `Tree::add_stamped_branch` now infer subtype metadata at compile time without magic strings or panics, with `Tree::add_object_branch` and `Tree::add_stamped_object_branch` for custom `"Object"` types. `FeedTree::new` and `FeedTree::new_stamped` infer `"SubType"` / `"ValueSubType"` from `T: CalcifyType`, with `FeedTree::new_object`, `FeedTree::new_stamped_object`, and `FeedTree::new_with_subtype` for custom or explicit subtypes.
* **Breaking (`&self` read APIs & direct branch downcasting):** `Branch` now stores an `Any`-capable payload so `Branch::as_collection`, `Branch::extract`, `Tree::get_branch`, `Tree::read_branch`, `Tree::read_branch_ref`, `Tree::read_stamped_branch`, `Tree::read_stamped_branch_ref`, and `FeedTree::get_feed` take `&self` without `MsgPack` round-trip caching.
* **Breaking (`CalcifyError` context & deprecation cleanup):** Expand `CalcifyError` with `KeyAlreadyExists(String)`, `MissingKey(String)`, `InvalidSubtype(String)`, `SubtypeMismatch { expected, found }`, and `ParseError(String)`, derive `PartialEq, Eq`, and remove the deprecated `Error::description` implementation.
* **Breaking (`ToFile` / `FromFile`):** Accept `impl AsRef<Path>` instead of `&str`, explicitly flush `BufWriter` in `write_json` / `write_msg` so write errors are never dropped, and reject trailing bytes in `read_msg`.
* **JSON via `serde_json` (`Option 1`):** Replace all home-rolled JSON parsers, escapers, and string splitters across `serializable.rs`, `Bin`, `Point`, `PointBin`, `ThreeVec`, `ThreeMat`, `FourVec`, `FourMat`, `Collection`, `StampedSample`, `Branch`, `Tree`, and `FeedTree` with `serde` (no proc-macro `derive`) + `serde_json` (`raw_value`). All built-in structs, `Collection<T>`, `StampedSample<T>`, and `FeedTree<T>` implement `serde::Serialize` and `serde::Deserialize`; `Bin` / `PointBin` parse `count` directly as `u64`; non-finite `f64` values serialize as valid JSON `null`.
* **Fix check-before-insert on duplicate keys:** `Tree::add_field`, `Tree::add_branch`, `Tree::add_stamped_branch`, `Tree::add_object_branch`, `Tree::add_stamped_object_branch`, `FeedTree::add_field`, and `FeedTree::add_feed` check key presence before inserting so returning `KeyAlreadyExists` never overwrites existing entries.
* **Fix 1D & 2D histograms and add fixed-range variants:** Fix `Collection<f64>::hist` and `Collection<Point>::hist` (gapped-data misbinning, empty-collection panics/underflows, `NaN` panics, hardcoded `+ 0.01` bin-width distortion, degenerate constant ranges, and extreme magnitudes), lower minimum bin count to `1`, and add `hist_with_range` on `Collection<f64>`, `Collection<Point>`, `Collection<StampedSample<f64>>`, and `Collection<StampedSample<Point>>`.
* **`Collection` ergonomics & descriptive stats:** Implement `get`, `get_mut`, `as_slice`, `as_mut_slice`, `iter`, `iter_mut`, `Deref<Target = [T]>`, `DerefMut`, `AsRef<[T]>`, `Index`, `IndexMut`, and `&Collection` / `&mut Collection` `IntoIterator` on `Collection<T>`, plus `sum`, `min`, `max`, `mean`, `variance`, `sample_variance`, `std_dev`, `sample_std_dev`, `quantile`, and `median` on `Collection<f64>`.
* **Introspection on `Tree` & `FeedTree`:** Add `fields`, `field_names`, `field_count`, `contains_field` to both, `branches`, `branch_names`, `branch_count`, `contains_branch`, `Branch::len`, `Branch::is_empty` to `Tree`/`Branch`, and `feeds`, `feed_names`, `feed_count`, `contains_feed` to `FeedTree`.
* **Update & feature-gate `rand`:** Upgrade `rand` from `0.6` to `0.10` behind a default-enabled `rand` feature (`ThreeVec::random`, `ThreeMat::random`, `Point::random`), and refresh `examples` and `README.md` for Rust 2024.

### 0.9.0 2026-11-19:
* Add generic `StampedSample<T: Serializable>` type (`timestamp: i64`, `value: T`) with JSON (`{"timestamp":...,"value":...}`) and MessagePack (`[timestamp, value]`) serialization and deserialization
* Add timestamped stream operations on `Collection<StampedSample<T>>`: `stream`, `push_sample`, `timestamps`, `values`, `map_values`, `window`, `sort_by_timestamp`, `sorted_by_timestamp`, `is_monotonic`, `time_bounds`, and `deltas`, plus `to_points`, `to_points_since`, and `hist` on scalar/`Point` streams
* Add `Tree` and `Branch` stamped stream support (`Tree::add_stamped_branch`, `Tree::read_stamped_branch`, `Tree::get_field`, `Branch::new_stamped`, `Branch::subtype`, `Branch::value_subtype`) using a separate `"value_subtype"` metadata field
* Add `FeedTree` stamped stream helpers (`FeedTree::new_stamped`, `FeedTree::add_stream`, `FeedTree::write_sample`, `FeedTree::write_samples`, `FeedTree::get_field`, `FeedTree::subtype`, `FeedTree::value_subtype`, `FeedTree::get_feed_mut`) using `"SubType": "StampedSample"` and `"ValueSubType"`
* Implement `Serializable`/`Deserializable` for `i64` and `Deserializable` for `String`, add `"i64"` and `"u64"` as built-in branch subtypes, and harden JSON/MessagePack parsing for `Collection`, `Branch`, `Tree`, and `FeedTree` (nested objects, empty collections/maps, arbitrary key order, and error propagation)
* Add `decay_signals` example (`examples/decay_signals.rs`) generating 10,000 timestamped Gaussian energy events over 24 hours with a decaying event rate

### 0.8.3 2026-11-10:
* Update to Rust 2024 edition and modern toolchains
* Fix MessagePack deserialization map ordering for `Tree` and `FeedTree`
* Fix `"Bin"`, `"Point"`, and `"PointBin"` MessagePack branch deserialization
* Add `Collection::is_empty`

### 0.8.2 2022-01-16:
* Add PointBin 2D hist type

### 0.8.1 2020-06-13:
* Implement From<Vec<T: Serializable>> and From<&[T: Serializable + Clone]> for Collection

### 0.8 2020-06-06:
* **File IO for Trees and FeedTrees**
* **Remove support for jsonc.**
* Update extract.
* Support "Object" Branches.
* Reorganize files around branch.

### 0.7: 2020-04-18
* Added CalcifyError. Dropped lifetime of Tree keys to 'a from 'static. Upgraded Point to be more like a TwoVec.

### 0.6
* Implemented std::iter::Sum for ThreeVec and FourVec

* ThreeField and ThreeVecField updated to use ThreeVec as input. This was the correct design decision from the beginning.

* New types, ThreeField and ThreeVecField. These are wrapper function around function pointers. They will have limited functionality for most users, but they will be important for typing in CalcifyLab.

### Previous
* Added Feed trait and FeedTree type for mutable data feeds.

* Updated organization of source files.

* Fixed Warnings in example and fixed deprecated code in LightSpeedError

* Added LightSpeedError type for beta()

* Added a compact json format to Serialization as `to_jsonc()`, which is array intensive, instead to object intensive. Also added binary Serialization to MessagePack using the rmp crate as `to_msg()`. The format is like jsonc, not json. The on disk savings of jsonc over json can be ~20%, and the savings for msg over json can be ~63%.

* Now includes example of a many body simulation "universe_in_a_box" use `cargo run --example universe_in_a_box --release` This could take several seconds.

* Branches can now be extracted from Trees, but this is not an ideal process. Trees should still be seen as containers for output only.

* All physics constants are exported in the top in SI units. To retrieve them in Planck or natural units call calcify::Consts::planck() or calcify::Consts::natural().

* FourMat::lambda() has been replaced by fn boost(initial: FourVec, v: ThreeVec). The math has been changed.
