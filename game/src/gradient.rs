use crate::map::Map;
use std::collections::VecDeque;

pub const UNREACHABLE: u16 = u16::MAX;

pub struct GradientField {
    pub distances: Vec<u16>,
    width: u32,
    height: u32,
    queue: VecDeque<(u32, u32)>,
}

const DIRS: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];

impl GradientField {
    pub fn new(width: u32, height: u32) -> Self {
        let capacity = (width * height) as usize;
        GradientField {
            distances: vec![UNREACHABLE; capacity],
            width,
            height,
            queue: VecDeque::with_capacity(capacity / 4),
        }
    }

    /// Recompute the gradient from cursor position using BFS.
    pub fn compute(&mut self, map: &Map, cx: u32, cy: u32) {
        let w = self.width;
        let h = self.height;

        // Reset
        for d in self.distances.iter_mut() {
            *d = UNREACHABLE;
        }
        self.queue.clear();

        if cx >= w || cy >= h || !map.is_open(cx, cy) {
            return;
        }

        self.distances[(cy * w + cx) as usize] = 0;
        self.queue.push_back((cx, cy));

        while let Some((x, y)) = self.queue.pop_front() {
            let current_dist = self.distances[(y * w + x) as usize];
            let next_dist = current_dist + 1;

            for (dx, dy) in &DIRS {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }

                let nx = nx as u32;
                let ny = ny as u32;
                let idx = (ny * w + nx) as usize;

                if map.is_open(nx, ny) && self.distances[idx] == UNREACHABLE {
                    self.distances[idx] = next_dist;
                    self.queue.push_back((nx, ny));
                }
            }
        }
    }

    /// Get the best direction for a fighter at (x, y) to move toward the cursor.
    /// Returns (dx, dy) in {-1, 0, 1}.
    #[inline]
    pub fn best_direction(&self, map: &Map, x: u32, y: u32) -> (i32, i32) {
        let w = self.width;
        let current = self.distances[(y * w + x) as usize];

        if current == 0 || current == UNREACHABLE {
            return (0, 0);
        }

        let mut best_dist = current;
        let mut best_dir = (0i32, 0i32);

        for dx in -1..=1i32 {
            for dy in -1..=1i32 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= self.height as i32 {
                    continue;
                }

                let nx = nx as u32;
                let ny = ny as u32;

                if !map.is_open(nx, ny) {
                    continue;
                }

                let d = self.distances[(ny * w + nx) as usize];
                if d < best_dist {
                    best_dist = d;
                    best_dir = (dx, dy);
                }
            }
        }

        best_dir
    }
}
