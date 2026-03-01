use crate::constants::*;
use crate::map::Map;

const MESH_MAX_ELEM_SIZE: i32 = 16;

/// Per-team state on each mesh node (matching LW5 MESH_INFO).
#[derive(Clone, Copy)]
pub struct MeshInfo {
    /// Gradient value: distance estimate to cursor. Lower = closer.
    /// Initialized to AREA_START_GRADIENT (2_000_000).
    pub grad: i32,
    /// Best direction toward cursor (0..11).
    pub dir: i8,
    /// Update time. Negative = -GLOBAL_CLOCK when direction was last computed.
    /// Non-negative = cursor is on this mesh cell (encodes cursor position).
    pub update_time: i32,
    /// Cursor position stored when cursor is on this cell.
    pub cursor_x: i16,
    pub cursor_y: i16,
}

impl MeshInfo {
    fn new() -> Self {
        MeshInfo {
            grad: AREA_START_GRADIENT,
            dir: 0,
            update_time: -1,
            cursor_x: 0,
            cursor_y: 0,
        }
    }
}

/// A mesh node in the hierarchical grid (matching LW5 MESH).
#[derive(Clone)]
pub struct MeshNode {
    /// Top-left pixel coordinate of this mesh element.
    pub x: i16,
    pub y: i16,
    /// Physical size in pixels (1, 2, 4, 8, or 16).
    pub size: i32,
    /// Per-team gradient/direction state.
    pub info: [MeshInfo; NB_TEAMS],
    /// Links to neighbor mesh nodes in 12 directions. -1 = no neighbor.
    pub link: [i32; NB_DIRS],
}

impl MeshNode {
    fn new(x: i16, y: i16, size: i32) -> Self {
        MeshNode {
            x,
            y,
            size,
            info: [MeshInfo::new(); NB_TEAMS],
            link: [-1; NB_DIRS],
        }
    }
}

/// Temporary construction node (matching LW5 MESHER).
struct Mesher {
    used: bool,
    size: i32,
    link: [i32; NB_DIRS], // indices into mesher array, -1 = no link
    corres: i32,          // maps to final mesh array index
}

pub struct Mesh {
    pub nodes: Vec<MeshNode>,
}

