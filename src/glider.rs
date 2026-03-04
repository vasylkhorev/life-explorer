use std::collections::HashMap;
use crate::grid::Grid2D;
use crate::rule::HalfLifeRule;

/// Verify that a cropped pattern is a true glider/ship by proving it has
/// periodic translation: it returns to the same shape after P steps,
/// but displaced by a non-zero (dx, dy).
pub fn verify_glider(
    crop: &Grid2D,
    rule: &HalfLifeRule,
    universe_size: usize,
    max_steps: usize,
) -> Option<Grid2D> {
    if crop.height >= universe_size || crop.width >= universe_size {
        return None;
    }
    if crop.width == 0 || crop.height == 0 {
        return None;
    }

    let mut grid = Grid2D::new(universe_size, universe_size);
    let sy = (universe_size - crop.height) / 2;
    let sx = (universe_size - crop.width) / 2;

    for y in 0..crop.height {
        for x in 0..crop.width {
            grid.set(sx + x, sy + y, crop.get(x, y));
        }
    }

    // Track canonical crop → (step, bb_y_min, bb_x_min)
    // Same shape at different position = glider
    let mut history: HashMap<Vec<u8>, (usize, usize, usize)> = HashMap::new();

    // Record initial state
    if let Some((y_min, _y_max, x_min, _x_max)) = grid.bounding_box() {
        if let Some(crop_now) = get_tight_crop(&grid) {
            let canon = canonical_orientation(&crop_now);
            let sig = canon.as_bytes().to_vec();
            history.insert(sig, (0, y_min, x_min));
        }
    }

    for step in 1..=max_steps {
        let mut next_grid = Grid2D::new(universe_size, universe_size);
        rule.step_in_place(&grid, &mut next_grid);
        grid = next_grid;

        if grid.is_empty() {
            return None;
        }

        if grid.alive_count() > 200 {
            return None;
        }

        if grid.hit_edge() {
            return None;
        }

        let (y_min, _y_max, x_min, _x_max) = match grid.bounding_box() {
            Some(bb) => bb,
            None => return None,
        };

        let crop_now = match get_tight_crop(&grid) {
            Some(c) => c,
            None => continue,
        };

        let canon = canonical_orientation(&crop_now);
        let sig = canon.as_bytes().to_vec();

        if let Some(&(prev_step, prev_y, prev_x)) = history.get(&sig) {
            let period = step - prev_step;

            // Position changed = translating = glider!
            if (y_min != prev_y || x_min != prev_x) && period <= 60 {
                // Verify: run one more period, confirm position still changes
                let dy = y_min as isize - prev_y as isize;
                let dx = x_min as isize - prev_x as isize;
                if verify_still_moving(&grid, rule, universe_size, period, dy, dx) {
                    return Some(canon);
                }
            }

            // Same position = oscillator, not glider. Keep searching.
        } else {
            history.insert(sig, (step, y_min, x_min));
        }
    }

    None
}

/// Run one more period and confirm the bounding box shifts by ~(dy, dx).
fn verify_still_moving(
    grid: &Grid2D,
    rule: &HalfLifeRule,
    universe_size: usize,
    period: usize,
    expected_dy: isize,
    expected_dx: isize,
) -> bool {
    let start_bb = match grid.bounding_box() {
        Some((y_min, _, x_min, _)) => (y_min, x_min),
        None => return false,
    };

    let mut g = grid.clone();
    for _ in 0..period {
        let mut next = Grid2D::new(universe_size, universe_size);
        rule.step_in_place(&g, &mut next);
        g = next;

        if g.is_empty() || g.alive_count() > 200 || g.hit_edge() {
            return false;
        }
    }

    let end_bb = match g.bounding_box() {
        Some((y_min, _, x_min, _)) => (y_min, x_min),
        None => return false,
    };

    let actual_dy = end_bb.0 as isize - start_bb.0 as isize;
    let actual_dx = end_bb.1 as isize - start_bb.1 as isize;

    // Must move in the same direction
    actual_dy == expected_dy && actual_dx == expected_dx
}

/// Get the tight bounding-box crop of a grid (all alive cells).
fn get_tight_crop(grid: &Grid2D) -> Option<Grid2D> {
    if let Some((y_min, y_max, x_min, x_max)) = grid.bounding_box() {
        Some(grid.crop(y_min, y_max, x_min, x_max))
    } else {
        None
    }
}

fn get_center_of_mass(grid: &Grid2D) -> Option<(f64, f64)> {
    let mut sum_y = 0;
    let mut sum_x = 0;
    let mut count = 0;

    for y in 0..grid.height {
        for x in 0..grid.width {
            if grid.get(x, y) > 0 {
                sum_y += y;
                sum_x += x;
                count += 1;
            }
        }
    }

    if count == 0 {
        None
    } else {
        Some((sum_y as f64 / count as f64, sum_x as f64 / count as f64))
    }
}

pub fn rot90(grid: &Grid2D) -> Grid2D {
    let mut rot = Grid2D::new(grid.height, grid.width);
    for y in 0..grid.height {
        for x in 0..grid.width {
            rot.set(grid.height - 1 - y, x, grid.get(x, y));
        }
    }
    rot
}

pub fn fliplr(grid: &Grid2D) -> Grid2D {
    let mut flipped = Grid2D::new(grid.width, grid.height);
    for y in 0..grid.height {
        for x in 0..grid.width {
            flipped.set(grid.width - 1 - x, y, grid.get(x, y));
        }
    }
    flipped
}

pub fn flipud(grid: &Grid2D) -> Grid2D {
    let mut flipped = Grid2D::new(grid.width, grid.height);
    for y in 0..grid.height {
        for x in 0..grid.width {
            flipped.set(x, grid.height - 1 - y, grid.get(x, y));
        }
    }
    flipped
}

pub fn canonical_orientation(grid: &Grid2D) -> Grid2D {
    // Determine bounding box
    let (mut min_y, mut max_y, mut min_x, mut max_x) = (grid.height, 0, grid.width, 0);
    let mut found = false;
    for y in 0..grid.height {
        for x in 0..grid.width {
            if grid.get(x, y) > 0 {
                found = true;
                if x < min_x { min_x = x; }
                if x > max_x { max_x = x; }
                if y < min_y { min_y = y; }
                if y > max_y { max_y = y; }
            }
        }
    }
    
    let tight_grid = if found {
        grid.crop(min_y, max_y, min_x, max_x)
    } else {
        grid.clone()
    };

    let mut variants = Vec::new();
    let mut current = tight_grid;

    for _ in 0..4 {
        variants.push(current.clone());
        variants.push(fliplr(&current));
        variants.push(flipud(&current));
        current = rot90(&current);
    }

    // Sort by shape, sum, then bytes
    variants.sort_by(|a, b| {
        let a_area = a.width * a.height;
        let b_area = b.width * b.height;
        let area_cmp = a_area.cmp(&b_area);
        if area_cmp != std::cmp::Ordering::Equal {
            return area_cmp;
        }

        let a_sum = a.data.iter().map(|&v| v as i32).sum::<i32>();
        let b_sum = b.data.iter().map(|&v| v as i32).sum::<i32>();
        // -np.sum(x) means higher sum comes FIRST
        let sum_cmp = b_sum.cmp(&a_sum);
        if sum_cmp != std::cmp::Ordering::Equal {
            return sum_cmp;
        }

        a.as_bytes().cmp(b.as_bytes())
    });

    variants[0].clone()
}
