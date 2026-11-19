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

    ftree.add_feed("init_state", init_state)?;
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

## Stamped Samples

`StampedSample<T: Serializable>` pairs an exact `i64` integer timestamp with any `Serializable` Calcify payload (`f64`, `i64`, `u64`, `String`, `ThreeVec`, `ThreeMat`, `FourVec`, `FourMat`, `Bin`, `Point`, `PointBin`, or custom `Object` types).

### Why use `StampedSample`?

* **Full integer timestamp precision:** Unlike storing time in `Point` (`f64`, which loses integer precision above 2^53), `i64` timestamps represent Unix epoch milliseconds, microseconds, nanoseconds, or signed simulation ticks without rounding error.
* **Works with any payload type:** Timestamp streams can wrap scalars, vectors, matrices, histograms, strings, or custom simulation structs uniformly in `Collection<StampedSample<T>>`, `Tree`, and `FeedTree`.
* **Built-in stream operations:** `Collection<StampedSample<T>>` provides `window`, `sort_by_timestamp`, `sorted_by_timestamp`, `is_monotonic`, `time_bounds`, `deltas`, `map_values`, `timestamps`, `values`, plus `to_points`, `to_points_since`, and `hist` on scalar/`Point` streams.
* **Native ICalcify support:** `Tree` (`value_subtype`) and `FeedTree` (`ValueSubType`) preserve the inner value type so `ICalcify` automatically loads stamped streams as `StampedSampleBranch` objects ready for windowing, `time_hist`, `hist`, `hist2d`, plotting, and curve fitting.

### Stamped Stream Example

```rust
use std::error;

use calcify::{Collection, FeedTree, StampedSample, Tree};
use calcify::io::ToFile;

fn main() -> Result<(), Box<dyn error::Error>> {
    // Collect timestamped scalar readings (e.g., epoch milliseconds)
    let mut events: Collection<StampedSample<f64>> = Collection::empty();
    events.push_sample(1_700_000_000_250, 5.12);
    events.push_sample(1_700_000_000_100, 4.87);
    events.push_sample(1_700_000_000_500, 5.34);

    // Sort chronologically, slice a time window, and inspect inter-arrival deltas
    events.sort_by_timestamp();
    assert!(events.is_monotonic());
    let early = events.window(1_700_000_000_000, 1_700_000_000_300);
    let inter_arrival_ms = events.deltas();

    // Save into a Tree using add_stamped_branch
    let mut tree = Tree::new("detector_run");
    tree.add_stamped_branch("events", events.clone(), "f64")?;
    tree.add_stamped_branch("early_events", early, "f64")?;
    tree.add_branch("inter_arrival_ms", inter_arrival_ms, "i64")?;
    tree.write_msg("detector_run.msg")?;

    // Or append incrementally into a stamped FeedTree
    let mut feed: FeedTree<StampedSample<f64>> = FeedTree::new_stamped("live_feed", "f64");
    feed.add_stream("channel_a")?;
    feed.write_samples("channel_a", events)?;
    feed.write_sample("channel_a", 1_700_000_000_900, 4.95)?;
    feed.write_msg("live_feed.msg")?;

    Ok(())
}
```

## Examples

* `cargo run --example universe_in_a_box --release` — Many-body gravitational particle simulation writing `Tree` and `FeedTree` snapshots.
* `cargo run --example decay_signals` — Generates 10,000 `StampedSample<f64>` detector energy events over 24 hours with a decaying event rate and a Gaussian energy distribution, ready to histogram and fit in ICalcify.

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
