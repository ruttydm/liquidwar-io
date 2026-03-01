use crate::constants::*;

/// Cursor state per team (matching LW5 CURSOR struct).
#[derive(Clone)]
pub struct Cursor {
    /// Gradient value poked into mesh; decreases each round.
    pub val: i32,
    /// Cursor position on the map.
    pub x: i32,
    pub y: i32,
    /// Whether this cursor is active.
    pub active: bool,
    /// Team index this cursor belongs to.
    pub team: usize,
    /// TIME_ELAPSED when this team lost (-1 if still playing).
    pub loose_time: i32,
    /// Finishing position (1st, 2nd, etc).
    pub score_order: i32,
    /// Bitmask of direction keys: 1=up, 2=right, 4=down, 8=left.
    pub key_state: u8,
}

pub const CURSOR_KEY_UP: u8 = 1;
pub const CURSOR_KEY_RIGHT: u8 = 2;
pub const CURSOR_KEY_DOWN: u8 = 4;
pub const CURSOR_KEY_LEFT: u8 = 8;

impl Cursor {
    pub fn new() -> Self {
        Cursor {
            val: 0,
            x: 0,
            y: 0,
            active: false,
            team: 0,
            loose_time: -1,
            score_order: 0,
            key_state: 0,
        }
    }

    pub fn init(&mut self, team: usize, x: i32, y: i32) {
        self.active = true;
        self.team = team;
        self.x = x;
        self.y = y;
        self.val = AREA_START_GRADIENT / 2; // = 1_000_000
        self.loose_time = -1;
        self.score_order = 0;
        self.key_state = 0;
    }
}
