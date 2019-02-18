# Calcify

 A crate for 3-D and 4-D vector and matrix algebra, conceived for use in physics simulations. Builds out from a basic ThreeVec struct including most commonly used operations built in.

## Notes

* All physics constants are exported in the top in SI units. To retrieve them in Planck or natural units call calcify::Consts::planck() or calcify::Consts::natural().

* FourMat::lambda() has been replaced by fn boost(initial: FourVec, v: ThreeVec). The math has been changed.

## todo

* from_json
* to_bytes
* Four-Momentum

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
