#[derive(Clone, Copy, PartialEq)]
pub enum Cell {
    Open,
    Wall,
}

pub struct Map {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<Cell>,
}

impl Map {
    pub fn new(width: u32, height: u32) -> Self {
        let mut cells = vec![Cell::Open; (width * height) as usize];

        // Border walls
        for x in 0..width {
            cells[x as usize] = Cell::Wall;
            cells[((height - 1) * width + x) as usize] = Cell::Wall;
        }
        for y in 0..height {
            cells[(y * width) as usize] = Cell::Wall;
            cells[(y * width + width - 1) as usize] = Cell::Wall;
        }

        Map {
            width,
            height,
            cells,
        }
    }

    #[inline]
    pub fn is_open(&self, x: u32, y: u32) -> bool {
        self.cells[(y * self.width + x) as usize] == Cell::Open
    }

    /// Create a map with obstacles for interesting gameplay.
    pub fn with_obstacles(width: u32, height: u32) -> Self {
        let mut map = Self::new(width, height);

        // Central cross obstacle
        let cx = width / 2;
        let cy = height / 2;
        // Horizontal bar
        for x in (cx - 30)..=(cx + 30) {
            for y in (cy - 2)..=(cy + 2) {
                map.cells[(y * width + x) as usize] = Cell::Wall;
            }
        }
        // Vertical bar
        for y in (cy - 20)..=(cy + 20) {
            for x in (cx - 2)..=(cx + 2) {
                map.cells[(y * width + x) as usize] = Cell::Wall;
            }
        }

        // Corner pillars
        let pillar_size = 8u32;
        let offsets = [
            (width / 4, height / 4),
            (3 * width / 4, height / 4),
            (width / 4, 3 * height / 4),
            (3 * width / 4, 3 * height / 4),
        ];
        for (px, py) in offsets {
            for dy in 0..pillar_size {
                for dx in 0..pillar_size {
                    let x = px - pillar_size / 2 + dx;
                    let y = py - pillar_size / 2 + dy;
                    if x > 0 && x < width - 1 && y > 0 && y < height - 1 {
                        map.cells[(y * width + x) as usize] = Cell::Wall;
                    }
                }
            }
        }

        map
    }

    /// Produce a flat byte buffer of wall data (0 = open, 1 = wall).
    pub fn to_bytes(&self) -> Vec<u8> {
        self.cells
            .iter()
            .map(|c| match c {
                Cell::Open => 0,
                Cell::Wall => 1,
            })
            .collect()
    }
}
