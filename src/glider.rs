use crate::grid::Grid2D;
use crate::rule::HalfLifeRule;
use crate::components::get_components;

pub fn verify_glider(
    crop: &Grid2D,
    rule: &HalfLifeRule,
    universe_size: usize,
    max_steps: usize,
) -> Option<Grid2D> {
    if crop.height >= universe_size || crop.width >= universe_size {
        return None;
    }

    let mut grid1 = Grid2D::new(universe_size, universe_size);
    let mut grid2 = Grid2D::new(universe_size, universe_size);
    let sy = (universe_size - crop.height) / 2;
    let sx = (universe_size - crop.width) / 2;

    for y in 0..crop.height {
        for x in 0..crop.width {
            grid1.set(sx + x, sy + y, crop.get(x, y));
        }
    }

    let (initial_y, initial_x) = match get_center_of_mass(&grid1) {
        Some(pos) => pos,
        None => return None,
    };

    let mut history: Vec<Vec<i8>> = vec![vec![0; universe_size * universe_size]; 60];
    let mut history_len = 0;
    let mut history_idx = 0;

    for _ in 1..max_steps {
        rule.step_in_place(&grid1, &mut grid2);
        std::mem::swap(&mut grid1, &mut grid2);

        if grid1.is_empty() {
            return None;
        }

        for i in 0..history_len {
            if history[i] == grid1.data {
                return None;
            }
        }

        history[history_idx].copy_from_slice(&grid1.data);
        history_idx = (history_idx + 1) % 60;
        if history_len < 60 {
            history_len += 1;
        }

        if grid1.hit_edge() {
            let (final_y, final_x) = match get_center_of_mass(&grid1) {
                Some(pos) => pos,
                None => return None,
            };

            let dy = final_y - initial_y;
            let dx = final_x - initial_x;
            let dist_sq = dy * dy + dx * dx;

            if dist_sq > 4.0 {
                let comps = get_components(&grid1);
                let main_comps: Vec<_> = comps.into_iter().filter(|c| c.size > 2).collect();

                if main_comps.len() == 1 {
                    let main_comp = &main_comps[0];
                    if main_comp.crop.height < 16 && main_comp.crop.width < 16 {
                        return Some(main_comp.crop.clone());
                    }
                }
            }
            return None;
        }
    }

    None
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

pub fn canonical_signature(phases: &[Grid2D]) -> Option<Vec<u8>> {
    let mut valid_phases: Vec<&Grid2D> = phases.iter().filter(|p| !p.is_empty()).collect();
    if valid_phases.is_empty() {
        return None;
    }

    valid_phases.sort_by(|a, b| {
        let a_area = a.width * a.height;
        let b_area = b.width * b.height;
        let area_cmp = a_area.cmp(&b_area);
        if area_cmp != std::cmp::Ordering::Equal {
            return area_cmp;
        }

        let a_sum = a.data.iter().map(|&v| v as i32).sum::<i32>();
        let b_sum = b.data.iter().map(|&v| v as i32).sum::<i32>();
        // -np.sum(x)
        b_sum.cmp(&a_sum)
    });

    Some(valid_phases[0].as_bytes().to_vec())
}
