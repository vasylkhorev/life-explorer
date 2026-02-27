use std::collections::VecDeque;
use crate::grid::Grid2D;
use crate::rule::HalfLifeRule;
use crate::components::get_components;
use crate::glider::{verify_glider, canonical_signature};

#[derive(Debug, Clone)]
pub enum PatternResult {
    Dead,
    Explode,
    Glider(Vec<u8>, Grid2D), // signature, canonical_crop
    Oscillator { period: usize, signature: Vec<u8>, best_phase: Grid2D },
    Chaos,
}

pub fn get_crop(grid: &Grid2D) -> Option<Grid2D> {
    if let Some((y_min, y_max, x_min, x_max)) = grid.bounding_box() {
        Some(grid.crop(y_min, y_max, x_min, x_max))
    } else {
        None
    }
}

pub fn analyze_pattern(
    base_grid: &Grid2D,
    rule: &HalfLifeRule,
    max_steps: usize,
) -> PatternResult {
    let mut grid = base_grid.clone();
    let mut history: VecDeque<Vec<u8>> = VecDeque::with_capacity(60);

    // Warmup
    for _ in 0..30 {
        grid = rule.step(&grid);
        if grid.is_empty() {
            return PatternResult::Dead;
        }
        if grid.alive_count() > 400 {
            return PatternResult::Explode;
        }
    }

    for _ in 0..max_steps {
        grid = rule.step(&grid);

        if grid.is_empty() {
            return PatternResult::Dead;
        }
        
        if grid.alive_count() > 400 {
            return PatternResult::Explode;
        }

        if grid.hit_edge() {
            let components = get_components(&grid);
            for comp in components {
                if comp.size > 50 {
                    continue; // Too big to be a nice glider, probably explosion
                }
                
                if let Some(g_crop) = verify_glider(&comp.crop, rule, 48, 400) {
                    return PatternResult::Glider(g_crop.as_bytes().to_vec(), g_crop);
                }
            }
            return PatternResult::Explode;
        }

        let crop = match get_crop(&grid) {
            Some(c) => c,
            None => continue,
        };
        
        let crop_bytes = crop.as_bytes().to_vec();

        let mut found_idx: i32 = -1;
        for (i, h_bytes) in history.iter().enumerate() {
            if crop_bytes == *h_bytes {
                found_idx = i as i32;
                break;
            }
        }

        if found_idx != -1 {
            let period = history.len() - found_idx as usize;
            
            // Re-simulate to capture ALL phases
            let mut grid_stable = grid.clone();
            let mut phases = vec![crop.clone()];
            
            for _ in 0..(period - 1) {
                grid_stable = rule.step(&grid_stable);
                if let Some(ph_crop) = get_crop(&grid_stable) {
                    phases.push(ph_crop);
                }
            }
            
            // Choose the best phase (most cells active, prioritizing '2's)
            phases.sort_by(|a, b| {
                let score_a: i32 = a.data.iter().map(|&v| if v == 2 { 10 } else if v == 1 { 1 } else { 0 }).sum();
                let score_b: i32 = b.data.iter().map(|&v| if v == 2 { 10 } else if v == 1 { 1 } else { 0 }).sum();
                score_b.cmp(&score_a) // reverse order for best first
            });
            
            let best_phase = phases[0].clone();
            
            if let Some(sig) = canonical_signature(&phases) {
                return PatternResult::Oscillator { period, signature: sig, best_phase };
            }
        }

        if history.len() == 60 {
            history.pop_front();
        }
        history.push_back(crop_bytes);
    }

    PatternResult::Chaos
}
