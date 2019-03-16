extern crate calcify;
mod universe_in_a_box;
use universe_in_a_box::Particle;
use universe_in_a_box::Universe;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

use calcify::Tree;
use calcify::Collection;
use calcify::Bin;
use calcify::Point;
use calcify::ThreeVec;
use calcify::Serializable;

fn main() {
    let mut ttree = Tree::new("universe_in_a_box");

    let mut universe = Universe::cloud(1.0,400,0.001);
    let init_state: Collection<Particle> = Collection::from_vec(universe.state.clone());
    let init_hist: Collection<Bin> = init_state.map(|x| {x.r().r()}).hist(50);

    ttree.add_field("Desc","A Tree including branches for the universe in a box.");
    ttree.add_branch("init_state", init_state, "ThreeVec");
    ttree.add_branch("init_hist", init_hist, "f64");


    universe.run(6500,1);

    let mid1_state: Collection<Particle> = Collection::from_vec(universe.state.clone());
    let mid1_hist: Collection<Bin> = mid1_state.map(|x| {x.r().r()}).hist(50);

    ttree.add_branch("mid1_state", mid1_state, "ThreeVec");
    ttree.add_branch("mid1_hist", mid1_hist, "f64");

    universe.run(6500,1);

    let mid2_state: Collection<Particle> = Collection::from_vec(universe.state.clone());
    let mid2_hist: Collection<Bin> = mid2_state.map(|x| {x.r().r()}).hist(50);

    ttree.add_branch("mid2_state", mid2_state, "ThreeVec");
    ttree.add_branch("mid2_hist", mid2_hist, "f64");

    universe.run(6500,1);

    let fin_state: Collection<Particle> = Collection::from_vec(universe.state.clone());
    let fin_hist: Collection<Bin> = fin_state.map(|x| {x.r().r()}).hist(50);

    ttree.add_branch("fin_state", fin_state, "ThreeVec");
    ttree.add_branch("fin_hist", fin_hist, "f64");

    let f = File::create("universe.txt").unwrap();
    let mut wr = BufWriter::new(f);
    wr.write(ttree.to_json().as_bytes()).unwrap();

}
