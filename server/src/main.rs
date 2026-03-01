use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use game::constants::*;
use game::game::GameState;
use game::map::Map;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Duration};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

/// LW5 threshold: 6*R + 3*G + B > 315 means passable (light)
fn load_map_from_png(path: &Path) -> Map {
    let img = image::open(path).expect("Failed to load map image");
    let rgb = img.to_rgb8();
    let width = rgb.width();
    let height = rgb.height();

    let mut passable = vec![false; (width * height) as usize];
    for (i, pixel) in rgb.pixels().enumerate() {
        let r = pixel[0] as u32;
        let g = pixel[1] as u32;
        let b = pixel[2] as u32;
        passable[i] = (6 * r + 3 * g + b) > 315;
    }

    // Ensure border is wall
    for x in 0..width {
        passable[x as usize] = false;
        passable[((height - 1) * width + x) as usize] = false;
    }
    for y in 0..height {
        passable[(y * width) as usize] = false;
        passable[(y * width + width - 1) as usize] = false;
    }

    Map::new(width, height, passable)
}

#[derive(Deserialize, Serialize, Clone)]
struct MapInfo {
    id: String,
    name: String,
}

fn load_map_index(maps_dir: &Path) -> Vec<MapInfo> {
    let index_path = maps_dir.join("index.json");
    if index_path.exists() {
        if let Ok(data) = std::fs::read_to_string(&index_path) {
            if let Ok(entries) = serde_json::from_str::<Vec<MapInfo>>(&data) {
                return entries;
            }
        }
    }
    Vec::new()
}

#[derive(Deserialize)]
struct TeamSlotConfig {
    mode: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMsg {
    #[serde(rename = "join")]
    Join { name: String },
    #[serde(rename = "cursor")]
    Cursor { x: i32, y: i32 },
    #[serde(rename = "key_state")]
    KeyState { keys: u8 },
    #[serde(rename = "cursor_speed")]
    CursorSpeed { speed: u8 },
    #[serde(rename = "team_config")]
    TeamConfig { teams: Vec<TeamSlotConfig> },
    #[serde(rename = "select_map")]
    SelectMap { id: String },
    #[serde(rename = "start_game")]
    StartGame,
}

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
enum ServerMsg {
    #[serde(rename = "map_list")]
    MapList { maps: Vec<MapInfo> },
    #[serde(rename = "welcome")]
    Welcome {
        #[serde(rename = "playerId")]
        player_id: usize,
        #[serde(rename = "mapWidth")]
        map_width: u32,
        #[serde(rename = "mapHeight")]
        map_height: u32,
        #[serde(rename = "mapData")]
        map_data: Vec<u8>,
        #[serde(rename = "mapId")]
        map_id: String,
    },
    #[serde(rename = "state")]
    State {
        tick: i32,
        bitmap: String,
        scores: [u32; NB_TEAMS],
        cursors: Vec<Option<(i32, i32)>>,
    },
    #[serde(rename = "player_joined")]
    PlayerJoined {
        #[serde(rename = "playerId")]
        player_id: usize,
        name: String,
    },
    #[serde(rename = "player_left")]
    PlayerLeft {
        #[serde(rename = "playerId")]
        player_id: usize,
    },
}

struct PlayerSlot {
    id: usize,
    name: String,
    tx: mpsc::UnboundedSender<ServerMsg>,
}

struct Room {
    game: Option<GameState>,
    players: HashMap<usize, PlayerSlot>,
    map_data: Vec<u8>,
    map_index: Vec<MapInfo>,
    maps_dir: PathBuf,
    current_map_id: String,
    team_config: Vec<TeamSlotConfig>,
    cpu_teams: Vec<usize>,
}

impl Room {
    fn new(maps_dir: PathBuf) -> Self {
        let map_index = load_map_index(&maps_dir);
        Room {
            game: None,
            players: HashMap::new(),
            map_data: Vec::new(),
            map_index,
            maps_dir,
            current_map_id: String::new(),
            team_config: Vec::new(),
            cpu_teams: Vec::new(),
        }
    }

