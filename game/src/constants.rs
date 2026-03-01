pub const NB_TEAMS: usize = 32;
pub const NB_DIRS: usize = 12;
pub const MAX_FIGHTER_HEALTH: i16 = 16384;
pub const AREA_START_GRADIENT: i32 = 2_000_000;

// Direction indices
pub const DIR_NNE: usize = 0;
pub const DIR_NE: usize = 1;
pub const DIR_ENE: usize = 2;
pub const DIR_ESE: usize = 3;
pub const DIR_SE: usize = 4;
pub const DIR_SSE: usize = 5;
pub const DIR_SSW: usize = 6;
pub const DIR_SW: usize = 7;
pub const DIR_WSW: usize = 8;
pub const DIR_WNW: usize = 9;
pub const DIR_NW: usize = 10;
pub const DIR_NNW: usize = 11;

// Direction to pixel offsets (from LW5 fighter.c)
pub const DIR_X: [i32; NB_DIRS] = [0, 1, 1, 1, 1, 0, 0, -1, -1, -1, -1, 0];
pub const DIR_Y: [i32; NB_DIRS] = [-1, -1, 0, 0, 1, 1, 1, 1, 0, 0, -1, -1];

// Directions that iterate forward vs backward in gradient spreading
// Forward (SE half): ENE, ESE, SE, SSE, SSW, SW
// Backward (NW half): WSW, WNW, NW, NNW, NNE, NE
pub const DIR_IS_FORWARD: [bool; NB_DIRS] = [
    false, // NNE
    false, // NE
    true,  // ENE
    true,  // ESE
    true,  // SE
    true,  // SSE
    true,  // SSW
    true,  // SW
    false, // WSW
    false, // WNW
    false, // NW
    false, // NNW
];

// Movement direction priority tables (from LW5 fighter.c)
// MOVE_DIR[sens][dir][priority] = direction to try
pub const MOVE_DIR: [[[usize; 5]; NB_DIRS]; 2] = [
    // Sense 0 (counterclockwise preference)
    [
        [DIR_NNE, DIR_NE, DIR_NW, DIR_ENE, DIR_WNW],   // from NNE
        [DIR_NE, DIR_ENE, DIR_NNE, DIR_SE, DIR_NW],     // from NE
        [DIR_ENE, DIR_NE, DIR_SE, DIR_NNE, DIR_SSE],    // from ENE
        [DIR_ESE, DIR_SE, DIR_NE, DIR_SSE, DIR_NNE],    // from ESE
        [DIR_SE, DIR_SSE, DIR_ESE, DIR_SW, DIR_NE],     // from SE
        [DIR_SSE, DIR_SE, DIR_SW, DIR_ESE, DIR_WSW],    // from SSE
        [DIR_SSW, DIR_SW, DIR_SE, DIR_WSW, DIR_ESE],    // from SSW
        [DIR_SW, DIR_WSW, DIR_SSW, DIR_NW, DIR_SE],     // from SW
        [DIR_WSW, DIR_SW, DIR_NW, DIR_SSW, DIR_NNW],    // from WSW
        [DIR_WNW, DIR_NW, DIR_SW, DIR_NNW, DIR_SSE],    // from WNW
        [DIR_NW, DIR_NNW, DIR_WNW, DIR_NE, DIR_SW],     // from NW
        [DIR_NNW, DIR_NW, DIR_NE, DIR_WNW, DIR_ENE],    // from NNW
    ],
    // Sense 1 (clockwise preference)
    [
        [DIR_NNE, DIR_NE, DIR_NW, DIR_ENE, DIR_WNW],    // from NNE
        [DIR_NE, DIR_NNE, DIR_ENE, DIR_NW, DIR_SE],     // from NE
        [DIR_ENE, DIR_NE, DIR_SE, DIR_NNE, DIR_SSE],    // from ENE
        [DIR_ESE, DIR_SE, DIR_NE, DIR_SSE, DIR_NNE],    // from ESE
        [DIR_SE, DIR_ESE, DIR_SSE, DIR_NE, DIR_SW],     // from SE
        [DIR_SSE, DIR_SE, DIR_SW, DIR_ESE, DIR_WSW],    // from SSE
        [DIR_SSW, DIR_SW, DIR_SE, DIR_WSW, DIR_ESE],    // from SSW
        [DIR_SW, DIR_SSW, DIR_WSW, DIR_SE, DIR_NW],     // from SW
        [DIR_WSW, DIR_SW, DIR_NW, DIR_SSW, DIR_NNW],    // from WSW
        [DIR_WNW, DIR_NW, DIR_SW, DIR_NNW, DIR_SSE],    // from WNW
        [DIR_NW, DIR_WNW, DIR_NNW, DIR_SW, DIR_NE],     // from NW
        [DIR_NNW, DIR_NW, DIR_NE, DIR_WNW, DIR_ENE],    // from NNW
    ],
];

// LOCAL_DIR table for cursor-relative direction (from LW5 fighter.c)
// Maps (code_dir - 1) * 2 + sens to a direction index
// code_dir bits: 1=above, 2=right, 4=below, 8=left
pub const LOCAL_DIR: [usize; 30] = [
    DIR_NNW, DIR_NNE, // code 1: above
    DIR_ENE, DIR_ESE, // code 2: right
    DIR_NE, DIR_NE,   // code 3: above-right
    DIR_SSE, DIR_SSW, // code 4: below
    0, 0,             // code 5: above+below (invalid)
    DIR_SE, DIR_SE,   // code 6: below-right
    0, 0,             // code 7: (invalid)
    DIR_WSW, DIR_WNW, // code 8: left
    DIR_NW, DIR_NW,   // code 9: above-left
    0, 0,             // code 10: (invalid)
    0, 0,             // code 11: (invalid)
    DIR_SW, DIR_SW,   // code 12: below-left
    0, 0,             // code 13: (invalid)
    0, 0,             // code 14: (invalid)
    0, 0,             // code 15: (invalid)
];

// Army fill table (from LW5 army.c) — maps config slider 0-32 to fill percentage
pub const FILL_TABLE: [u32; 33] = [
    1, 2, 3, 4, 5, 6, 8, 9, 10, 12, 14, 16, 18, 20, 22, 24, 25, 27, 29, 31, 33, 36, 40, 45, 50,
    55, 60, 65, 70, 75, 80, 90, 99,
];

// Side attack factor: side attacks do damage >> 4 (1/16th of front)
pub const SIDE_ATTACK_FACTOR: i32 = 4;

// Default config values matching LW5
pub const DEFAULT_FIGHTER_NUMBER: usize = 16; // index into FILL_TABLE
pub const DEFAULT_FIGHTER_ATTACK: u32 = 8;
pub const DEFAULT_FIGHTER_DEFENSE: u32 = 8;
pub const DEFAULT_FIGHTER_NEW_HEALTH: u32 = 8;
pub const DEFAULT_NUMBER_INFLUENCE: i32 = 8;

// Map defaults
pub const MAP_WIDTH: u32 = 320;
pub const MAP_HEIGHT: u32 = 200;
pub const MAX_PLAYERS: usize = NB_TEAMS;

// Bitmap encoding constants for get_bitmap()/renderer
pub const BITMAP_EMPTY: u8 = 0;
pub const BITMAP_WALL: u8 = 254;
pub const BITMAP_HEALTH_LEVELS: u8 = 7;
