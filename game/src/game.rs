use crate::constants::*;
use crate::cursor::*;
use crate::fighter::*;
use crate::map::Map;
use crate::mesh::Mesh;

pub struct GameState {
    pub map: Map,
    pub mesh: Mesh,
    pub fighters: Vec<Fighter>,
    pub places: Vec<Place>,
    pub cursors: [Cursor; NB_TEAMS],
    pub active_fighters: [i32; NB_TEAMS],
    pub playing_teams: usize,
    pub global_clock: i32,
    pub army_size: i32,
    // Config
    pub fighter_attack: u32,
    pub fighter_defense: u32,
    pub fighter_new_health: u32,
    pub number_influence: i32,
    pub cursor_speed: i32,
    pub fighter_number: usize, // index into FILL_TABLE (0-32)
}

impl GameState {
    pub fn new(map: Map) -> Self {
        let w = map.width;
        let h = map.height;
        let size = (w * h) as usize;

        // Build hierarchical mesh from map
        let mut mesh = Mesh::build(&map);

        // Create place array: link each pixel to its mesh node
        let mut places = vec![Place::new(); size];
        for (node_idx, node) in mesh.nodes.iter().enumerate() {
            // A mesh node covers a square from (x, y) to (x + size - 1, y + size - 1)
            let nx = node.x as i32;
            let ny = node.y as i32;
            let ns = node.size;
            for dy in 0..ns {
                for dx in 0..ns {
                    let px = nx + dx;
                    let py = ny + dy;
                    if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                        let pi = (py as u32 * w + px as u32) as usize;
                        places[pi].mesh_idx = node_idx as i32;
                    }
                }
            }
        }

        mesh.reset_gradients();
        mesh.reset_directions();

