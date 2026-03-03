use std::fmt;
use crate::grid::Grid2D;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HalfLifeRule {
    pub b_min: i32,
    pub b_max: i32,
    pub s_min: i32,
    pub s_max: i32,
    pub s2_min: Option<i32>,
    pub s2_max: Option<i32>,
}

impl HalfLifeRule {
    pub fn new(b_min: i32, b_max: i32, s_min: i32, s_max: i32) -> Self {
        Self { b_min, b_max, s_min, s_max, s2_min: None, s2_max: None }
    }

    pub fn new_with_s2(b_min: i32, b_max: i32, s_min: i32, s_max: i32, s2_min: i32, s2_max: i32) -> Self {
        Self { b_min, b_max, s_min, s_max, s2_min: Some(s2_min), s2_max: Some(s2_max) }
    }

    /// Convert from 0,0.5,1 scale to internal 0,1,2 scale
    pub fn from_fuzzy_scale(b_min: f32, b_max: f32, s_min: f32, s_max: f32) -> Self {
        Self {
            b_min: (b_min * 2.0).round() as i32,
            b_max: (b_max * 2.0).round() as i32,
            s_min: (s_min * 2.0).round() as i32,
            s_max: (s_max * 2.0).round() as i32,
            s2_min: None,
            s2_max: None,
        }
    }

    /// Convert from 0,0.5,1 scale to internal 0,1,2 scale with second survival interval
    pub fn from_fuzzy_scale_with_s2(b_min: f32, b_max: f32, s_min: f32, s_max: f32, s2_min: f32, s2_max: f32) -> Self {
        Self {
            b_min: (b_min * 2.0).round() as i32,
            b_max: (b_max * 2.0).round() as i32,
            s_min: (s_min * 2.0).round() as i32,
            s_max: (s_max * 2.0).round() as i32,
            s2_min: Some((s2_min * 2.0).round() as i32),
            s2_max: Some((s2_max * 2.0).round() as i32),
        }
    }

    /// Convert internal value to RLE character (0,1,2 -> b,A,B)
    pub fn value_to_rle_char(val: i32) -> char {
        match val {
            2 => 'B',  // 1.0
            1 => 'A',  // 0.5
            _ => 'b'   // 0
        }
    }

    /// Convert RLE character to internal value (b,A,B -> 0,1,2)
    pub fn rle_char_to_value(ch: char) -> i32 {
        match ch {
            'B' => 2,  // 1.0
            'A' => 1,  // 0.5
            _ => 0     // b = 0
        }
    }

    pub fn format_val(v: i32) -> String {
        if v % 2 == 0 {
            format!("{}", v / 2)
        } else {
            format!("{}.5", v / 2)
        }
    }

    pub fn step(&self, grid: &Grid2D) -> Grid2D {
        let mut next_grid = Grid2D::new(grid.width, grid.height);
        self.step_in_place(grid, &mut next_grid);
        next_grid
    }

    pub fn step_in_place(&self, grid: &Grid2D, next_grid: &mut Grid2D) {
        let w = grid.width as isize;
        let h = grid.height as isize;
        
        for y in 0..grid.height {
            let ry = y as isize;
            for x in 0..grid.width {
                let current_val = grid.get(x, y);
                let mut sum: i32 = 0;
                
                let rx = x as isize;
                for dy in [-1, 0, 1] {
                    for dx in [-1, 0, 1] {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        
                        let mut nx = rx + dx;
                        let mut ny = ry + dy;
                        
                        if nx < 0 {
                            nx = w - 1;
                        } else if nx >= w {
                            nx = 0;
                        }
                        
                        if ny < 0 {
                            ny = h - 1;
                        } else if ny >= h {
                            ny = 0;
                        }
                        
                        sum += grid.get(nx as usize, ny as usize) as i32;
                    }
                }
                
                let is_birth = current_val == 0 && sum >= self.b_min && sum <= self.b_max;
                let mut is_survive = current_val >= 1 && sum >= self.s_min && sum <= self.s_max;
                
                if !is_survive && current_val >= 1 {
                    if let (Some(min2), Some(max2)) = (self.s2_min, self.s2_max) {
                        if sum >= min2 && sum <= max2 {
                            is_survive = true;
                        }
                    }
                }
                
                let target = if is_birth || is_survive { 2 } else { 0 };
                
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
    }
}

impl fmt::Display for HalfLifeRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = if self.b_min == self.b_max {
            format!("B{}", Self::format_val(self.b_min))
        } else {
            format!("B{}-{}", Self::format_val(self.b_min), Self::format_val(self.b_max))
        };
        
        let s2_str = if let (Some(min2), Some(max2)) = (self.s2_min, self.s2_max) {
            if min2 == max2 {
                format!(",{}", Self::format_val(min2))
            } else {
                format!(",{}-{}", Self::format_val(min2), Self::format_val(max2))
            }
        } else {
            "".to_string()
        };
        
        let s = if self.s_min == self.s_max {
            format!("S{}{}", Self::format_val(self.s_min), s2_str)
        } else {
            format!("S{}-{}{}", Self::format_val(self.s_min), Self::format_val(self.s_max), s2_str)
        };
        
        write!(f, "{}/{}", b, s)
    }
}
