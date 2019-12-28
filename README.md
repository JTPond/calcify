# Calcify

A crate for 3-D and 4-D vector and matrix algebra, conceived for use in physics simulations. Builds out from a basic ThreeVec struct including most commonly used operations built in.
Includes physics constants, 3 and 4-D vectors and matrices and many associated operations, collections, histograms, and output trees, which are serialized in json or MessagePack.

## CalcifyLab

Beginning work on a new lib crate to make simulations (such as the example) more straightforward.
Look forward to that in the coming months.

## ICalcify

Check it out [here!](https://github.com/JTPond/ICalcify "ICalcify GitHub")

## Notes
* New types, ThreeField and ThreeVecField. These are wrapper function around function pointers. They will have limited functionality for most users, but they will be important for typing in CalcifyLab.

* Added Feed trait and FeedTree type for mutable data feeds.

* Updated organization of source files.

* Fixed Warnings in example and fixed deprecated code in LightSpeedError

* Added LightSpeedError type for beta()

* Added a compact json format to Serialization as `to_jsonc()`, which is array intensive, instead to object intensive. Also added binary Serialization to MessagePack using the rmp crate as `to_msg()`. The format is like jsonc, not json. The on disk savings of jsonc over json can be ~20%, and the savings for msg over json can be ~63%.

* Now includes example of a many body simulation "universe_in_a_box" use `cargo run --example universe_in_a_box` This could take several seconds.

* Branches can now be extracted from Trees, but this is not an ideal process. Trees should still be seen as containers for output only.

* All physics constants are exported in the top in SI units. To retrieve them in Planck or natural units call calcify::Consts::planck() or calcify::Consts::natural().

* FourMat::lambda() has been replaced by fn boost(initial: FourVec, v: ThreeVec). The math has been changed.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