        GameState {
            map,
            mesh,
            fighters: Vec::new(),
            places,
            cursors: std::array::from_fn(|_| Cursor::new()),
            active_fighters: [0; NB_TEAMS],
            playing_teams: 0,
            global_clock: 2, // LW5 starts at 2
            army_size: 0,
            fighter_attack: DEFAULT_FIGHTER_ATTACK,
            fighter_defense: DEFAULT_FIGHTER_DEFENSE,
            fighter_new_health: DEFAULT_FIGHTER_NEW_HEALTH,
            number_influence: DEFAULT_NUMBER_INFLUENCE,
            cursor_speed: 1,
            fighter_number: DEFAULT_FIGHTER_NUMBER,
        }
    }

    /// Add a player. Places fighters in a spiral from the spawn point.
    /// `total_teams` is the total number of teams that will be added, so each
    /// team gets an equal share of the army.
    pub fn add_player(&mut self, player_id: usize, total_teams: usize) {
        if player_id >= NB_TEAMS {
            return;
        }

        let w = self.map.width as i32;
        let h = self.map.height as i32;

        // Spawn positions: teams 0-5 use classic LW5 2x3 grid,
        // teams 6+ use farthest-point heuristic for smart distribution
        let (sx, sy) = if player_id < 6 {
            let raw = match player_id {
                0 => (w / 6, h / 4),
                1 => (w / 2, h / 4),
                2 => (5 * w / 6, h / 4),
                3 => (w / 6, 3 * h / 4),
                4 => (w / 2, 3 * h / 4),
                _ => (5 * w / 6, 3 * h / 4),
            };
            self.find_passable_near(raw.0, raw.1)
        } else {
            let mut existing: Vec<(i32, i32)> = Vec::new();
            for c in &self.cursors {
                if c.active { existing.push((c.x, c.y)); }
            }
            self.find_farthest_spawn(&existing)
        };

        // Compute fighters per team from battle room and fill table
        let battle_room = self.mesh.battle_room();
        let fill_pct = FILL_TABLE[self.fighter_number.min(FILL_TABLE.len() - 1)] as i32;
        let total_fighters = (battle_room * fill_pct / 100).max(1);
        let fighters_per_team = (total_fighters / total_teams.max(1) as i32).max(1);

        let health = MAX_FIGHTER_HEALTH - 1;
        let team = player_id as i8;
        let mut placed = 0i32;

        // Spiral placement (matching LW5 army.c place_team)
        let mut x_min = sx;
        let mut x_max = sx;
        let mut y_min = sy;
        let mut y_max = sy;

        while placed < fighters_per_team {
            // Top edge: left to right at y_min
            let mut x = x_min;
            while x <= x_max && placed < fighters_per_team {
                if self.try_add_fighter(x, y_min, team, health) {
                    placed += 1;
                }
                x += 1;
            }
            if x_max < w - 2 {
                x_max += 1;
            }

            // Right edge: top to bottom at x_max
            let mut y = y_min;
            while y <= y_max && placed < fighters_per_team {
                if self.try_add_fighter(x_max, y, team, health) {
                    placed += 1;
                }
                y += 1;
            }
            if y_max < h - 2 {
                y_max += 1;
            }

            // Bottom edge: right to left at y_max
            x = x_max;
            while x >= x_min && placed < fighters_per_team {
                if self.try_add_fighter(x, y_max, team, health) {
                    placed += 1;
                }
                x -= 1;
            }
            if x_min > 1 {
                x_min -= 1;
            }

            // Left edge: bottom to top at x_min
            y = y_max;
            while y >= y_min && placed < fighters_per_team {
                if self.try_add_fighter(x_min, y, team, health) {
                    placed += 1;
                }
                y -= 1;
            }
            if y_min > 1 {
                y_min -= 1;
            }

            // Safety: if spiral has expanded to map bounds and still can't place, break
            if x_min <= 1 && x_max >= w - 2 && y_min <= 1 && y_max >= h - 2 {
                break;
            }
        }

        // Init cursor at center of mass of placed fighters
        let mut cx = 0i64;
        let mut cy = 0i64;
        let mut count = 0i64;
        for f in &self.fighters {
            if f.team == team {
                cx += f.x as i64;
                cy += f.y as i64;
                count += 1;
            }
        }
        if count > 0 {
            let avg_x = (cx / count) as i32;
            let avg_y = (cy / count) as i32;
            // Spiral outward to find a passable position for the cursor
            let (cur_x, cur_y) = self.find_passable_near(avg_x, avg_y);
            self.cursors[player_id].init(player_id, cur_x, cur_y);
        }

        self.playing_teams += 1;
        self.army_size = self.fighters.len() as i32;
    }

    fn try_add_fighter(&mut self, x: i32, y: i32, team: i8, health: i16) -> bool {
        if x < 1 || y < 1 || x >= self.map.width as i32 - 1 || y >= self.map.height as i32 - 1 {
            return false;
        }
        let idx = self.map.idx(x, y);
        if self.places[idx].mesh_idx < 0 || self.places[idx].fighter_idx >= 0 {
            return false;
        }

        let fighter_idx = self.fighters.len() as i32;
        self.fighters.push(Fighter::new(x as i16, y as i16, team, health));
        self.places[idx].fighter_idx = fighter_idx;
        true
    }

    /// Find the passable position farthest from all existing spawn points.
    /// Samples every 4 pixels for performance.
    fn find_farthest_spawn(&self, existing: &[(i32, i32)]) -> (i32, i32) {
        let w = self.map.width as i32;
        let h = self.map.height as i32;
        let step = 4usize;
        let mut best = (w / 2, h / 2);
        let mut best_dist = 0i64;
        let mut y = 2;
        while y < h - 2 {
            let mut x = 2;
            while x < w - 2 {
                if self.map.is_passable(x, y) {
                    let min_d = existing.iter()
                        .map(|(sx, sy)| {
                            let dx = (x - sx) as i64;
                            let dy = (y - sy) as i64;
                            dx * dx + dy * dy
                        })
                        .min().unwrap_or(i64::MAX);
                    if min_d > best_dist {
                        best_dist = min_d;
                        best = (x, y);
                    }
                }
                x += step as i32;
            }
            y += step as i32;
        }
        best
    }

    fn find_passable_near(&self, x: i32, y: i32) -> (i32, i32) {
        if self.map.is_passable(x, y) {
            return (x, y);
        }
        // Spiral search
        for r in 1..100 {
            for dx in -r..=r {
                for &dy in &[-r, r] {
                    let nx = x + dx;
                    let ny = y + dy;
                    if self.map.is_passable(nx, ny) {
                        return (nx, ny);
                    }
                }
            }
            for dy in -r + 1..r {
                for &dx in &[-r, r] {
                    let nx = x + dx;
                    let ny = y + dy;
                    if self.map.is_passable(nx, ny) {
                        return (nx, ny);
                    }
                }
            }
        }
        (x, y)
    }

    /// Set cursor key state bitmask (1=up, 2=right, 4=down, 8=left).
    pub fn set_key_state(&mut self, player_id: usize, key_state: u8) {
        if player_id < NB_TEAMS && self.cursors[player_id].active {
            self.cursors[player_id].key_state = key_state;
        }
    }

    /// Move cursor based on key_state bitmask, matching LW5 move_cursor.
    /// Called once per tick per active cursor.
    fn move_cursor_by_keys(&mut self, player_id: usize) {
        if player_id >= NB_TEAMS || !self.cursors[player_id].active {
            return;
        }
        let keys = self.cursors[player_id].key_state;
        if keys == 0 {
            return;
        }

        let w = self.map.width as i32;
        let h = self.map.height as i32;
        let speed = self.cursor_speed;

        let mut dx: i32 = 0;
        let mut dy: i32 = 0;
        if keys & CURSOR_KEY_UP != 0 { dy -= speed; }
        if keys & CURSOR_KEY_DOWN != 0 { dy += speed; }
        if keys & CURSOR_KEY_LEFT != 0 { dx -= speed; }
        if keys & CURSOR_KEY_RIGHT != 0 { dx += speed; }

        let new_x = (self.cursors[player_id].x + dx).clamp(1, w - 2);
        let new_y = (self.cursors[player_id].y + dy).clamp(1, h - 2);
        self.set_cursor(player_id, new_x, new_y);
    }

    /// Set cursor position directly (for server-mode: client sends absolute position).
    pub fn set_cursor(&mut self, player_id: usize, x: i32, y: i32) {
        if player_id < NB_TEAMS && self.cursors[player_id].active {
            let w = self.map.width as i32;
            let h = self.map.height as i32;
            let new_x = x.clamp(1, w - 2);
            let new_y = y.clamp(1, h - 2);

            let old_x = self.cursors[player_id].x;
            let old_y = self.cursors[player_id].y;
            let moved = new_x != old_x || new_y != old_y;

            // Mark old mesh position as needing update
            let old_idx = self.map.idx(old_x, old_y);
            let old_mesh = self.places[old_idx].mesh_idx;
            if old_mesh >= 0 {
                self.mesh.nodes[old_mesh as usize].info[player_id].update_time = -1;
            }

            self.cursors[player_id].x = new_x;
            self.cursors[player_id].y = new_y;

            // Store cursor position in new mesh node
            let new_idx = self.map.idx(new_x, new_y);
            let new_mesh = self.places[new_idx].mesh_idx;
            if new_mesh >= 0 {
                let info = &mut self.mesh.nodes[new_mesh as usize].info[player_id];
                info.cursor_x = new_x as i16;
                info.cursor_y = new_y as i16;
                info.update_time = 1; // positive = cursor is here
            }

            // Decrement cursor val when moved or every 13 ticks
            if moved || (self.global_clock % (NB_DIRS as i32 + 1)) == 0 {
                self.cursors[player_id].val -= 1;
            }
        }
    }

    /// One game tick (matching LW5 logic() exactly).
    pub fn tick(&mut self) {
        // Step 1: move_all_cursors based on key_state
        for i in 0..NB_TEAMS {
            self.move_cursor_by_keys(i);
        }

        // Step 2: apply_all_cursor — poke cursor values into mesh
        self.apply_all_cursors();

        // Step 3: spread_single_gradient — one direction per tick
        self.mesh.spread_gradient(self.global_clock, self.playing_teams);

        // Step 4: move_fighters — movement + combat + healing
        self.move_fighters();

        // Step 5: check_loose_team — eliminate teams with 0 fighters
        self.check_loose_team();

        // Step 6: increment clock
        self.global_clock += 1;
    }

    /// Poke cursor gradient values into the mesh (matching LW5 apply_all_cursor).
    fn apply_all_cursors(&mut self) {
        for i in 0..NB_TEAMS {
            if self.cursors[i].active {
                let x = self.cursors[i].x;
                let y = self.cursors[i].y;
                let idx = self.map.idx(x, y);
                let mesh_idx = self.places[idx].mesh_idx;
                if mesh_idx >= 0 {
                    self.mesh.nodes[mesh_idx as usize].info[self.cursors[i].team].grad =
                        self.cursors[i].val;
                }
            }
        }
    }

    /// Move all fighters (matching LW5 fighter.c move_fighters).
    fn move_fighters(&mut self) {
        let start_base = ((self.global_clock / 6) % NB_DIRS as i32) as usize;
        let table = ((self.global_clock / 3) % 2) as usize;
        let mut sens: usize = 0;
        let mut start = start_base;

        // Compute combat parameters
        let (attack, defense, new_health) = compute_combat_params(
            &self.active_fighters,
            self.playing_teams,
            self.army_size,
            self.fighter_attack,
            self.fighter_defense,
            self.fighter_new_health,
            self.number_influence,
        );

        // Reset active fighter counts
        for a in self.active_fighters.iter_mut() {
            *a = 0;
        }

        let w = self.map.width as i32;

        for fi in 0..self.fighters.len() {
            let team = self.fighters[fi].team as usize;
            if team >= self.playing_teams {
                continue;
            }

            self.active_fighters[team] += 1;
            start = if start < NB_DIRS - 1 { start + 1 } else { 0 };

            let fx = self.fighters[fi].x as i32;
            let fy = self.fighters[fi].y as i32;
            let place_idx = (fy * w + fx) as usize;
            let mesh_idx = self.places[place_idx].mesh_idx;
            if mesh_idx < 0 {
                continue;
            }
            let mi = mesh_idx as usize;

            // Determine direction
            let update_time = self.mesh.nodes[mi].info[team].update_time;
            if update_time >= 0 {
                // Cursor is on this mesh cell: use close_dir
                let dir = self.mesh.get_close_dir(
                    mi,
                    self.fighters[fi].x,
                    self.fighters[fi].y,
                    team,
                    (sens % 2) != 0,
                    start,
                );
                self.mesh.nodes[mi].info[team].dir = dir as i8;
                sens += 1;
            } else if (-update_time) < self.global_clock {
                // Direction is stale: recompute from gradient
                let dir = self.mesh.get_main_dir(mi, team, (sens % 2) != 0, start);
                self.mesh.nodes[mi].info[team].dir = dir as i8;
                self.mesh.nodes[mi].info[team].update_time = -self.global_clock;
                sens += 1;
            }

            let dir = self.mesh.nodes[mi].info[team].dir as usize;

            // Try 5 movement directions
            let move_dirs = MOVE_DIR[table][dir];
            let mut moved = false;

            // Try each of the 5 priority directions for movement
            let mut try_results: [(i32, i32, usize); 5] = [(0, 0, 0); 5];
            for p in 0..5 {
                let try_dir = move_dirs[p];
                let dx = DIR_X[try_dir];
                let dy = DIR_Y[try_dir];
                let nx = fx + dx;
                let ny = fy + dy;
                let ni = (ny * w + nx) as usize;
                try_results[p] = (nx, ny, ni);
            }

            // Try to move to an empty passable cell
            for p in 0..5 {
                let (nx, ny, ni) = try_results[p];
                if nx >= 0
                    && ny >= 0
                    && nx < w
                    && ny < self.map.height as i32
                    && self.places[ni].mesh_idx >= 0
                    && self.places[ni].fighter_idx < 0
                {
                    // Move fighter
                    self.places[place_idx].fighter_idx = -1;
                    self.places[ni].fighter_idx = fi as i32;
                    self.fighters[fi].x = nx as i16;
                    self.fighters[fi].y = ny as i16;
                    moved = true;
                    break;
                }
            }

            if moved {
                continue;
            }

            // All 5 directions blocked — try attack then heal
            // Front attack (p0, full damage)
            let (_, _, ni0) = try_results[0];
            if ni0 < self.places.len()
                && self.places[ni0].mesh_idx >= 0
                && self.places[ni0].fighter_idx >= 0
            {
                let target_fi = self.places[ni0].fighter_idx as usize;
                if self.fighters[target_fi].team as usize != team {
                    // Front attack: full damage
                    self.fighters[target_fi].health -= attack[team] as i16;
                    if self.fighters[target_fi].health < 0 {
                        while self.fighters[target_fi].health < 0 {
                            self.fighters[target_fi].health += new_health[team] as i16;
                        }
                        self.fighters[target_fi].team = team as i8;
                    }
                    continue;
                }
            }

            // Side attack p1 (damage >> SIDE_ATTACK_FACTOR)
            let (_, _, ni1) = try_results[1];
            if ni1 < self.places.len()
                && self.places[ni1].mesh_idx >= 0
                && self.places[ni1].fighter_idx >= 0
            {
                let target_fi = self.places[ni1].fighter_idx as usize;
                if self.fighters[target_fi].team as usize != team {
                    let side_damage = (attack[team] >> SIDE_ATTACK_FACTOR).max(1) as i16;
                    self.fighters[target_fi].health -= side_damage;
                    if self.fighters[target_fi].health < 0 {
                        while self.fighters[target_fi].health < 0 {
                            self.fighters[target_fi].health += new_health[team] as i16;
                        }
                        self.fighters[target_fi].team = team as i8;
                    }
                    continue;
                }
            }

            // Side attack p2 (damage >> SIDE_ATTACK_FACTOR)
            let (_, _, ni2) = try_results[2];
            if ni2 < self.places.len()
                && self.places[ni2].mesh_idx >= 0
                && self.places[ni2].fighter_idx >= 0
            {
                let target_fi = self.places[ni2].fighter_idx as usize;
                if self.fighters[target_fi].team as usize != team {
                    let side_damage = (attack[team] >> SIDE_ATTACK_FACTOR).max(1) as i16;
                    self.fighters[target_fi].health -= side_damage;
                    if self.fighters[target_fi].health < 0 {
                        while self.fighters[target_fi].health < 0 {
                            self.fighters[target_fi].health += new_health[team] as i16;
                        }
                        self.fighters[target_fi].team = team as i8;
                    }
                    continue;
                }
            }

            // Heal: if p0 has a friendly fighter
            if ni0 < self.places.len()
                && self.places[ni0].mesh_idx >= 0
                && self.places[ni0].fighter_idx >= 0
            {
                let target_fi = self.places[ni0].fighter_idx as usize;
                if self.fighters[target_fi].team as usize == team {
                    self.fighters[target_fi].health += defense[team] as i16;
                    if self.fighters[target_fi].health >= MAX_FIGHTER_HEALTH {
                        self.fighters[target_fi].health = MAX_FIGHTER_HEALTH - 1;
                    }
                }
            }
        }
    }

    /// Eliminate teams with 0 active fighters.
    /// Unlike LW5, we do NOT shift team indices — this preserves stable
    /// team→color mapping for the client renderer.
    fn check_loose_team(&mut self) {
        for i in 0..self.playing_teams {
            if self.active_fighters[i] == 0 {
                // Deactivate cursor for this team (if not already)
                for c in 0..NB_TEAMS {
                    if self.cursors[c].team == i && self.cursors[c].active {
                        self.cursors[c].active = false;
                        self.cursors[c].loose_time = self.global_clock;
                    }
                }
            }
        }
    }

    /// Produce per-cell bitmap for rendering.
    /// Encoding: 0 = empty, 254 = wall, 1..224 = fighter (team*7 + health_level + 1).
    /// Supports up to 32 teams with 7 health brightness levels.
    pub fn get_bitmap(&self) -> Vec<u8> {
        let w = self.map.width;
        let h = self.map.height;
        let size = (w * h) as usize;
        let mut buf = vec![BITMAP_EMPTY; size];

        // Mark walls
        for i in 0..size {
            if !self.map.passable[i] {
                buf[i] = BITMAP_WALL;
            }
        }

        // Mark fighters
        let hl = BITMAP_HEALTH_LEVELS as u32;
        for fighter in &self.fighters {
            let idx = fighter.y as usize * w as usize + fighter.x as usize;
            if idx < size {
                let team = fighter.team as u8;
                let health_level = ((fighter.health as u32 * (hl - 1)) / (MAX_FIGHTER_HEALTH as u32 - 1)).min(hl - 1) as u8;
                buf[idx] = team * BITMAP_HEALTH_LEVELS + health_level + 1;
            }
        }

        buf
    }

    pub fn get_scores(&self) -> [u32; NB_TEAMS] {
        let mut scores = [0u32; NB_TEAMS];
        for f in &self.fighters {
            let t = f.team as usize;
            if t < NB_TEAMS {
                scores[t] += 1;
            }
        }
        scores
    }

    pub fn get_cursors(&self) -> [(i32, i32, bool); NB_TEAMS] {
        let mut result = [(0i32, 0i32, false); NB_TEAMS];
        for (i, c) in self.cursors.iter().enumerate() {
            result[i] = (c.x, c.y, c.active);
        }
        result
    }

    pub fn map_width(&self) -> u32 {
        self.map.width
    }

    pub fn map_height(&self) -> u32 {
        self.map.height
    }
}
