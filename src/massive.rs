use std::collections::HashSet;
use rayon::prelude::*;
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::io::{Write, BufWriter};
use std::fs::OpenOptions;
use std::sync::{Mutex, Arc};

use crate::rule::HalfLifeRule;
use crate::grid::Grid2D;
use crate::analysis::{analyze_pattern, PatternResult};

pub fn run_massive_search(rule: HalfLifeRule, num_patterns: usize, max_period: usize, max_steps: usize, max_size: usize, _threads: usize) {
    println!("Running Massive Search for rule: {}", rule);
    println!("Patterns: {}, Max Steps: {}, Max Period: {}, Max Size: {}x{}", num_patterns, max_steps, max_period, max_size, max_size);

    let progress_counter = Arc::new(AtomicUsize::new(0));
    let print_interval = (num_patterns / 20).max(1); // Print ~20 times total (every 5%)
    
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("massive_discoveries.jsonl")
        .unwrap();
    let patterns_file = Arc::new(Mutex::new(BufWriter::new(file)));

    // Track unique gliders (signatures)
    let global_gliders_sig = Arc::new(Mutex::new(HashSet::new()));
    
    // Track up to 5 smallest oscillators per period
    // Map: period -> Vec<(area, width, height, rle_body)>
    let best_oscillators = Arc::new(Mutex::new(std::collections::HashMap::new()));

    (0..num_patterns).into_par_iter()
        .for_each(|_| {
            let current = progress_counter.fetch_add(1, Ordering::Relaxed) + 1;
            if current % print_interval == 0 || current == num_patterns {
                let percent = (current as f64 / num_patterns as f64) * 100.0;
                println!("Progress: {} / {} patterns ({:.1}%)", current, num_patterns, percent);
            }
            
            let mut rng = rand::thread_rng();
            
            // Random sizes up to max_size x max_size
            let p_size = rng.gen_range(3..=max_size);
            
            // Massive variance in density: from very sparse to very dense
            let density: f64 = rng.gen_range(0.01..0.95);
            
            // A sufficiently large grid to avoid immediate edge collisions
            // 100x100 grid with up to 20x20 seed gives 40 padding on each side
            let gs = 100;
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

            match analyze_pattern(&grid, &rule, max_steps, max_period) {
                PatternResult::Dead => {},
                PatternResult::Explode => {},
                PatternResult::Chaos => {
                     // Omitted to focus purely on oscillators and gliders
                },
                PatternResult::Glider(sig, crop) => {
                    let is_new = {
                        let mut gliders_sig = global_gliders_sig.lock().unwrap();
                        gliders_sig.insert(sig)
                    };
                    if is_new {
                        let rle_body = crop.to_rle();
                        let mut f = patterns_file.lock().unwrap();
                        let json = format!(r#"{{"rule":"{}","type":"glider","period":0,"rle":"x = {}, y = {}, rule = FuzzyLife/3\n{}"}}"#, rule, crop.width, crop.height, rle_body);
                        f.write_all(json.as_bytes()).unwrap();
                        f.write_all("\n".as_bytes()).unwrap();
                    }
                },
                PatternResult::Oscillator { period, signature: _, best_phase } => {
                    // We only log if period > 15 (filtering out noise of common low period oscillators)
                    if period > 15 {
                        let area = best_phase.width * best_phase.height;
                        let rle_body = best_phase.to_rle();
                        let new_entry = (area, best_phase.width, best_phase.height, rle_body);
                        
                        let mut osc_map = best_oscillators.lock().unwrap();
                        let entry = osc_map.entry(period).or_insert_with(Vec::new);
                        
                        // Check if we already have this exact rle_body to avoid duplicates
                        let is_duplicate = entry.iter().any(|(_, _, _, ref r)| *r == new_entry.3);
                        
                        if !is_duplicate {
                            entry.push(new_entry);
                            // Sort by area ascending
                            entry.sort_by_key(|e| e.0);
                            // Keep only the 5 smallest
                            if entry.len() > 5 {
                                entry.pop();
                            }
                        }
                    }
                }
            }
        });
        
    // Write out the best oscillators at the end
    {
        let mut f = patterns_file.lock().unwrap();
        let osc_map = best_oscillators.lock().unwrap();
        for (period, entries) in osc_map.iter() {
            for (_, width, height, rle_body) in entries {
                let json = format!(r#"{{"rule":"{}","type":"oscillator","period":{},"rle":"x = {}, y = {}, rule = FuzzyLife/3\n{}"}}"#, rule, period, width, height, rle_body);
                f.write_all(json.as_bytes()).unwrap();
                f.write_all("\n".as_bytes()).unwrap();
            }
        }
    }
        
    println!("Massive search completed. Check massive_discoveries.jsonl for new long-lived discoveries.");
}
