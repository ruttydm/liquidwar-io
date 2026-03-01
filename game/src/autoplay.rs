use crate::constants::*;
use crate::cursor::{CURSOR_KEY_UP, CURSOR_KEY_RIGHT, CURSOR_KEY_DOWN, CURSOR_KEY_LEFT};
use crate::game::GameState;

const RANDOM_LIMIT: usize = 10000;

/// Bot AI. Picks a nearby enemy fighter, moves the cursor toward it while
/// course-correcting, then pauses at the target before picking a new one.
pub struct ComputerAI {
    target_x: [i32; NB_TEAMS],
    target_y: [i32; NB_TEAMS],
    /// Ticks of active movement remaining.
    move_left: [i32; NB_TEAMS],
    /// Ticks of pause remaining (cursor holds position).
    wait_left: [i32; NB_TEAMS],
    rng: u64,
}

impl ComputerAI {
    pub fn new() -> Self {
        ComputerAI {
            target_x: [0; NB_TEAMS],
            target_y: [0; NB_TEAMS],
            move_left: [0; NB_TEAMS],
            wait_left: [0; NB_TEAMS],
            rng: 12345,
        }
    }

    fn next_rand(&mut self) -> u64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        self.rng
    }

    pub fn get_next_move(&mut self, game: &GameState, team: usize) -> u8 {
        // Pausing at target — hold position so fighters converge
        if self.wait_left[team] > 0 {
            self.wait_left[team] -= 1;
            return 0;
        }

        // Still moving toward target — recalculate direction each tick
        if self.move_left[team] > 0 {
            let cx = game.cursors[team].x;
            let cy = game.cursors[team].y;
            let tx = self.target_x[team];
            let ty = self.target_y[team];

            // Close enough to target — start pause
            if (cx - tx).abs() <= 2 && (cy - ty).abs() <= 2 {
                self.move_left[team] = 0;
                let pause = 40 + (self.next_rand() % 80) as i32;
                self.wait_left[team] = pause;
                return 0;
            }

            self.move_left[team] -= 1;

            // move_left just hit 0 without reaching target — start pause
            if self.move_left[team] == 0 {
                let pause = 40 + (self.next_rand() % 80) as i32;
                self.wait_left[team] = pause;
            }

            let mut keys: u8 = 0;
            if cy > ty { keys |= CURSOR_KEY_UP; }
            if cy < ty { keys |= CURSOR_KEY_DOWN; }
            if cx < tx { keys |= CURSOR_KEY_RIGHT; }
            if cx > tx { keys |= CURSOR_KEY_LEFT; }
            return keys;
        }

        // Pick a new target
        if let Some((tx, ty)) = self.random_enemy_fighter_pos(game, team) {
            self.target_x[team] = tx;
            self.target_y[team] = ty;

            let cx = game.cursors[team].x;
            let cy = game.cursors[team].y;
            let dx = (tx - cx).unsigned_abs() as i32;
            let dy = (ty - cy).unsigned_abs() as i32;
            let dist = dx.max(dy).min(100);
            // Move for enough ticks to reach target, plus a small buffer
            self.move_left[team] = dist + 5;
            // Pause will be set when we arrive

            let mut keys: u8 = 0;
            if cy > ty { keys |= CURSOR_KEY_UP; }
            if cy < ty { keys |= CURSOR_KEY_DOWN; }
            if cx < tx { keys |= CURSOR_KEY_RIGHT; }
            if cx > tx { keys |= CURSOR_KEY_LEFT; }
            return keys;
        }

        0
    }

    /// Pick a nearby enemy fighter with some variation. Samples random enemies,
    /// keeps the closest few, then picks one at random from those.
    fn random_enemy_fighter_pos(&mut self, game: &GameState, team: usize) -> Option<(i32, i32)> {
        let army_size = game.fighters.len();
        if army_size == 0 {
            return None;
        }

        let cx = game.cursors[team].x;
        let cy = game.cursors[team].y;

        // Collect up to 20 enemy samples with their distances
        let mut candidates: [(i32, i32, i32); 20] = [(0, 0, i32::MAX); 20];
        let mut count = 0usize;

        for _ in 0..RANDOM_LIMIT {
            let i = (self.next_rand() as usize) % army_size;
            let f = &game.fighters[i];
            if f.health > 0 && f.team != team as i8 {
                let dx = (f.x as i32 - cx).abs();
                let dy = (f.y as i32 - cy).abs();
                let dist = dx + dy;
                if count < 20 {
                    candidates[count] = (f.x as i32, f.y as i32, dist);
                    count += 1;
                }
                if count >= 20 {
                    break;
                }
            }
        }

        if count == 0 {
            return None;
        }

        // Sort by distance, pick randomly from the closest 5
        candidates[..count].sort_unstable_by_key(|c| c.2);
        let pool = count.min(5);
        let pick = (self.next_rand() as usize) % pool;
        Some((candidates[pick].0, candidates[pick].1))
    }
}
