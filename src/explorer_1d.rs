use std::collections::VecDeque;
use rayon::prelude::*;
use rand::Rng;
use indicatif::{ProgressBar, ParallelProgressIterator};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern1DResult {
    Dead,
    Expand,
    Chaos,
    Spaceship,
    Oscillator,
}

#[derive(Debug)]
pub struct Rule1DStats {
    pub weights: (f64, f64, f64),
    pub b_min: i32,
    pub b_max: i32,
    pub s_min: i32,
    pub s_max: i32,
    pub dead: usize,
    pub expand: usize,
    pub chaos: usize,
    pub spaceship: usize,
    pub oscillator: usize,
}

fn step_1d_int(grid: &[i8], b_min: i32, b_max: i32, s_min: i32, s_max: i32, kernel: &[i32; 7]) -> Vec<i8> {
    let mut next = vec![0; grid.len()];
    
    // We only iterate where the kernel can fit (padding 3 on each side)
    for i in 3..(grid.len() - 3) {
        let mut sum = 0;
        for k in 0..7 {
            sum += grid[i - 3 + k] as i32 * kernel[k];
        }
        
        let current = grid[i];
        let birth = sum >= b_min && sum <= b_max;
        let survival = current == 1 && sum >= s_min && sum <= s_max;
        
        if birth || survival {
            next[i] = 1;
        } else {
            next[i] = 0;
        }
    }
    
    next
}

fn analyze_1d_pattern(
    seed_grid: &[i8],
    b_min: i32,
    b_max: i32,
    s_min: i32,
    s_max: i32,
    kernel: &[i32; 7],
    max_steps: usize,
) -> Pattern1DResult {
    let mut grid = seed_grid.to_vec();
    let mut history: VecDeque<(Vec<u8>, usize)> = VecDeque::with_capacity(200);
    
    // Warm-up
    for _ in 0..20 {
        grid = step_1d_int(&grid, b_min, b_max, s_min, s_max, kernel);
        
        let sum: usize = grid.iter().map(|&x| x as usize).sum();
        if sum == 0 {
            return Pattern1DResult::Dead;
        }
        if sum > grid.len() * 7 / 10 {
            return Pattern1DResult::Expand;
        }
    }
    
    for _ in 0..max_steps {
        grid = step_1d_int(&grid, b_min, b_max, s_min, s_max, kernel);
        
        let alive_coords: Vec<usize> = grid.iter().enumerate().filter(|(_, &v)| v == 1).map(|(i, _)| i).collect();
        
        if alive_coords.is_empty() {
            return Pattern1DResult::Dead;
        }
        
        let left = alive_coords[0];
        let right = *alive_coords.last().unwrap();
        
        if left < 20 || right > grid.len() - 20 {
            return Pattern1DResult::Expand;
        }
        
        let crop = grid[left..=right].to_vec();
        // convert to u8 for equality
        let crop_bytes: Vec<u8> = crop.iter().map(|&x| x as u8).collect();
        
        let mut found_idx = -1;
        for (i, (h_crop, _)) in history.iter().enumerate() {
            if crop_bytes == *h_crop {
                found_idx = i as i32;
                break;
            }
        }
        
        if found_idx != -1 {
            let shift = left as i32 - history[found_idx as usize].1 as i32;
            if shift == 0 {
                return Pattern1DResult::Oscillator;
            } else {
                return Pattern1DResult::Spaceship;
            }
        }
        
        if history.len() == 200 {
            history.pop_front();
        }
        history.push_back((crop_bytes, left));
    }
    
    Pattern1DResult::Chaos
}

pub fn explore_1d(num_patterns: usize, _threads: usize) -> Vec<Rule1DStats> {
    let weight_sets: [(f64, f64, f64); 9] = [
        (1.0, 0.5, 0.5),
        (1.0, 0.25, 0.25),
        (1.0, 1.0, 0.25),
        (1.0, 0.75, 0.5),
        (1.0, 0.5, 0.25),
        (1.0, 1.0, 0.5),
        (1.0, 1.0, 1.0),
        (0.75, 0.5, 0.25),
        (0.5, 0.5, 0.5)
    ];
    
    let mut all_rules = Vec::new();
    for ws in weight_sets {
        let max_sum = (2.0 * (ws.0 + ws.1 + ws.2)).ceil() as i32;
        
        let mut intervals = Vec::new();
        for min_val in 0..=max_sum {
            for max_val in min_val..=max_sum {
                intervals.push((min_val, max_val));
            }
        }
        
        for b_int in &intervals {
            for s_int in &intervals {
                all_rules.push((ws, *b_int, *s_int));
            }
        }
    }
    
    let total_rules = all_rules.len();
    println!("Exploring 1D rules...");
    println!("Testing {} weight configurations.", weight_sets.len());
    println!("Total rules to test: {} ({} patterns each)", total_rules, num_patterns);
    
    let pb = ProgressBar::new(total_rules as u64);
    
    let mut results: Vec<Rule1DStats> = all_rules.into_par_iter()
        .progress_with(pb)
        .map(|(ws, b_int, s_int)| {
            let (w1, w2, w3) = ws;
            let (b_min, b_max) = b_int;
            let (s_min, s_max) = s_int;
            
            let b_min_scaled = b_min * 4;
            let b_max_scaled = b_max * 4;
            let s_min_scaled = s_min * 4;
            let s_max_scaled = s_max * 4;
            
            let kernel = [
                (w3 * 4.0) as i32,
                (w2 * 4.0) as i32,
                (w1 * 4.0) as i32,
                0,
                (w1 * 4.0) as i32,
                (w2 * 4.0) as i32,
                (w3 * 4.0) as i32
            ];
            
            let mut stats = Rule1DStats {
                weights: ws,
                b_min, b_max, s_min, s_max,
                dead: 0, expand: 0, chaos: 0, spaceship: 0, oscillator: 0
            };
            
            let mut rng = rand::thread_rng();
            let grid_size = 400;
            
            for _ in 0..num_patterns {
                let mut grid = vec![0; grid_size];
                let seed_len = rng.gen_range(5..20);
                let density = rng.gen_range(0.3..0.7);
                
                let start = (grid_size - seed_len) / 2;
                let mut sum = 0;
                for i in 0..seed_len {
                    let r: f64 = rng.gen();
                    if r < density {
                        grid[start + i] = 1;
                        sum += 1;
                    }
                }
                
                if sum == 0 {
                    grid[start + seed_len / 2] = 1;
                }
                
                match analyze_1d_pattern(&grid, b_min_scaled, b_max_scaled, s_min_scaled, s_max_scaled, &kernel, 400) {
                    Pattern1DResult::Dead => stats.dead += 1,
                    Pattern1DResult::Expand => stats.expand += 1,
                    Pattern1DResult::Chaos => stats.chaos += 1,
                    Pattern1DResult::Spaceship => stats.spaceship += 1,
                    Pattern1DResult::Oscillator => stats.oscillator += 1,
                }
            }
            stats
        }).collect();
        
    results.sort_by(|a, b| {
        let spaceship_cmp = b.spaceship.cmp(&a.spaceship);
        if spaceship_cmp == std::cmp::Ordering::Equal {
            b.chaos.cmp(&a.chaos)
        } else {
            spaceship_cmp
        }
    });
    
    results
}
