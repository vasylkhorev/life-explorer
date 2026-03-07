use std::collections::{HashMap, HashSet};
use rayon::prelude::*;
use rand::Rng;
use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use std::io::{Write, BufWriter};
use std::fs::OpenOptions;
use std::sync::{Mutex, Arc};

use crate::rule::HalfLifeRule;
use crate::grid::Grid2D;
use crate::analysis::{analyze_pattern, PatternResult};

#[derive(Debug)]
pub struct RuleStats {
    pub rule: HalfLifeRule,
    pub dead: usize,
    pub explode: usize,
    pub chaos: usize,
    pub gliders_count: usize,
    pub oscillators_count: usize,
    pub oscillators_by_period: HashMap<usize, usize>, // period -> count
}

pub fn explore_2d(rules: Vec<HalfLifeRule>, num_patterns: usize, _threads: usize) -> Vec<RuleStats> {
    let total_rules = rules.len();
    println!("Exploring {} rules with {} patterns each.", total_rules, num_patterns);

    let pb = ProgressBar::new(total_rules as u64);
    let is_tty = console::user_attended();
    let counter = std::sync::atomic::AtomicUsize::new(0);
    
    // Create a global file to continuously append discovered patterns
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("discovered_patterns.jsonl")
        .unwrap();
    let patterns_file = Arc::new(Mutex::new(BufWriter::new(file)));

    let mut results: Vec<RuleStats> = rules.into_par_iter()
        .progress_with(pb)
        .map(|rule| {
            let current = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            
            if !is_tty && (current == 1 || current % 50 == 0 || current == total_rules) {
                println!("Progress: {}/{} rules processed...", current, total_rules);
                let _ = std::io::stdout().flush();
            }

            let mut dead = 0;
            let mut explode = 0;
            let mut chaos = 0;
            let mut gliders = HashSet::new();
            let mut oscillators: HashMap<usize, HashSet<Vec<u8>>> = HashMap::new();

            // Pre-allocate grids to reduce allocations
            let gs = 48;
            let _p_size = 12;
            let mut grid_pool = Vec::with_capacity(num_patterns);
            for _ in 0..num_patterns {
                grid_pool.push(Grid2D::new(gs, gs));
            }

            // We use a thread-local RNG
            let mut rng = rand::thread_rng();

            for grid in &mut grid_pool {
                let p_size = rng.gen_range(3..9);
                let density = rng.gen_range(0.15..0.45);
                let sy = (gs - p_size) / 2;
                let sx = (gs - p_size) / 2;

                for y in 0..p_size {
                    for x in 0..p_size {
                        let r: f64 = rng.gen();
                        let val = if r < density / 2.0 {
                            2
                        } else if r < density {
                            1
                        } else {
                            0
                        };
                        grid.set(sx + x, sy + y, val);
                    }
                }

                match analyze_pattern(&grid, &rule, 120, 60) {
                    PatternResult::Dead => dead += 1,
                    PatternResult::Explode => explode += 1,
                    PatternResult::Chaos => chaos += 1,
                    PatternResult::Glider(sig, crop) => {
                        if gliders.insert(sig) {
                            let mut f = patterns_file.lock().unwrap();
                            let rle_body = crop.to_rle();
                            let json = format!(r#"{{"rule":"{}","type":"glider","period":0,"rle":"x = {}, y = {}, rule = FuzzyLife/3\n{}"}}"#, rule, crop.width, crop.height, rle_body);
                            f.write_all(json.as_bytes()).unwrap();
                            f.write_all("\n".as_bytes()).unwrap();
                        }
                    },
                    PatternResult::Oscillator { period, signature, best_phase } => {
                        let osc_set = oscillators.entry(period).or_default();
                        if osc_set.insert(signature) {
                            let mut f = patterns_file.lock().unwrap();
                            let rle_body = best_phase.to_rle();
                            let json = format!(r#"{{"rule":"{}","type":"oscillator","period":{},"rle":"x = {}, y = {}, rule = FuzzyLife/3\n{}"}}"#, rule, period, best_phase.width, best_phase.height, rle_body);
                            f.write_all(json.as_bytes()).unwrap();
                            f.write_all("\n".as_bytes()).unwrap();
                        }
                    }
                }
            }

            let mut oscillators_by_period = HashMap::new();
            let mut oscillators_count = 0;
            for (p, set) in oscillators.iter() {
                oscillators_by_period.insert(*p, set.len());
                oscillators_count += set.len();
            }

            RuleStats {
                rule,
                dead,
                explode,
                chaos,
                gliders_count: gliders.len(),
                oscillators_count,
                oscillators_by_period,
            }
        })
        .collect();

    // Sort by unique gliders, then total unique oscillators
    results.sort_by_cached_key(|a| {
        let oscs = a.oscillators_count;
        let gliders = a.gliders_count;
        (std::cmp::Reverse(gliders), std::cmp::Reverse(oscs))
    });

    results
}
