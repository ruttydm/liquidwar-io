use crate::constants::*;
use crate::fighter::Fighter;
use crate::gradient::GradientField;
use crate::map::Map;

pub struct Player {
    pub cursor_x: u32,
    pub cursor_y: u32,
    pub active: bool,
    pub fighter_count: u32,
}

pub struct GameState {
    pub map: Map,
    pub fighters: Vec<Fighter>,
    pub players: [Player; MAX_PLAYERS],
    gradients: Vec<GradientField>,
    pub tick: u32,
    // Reusable combat buffer: [cell_idx * MAX_PLAYERS + team] = count
    combat_buf: Vec<u16>,
}

impl GameState {
    pub fn new() -> Self {
        let map = Map::with_obstacles(MAP_WIDTH, MAP_HEIGHT);
        let mut gradients = Vec::with_capacity(MAX_PLAYERS);
        for _ in 0..MAX_PLAYERS {
            gradients.push(GradientField::new(MAP_WIDTH, MAP_HEIGHT));
        }
        let combat_buf = vec![0u16; (MAP_WIDTH * MAP_HEIGHT) as usize * MAX_PLAYERS];

        GameState {
            map,
            fighters: Vec::new(),
            players: std::array::from_fn(|_| Player {
                cursor_x: 0,
                cursor_y: 0,
                active: false,
                fighter_count: 0,
            }),
            gradients,
            tick: 0,
            combat_buf,
        }
    }

    pub fn add_player(&mut self, player_id: usize) {
        if player_id >= MAX_PLAYERS {
            return;
        }

        let (spawn_cx, spawn_cy) = match player_id {
            0 => (MAP_WIDTH / 4, MAP_HEIGHT / 4),
            1 => (3 * MAP_WIDTH / 4, 3 * MAP_HEIGHT / 4),
            2 => (3 * MAP_WIDTH / 4, MAP_HEIGHT / 4),
            3 => (MAP_WIDTH / 4, 3 * MAP_HEIGHT / 4),
            _ => return,
        };

        self.players[player_id].active = true;
        self.players[player_id].cursor_x = spawn_cx;
        self.players[player_id].cursor_y = spawn_cy;

        let spawn_radius = ((FIGHTERS_PER_PLAYER as f32).sqrt() / 2.0) as u32 + 1;
        let mut count = 0u32;

        for dy in 0..spawn_radius * 2 {
            for dx in 0..spawn_radius * 2 {
                if count >= FIGHTERS_PER_PLAYER {
                    break;
                }
                let x = spawn_cx.saturating_sub(spawn_radius) + dx;
                let y = spawn_cy.saturating_sub(spawn_radius) + dy;
                if x > 0 && x < MAP_WIDTH - 1 && y > 0 && y < MAP_HEIGHT - 1 && self.map.is_open(x, y) {
                    self.fighters.push(Fighter {
                        x: x as u16,
                        y: y as u16,
                        team: player_id as u8,
                        health: FIGHTER_MAX_HEALTH,
                    });
                    count += 1;
                }
            }
        }
    }

    pub fn set_cursor(&mut self, player_id: usize, x: u32, y: u32) {
        if player_id < MAX_PLAYERS && self.players[player_id].active {
            self.players[player_id].cursor_x = x.min(MAP_WIDTH - 1);
            self.players[player_id].cursor_y = y.min(MAP_HEIGHT - 1);
        }
    }

    pub fn tick(&mut self) {
        self.tick += 1;
        self.compute_gradients();
        self.move_fighters();
        self.resolve_combat();
        self.heal_fighters();
        self.update_counts();
    }

    fn compute_gradients(&mut self) {
        for i in 0..MAX_PLAYERS {
            if self.players[i].active {
                self.gradients[i].compute(
                    &self.map,
                    self.players[i].cursor_x,
                    self.players[i].cursor_y,
                );
            }
        }
    }

    fn move_fighters(&mut self) {
        for fighter in self.fighters.iter_mut() {
            let team = fighter.team as usize;
            if team >= MAX_PLAYERS || !self.players[team].active {
                continue;
            }

            let (dx, dy) = self.gradients[team].best_direction(
                &self.map,
                fighter.x as u32,
                fighter.y as u32,
            );

            let new_x = (fighter.x as i32 + dx).clamp(1, (MAP_WIDTH - 2) as i32) as u16;
            let new_y = (fighter.y as i32 + dy).clamp(1, (MAP_HEIGHT - 2) as i32) as u16;

            if self.map.is_open(new_x as u32, new_y as u32) {
                fighter.x = new_x;
                fighter.y = new_y;
            }
        }
    }

