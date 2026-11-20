use std::error;
use std::fs;

use chrono::prelude::*;
use rand::distr::{Distribution, Open01, Uniform};

use calcify::io::ToFile;
use calcify::{Collection, FeedTree, StampedSample, Tree};

const NUM_SAMPLES: usize = 10_000;
const DURATION_SECS: i64 = 24 * 60 * 60;
const DURATION_MS: i64 = DURATION_SECS * 1_000;
const HALF_LIFE_HOURS: f64 = 8.0;
const ENERGY_MEAN_MEV: f64 = 5.0;
const ENERGY_STD_MEV: f64 = 0.75;

fn sample_normal(rng: &mut impl rand::Rng, mean: f64, std_dev: f64) -> f64 {
    let u1: f64 = Open01.sample(rng);
    let u2: f64 = Open01.sample(rng);
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    mean + std_dev * z
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let tau_secs = HALF_LIFE_HOURS * 3_600.0 / 2.0_f64.ln();
    let cdf_max = 1.0 - (-(DURATION_SECS as f64) / tau_secs).exp();

    let now = Utc::now();
    let end_ms = now.timestamp_millis();
    let start_ms = end_ms - DURATION_MS;
    let run_on = now.with_timezone(&Local).format("%m/%d/%Y %H:%M").to_string();

    let mut rng = rand::rng();
    let uniform_u = Uniform::new(0.0_f64, 1.0_f64)?;

    let mut events: Collection<StampedSample<f64>> = Collection::empty();
    for _ in 0..NUM_SAMPLES {
        let u = uniform_u.sample(&mut rng);
        let offset_secs = -tau_secs * (1.0 - u * cdf_max).ln();
        let ts_ms = (start_ms + (offset_secs * 1_000.0).round() as i64)
            .clamp(start_ms, start_ms + DURATION_MS);
        let energy_mev = sample_normal(&mut rng, ENERGY_MEAN_MEV, ENERGY_STD_MEV).max(0.0);
        events.push_sample(ts_ms, energy_mev);
    }

    events.sort_by_timestamp();
    assert!(events.is_monotonic());

    let first_6h = events.window(start_ms, start_ms + 6 * 3_600_000);
    let last_6h = events.window(start_ms + 18 * 3_600_000, start_ms + DURATION_MS);
    let inter_arrival_ms = events.deltas();
    let energy_hist = events.hist(50);

    let details = format!(
        "Samples: {}, Duration: 24h ({} s), Half-life: {} h, Energy ~ N({}, {}^2) MeV",
        NUM_SAMPLES, DURATION_SECS, HALF_LIFE_HOURS, ENERGY_MEAN_MEV, ENERGY_STD_MEV
    );
    let start_ms_str = start_ms.to_string();
    let duration_secs_str = DURATION_SECS.to_string();
    let half_life_str = HALF_LIFE_HOURS.to_string();
    let mean_str = ENERGY_MEAN_MEV.to_string();
    let std_str = ENERGY_STD_MEV.to_string();

    let mut ftree: FeedTree<StampedSample<f64>> = FeedTree::new_stamped("decay_signals_feed");
    ftree.add_field(
        "Desc",
        "Timestamped Gaussian detector energy feed with exponentially decaying event rate over 24 hours.",
    )?;
    ftree.add_field("Details", &details)?;
    ftree.add_field("Run on", &run_on)?;
    ftree.add_field("StartTimestampMs", &start_ms_str)?;
    ftree.add_field("DurationSecs", &duration_secs_str)?;
    ftree.add_field("HalfLifeHours", &half_life_str)?;
    ftree.add_field("EnergyMeanMeV", &mean_str)?;
    ftree.add_field("EnergyStdMeV", &std_str)?;
    ftree.add_stream("detector_energy")?;
    ftree.write_samples("detector_energy", events.vec.clone())?;

    let mut ttree = Tree::new("decay_signals");
    ttree.add_field(
        "Desc",
        "Timestamped Gaussian detector energy branches with exponentially decaying event rate over 24 hours.",
    )?;
    ttree.add_field("Details", &details)?;
    ttree.add_field("Run on", &run_on)?;
    ttree.add_field("StartTimestampMs", &start_ms_str)?;
    ttree.add_field("DurationSecs", &duration_secs_str)?;
    ttree.add_field("HalfLifeHours", &half_life_str)?;
    ttree.add_field("EnergyMeanMeV", &mean_str)?;
    ttree.add_field("EnergyStdMeV", &std_str)?;

    ttree.add_stamped_branch("events", events.clone())?;
    ttree.add_stamped_branch("first_6h_events", first_6h.clone())?;
    ttree.add_stamped_branch("last_6h_events", last_6h.clone())?;
    ttree.add_branch("inter_arrival_ms", inter_arrival_ms)?;
    ttree.add_branch("energy_hist", energy_hist)?;

    fs::create_dir_all("./scratch")?;
    ftree.write_msg("./scratch/decay_signals_feed.msg")?;
    ttree.write_msg("./scratch/decay_signals.msg")?;

    println!("Generated {} timestamped events over 24 hours.", events.len());
    println!(
        "First 6h count: {} | Last 6h count: {}",
        first_6h.len(),
        last_6h.len()
    );
    println!("Wrote ./scratch/decay_signals.msg and ./scratch/decay_signals_feed.msg");
    println!("Inspect with:");
    println!("  icalcify ./scratch/decay_signals.msg ./scratch/decay_signals_feed.msg");

    Ok(())
}
