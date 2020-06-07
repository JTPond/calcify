# Calcify

A crate for 3-D and 4-D vector and matrix algebra, conceived for use in physics simulations. Builds out from a basic ThreeVec struct including most commonly used operations built in.
Includes physics constants, 3 and 4-D vectors and matrices and many associated operations, collections, histograms, and output trees, which are serialized in json or MessagePack.

## ICalcify

Python command line utility and module for analyzing Tree files.

Check it out [here!](https://github.com/JTPond/ICalcify "ICalcify GitHub")

## Usage

```rust
use std::error;

extern crate calcify;

use calcify::Tree;
use calcify::FeedTree;
use calcify::Collection;
use calcify::Bin;
use calcify::Point;
use calcify::io::ToFile;

mod dummy_experiment_lib;

use dummy_experiment_lib::Projectile;
use dummy_experiment_lib::Lab;

fn main() -> Result<(),Box<dyn error::Error>> {
    let mut ftree = FeedTree::<Projectile>::new("Dummy_States","Object");
    let mut ttree = Tree::new("Dummy_Data");

    ftree.add_field("Desc","A FeedTree of states for a simulation that does not exist.")?;
    ttree.add_field("Desc","A Tree data branches for a simulation that does not exist.")?;

    let mut dummy_lab = Lab::new();

    let init_state: Collection<Projectile> = Collection::from(dummy_lab.state.clone());
    let init_hist: Collection<Bin> = init_state.map(|x| {x.r().r()}).hist(500);
    let init_spread: Collection<Point> = Collection::plot(&init_state.map(|x| {*x.r().x0()}).vec,
                                                          &init_state.map(|x| {*x.r().x1()}).vec)
                                                          .cut(|p| p.r() <= 1.0);

    ftree.add_feed("init_state", init_state, "Object")?;
    ttree.add_branch("init_hist", init_hist, "Bin")?;
    ttree.add_branch("init_spread", init_spread, "Point")?;

    dummy_lab.run(1000);

    let fin_state: Collection<Projectile> = Collection::from(dummy_lab.state.clone());
    let fin_hist: Collection<Bin> = fin_state.map(|x| {x.r().r()}).hist(500);
    let fin_spread: Collection<Point> = Collection::plot(&fin_state.map(|x| {*x.r().x0()}).vec,
                                                         &fin_state.map(|x| {*x.r().x1()}).vec)
                                                         .cut(|p| p.r() <= 1.0);

    ftree.add_feed("fin_state", fin_state)?;
    ttree.add_branch("fin_hist", fin_hist, "Bin")?;
    ttree.add_branch("fin_spread", fin_spread, "Point")?;

    ftree.write_msg("dummy_states.msg")?;
    ttree.write_msg("dummy_data.msg")?;
    Ok(())
}
```

## Example

`cargo run --example universe_in_a_box --release`

## Notes about File IO

* Even though json is supported for both reading and writing, it's not as efficiently implemented and may lead to slowdowns when reading large files. Consider only using it for debugging, so that you can read the results of tests, otherwise use msg.

* Feel free to use Serde when implementing the Serialization traits for your types

### Trees

| Write      | Read |
| ----------- | ----------- |
| Supports all subtypes      | Internal types only, and not `Object`|

### FeedTrees

| Write| Read |
| -----| -----|
| Supports all subtypes| Supports all subtypes|

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
