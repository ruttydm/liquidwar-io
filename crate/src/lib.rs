use js_sys::Float32Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Initialize a flat grid of particles. Returns a Float32Array of [x, y, z, ...] positions.
#[wasm_bindgen]
pub fn init_particle_grid(count: u32, spacing: f32) -> Float32Array {
    let grid_size = (count as f32).sqrt() as u32;
    let mut positions = Vec::with_capacity((count * 3) as usize);

    for i in 0..grid_size {
        for j in 0..grid_size {
            positions.push(i as f32 * spacing); // x
            positions.push(0.0); // y
            positions.push(j as f32 * spacing); // z
        }
    }

    let array = Float32Array::new_with_length(positions.len() as u32);
    array.copy_from(&positions);
    array
}

/// Update particle Y-positions with a multi-wave pattern. Mutates the buffer in place.
#[wasm_bindgen]
pub fn update_particle_positions(positions: &Float32Array, count: u32, time: f32) {
    let len = (count * 3) as usize;
    let mut data = vec![0.0f32; len];
    positions.copy_to(&mut data);

    let grid_size = (count as f32).sqrt() as u32;
    let half = grid_size as f32 / 2.0;

    for i in 0..count {
        let idx = (i * 3) as usize;
        let x = data[idx];
        let z = data[idx + 2];

        let dist = ((x - half).powi(2) + (z - half).powi(2)).sqrt();
        let y = (dist * 0.3 + time * 2.0).sin() * 2.0
            + (x * 0.5 + time * 1.5).cos() * 1.0
            + (z * 0.4 - time).sin() * 0.8;

        data[idx + 1] = y;
    }

    positions.copy_from(&data);
}
