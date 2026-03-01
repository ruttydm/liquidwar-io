/// Bicolor map: true = passable, false = wall.
/// Matching LW5's area bitmap where foreground (MESH_FG=1) = wall.
pub struct Map {
    pub width: u32,
    pub height: u32,
    pub passable: Vec<bool>,
}

impl Map {
    /// Create a map from a boolean grid (true = passable).
    pub fn new(width: u32, height: u32, passable: Vec<bool>) -> Self {
        assert_eq!(passable.len(), (width * height) as usize);
        Map {
            width,
            height,
            passable,
        }
    }

    /// Create a simple rectangular map with border walls.
    pub fn empty(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        let mut passable = vec![true; size];

        // Border walls
        for x in 0..width {
            passable[x as usize] = false;
            passable[((height - 1) * width + x) as usize] = false;
        }
        for y in 0..height {
            passable[(y * width) as usize] = false;
            passable[(y * width + width - 1) as usize] = false;
        }

        Map {
            width,
            height,
            passable,
        }
    }

    /// Create a map from raw pixel data (grayscale or RGBA).
    /// Pixels darker than threshold are walls.
    pub fn from_pixels(width: u32, height: u32, pixels: &[u8], bytes_per_pixel: usize, threshold: u8) -> Self {
        let size = (width * height) as usize;
        let mut passable = vec![false; size];

        for i in 0..size {
            let brightness = if bytes_per_pixel == 1 {
                pixels[i]
            } else if bytes_per_pixel >= 3 {
                let r = pixels[i * bytes_per_pixel] as u16;
                let g = pixels[i * bytes_per_pixel + 1] as u16;
                let b = pixels[i * bytes_per_pixel + 2] as u16;
                ((r + g + b) / 3) as u8
            } else {
                pixels[i * bytes_per_pixel]
            };
            passable[i] = brightness >= threshold;
        }

        // Ensure border is always wall (matching LW5)
        for x in 0..width {
            passable[x as usize] = false;
            passable[((height - 1) * width + x) as usize] = false;
        }
        for y in 0..height {
            passable[(y * width) as usize] = false;
            passable[(y * width + width - 1) as usize] = false;
        }

        Map {
            width,
            height,
            passable,
        }
    }

    #[inline]
    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }
        self.passable[(y as u32 * self.width + x as u32) as usize]
    }

    #[inline]
    pub fn idx(&self, x: i32, y: i32) -> usize {
        (y as u32 * self.width + x as u32) as usize
    }

    /// Create a default map with some obstacles for testing.
    pub fn with_obstacles(width: u32, height: u32) -> Self {
        let mut map = Self::empty(width, height);

        // Central cross obstacle
        let cx = width / 2;
        let cy = height / 2;
        for x in cx.saturating_sub(30)..=(cx + 30).min(width - 2) {
            for y in cy.saturating_sub(2)..=(cy + 2).min(height - 2) {
                map.passable[(y * width + x) as usize] = false;
            }
        }
        for y in cy.saturating_sub(20)..=(cy + 20).min(height - 2) {
            for x in cx.saturating_sub(2)..=(cx + 2).min(width - 2) {
                map.passable[(y * width + x) as usize] = false;
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
                    let x = px.wrapping_sub(pillar_size / 2) + dx;
                    let y = py.wrapping_sub(pillar_size / 2) + dy;
                    if x > 0 && x < width - 1 && y > 0 && y < height - 1 {
                        map.passable[(y * width + x) as usize] = false;
                    }
                }
            }
        }

        map
    }

    /// Count passable pixels.
    pub fn passable_count(&self) -> u32 {
        self.passable.iter().filter(|&&p| p).count() as u32
    }
}
