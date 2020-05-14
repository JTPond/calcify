# Calcify

A crate for 3-D and 4-D vector and matrix algebra, conceived for use in physics simulations. Builds out from a basic ThreeVec struct including most commonly used operations built in.
Includes physics constants, 3 and 4-D vectors and matrices and many associated operations, collections, histograms, and output trees, which are serialized in json or MessagePack.

## ICalcify

Python command line utility and module for analyzing Tree files. 

Check it out [here!](https://github.com/JTPond/ICalcify "ICalcify GitHub")

## Usage

```rust
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;
use std::error;

extern crate calcify;

use calcify::Tree;
use calcify::Collection;
use calcify::Bin;
use calcify::Serializable;

mod dummy_experiment_lib;

use dummy_experiment_lib::Projectile;
use dummy_experiment_lib::Lab;

fn main() -> Result<(),Box<dyn error::Error>> {
    let mut ttree = Tree::new("Dummy_Exp");

    let mut dummy_lab = Lab::new();

    let init_state: Collection<Projectile> = Collection::from_vec(dummy_lab.state.clone());
    let init_hist: Collection<Bin> = init_state.map(|x| {x.r().r()}).hist(500);

    ttree.add_field("Desc","A Tree for an example that does not exist")?;

    ttree.add_branch("init_state", init_state, "Object")?;
    ttree.add_branch("init_hist", init_hist, "Bin")?;


    dummy_lab.run(1000);

    let f = File::create("lab.msg").unwrap();
    let mut wr = BufWriter::new(f);
    wr.write(ttree.to_msg().expect("Failed to convert to msg.").as_slice()).expect("Failed to write to file.");
    Ok(())
}
```

## Example

`cargo run --example universe_in_a_box --release`

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
