use std::collections::VecDeque;
use crate::grid::Grid2D;
use crate::rule::HalfLifeRule;
use crate::components::get_components;
use crate::glider::{verify_glider, canonical_orientation};

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
    max_history: usize,
) -> PatternResult {
    let mut grid1 = base_grid.clone();
    let mut grid2 = Grid2D::new(grid1.width, grid1.height);
    let mut history: VecDeque<Vec<u8>> = VecDeque::with_capacity(max_history);
    let mut prev_bounding_box: Option<(usize, usize)> = None; // (y_min, x_min)

    // Warmup
    for _ in 0..30 {
        rule.step_in_place(&grid1, &mut grid2);
        std::mem::swap(&mut grid1, &mut grid2);
        if grid1.is_empty() {
            return PatternResult::Dead;
        }
        if grid1.alive_count() > 400 {
            return PatternResult::Explode;
        }
    }

    for _ in 0..max_steps {
        rule.step_in_place(&grid1, &mut grid2);
        std::mem::swap(&mut grid1, &mut grid2);

        if grid1.is_empty() {
            return PatternResult::Dead;
        }
        
        if grid1.alive_count() > 400 {
            return PatternResult::Explode;
        }

        if grid1.hit_edge() {
            let components = get_components(&grid1);
            for comp in components {
                if comp.size > 100 {
                    continue; // Too big to be a nice glider, probably explosion
                }
                
                if let Some(g_crop) = verify_glider(&comp.crop, rule, 64, 500) {
                    let canon = canonical_orientation(&g_crop);
                    return PatternResult::Glider(canon.as_bytes().to_vec(), canon);
                }
            }
            return PatternResult::Explode;
        }

        let crop = match get_crop(&grid1) {
            Some(c) => c,
            None => continue,
        };
        
        let crop_bytes = crop.as_bytes().to_vec();

        // Track bounding box position to detect translation
        let bb = grid1.bounding_box();

        let mut found_idx: i32 = -1;
        for (i, h_bytes) in history.iter().enumerate() {
            if crop_bytes == *h_bytes {
                found_idx = i as i32;
                break;
            }
        }

        if found_idx != -1 {
            let period = history.len() - found_idx as usize;
            
            // Check if the pattern is translating by comparing bounding box position.
            // If it moved, this is likely a glider/ship, not an oscillator.
            if let Some((bb_y_min, _bb_y_max, bb_x_min, _bb_x_max)) = bb {
                if let Some(prev_bb) = prev_bounding_box {
                    let dy = (bb_y_min as isize - prev_bb.0 as isize).unsigned_abs();
                    let dx = (bb_x_min as isize - prev_bb.1 as isize).unsigned_abs();
                    
                    if dy > 0 || dx > 0 {
                        // Pattern is translating! Try to verify as a glider.
                        let components = get_components(&grid1);
                        for comp in components {
                            if comp.size > 100 {
                                continue;
                            }
                            if let Some(g_crop) = verify_glider(&comp.crop, rule, 64, 500) {
                                let canon = canonical_orientation(&g_crop);
                                return PatternResult::Glider(canon.as_bytes().to_vec(), canon);
                            }
                        }
                        // Translation detected but verify_glider failed — treat as chaos
                        // rather than misclassifying as oscillator
                        return PatternResult::Chaos;
                    }
                }
            }

            // Re-simulate to capture ALL phases
            let mut grid_stable = grid1.clone();
            let mut grid_stable_next = Grid2D::new(grid_stable.width, grid_stable.height);
            let mut phases = vec![crop.clone()];
            
            for _ in 0..(period - 1) {
                rule.step_in_place(&grid_stable, &mut grid_stable_next);
                std::mem::swap(&mut grid_stable, &mut grid_stable_next);
                if let Some(ph_crop) = get_crop(&grid_stable) {
                    phases.push(ph_crop);
                }
            }
            
            // Choose the best phase (more full alive cells prioritized)
            phases.sort_by(|a, b| {
                let score_a: f64 = a.data.iter().map(|&v| if v == 2 { 1.0 } else if v == 1 { 0.1 } else { 0.0 }).sum();
                let score_b: f64 = b.data.iter().map(|&v| if v == 2 { 1.0 } else if v == 1 { 0.1 } else { 0.0 }).sum();
                score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
            });
            
            let best_phase = phases[0].clone();
            let canon = canonical_orientation(&best_phase);
            return PatternResult::Oscillator { period, signature: canon.as_bytes().to_vec(), best_phase: canon };
        }

        // Track bounding box for next iteration
        if let Some((bb_y_min, _bb_y_max, bb_x_min, _bb_x_max)) = bb {
            prev_bounding_box = Some((bb_y_min, bb_x_min));
        }

        if history.len() >= max_history {
            history.pop_front();
        }
        history.push_back(crop_bytes);
    }
    // Pattern survived the full loop without oscillator match or edge-hit.
    // Try verifying the remaining pattern as a slow-moving glider/ship.
    let components = get_components(&grid1);
    for comp in components {
        if comp.size > 100 {
            continue;
        }
        if let Some(g_crop) = verify_glider(&comp.crop, rule, 64, 500) {
            let canon = canonical_orientation(&g_crop);
            return PatternResult::Glider(canon.as_bytes().to_vec(), canon);
        }
    }

    PatternResult::Chaos
}
