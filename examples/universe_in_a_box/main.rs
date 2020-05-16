#[macro_use]
extern crate lazy_static;

extern crate calcify;
extern crate chrono;

use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;
use std::error;

use chrono::prelude::*;

use calcify::Tree;
use calcify::Collection;
use calcify::Bin;
use calcify::Point;
use calcify::Serializable;

mod universe_in_a_box;

use universe_in_a_box::Particle;
use universe_in_a_box::Universe;

fn main() -> Result<(),Box<dyn error::Error>> {
    lazy_static!{
        static ref UNIVERSE_RANGE: f64 = 1.0;
        static ref UNIVERSE_NUM: usize = 500;
        static ref UNIVERSE_DT: f64 = 0.01;
        static ref RUN_T: usize = 200;
        static ref NOW: DateTime<Local> = Local::now();
        static ref NOWS: String = NOW.format("%m/%d/%Y %H:%M").to_string();
        static ref DETAILS: String = format!("Universe Range: {}, Number of Particles: {}, Delta T: {}, Time steps: {}, Total Time: {}", *UNIVERSE_RANGE, *UNIVERSE_NUM, *UNIVERSE_DT, *RUN_T, (*RUN_T as f64)*(*UNIVERSE_DT));
    }
    let mut ttree = Tree::new("universe_in_a_box");

    let mut universe = Universe::cloud(*UNIVERSE_RANGE,*UNIVERSE_NUM,*UNIVERSE_DT);

    let init_state: Collection<Particle> = Collection::from(universe.state.clone());
    let init_hist: Collection<Bin> = init_state.map(|x| {x.r().r()}).hist(500);
    let init_spread: Collection<Point> = Collection::plot(&init_state.map(|x| {*x.r().x0()}).vec,&init_state.map(|x| {*x.r().x1()}).vec).cut(|p| p.r() <= 1.0);

    ttree.add_field("Desc","A Tree including branches for the simple universe in a box multiparticle simulation.")?;
    ttree.add_field("Details", &*DETAILS)?;
    ttree.add_field("Run on",&*NOWS)?;

    ttree.add_branch("init_state", init_state, "Object")?;
    ttree.add_branch("init_hist", init_hist, "Bin")?;
    ttree.add_branch("init_spread", init_spread, "Point")?;

    universe.run(*RUN_T);

    let mid1_state: Collection<Particle> = Collection::from(universe.state.clone());
    let mid1_hist: Collection<Bin> = mid1_state.map(|x| {x.r().r()}).hist(500);

    ttree.add_branch("mid1_state", mid1_state, "Object")?;
    ttree.add_branch("mid1_hist", mid1_hist, "Bin")?;

    universe.run(*RUN_T);

    let mid2_state: Collection<Particle> = Collection::from(universe.state.clone());
    let mid2_hist: Collection<Bin> = mid2_state.map(|x| {x.r().r()}).hist(500);

    ttree.add_branch("mid2_state", mid2_state, "Object")?;
    ttree.add_branch("mid2_hist", mid2_hist, "Bin")?;

    universe.run(*RUN_T);

    let fin_state: Collection<Particle> = Collection::from(universe.state.clone());
    let fin_hist: Collection<Bin> = fin_state.map(|x| {x.r().r()}).hist(500);
    let fin_spread: Collection<Point> = Collection::plot(&fin_state.map(|x| {*x.r().x0()}).vec,&fin_state.map(|x| {*x.r().x1()}).vec).cut(|p| p.r() <= 1.0);

    ttree.add_branch("fin_state", fin_state, "Object")?;
    ttree.add_branch("fin_hist", fin_hist, "Bin")?;
    ttree.add_branch("fin_spread", fin_spread, "Point")?;

    let f = File::create("universe.msg").unwrap();
    let mut wr = BufWriter::new(f);
    wr.write(ttree.to_msg().expect("Failed to convert to msg.").as_slice()).expect("Failed to write to file.");
    Ok(())
}