impl Mesh {
    /// Build the hierarchical mesh from a map, matching LW5 mesh.c exactly.
    pub fn build(map: &Map) -> Self {
        let w = map.width as i32;
        let h = map.height as i32;
        let total = (w * h) as usize;

        // Step 1: Create 1x1 mesher grid (create_first_mesher)
        let mut mesher: Vec<Mesher> = Vec::with_capacity(total);
        for i in 0..total {
            mesher.push(Mesher {
                used: map.passable[i],
                size: 1,
                link: [-1; NB_DIRS],
                corres: -1,
            });
        }

        // Link each passable interior pixel to its 8-connected neighbors.
        // Cardinals get 2 direction slots each, diagonals get 1.
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let i = (y * w + x) as usize;
                if !mesher[i].used {
                    continue;
                }

                let i_n = ((y - 1) * w + x) as usize;
                let i_ne = ((y - 1) * w + x + 1) as usize;
                let i_e = (y * w + x + 1) as usize;
                let i_se = ((y + 1) * w + x + 1) as usize;
                let i_s = ((y + 1) * w + x) as usize;
                let i_sw = ((y + 1) * w + x - 1) as usize;
                let i_w = (y * w + x - 1) as usize;
                let i_nw = ((y - 1) * w + x - 1) as usize;

                // North → NNW and NNE
                if mesher[i_n].used {
                    mesher[i].link[DIR_NNW] = i_n as i32;
                    mesher[i].link[DIR_NNE] = i_n as i32;
                }
                // NE diagonal
                if mesher[i_ne].used {
                    mesher[i].link[DIR_NE] = i_ne as i32;
                }
                // East → ENE and ESE
                if mesher[i_e].used {
                    mesher[i].link[DIR_ENE] = i_e as i32;
                    mesher[i].link[DIR_ESE] = i_e as i32;
                }
                // SE diagonal
                if mesher[i_se].used {
                    mesher[i].link[DIR_SE] = i_se as i32;
                }
                // South → SSE and SSW
                if mesher[i_s].used {
                    mesher[i].link[DIR_SSE] = i_s as i32;
                    mesher[i].link[DIR_SSW] = i_s as i32;
                }
                // SW diagonal
                if mesher[i_sw].used {
                    mesher[i].link[DIR_SW] = i_sw as i32;
                }
                // West → WSW and WNW
                if mesher[i_w].used {
                    mesher[i].link[DIR_WSW] = i_w as i32;
                    mesher[i].link[DIR_WNW] = i_w as i32;
                }
                // NW diagonal
                if mesher[i_nw].used {
                    mesher[i].link[DIR_NW] = i_nw as i32;
                }
            }
        }

        // Step 2: Hierarchical 2x2 merging (group_mesher)
        let mut step = 1i32;
        while step <= MESH_MAX_ELEM_SIZE / 2 {
            let merges = Self::group_mesher(&mut mesher, w, h, step);
            if merges == 0 {
                break;
            }
            step *= 2;
        }

        // Step 3: Compact into final mesh array (mesher_to_mesh)
        let mut count = 0usize;
        for m in &mesher {
            if m.used {
                count += 1;
            }
        }

        let mut nodes: Vec<MeshNode> = Vec::with_capacity(count);
        let mut j = 0i32;
        for i in 0..total {
            if mesher[i].used {
                let x = (i as i32 % w) as i16;
                let y = (i as i32 / w) as i16;
                let mut node = MeshNode::new(x, y, mesher[i].size);
                // Copy links (still mesher indices)
                node.link = mesher[i].link;
                // Initialize per-team direction: (node_idx + team) % NB_DIRS
                for t in 0..NB_TEAMS {
                    node.info[t].dir = ((j as usize + t) % NB_DIRS) as i8;
                    node.info[t].update_time = -1;
                }
                nodes.push(node);
                mesher[i].corres = j;
                j += 1;
            }
        }

        // Remap links from mesher indices to final node indices
        for node in &mut nodes {
            for k in 0..NB_DIRS {
                let link = node.link[k];
                if link >= 0 {
                    node.link[k] = mesher[link as usize].corres;
                }
            }
        }

        Mesh { nodes }
    }

    /// Try merging 2x2 blocks of `step`-sized mesher cells into `step*2` cells.
    /// Returns the number of merges performed.
    fn group_mesher(mesher: &mut [Mesher], w: i32, h: i32, step: i32) -> usize {
        let mut merge_count = 0;

        let mut y = 0;
        while y < h - step {
            let mut x = 0;
            while x < w - step {
                let i_nw = (y * w + x) as usize;
                let i_ne = (y * w + x + step) as usize;
                let i_sw = ((y + step) * w + x) as usize;
                let i_se = ((y + step) * w + x + step) as usize;

                // All four must be used and same size
                if !mesher[i_nw].used || !mesher[i_ne].used
                    || !mesher[i_sw].used || !mesher[i_se].used
                {
                    x += step * 2;
                    continue;
                }
                if mesher[i_nw].size != step || mesher[i_ne].size != step
                    || mesher[i_sw].size != step || mesher[i_se].size != step
                {
                    x += step * 2;
                    continue;
                }

                // Check uniform edges (cardinal double-links must match)
                if mesher[i_ne].link[DIR_NNW] != mesher[i_ne].link[DIR_NNE]
                    || mesher[i_ne].link[DIR_ENE] != mesher[i_ne].link[DIR_ESE]
                    || mesher[i_se].link[DIR_ENE] != mesher[i_se].link[DIR_ESE]
                    || mesher[i_se].link[DIR_SSE] != mesher[i_se].link[DIR_SSW]
                    || mesher[i_sw].link[DIR_SSE] != mesher[i_sw].link[DIR_SSW]
                    || mesher[i_sw].link[DIR_WSW] != mesher[i_sw].link[DIR_WNW]
                    || mesher[i_nw].link[DIR_WSW] != mesher[i_nw].link[DIR_WNW]
                    || mesher[i_nw].link[DIR_NNW] != mesher[i_nw].link[DIR_NNE]
                {
                    x += step * 2;
                    continue;
                }

                // All four diagonal links must exist
                if mesher[i_ne].link[DIR_NE] < 0
                    || mesher[i_se].link[DIR_SE] < 0
                    || mesher[i_sw].link[DIR_SW] < 0
                    || mesher[i_nw].link[DIR_NW] < 0
                {
                    x += step * 2;
                    continue;
                }

                // Merge: NW absorbs NE, SE, SW
                mesher[i_ne].used = false;
                mesher[i_se].used = false;
                mesher[i_sw].used = false;
                mesher[i_nw].size = step * 2;

                // Inherit outer links from constituent quadrants
                mesher[i_nw].link[DIR_NNE] = mesher[i_ne].link[DIR_NNE];
                mesher[i_nw].link[DIR_NE] = mesher[i_ne].link[DIR_NE];
                mesher[i_nw].link[DIR_ENE] = mesher[i_ne].link[DIR_ENE];
                mesher[i_nw].link[DIR_ESE] = mesher[i_se].link[DIR_ESE];
                mesher[i_nw].link[DIR_SE] = mesher[i_se].link[DIR_SE];
                mesher[i_nw].link[DIR_SSE] = mesher[i_se].link[DIR_SSE];
                mesher[i_nw].link[DIR_SSW] = mesher[i_sw].link[DIR_SSW];
                mesher[i_nw].link[DIR_SW] = mesher[i_sw].link[DIR_SW];
                mesher[i_nw].link[DIR_WSW] = mesher[i_sw].link[DIR_WSW];
                // DIR_WNW, DIR_NW, DIR_NNW stay from NW quadrant (already correct)

                // Backlink fixup: redirect any neighbor pointing to NE/SE/SW → NW
                let nw = i_nw as i32;
                let ne = i_ne as i32;
                let se = i_se as i32;
                let sw = i_sw as i32;

                for j in 0..NB_DIRS {
                    let neighbor_idx = mesher[i_nw].link[j];
                    if neighbor_idx >= 0 {
                        let ni = neighbor_idx as usize;
                        for k in 0..NB_DIRS {
                            let target = mesher[ni].link[k];
                            if target == ne || target == se || target == sw {
                                mesher[ni].link[k] = nw;
                            }
                        }
                    }
                }

                merge_count += 1;
                x += step * 2;
            }
            y += step * 2;
        }

        merge_count
    }

    /// Reset all gradients to AREA_START_GRADIENT (matching LW5 reset_game_area).
    pub fn reset_gradients(&mut self) {
        for node in &mut self.nodes {
            for t in 0..NB_TEAMS {
                node.info[t].grad = AREA_START_GRADIENT;
            }
        }
    }

    /// Reset per-team direction state (matching LW5 reset_mesh).
    pub fn reset_directions(&mut self) {
        for (i, node) in self.nodes.iter_mut().enumerate() {
            for j in 0..NB_TEAMS {
                node.info[j].dir = ((i + j) % NB_DIRS) as i8;
                node.info[j].update_time = -1;
            }
        }
    }

    /// Spread gradient in one direction (matching LW5 grad.c spread_single_gradient).
    /// Called once per game tick. Direction = (global_clock * 7) % 12.
    pub fn spread_gradient(&mut self, global_clock: i32, playing_teams: usize) {
        let dir = ((global_clock * 7) % NB_DIRS as i32) as usize;
        let n = self.nodes.len();
        if n == 0 {
            return;
        }

        if DIR_IS_FORWARD[dir] {
            // Forward iteration (directions ENE, ESE, SE, SSE, SSW, SW)
            for idx in 0..n {
                let link_idx = self.nodes[idx].link[dir];
                if link_idx >= 0 {
                    let size = self.nodes[idx].size;
                    for t in 0..playing_teams {
                        let new_grad = self.nodes[idx].info[t].grad + size;
                        if self.nodes[link_idx as usize].info[t].grad > new_grad {
                            self.nodes[link_idx as usize].info[t].grad = new_grad;
                        }
                    }
                }
            }
        } else {
            // Backward iteration (directions WSW, WNW, NW, NNW, NNE, NE)
            for idx in (0..n).rev() {
                let link_idx = self.nodes[idx].link[dir];
                if link_idx >= 0 {
                    let size = self.nodes[idx].size;
                    for t in 0..playing_teams {
                        let new_grad = self.nodes[idx].info[t].grad + size;
                        if self.nodes[link_idx as usize].info[t].grad > new_grad {
                            self.nodes[link_idx as usize].info[t].grad = new_grad;
                        }
                    }
                }
            }
        }
    }

    /// Get the total "battle room" — sum of size^2 for all mesh nodes.
    /// This represents the total usable area in terms of mesh coverage.
    pub fn battle_room(&self) -> i32 {
        let mut total = 0i32;
        for node in &self.nodes {
            total += node.size * node.size;
        }
        total
    }

    /// Find direction toward lowest gradient neighbor (matching LW5 get_main_dir).
    pub fn get_main_dir(&self, mesh_idx: usize, team: usize, sens: bool, start: usize) -> usize {
        let node = &self.nodes[mesh_idx];
        let mut dist = AREA_START_GRADIENT;
        let mut dir: i32 = -1;
        let mut i = start;

        if sens {
            // Ascending scan
            loop {
                let link = node.link[i];
                if link >= 0 {
                    let neighbor_grad = self.nodes[link as usize].info[team].grad;
                    if neighbor_grad < dist {
                        dir = i as i32;
                        dist = neighbor_grad;
                    }
                }
                i = if i < NB_DIRS - 1 { i + 1 } else { 0 };
                if i == start {
                    break;
                }
            }
        } else {
            // Descending scan
            loop {
                let link = node.link[i];
                if link >= 0 {
                    let neighbor_grad = self.nodes[link as usize].info[team].grad;
                    if neighbor_grad < dist {
                        dir = i as i32;
                        dist = neighbor_grad;
                    }
                }
                i = if i > 0 { i - 1 } else { NB_DIRS - 1 };
                if i == start {
                    break;
                }
            }
        }

        if dir >= 0 {
            dir as usize
        } else {
            // Fallback: GLOBAL_CLOCK % NB_TEAMS (matching LW5)
            0
        }
    }

    /// Get direction directly toward cursor position (matching LW5 get_close_dir).
    pub fn get_close_dir(
        &self,
        mesh_idx: usize,
        fighter_x: i16,
        fighter_y: i16,
        team: usize,
        sens: bool,
        start: usize,
    ) -> usize {
        let info = &self.nodes[mesh_idx].info[team];
        let cursor_x = info.cursor_x as i32;
        let cursor_y = info.cursor_y as i32;
        let fx = fighter_x as i32;
        let fy = fighter_y as i32;

        let mut code_dir = 0u32;
        if cursor_y < fy {
            code_dir += 1;
        }
        if cursor_x > fx {
            code_dir += 2;
        }
        if cursor_y > fy {
            code_dir += 4;
        }
        if cursor_x < fx {
            code_dir += 8;
        }

        if code_dir > 0 {
            let idx = ((code_dir - 1) * 2 + if sens { 1 } else { 0 }) as usize;
            if idx < LOCAL_DIR.len() {
                LOCAL_DIR[idx]
            } else {
                start
            }
        } else {
            start
        }
    }
}
