# Release Notes

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
