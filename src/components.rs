use std::collections::VecDeque;
use crate::grid::Grid2D;

pub struct Component {
    pub crop: Grid2D,
    pub bbox: (usize, usize, usize, usize), // (y_min, y_max, x_min, x_max)
    pub size: usize,                        // Number of active cells
}

pub fn label_components(grid: &Grid2D) -> (Vec<i32>, usize) {
    let mut labeled = vec![0; grid.width * grid.height];
    let mut visited = vec![false; grid.width * grid.height];
    let mut label_count = 0;

    let offsets: [(isize, isize); 8] = [
        (-1, -1), (-1, 0), (-1, 1),
        (0, -1),           (0, 1),
        (1, -1),  (1, 0),  (1, 1)
    ];

    for y in 0..grid.height {
        for x in 0..grid.width {
            let idx = y * grid.width + x;
            if grid.data[idx] > 0 && !visited[idx] {
                label_count += 1;
                
                let mut queue = VecDeque::new();
                queue.push_back((y, x));
                visited[idx] = true;

                while let Some((cy, cx)) = queue.pop_front() {
                    let cidx = cy * grid.width + cx;
                    labeled[cidx] = label_count;

                    for &(dy, dx) in &offsets {
                        let ny = cy as isize + dy;
                        let nx = cx as isize + dx;

                        if ny >= 0 && ny < grid.height as isize && nx >= 0 && nx < grid.width as isize {
                            let n_idx = (ny as usize) * grid.width + (nx as usize);
                            if grid.data[n_idx] > 0 && !visited[n_idx] {
                                visited[n_idx] = true;
                                queue.push_back((ny as usize, nx as usize));
                            }
                        }
                    }
                }
            }
        }
    }

    (labeled, label_count as usize)
}

pub fn get_components(grid: &Grid2D) -> Vec<Component> {
    let (labeled, num_features) = label_components(grid);
    let mut components = Vec::new();

    for i in 1..=num_features as i32 {
        let mut min_x = grid.width;
        let mut min_y = grid.height;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut size = 0;

        // Find bounding box for this component
        let mut has_cells = false;
        for y in 0..grid.height {
            for x in 0..grid.width {
                let idx = y * grid.width + x;
                if labeled[idx] == i {
                    has_cells = true;
                    size += 1;
                    if x < min_x { min_x = x; }
                    if x > max_x { max_x = x; }
                    if y < min_y { min_y = y; }
                    if y > max_y { max_y = y; }
                }
            }
        }

        if !has_cells {
            continue;
        }

        let cw = max_x - min_x + 1;
        let ch = max_y - min_y + 1;
        let mut crop = Grid2D::new(cw, ch);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let idx = y * grid.width + x;
                if labeled[idx] == i {
                    crop.set(x - min_x, y - min_y, grid.get(x, y));
                }
            }
        }

        components.push(Component {
            crop,
            bbox: (min_y, max_y, min_x, max_x),
            size,
        });
    }

    components
}