    fn load_map(&mut self, map_id: &str) {
        let map_path = self.maps_dir.join(format!("{map_id}.png"));
        let map = if map_path.exists() {
            load_map_from_png(&map_path)
        } else {
            Map::with_obstacles(MAP_WIDTH, MAP_HEIGHT)
        };

        println!(
            "Loaded map: {} ({}x{}, {} passable)",
            map_id,
            map.width,
            map.height,
            map.passable_count()
        );

        self.map_data = map
            .passable
            .iter()
            .map(|&p| if p { 0u8 } else { 1u8 })
            .collect();
        self.current_map_id = map_id.to_string();
        let mut game = GameState::new(map);

        // Add all current players
        let mut ids: Vec<usize> = self.players.keys().cloned().collect();
        ids.sort();
        for id in ids {
            game.add_player(id);
        }

        self.game = Some(game);
    }

    fn add_player(&mut self, tx: mpsc::UnboundedSender<ServerMsg>) -> Option<(usize, ServerMsg)> {
        // Find first available ID
        let id = (0..MAX_PLAYERS).find(|i| !self.players.contains_key(i));
        let id = match id {
            Some(id) => id,
            None => return None,
        };

        self.players.insert(
            id,
            PlayerSlot {
                id,
                name: String::new(),
                tx,
            },
        );

        // If game is running, add player to it
        if let Some(ref mut game) = self.game {
            game.add_player(id);

            let welcome = ServerMsg::Welcome {
                player_id: id,
                map_width: game.map_width(),
                map_height: game.map_height(),
                map_data: self.map_data.clone(),
                map_id: self.current_map_id.clone(),
            };

            Some((id, welcome))
        } else {
            // No game yet — send map list so client can choose
            let msg = ServerMsg::MapList {
                maps: self.map_index.clone(),
            };
            Some((id, msg))
        }
    }

    fn remove_player(&mut self, id: usize) {
        self.players.remove(&id);
        let msg = ServerMsg::PlayerLeft { player_id: id };
        for slot in self.players.values() {
            let _ = slot.tx.send(msg.clone());
        }
    }

    fn start_game(&mut self, map_id: &str) {
        // Load the map
        let map_path = self.maps_dir.join(format!("{map_id}.png"));
        let map = if map_path.exists() {
            load_map_from_png(&map_path)
        } else {
            Map::with_obstacles(MAP_WIDTH, MAP_HEIGHT)
        };

        println!(
            "Starting game on map: {} ({}x{}, {} passable)",
            map_id,
            map.width,
            map.height,
            map.passable_count()
        );

        self.map_data = map
            .passable
            .iter()
            .map(|&p| if p { 0u8 } else { 1u8 })
            .collect();
        self.current_map_id = map_id.to_string();
        let mut game = GameState::new(map);

        // Determine which teams to add based on team config
        self.cpu_teams.clear();
        if !self.team_config.is_empty() {
            for (i, tc) in self.team_config.iter().enumerate() {
                if i >= NB_TEAMS { break; }
                match tc.mode.as_str() {
                    "human" => { game.add_player(i); }
                    "cpu" => {
                        game.add_player(i);
                        self.cpu_teams.push(i);
                    }
                    _ => {} // "off"
                }
            }
        } else {
            // Default: add all connected players + 1 CPU opponent
            let mut ids: Vec<usize> = self.players.keys().cloned().collect();
            ids.sort();
            for id in &ids {
                game.add_player(*id);
            }
            // Add CPU as team after last human
            let cpu_team = ids.last().map(|&id| id + 1).unwrap_or(1);
            if cpu_team < NB_TEAMS {
                game.add_player(cpu_team);
                self.cpu_teams.push(cpu_team);
            }
        }

        self.game = Some(game);

        // Send welcome to all players
        if let Some(ref game) = self.game {
            for (id, slot) in &self.players {
                let welcome = ServerMsg::Welcome {
                    player_id: *id,
                    map_width: game.map_width(),
                    map_height: game.map_height(),
                    map_data: self.map_data.clone(),
                    map_id: self.current_map_id.clone(),
                };
                let _ = slot.tx.send(welcome);
            }
        }
    }

    fn set_cursor_speed(&mut self, speed: u8) {
        if let Some(ref mut game) = self.game {
            game.cursor_speed = speed.clamp(1, 5) as i32;
        }
    }

    fn set_cursor(&mut self, player_id: usize, x: i32, y: i32) {
        if let Some(ref mut game) = self.game {
            game.set_cursor(player_id, x, y);
        }
    }

    fn set_key_state(&mut self, player_id: usize, keys: u8) {
        if let Some(ref mut game) = self.game {
            game.set_key_state(player_id, keys);
        }
    }

