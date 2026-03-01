use crate::constants::*;

#[derive(Clone, Copy)]
pub struct Fighter {
    pub x: i16,
    pub y: i16,
    pub health: i16,
    pub team: i8,
    pub last_dir: i8,
}

impl Fighter {
    pub fn new(x: i16, y: i16, team: i8, health: i16) -> Self {
        let last_dir = ((y as i32 * 320 + x as i32) % NB_DIRS as i32) as i8;
        Fighter {
            x,
            y,
            health,
            team,
            last_dir,
        }
    }
}

/// Per-cell lookup: which mesh node covers this pixel, and which fighter occupies it.
#[derive(Clone, Copy)]
pub struct Place {
    pub mesh_idx: i32,    // -1 = wall, else index into mesh array
    pub fighter_idx: i32, // -1 = empty, else index into fighter array
}

impl Place {
    pub fn new() -> Self {
        Place {
            mesh_idx: -1,
            fighter_idx: -1,
        }
    }
}

/// fixsqrt: Allegro-compatible 16.16 fixed-point square root.
/// Input and output are both 16.16 fixed-point integers.
pub fn fixsqrt(x: i32) -> i32 {
    if x <= 0 {
        return 0;
    }
    ((x as f64 / 65536.0).sqrt() * 65536.0) as i32
}

/// Compute attack/defense/new_health values per team, matching LW5 fighter.c lines 321-370.
pub fn compute_combat_params(
    active_fighters: &[i32; NB_TEAMS],
    playing_teams: usize,
    total_army_size: i32,
    fighter_attack: u32,
    fighter_defense: u32,
    fighter_new_health: u32,
    number_influence: i32,
) -> ([i32; NB_TEAMS], [i32; NB_TEAMS], [i32; NB_TEAMS]) {
    let mut attack = [1i32; NB_TEAMS];
    let mut defense = [1i32; NB_TEAMS];
    let mut new_health = [1i32; NB_TEAMS];

    if total_army_size <= 0 || playing_teams == 0 {
        return (attack, defense, new_health);
    }

    for i in 0..playing_teams {
        // coef: number-influence scaling factor
        let mut coef: i32 = active_fighters[i] * playing_teams as i32 - total_army_size;
        coef *= 256;
        coef /= total_army_size;
        if coef > 256 {
            coef = 256;
        }

        coef *= (number_influence - 8) * (number_influence - 8);
        coef /= 64;
        if number_influence < 8 {
            coef = -coef;
        }
        if coef < 0 {
            coef /= 2;
        }
        coef += 256;

        // cpu_influence is 0 for human players (we don't implement CPU players yet)
        let cpu_influence: u32 = 0;

        attack[i] = (coef as i64
            * fixsqrt(fixsqrt(1i32 << (fighter_attack + cpu_influence))) as i64
            / (256 * 8)) as i32;
        if attack[i] >= MAX_FIGHTER_HEALTH as i32 {
            attack[i] = MAX_FIGHTER_HEALTH as i32 - 1;
        }
        if attack[i] < 1 {
            attack[i] = 1;
        }

        defense[i] = (coef as i64
            * fixsqrt(fixsqrt(1i32 << (fighter_defense + cpu_influence))) as i64
            / (256 * 256)) as i32;
        if defense[i] >= MAX_FIGHTER_HEALTH as i32 {
            defense[i] = MAX_FIGHTER_HEALTH as i32 - 1;
        }
        if defense[i] < 1 {
            defense[i] = 1;
        }

        new_health[i] = (coef as i64
            * fixsqrt(fixsqrt(1i32 << (fighter_new_health + cpu_influence))) as i64
            / (256 * 4)) as i32;
        if new_health[i] >= MAX_FIGHTER_HEALTH as i32 {
            new_health[i] = MAX_FIGHTER_HEALTH as i32 - 1;
        }
        if new_health[i] < 1 {
            new_health[i] = 1;
        }
    }

    (attack, defense, new_health)
}