    fn resolve_combat(&mut self) {
        // Clear combat buffer
        for v in self.combat_buf.iter_mut() {
            *v = 0;
        }

        // Count fighters per team per cell
        for fighter in &self.fighters {
            let idx = fighter.y as usize * MAP_WIDTH as usize + fighter.x as usize;
            self.combat_buf[idx * MAX_PLAYERS + fighter.team as usize] += 1;
        }

        // Apply combat
        for fighter in self.fighters.iter_mut() {
            let idx = fighter.y as usize * MAP_WIDTH as usize + fighter.x as usize;
            let my_team = fighter.team as usize;

            let mut enemy_count = 0u16;
            for t in 0..MAX_PLAYERS {
                if t != my_team {
                    enemy_count += self.combat_buf[idx * MAX_PLAYERS + t];
                }
            }

            if enemy_count > 0 {
                let damage = (enemy_count as u8).min(ATTACK_DAMAGE).min(fighter.health);
                fighter.health = fighter.health.saturating_sub(damage);

                if fighter.health == 0 {
                    // Convert to dominant enemy team
                    let mut best_team = my_team;
                    let mut best_count = 0u16;
                    for t in 0..MAX_PLAYERS {
                        if t != my_team {
                            let c = self.combat_buf[idx * MAX_PLAYERS + t];
                            if c > best_count {
                                best_count = c;
                                best_team = t;
                            }
                        }
                    }
                    fighter.team = best_team as u8;
                    fighter.health = FIGHTER_MAX_HEALTH / 2;
                }
            }
        }
    }

    fn heal_fighters(&mut self) {
        for fighter in self.fighters.iter_mut() {
            if fighter.health < FIGHTER_MAX_HEALTH {
                // Check if no enemies at this cell
                let idx = fighter.y as usize * MAP_WIDTH as usize + fighter.x as usize;
                let my_team = fighter.team as usize;
                let mut has_enemy = false;
                for t in 0..MAX_PLAYERS {
                    if t != my_team && self.combat_buf[idx * MAX_PLAYERS + t] > 0 {
                        has_enemy = true;
                        break;
                    }
                }
                if !has_enemy {
                    fighter.health = (fighter.health + HEAL_AMOUNT).min(FIGHTER_MAX_HEALTH);
                }
            }
        }
    }

    fn update_counts(&mut self) {
        for p in self.players.iter_mut() {
            p.fighter_count = 0;
        }
        for fighter in &self.fighters {
            let t = fighter.team as usize;
            if t < MAX_PLAYERS {
                self.players[t].fighter_count += 1;
            }
        }
    }

    /// Produce per-cell bitmap. Each byte:
    /// 0xFE = wall, 0xFF = empty, else (team << 4) | health
    pub fn get_bitmap(&self) -> Vec<u8> {
        let size = (MAP_WIDTH * MAP_HEIGHT) as usize;
        let mut buf = vec![0xFFu8; size];

        // Mark walls
        for i in 0..size {
            if self.map.cells[i] == crate::map::Cell::Wall {
                buf[i] = 0xFE;
            }
        }

        // Mark fighters (last writer wins per cell — good enough)
        for fighter in &self.fighters {
            let idx = fighter.y as usize * MAP_WIDTH as usize + fighter.x as usize;
            buf[idx] = ((fighter.team & 0x0F) << 4) | (fighter.health & 0x0F);
        }

        buf
    }

    pub fn get_scores(&self) -> [u32; MAX_PLAYERS] {
        let mut scores = [0u32; MAX_PLAYERS];
        for (i, p) in self.players.iter().enumerate() {
            scores[i] = p.fighter_count;
        }
        scores
    }

    pub fn get_cursors(&self) -> [(u32, u32, bool); MAX_PLAYERS] {
        let mut cursors = [(0u32, 0u32, false); MAX_PLAYERS];
        for (i, p) in self.players.iter().enumerate() {
            cursors[i] = (p.cursor_x, p.cursor_y, p.active);
        }
        cursors
    }
}
