# Release Notes


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
