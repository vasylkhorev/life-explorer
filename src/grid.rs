use crate::rule::HalfLifeRule;
use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Grid2D {
    pub width: usize,
    pub height: usize,
    pub data: Vec<i8>,
}

impl Grid2D {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0; width * height],
        }
    }

    pub fn from_vec(width: usize, height: usize, data: Vec<i8>) -> Self {
        assert_eq!(width * height, data.len());
        Self { width, height, data }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> i8 {
        self.data[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: i8) {
        self.data[y * self.width + x] = val;
    }

    pub fn hit_edge(&self) -> bool {
        // Top and bottom rows
        for x in 0..self.width {
            if self.data[x] > 0 || self.data[(self.height - 1) * self.width + x] > 0 {
                return true;
            }
        }
        // Left and right columns
        for y in 0..self.height {
            if self.data[y * self.width] > 0 || self.data[y * self.width + self.width - 1] > 0 {
                return true;
            }
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        self.data.iter().all(|&v| v == 0)
    }

    pub fn alive_count(&self) -> usize {
        self.data.iter().filter(|&&v| v > 0).count()
    }

    pub fn bounding_box(&self) -> Option<(usize, usize, usize, usize)> {
        let mut min_x = self.width;
        let mut min_y = self.height;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut found = false;

        for y in 0..self.height {
            for x in 0..self.width {
                if self.get(x, y) > 0 {
                    found = true;
                    if x < min_x { min_x = x; }
                    if x > max_x { max_x = x; }
                    if y < min_y { min_y = y; }
                    if y > max_y { max_y = y; }
                }
            }
        }

        if found {
            Some((min_y, max_y, min_x, max_x))
        } else {
            None
        }
    }

    pub fn crop(&self, y_min: usize, y_max: usize, x_min: usize, x_max: usize) -> Self {
        let new_w = x_max - x_min + 1;
        let new_h = y_max - y_min + 1;
        let mut new_grid = Self::new(new_w, new_h);

        for y in y_min..=y_max {
            for x in x_min..=x_max {
                new_grid.set(x - x_min, y - y_min, self.get(x, y));
            }
        }
        new_grid
    }

    pub fn as_bytes(&self) -> &[u8] {
        // Safe because Vec<i8> and Vec<u8> have the same memory layout.
        // We use this for signatures.
        unsafe {
            std::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.data.len())
        }
    }
}

impl fmt::Display for Grid2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let val = self.get(x, y);
                let c = match val {
                    2 => "##",
                    1 => "::",
                    _ => "  ",
                };
                write!(f, "{}", c)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Grid2D {
    pub fn to_rle(&self) -> String {
        let mut rle = String::new();
        
        for y in 0..self.height {
            let mut current_char = ' ';
            let mut run_count = 0;
            
            for x in 0..self.width {
                let val = self.get(x, y);

                let char_actual = HalfLifeRule::value_to_rle_char(val as i32);

                if current_char == ' ' {
                    current_char = char_actual;
                    run_count = 1;
                } else if current_char == char_actual {
                    run_count += 1;
                } else {
                    if run_count > 1 {
                        rle.push_str(&run_count.to_string());
                    }
                    rle.push(current_char);
                    current_char = char_actual;
                    run_count = 1;
                }
            }
            
            // End of row
            if current_char != 'b' || true { // We trim trailing 'b's manually later or just emit them. Often RLE omits trailing dead cells before $. Let's omit them.
                if current_char != 'b' {
                    if run_count > 1 {
                        rle.push_str(&run_count.to_string());
                    }
                    rle.push(current_char);
                }
            }
            
            if y < self.height - 1 {
                rle.push('$');
            } else {
                rle.push('!');
            }
        }
        
        // Clean up redundant trailing dollar signs before ! if needed, but not strictly necessary
        rle = rle.replace("$!", "!");
        let mut cleaned = rle;
        while cleaned.contains("$$!") {
            cleaned = cleaned.replace("$$!", "$!");
        }
        
        cleaned
    }
}
