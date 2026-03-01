use js_sys::Float32Array;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

mod constants;
mod grid;
mod kernels;
mod particle;
mod sph;

thread_local! {
    static SIM: RefCell<Option<sph::SphSimulation>> = RefCell::new(None);
}

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Create a new SPH simulation. Returns actual particle count.
#[wasm_bindgen]
pub fn create_simulation(count: u32) -> u32 {
    let sim = sph::SphSimulation::new(count as usize);
    let actual_count = sim.particles.count as u32;
    SIM.with(|s| *s.borrow_mut() = Some(sim));
    actual_count
}

/// Advance simulation by one frame (multiple substeps).
#[wasm_bindgen]
pub fn step_simulation() {
    SIM.with(|s| {
        if let Some(ref mut sim) = *s.borrow_mut() {
            for _ in 0..constants::SUBSTEPS {
                sim.step();
            }
            sim.fill_buffers();
        }
    });
}

/// Get particle positions as Float32Array [x0, y0, z0, x1, y1, z1, ...].
#[wasm_bindgen]
pub fn get_positions() -> Float32Array {
    SIM.with(|s| {
        let borrow = s.borrow();
        let sim = borrow.as_ref().unwrap();
        let arr = Float32Array::new_with_length(sim.position_buffer.len() as u32);
        arr.copy_from(&sim.position_buffer);
        arr
    })
}

/// Get particle speeds as Float32Array [s0, s1, s2, ...].
#[wasm_bindgen]
pub fn get_speeds() -> Float32Array {
    SIM.with(|s| {
        let borrow = s.borrow();
        let sim = borrow.as_ref().unwrap();
        let arr = Float32Array::new_with_length(sim.speed_buffer.len() as u32);
        arr.copy_from(&sim.speed_buffer);
        arr
    })
}

/// Get particle count.
#[wasm_bindgen]
pub fn get_particle_count() -> u32 {
    SIM.with(|s| {
        let borrow = s.borrow();
        borrow.as_ref().map_or(0, |sim| sim.particles.count as u32)
    })
}
