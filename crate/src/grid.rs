use crate::constants::SMOOTHING_RADIUS;

const CELL_SIZE: f32 = SMOOTHING_RADIUS;

pub struct SpatialGrid {
    cell_count: Vec<u32>,
    cell_start: Vec<u32>,
    indices: Vec<u32>,
    table_size: usize,
}

impl SpatialGrid {
    pub fn new(max_particles: usize) -> Self {
        // Use a prime table size >= 2 * particle count
        let table_size = Self::next_prime(max_particles * 2);
        SpatialGrid {
            cell_count: vec![0; table_size],
            cell_start: vec![0; table_size],
            indices: vec![0; max_particles],
            table_size,
        }
    }

    fn next_prime(n: usize) -> usize {
        let mut candidate = n | 1; // make odd
        loop {
            if Self::is_prime(candidate) {
                return candidate;
            }
            candidate += 2;
        }
    }

    fn is_prime(n: usize) -> bool {
        if n < 2 {
            return false;
        }
        if n == 2 || n == 3 {
            return true;
        }
        if n % 2 == 0 || n % 3 == 0 {
            return false;
        }
        let mut i = 5;
        while i * i <= n {
            if n % i == 0 || n % (i + 2) == 0 {
                return false;
            }
            i += 6;
        }
        true
    }

    #[inline(always)]
    fn hash_cell(&self, ix: i32, iy: i32, iz: i32) -> usize {
        let h = ((ix.wrapping_mul(73856093))
            ^ (iy.wrapping_mul(19349663))
            ^ (iz.wrapping_mul(83492791))) as usize;
        h % self.table_size
    }

    #[inline(always)]
    fn cell_coords(x: f32, y: f32, z: f32) -> (i32, i32, i32) {
        (
            (x / CELL_SIZE).floor() as i32,
            (y / CELL_SIZE).floor() as i32,
            (z / CELL_SIZE).floor() as i32,
        )
    }

    pub fn rebuild(&mut self, pos_x: &[f32], pos_y: &[f32], pos_z: &[f32], count: usize) {
        // Clear counts
        for c in self.cell_count.iter_mut() {
            *c = 0;
        }

        // Count particles per cell
        for i in 0..count {
            let (cx, cy, cz) = Self::cell_coords(pos_x[i], pos_y[i], pos_z[i]);
            let hash = self.hash_cell(cx, cy, cz);
            self.cell_count[hash] += 1;
        }

        // Prefix sum
        self.cell_start[0] = 0;
        for i in 1..self.table_size {
            self.cell_start[i] = self.cell_start[i - 1] + self.cell_count[i - 1];
        }

        // Reset counts for insertion pass
        let mut offsets = vec![0u32; self.table_size];

        // Insert particle indices
        for i in 0..count {
            let (cx, cy, cz) = Self::cell_coords(pos_x[i], pos_y[i], pos_z[i]);
            let hash = self.hash_cell(cx, cy, cz);
            let idx = self.cell_start[hash] + offsets[hash];
            self.indices[idx as usize] = i as u32;
            offsets[hash] += 1;
        }
    }

    /// Iterate over all particles in the 27 neighboring cells of position (x, y, z).
    #[inline]
    pub fn for_each_neighbor<F>(&self, x: f32, y: f32, z: f32, mut callback: F)
    where
        F: FnMut(usize),
    {
        let (cx, cy, cz) = Self::cell_coords(x, y, z);

        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let hash = self.hash_cell(cx + dx, cy + dy, cz + dz);
                    let start = self.cell_start[hash] as usize;
                    let count = self.cell_count[hash] as usize;

                    for k in start..start + count {
                        callback(self.indices[k] as usize);
                    }
                }
            }
        }
    }
}
