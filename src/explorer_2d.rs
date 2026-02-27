use std::collections::{HashSet, HashMap};
use rayon::prelude::*;
use rand::Rng;
use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use std::io::Write;
use std::sync::atomic::AtomicUsize;

use crate::rule::HalfLifeRule;
use crate::grid::Grid2D;
use crate::analysis::{analyze_pattern, PatternResult};

#[derive(Debug)]
pub struct RuleStats {
    pub rule: HalfLifeRule,
    pub dead: usize,
    pub explode: usize,
    pub chaos: usize,
    pub gliders: HashSet<Vec<u8>>,
    pub oscillators: HashMap<usize, HashSet<Vec<u8>>>, // period -> signatures
}

pub fn explore_2d(rules: Vec<HalfLifeRule>, num_patterns: usize, _threads: usize) -> Vec<RuleStats> {
    let total_rules = rules.len();
    println!("Exploring {} rules with {} patterns each.", total_rules, num_patterns);

    let pb = ProgressBar::new(total_rules as u64);
    let is_tty = console::user_attended();
    let counter = std::sync::atomic::AtomicUsize::new(0);

    let mut results: Vec<RuleStats> = rules.into_par_iter()
        .progress_with(pb)
        .map(|rule| {
            let current = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            
            if !is_tty && (current == 1 || current % 50 == 0 || current == total_rules) {
                println!("Progress: {}/{} rules processed...", current, total_rules);
                let _ = std::io::stdout().flush();
            }

            let mut stats = RuleStats {
                rule,
                dead: 0,
                explode: 0,
                chaos: 0,
                gliders: HashSet::new(),
                oscillators: HashMap::new(),
            };

            // We use a thread-local RNG
            let mut rng = rand::thread_rng();

            for _ in 0..num_patterns {
                let gs = 32;
                let p_size = rng.gen_range(3..9);
                let density = rng.gen_range(0.15..0.45);

                let mut grid = Grid2D::new(gs, gs);
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

                match analyze_pattern(&grid, &rule, 120) {
                    PatternResult::Dead => stats.dead += 1,
                    PatternResult::Explode => stats.explode += 1,
                    PatternResult::Chaos => stats.chaos += 1,
                    PatternResult::Glider(sig, _) => {
                        stats.gliders.insert(sig);
                    },
                    PatternResult::Oscillator { period, signature, .. } => {
                        stats.oscillators.entry(period).or_default().insert(signature);
                    }
                }
            }

            stats
        })
        .collect();

    // Sort by unique gliders, then total unique oscillators
    results.sort_by(|a, b| {
        let a_oscs: usize = a.oscillators.values().map(|s| s.len()).sum();
        let b_oscs: usize = b.oscillators.values().map(|s| s.len()).sum();
        
        let glider_cmp = b.gliders.len().cmp(&a.gliders.len());
        if glider_cmp == std::cmp::Ordering::Equal {
            b_oscs.cmp(&a_oscs)
        } else {
            glider_cmp
        }
    });

    results
}
