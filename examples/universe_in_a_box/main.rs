#[macro_use]
extern crate lazy_static;

extern crate calcify;
extern crate chrono;

use std::error;

use chrono::prelude::*;

use calcify::Tree;
use calcify::FeedTree;
use calcify::Collection;
use calcify::Bin;
use calcify::Point;
use calcify::io::ToFile;

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
    let mut ftree = FeedTree::<Particle>::new("states","Object");
    let mut ttree = Tree::new("data");

    let mut universe = Universe::cloud(*UNIVERSE_RANGE,*UNIVERSE_NUM,*UNIVERSE_DT);

    let init_state: Collection<Particle> = Collection::from(universe.state.clone());
    let init_hist: Collection<Bin> = init_state.map(|x| {x.r().r()}).hist(500);
    let init_spread: Collection<Point> = Collection::plot(&init_state.map(|x| {*x.r().x0()}).vec,&init_state.map(|x| {*x.r().x1()}).vec).cut(|p| p.r() <= 1.0);

    ftree.add_field("Desc","A FeedTree of states for the simple universe in a box multiparticle simulation.")?;
    ftree.add_field("Details", &*DETAILS)?;
    ftree.add_field("Run on",&*NOWS)?;

    ttree.add_field("Desc","A Tree data branches for the simple universe in a box multiparticle simulation.")?;
    ttree.add_field("Details", &*DETAILS)?;
    ttree.add_field("Run on",&*NOWS)?;



    ftree.add_feed("init_state", init_state)?;

    ttree.add_branch("init_hist", init_hist, "Bin")?;
    ttree.add_branch("init_spread", init_spread, "Point")?;

    universe.run(*RUN_T);

    let mid1_state: Collection<Particle> = Collection::from(universe.state.clone());
    let mid1_hist: Collection<Bin> = mid1_state.map(|x| {x.r().r()}).hist(500);

    ftree.add_feed("mid1_state", mid1_state)?;
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

    ftree.add_feed("fin_state", fin_state)?;
    ttree.add_branch("fin_hist", fin_hist, "Bin")?;
    ttree.add_branch("fin_spread", fin_spread, "Point")?;

    ftree.write_msg("universe_states.msg")?;
    ttree.write_msg("universe_data.msg")?;
    Ok(())
}