    fn tick(&mut self) {
        if let Some(ref mut game) = self.game {
            // Simple CPU AI: change direction randomly every ~20 ticks
            for &cpu_team in &self.cpu_teams {
                if game.global_clock % 20 == (cpu_team as i32 * 7) % 20 {
                    // Pick a random direction (1=up, 2=right, 4=down, 8=left, or combos)
                    let directions: [u8; 8] = [1, 2, 4, 8, 3, 6, 9, 12];
                    let idx = ((game.global_clock as usize).wrapping_mul(cpu_team + 1).wrapping_add(cpu_team * 37)) % directions.len();
                    game.set_key_state(cpu_team, directions[idx]);
                }
            }

            game.tick();

            let bitmap = game.get_bitmap();
            let bitmap_b64 = base64::engine::general_purpose::STANDARD.encode(&bitmap);
            let scores = game.get_scores();
            let cursor_data = game.get_cursors();

            let cursors: Vec<Option<(i32, i32)>> = cursor_data
                .iter()
                .map(|(x, y, active)| if *active { Some((*x, *y)) } else { None })
                .collect();

            let msg = ServerMsg::State {
                tick: game.global_clock,
                bitmap: bitmap_b64,
                scores,
                cursors,
            };

            for slot in self.players.values() {
                let _ = slot.tx.send(msg.clone());
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3001";
    let maps_dir = PathBuf::from("public/maps");

    let mut room = Room::new(maps_dir);

    // If map specified on command line, start game immediately
    if let Some(map_id) = std::env::args().nth(1) {
        room.load_map(&map_id);
    }

    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("Liquid War server listening on ws://{addr}");

    let room = Arc::new(Mutex::new(room));

    // Game loop
    let room_tick = room.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(50)); // 20 Hz
        loop {
            ticker.tick().await;
            let mut room = room_tick.lock().await;
            if !room.players.is_empty() {
                room.tick();
            }
        }
    });

    // Accept connections
    while let Ok((stream, addr)) = listener.accept().await {
        let room = room.clone();
        tokio::spawn(async move {
            let ws_stream = match accept_async(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("WebSocket handshake failed for {addr}: {e}");
                    return;
                }
            };

            let (mut ws_tx, mut ws_rx) = ws_stream.split();
            let (tx, mut rx) = mpsc::unbounded_channel::<ServerMsg>();

            // Register player
            let player_id;
            {
                let mut room = room.lock().await;
                match room.add_player(tx.clone()) {
                    Some((id, msg)) => {
                        player_id = id;
                        let text = serde_json::to_string(&msg).unwrap();
                        if ws_tx.send(Message::Text(text.into())).await.is_err() {
                            return;
                        }
                        println!("Player {id} connected from {addr}");
                    }
                    None => {
                        let _ = ws_tx.send(Message::Close(None)).await;
                        return;
                    }
                }
            }

            // Forward server messages to WebSocket
            let send_task = tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    let text = serde_json::to_string(&msg).unwrap();
                    if ws_tx.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
            });

            // Read client messages
            while let Some(Ok(msg)) = ws_rx.next().await {
                if let Message::Text(text) = msg {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&text) {
                        let mut room = room.lock().await;
                        match client_msg {
                            ClientMsg::Join { name } => {
                                if let Some(slot) = room.players.get_mut(&player_id) {
                                    slot.name = name.clone();
                                }
                                let msg = ServerMsg::PlayerJoined {
                                    player_id,
                                    name,
                                };
                                for slot in room.players.values() {
                                    if slot.id != player_id {
                                        let _ = slot.tx.send(msg.clone());
                                    }
                                }
                            }
                            ClientMsg::Cursor { x, y } => {
                                room.set_cursor(player_id, x, y);
                            }
                            ClientMsg::KeyState { keys } => {
                                room.set_key_state(player_id, keys);
                            }
                            ClientMsg::CursorSpeed { speed } => {
                                if player_id == 0 {
                                    room.set_cursor_speed(speed);
                                }
                            }
                            ClientMsg::TeamConfig { teams } => {
                                if player_id == 0 {
                                    room.team_config = teams;
                                }
                            }
                            ClientMsg::SelectMap { id } => {
                                // Player 0 (host) can select map
                                if player_id == 0 {
                                    room.start_game(&id);
                                }
                            }
                            ClientMsg::StartGame => {
                                if player_id == 0 && room.game.is_none() {
                                    // Default map if none selected
                                    room.start_game("rect");
                                }
                            }
                        }
                    }
                }
            }

            // Disconnect
            {
                let mut room = room.lock().await;
                room.remove_player(player_id);
                println!("Player {player_id} disconnected");
            }
            send_task.abort();
        });
    }
}
