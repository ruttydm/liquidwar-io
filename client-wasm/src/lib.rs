use game::constants::*;
use game::game::GameState;
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

/// Create a new game. Call add_player() after.
#[wasm_bindgen]
pub fn create_game() {
    GAME.with(|g| *g.borrow_mut() = Some(GameState::new()));
}

#[wasm_bindgen]
pub fn add_player(player_id: u32) {
    GAME.with(|g| {
        if let Some(ref mut state) = *g.borrow_mut() {
            state.add_player(player_id as usize);
        }
    });
}

#[wasm_bindgen]
pub fn set_cursor(player_id: u32, x: u32, y: u32) {
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
        let data = state.map.to_bytes();
        let arr = Uint8Array::new_with_length(data.len() as u32);
        arr.copy_from(&data);
        arr
    })
}
