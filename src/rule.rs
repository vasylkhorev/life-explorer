use std::fmt;
use crate::grid::Grid2D;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HalfLifeRule {
    pub b_min: i32,
    pub b_max: i32,
    pub s_min: i32,
    pub s_max: i32,
}

impl HalfLifeRule {
    pub fn new(b_min: i32, b_max: i32, s_min: i32, s_max: i32) -> Self {
        Self { b_min, b_max, s_min, s_max }
    }

    /// Executed exactly one step of Half-Life using true sums (0-16) and 3-states.
    pub fn step(&self, grid: &Grid2D) -> Grid2D {
        let mut next_grid = Grid2D::new(grid.width, grid.height);
        
        // Compute neighbor sums (8-connected, wrapping/circular padding)
        for y in 0..grid.height {
            for x in 0..grid.width {
                let current_val = grid.get(x, y);
                let mut sum: i32 = 0;
                
                // Optimized 8-neighbor sum with wrapping
                for dy in [-1, 0, 1] {
                    for dx in [-1, 0, 1] {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        
                        let nx = (x as isize + dx).rem_euclid(grid.width as isize) as usize;
                        let ny = (y as isize + dy).rem_euclid(grid.height as isize) as usize;
                        
                        sum += grid.get(nx, ny) as i32;
                    }
                }
                
                let is_birth = current_val == 0 && sum >= self.b_min && sum <= self.b_max;
                let is_survive = current_val >= 1 && sum >= self.s_min && sum <= self.s_max;
                
                let target = if is_birth || is_survive { 2 } else { 0 };
                
                // diff = sign(target - current)
                let diff = if target > current_val {
                    1
                } else if target < current_val {
                    -1
                } else {
                    0
                };
                
                next_grid.set(x, y, current_val + diff);
            }
        }
        
        next_grid
    }
}

impl fmt::Display for HalfLifeRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = if self.b_min == self.b_max {
            format!("B{}", self.b_min)
        } else {
            format!("B{}-{}", self.b_min, self.b_max)
        };
        
        let s = if self.s_min == self.s_max {
            format!("S{}", self.s_min)
        } else {
            format!("S{}-{}", self.s_min, self.s_max)
        };
        
        write!(f, "{}/{}", b, s)
    }
}
