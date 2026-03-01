use game::constants::*;
use game::game::GameState;
use game::map::Map;
use js_sys::Uint8Array;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static GAME: RefCell<Option<GameState>> = RefCell::new(None);
}

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn map_width() -> u32 {
    MAP_WIDTH
}

#[wasm_bindgen]
pub fn map_height() -> u32 {
    MAP_HEIGHT
}

/// Create a new game with default obstacle map. Call add_player() after.
#[wasm_bindgen]
pub fn create_game() {
    let map = Map::with_obstacles(MAP_WIDTH, MAP_HEIGHT);
    GAME.with(|g| *g.borrow_mut() = Some(GameState::new(map)));
}

#[wasm_bindgen]
pub fn add_player(player_id: u32, total_teams: u32) {
    GAME.with(|g| {
        if let Some(ref mut state) = *g.borrow_mut() {
            state.add_player(player_id as usize, total_teams as usize);
        }
    });
}

#[wasm_bindgen]
pub fn set_cursor(player_id: u32, x: i32, y: i32) {
    GAME.with(|g| {
        if let Some(ref mut state) = *g.borrow_mut() {
            state.set_cursor(player_id as usize, x, y);
        }
    });
}

#[wasm_bindgen]
pub fn tick() {
    GAME.with(|g| {
        if let Some(ref mut state) = *g.borrow_mut() {
            state.tick();
        }
    });
}

#[wasm_bindgen]
pub fn get_bitmap() -> Uint8Array {
    GAME.with(|g| {
        let borrow = g.borrow();
        let state = borrow.as_ref().unwrap();
        let bitmap = state.get_bitmap();
        let arr = Uint8Array::new_with_length(bitmap.len() as u32);
        arr.copy_from(&bitmap);
        arr
    })
}

#[wasm_bindgen]
pub fn get_map_data() -> Uint8Array {
    GAME.with(|g| {
        let borrow = g.borrow();
        let state = borrow.as_ref().unwrap();
        let data: Vec<u8> = state.map.passable.iter().map(|&p| if p { 0u8 } else { 1u8 }).collect();
        let arr = Uint8Array::new_with_length(data.len() as u32);
        arr.copy_from(&data);
        arr
    })
}
