use flate2::{write::DeflateEncoder, Compression};
use futures_util::{SinkExt, StreamExt};
use game::autoplay::ComputerAI;
use game::constants::*;
use game::game::GameState;
use game::map::Map;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Duration};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

/// Messages sent to a connection — either JSON (control) or binary (game state)
enum OutMsg {
    Json(ServerMsg),
    Binary(Vec<u8>),
}

// ---------------------------------------------------------------------------
// Map loading
// ---------------------------------------------------------------------------

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

#[derive(Deserialize, Serialize, Clone, Debug)]
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

// ---------------------------------------------------------------------------
// Room code generation
// ---------------------------------------------------------------------------

const CODE_CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const CODE_LEN: usize = 4;

fn generate_room_code(existing: &HashMap<String, Room>) -> String {
    let mut rng = rand::thread_rng();
    loop {
        let code: String = (0..CODE_LEN)
            .map(|_| CODE_CHARSET[rng.gen_range(0..CODE_CHARSET.len())] as char)
            .collect();
        if !existing.contains_key(&code) {
            return code;
        }
    }
}

// ---------------------------------------------------------------------------
// Protocol
// ---------------------------------------------------------------------------

#[derive(Deserialize, Clone, Debug)]
struct TeamSlotConfig {
    mode: String,  // "human", "cpu", "off"
    name: String,
}

#[derive(Deserialize, Clone, Debug)]
struct GameConfig {
    teams: Vec<TeamSlotConfig>,
    #[serde(default = "default_8")]
    fighter_attack: u8,
    #[serde(default = "default_8")]
    fighter_defense: u8,
    #[serde(default = "default_8")]
    fighter_new_health: u8,
    #[serde(default = "default_8")]
    number_influence: u8,
    #[serde(default = "default_16")]
    fighter_number: u8,
    #[serde(default = "default_1")]
    cursor_speed: u8,
}

fn default_8() -> u8 { 8 }
fn default_16() -> u8 { 16 }
fn default_1() -> u8 { 1 }

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMsg {
    #[serde(rename = "join")]
    Join { name: String },
    #[serde(rename = "key_state")]
    KeyState { keys: u8 },
    #[serde(rename = "cursor_speed")]
    CursorSpeed { speed: u8 },
    #[serde(rename = "create_room")]
    CreateRoom {
        #[serde(default)]
        is_public: bool,
        #[serde(default)]
        map_id: Option<String>,
        #[serde(default)]
        bot_count: Option<u8>,
        #[serde(default)]
        is_vanilla: Option<bool>,
    },
    #[serde(rename = "join_room")]
    JoinRoom { code: String },
    #[serde(rename = "quick_play")]
    QuickPlay,
    #[serde(rename = "list_rooms")]
    ListRooms,
    #[serde(rename = "leave_room")]
    LeaveRoom,
    #[serde(rename = "toggle_ready")]
    ToggleReady,
    #[serde(rename = "set_map")]
    SetMap { map_id: String },
    #[serde(rename = "set_bots")]
    SetBots { count: u8 },
    #[serde(rename = "set_public")]
    SetPublic { is_public: bool },
    #[serde(rename = "set_vanilla")]
    SetVanilla { is_vanilla: bool },
    #[serde(rename = "start_game")]
    StartGame {
        #[serde(default)]
        config: Option<GameConfig>,
    },
    #[serde(rename = "start_single_player")]
    StartSinglePlayer { map_id: String, config: GameConfig },
}

#[derive(Serialize, Clone, Debug)]
struct LobbyPlayer {
    id: usize,
    name: String,
    ready: bool,
}

#[derive(Serialize, Clone, Debug)]
struct RoomListEntry {
    code: String,
    #[serde(rename = "playerCount")]
    player_count: usize,
    #[serde(rename = "maxPlayers")]
    max_players: u8,
    #[serde(rename = "mapId")]
    map_id: String,
    #[serde(rename = "hostName")]
    host_name: String,
    #[serde(rename = "isVanilla")]
    is_vanilla: bool,
}

#[derive(Serialize, Clone, Debug)]
struct LobbySettings {
    map_id: String,
    bot_count: u8,
    is_public: bool,
    max_players: u8,
    is_vanilla: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type")]
enum ServerMsg {
    #[serde(rename = "map_list")]
    MapList {
        maps: Vec<MapInfo>,
        #[serde(rename = "yourId")]
        your_id: usize,
    },
    #[serde(rename = "room_created")]
    RoomCreated { code: String },
    #[serde(rename = "room_list")]
    RoomList { rooms: Vec<RoomListEntry> },
    #[serde(rename = "lobby_update")]
    LobbyUpdate {
        code: String,
        host_id: usize,
        players: Vec<LobbyPlayer>,
        settings: LobbySettings,
        phase: String,
    },
    #[serde(rename = "countdown")]
    Countdown { seconds: u8 },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "left_room")]
    LeftRoom,
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
    #[serde(rename = "game_over")]
    GameOver {
        #[serde(rename = "winnerTeam")]
        winner_team: usize,
        #[serde(rename = "winnerName")]
        winner_name: String,
    },
}

// ---------------------------------------------------------------------------
// Room structures
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct RoomSettings {
    map_id: String,
    bot_count: u8,
    is_public: bool,
    max_players: u8,
    is_vanilla: bool,
}

impl Default for RoomSettings {
    fn default() -> Self {
        RoomSettings {
            map_id: String::new(),
            bot_count: 1,
            is_public: false,
            max_players: 32,
            is_vanilla: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum RoomPhase {
    Waiting,
    Countdown(i32),  // tick when countdown started
    Playing,
    GameOver(i32),   // tick when game ended
}

impl RoomPhase {
    fn as_str(&self) -> &str {
        match self {
            RoomPhase::Waiting => "waiting",
            RoomPhase::Countdown(_) => "countdown",
            RoomPhase::Playing => "playing",
            RoomPhase::GameOver(_) => "game_over",
        }
    }
}

struct PlayerSlot {
    conn_id: usize,
    team_id: usize,   // assigned team index in game (0..5)
    name: String,
    ready: bool,
}

struct Room {
    code: String,
    host_id: usize,
    settings: RoomSettings,
    players: HashMap<usize, PlayerSlot>,  // conn_id → slot
    team_names: HashMap<usize, String>,   // team_id → display name (all teams)
    phase: RoomPhase,
    game: Option<GameState>,
    map_data: Vec<u8>,
    prev_bitmap: Vec<u8>,
    cpu_teams: Vec<usize>,
    cpu_ai: ComputerAI,
    is_single_player: bool,
    tick_count: i32,
}

impl Room {
    fn new(code: String, host_id: usize, is_public: bool) -> Self {
        Room {
            code,
            host_id,
            settings: RoomSettings {
                is_public,
                ..Default::default()
            },
            players: HashMap::new(),
            team_names: HashMap::new(),
            phase: RoomPhase::Waiting,
            game: None,
            map_data: Vec::new(),
            prev_bitmap: Vec::new(),
            cpu_teams: Vec::new(),
            cpu_ai: ComputerAI::new(),
            is_single_player: false,
            tick_count: 0,
        }
    }

    fn player_count(&self) -> usize {
        self.players.len()
    }

    fn is_full(&self) -> bool {
        self.players.len() >= self.settings.max_players as usize
    }

    fn add_player(&mut self, conn_id: usize, name: &str) -> Option<usize> {
        if self.is_full() {
            return None;
        }
        // Assign first free team_id (0..5)
        let used_teams: Vec<usize> = self.players.values().map(|s| s.team_id).collect();
        let team_id = (0..NB_TEAMS).find(|t| !used_teams.contains(t))?;

        self.players.insert(conn_id, PlayerSlot {
            conn_id,
            team_id,
            name: if name.is_empty() { format!("Player {}", team_id + 1) } else { name.to_string() },
            ready: false,
        });
        Some(team_id)
    }

    fn remove_player(&mut self, conn_id: usize) {
        self.players.remove(&conn_id);
        // Transfer host if needed
        if self.host_id == conn_id && !self.players.is_empty() {
            let new_host = *self.players.keys().min().unwrap();
            self.host_id = new_host;
        }
    }

    fn get_lobby_players(&self) -> Vec<LobbyPlayer> {
        let mut players: Vec<LobbyPlayer> = self.players.values().map(|s| LobbyPlayer {
            id: s.conn_id,
            name: s.name.clone(),
            ready: s.ready,
        }).collect();
        players.sort_by_key(|p| p.id);
        players
    }

    fn get_lobby_settings(&self) -> LobbySettings {
        LobbySettings {
            map_id: self.settings.map_id.clone(),
            bot_count: self.settings.bot_count,
            is_public: self.settings.is_public,
            max_players: self.settings.max_players,
            is_vanilla: self.settings.is_vanilla,
        }
    }

    fn lobby_update_msg(&self) -> ServerMsg {
        ServerMsg::LobbyUpdate {
            code: self.code.clone(),
            host_id: self.host_id,
            players: self.get_lobby_players(),
            settings: self.get_lobby_settings(),
            phase: self.phase.as_str().to_string(),
        }
    }

    fn start_game(&mut self, maps_dir: &Path, config: Option<&GameConfig>) {
        let map_id = &self.settings.map_id;
        let map_path = maps_dir.join(format!("{map_id}.png"));
        let map = if map_path.exists() {
            load_map_from_png(&map_path)
        } else {
            println!("Map file not found: {:?}, using fallback", map_path);
            Map::with_obstacles(MAP_WIDTH, MAP_HEIGHT)
        };

        println!(
            "Room {} starting game on map: {} ({}x{})",
            self.code, map_id, map.width, map.height,
        );

        self.map_data = map
            .passable
            .iter()
            .map(|&p| if p { 0u8 } else { 1u8 })
            .collect();

        let mut game = GameState::new(map);

        // Apply game config if provided
        if let Some(cfg) = config {
            game.fighter_attack = cfg.fighter_attack.min(32) as u32;
            game.fighter_defense = cfg.fighter_defense.min(32) as u32;
            game.fighter_new_health = cfg.fighter_new_health.min(32) as u32;
            game.number_influence = cfg.number_influence.min(32) as i32;
            game.fighter_number = cfg.fighter_number.min(32) as usize;
            game.cursor_speed = cfg.cursor_speed.clamp(1, 5) as i32;
        }

        self.cpu_teams.clear();
        self.team_names.clear();

        if let Some(cfg) = config {
            if !cfg.teams.is_empty() {
                // Config-driven team setup (single player with full team config)
                let total_teams = cfg.teams.iter().take(NB_TEAMS)
                    .filter(|s| s.mode == "human" || s.mode == "cpu")
                    .count();
                for (team_id, slot) in cfg.teams.iter().enumerate().take(NB_TEAMS) {
                    match slot.mode.as_str() {
                        "human" | "cpu" => {
                            self.team_names.insert(team_id, slot.name.clone());
                        }
                        _ => {}
                    }
                    match slot.mode.as_str() {
                        "human" => {
                            game.add_player(team_id, total_teams);
                        }
                        "cpu" => {
                            game.add_player(team_id, total_teams);
                            self.cpu_teams.push(team_id);
                        }
                        _ => {} // "off" — skip
                    }
                }

                // Re-assign team_ids for human players based on config
                let human_teams: Vec<usize> = cfg.teams.iter().enumerate()
                    .filter(|(_, s)| s.mode == "human")
                    .map(|(i, _)| i)
                    .collect();

                // Assign each room player to a human team slot
                let mut player_list: Vec<usize> = self.players.keys().cloned().collect();
                player_list.sort();
                for (pi, conn_id) in player_list.iter().enumerate() {
                    if let Some(&team_id) = human_teams.get(pi) {
                        if let Some(slot) = self.players.get_mut(conn_id) {
                            slot.team_id = team_id;
                        }
                    }
                }

                println!(
                    "  Config-driven: {} human, {} cpu teams",
                    human_teams.len(), self.cpu_teams.len()
                );
            } else {
                self.start_game_default_teams(&mut game);
            }
        } else {
            self.start_game_default_teams(&mut game);
        }

        self.game = Some(game);
        self.phase = RoomPhase::Playing;
        self.tick_count = 0;
    }

    /// Default team setup: each connected player = own team + fill with bots
    fn start_game_default_teams(&mut self, game: &mut GameState) {
        let mut team_ids: Vec<(usize, usize, String)> = self.players.values()
            .map(|s| (s.conn_id, s.team_id, s.name.clone()))
            .collect();
        team_ids.sort_by_key(|(_, tid, _)| *tid);

        let total_teams = team_ids.len() + self.settings.bot_count as usize;

        for (_, team_id, name) in &team_ids {
            self.team_names.insert(*team_id, name.clone());
            game.add_player(*team_id, total_teams);
        }

        let used_teams: Vec<usize> = team_ids.iter().map(|(_, t, _)| *t).collect();
        let mut bots_added = 0u8;
        let bot_names = [
            "Napoleon", "Clovis", "Henri IV", "Cesar", "Geronimo", "Attila",
            "Genghis", "Cleopatra", "Alexander", "Hannibal", "Spartacus", "Boudicca",
            "Saladin", "Charlemagne", "Leonidas", "Ramses", "Montezuma", "Tokugawa",
            "Bismarck", "Victoria", "Shaka", "Suleiman", "Cyrus", "Pachacuti",
            "Ragnar", "Tamerlane", "Darius", "Barbarossa", "Ashoka", "Cortez",
            "Drake", "Bolivar",
        ];
        for t in 0..NB_TEAMS {
            if bots_added >= self.settings.bot_count {
                break;
            }
            if !used_teams.contains(&t) {
                self.team_names.insert(t, bot_names[t % bot_names.len()].to_string());
                game.add_player(t, total_teams);
                self.cpu_teams.push(t);
                bots_added += 1;
            }
        }
    }

    fn reset_to_waiting(&mut self) {
        self.game = None;
        self.cpu_teams.clear();
        self.phase = RoomPhase::Waiting;
        self.tick_count = 0;
        // Reset ready states
        for slot in self.players.values_mut() {
            slot.ready = false;
        }
    }
}

// ---------------------------------------------------------------------------
// Connection info (lives outside rooms)
// ---------------------------------------------------------------------------

struct ConnInfo {
    tx: mpsc::UnboundedSender<OutMsg>,
    name: String,
    room_code: Option<String>,
}

// ---------------------------------------------------------------------------
// Room Manager
// ---------------------------------------------------------------------------

struct RoomManager {
    rooms: HashMap<String, Room>,
    connections: HashMap<usize, ConnInfo>,
    map_index: Vec<MapInfo>,
    maps_dir: PathBuf,
}

impl RoomManager {
    fn new(maps_dir: PathBuf) -> Self {
        let map_index = load_map_index(&maps_dir);
        RoomManager {
            rooms: HashMap::new(),
            connections: HashMap::new(),
            map_index,
            maps_dir,
        }
    }

    fn send_to(&self, conn_id: usize, msg: ServerMsg) {
        if let Some(conn) = self.connections.get(&conn_id) {
            let _ = conn.tx.send(OutMsg::Json(msg));
        }
    }

    fn send_error(&self, conn_id: usize, message: &str) {
        self.send_to(conn_id, ServerMsg::Error { message: message.to_string() });
    }

    fn broadcast_lobby_update(&self, code: &str) {
        if let Some(room) = self.rooms.get(code) {
            let msg = room.lobby_update_msg();
            for conn_id in room.players.keys() {
                if let Some(conn) = self.connections.get(conn_id) {
                    let _ = conn.tx.send(OutMsg::Json(msg.clone()));
                }
            }
        }
    }

    /// Build a binary state frame: [flags:1][tick:4][num_teams:1][scores:N*4][cursors:N*5][deflate_bitmap]
    fn build_binary_state(
        tick: i32,
        bitmap: &[u8],
        prev_bitmap: &[u8],
        scores: &[u32; NB_TEAMS],
        cursors: &[(i32, i32, bool)],
    ) -> Vec<u8> {
        let is_delta = !prev_bitmap.is_empty() && prev_bitmap.len() == bitmap.len();
        let num_teams = NB_TEAMS as u8;

        // Header
        let mut buf: Vec<u8> = Vec::with_capacity(6 + NB_TEAMS * 9 + bitmap.len() / 4);
        buf.push(if is_delta { 1 } else { 0 }); // flags
        buf.extend_from_slice(&tick.to_le_bytes()); // tick
        buf.push(num_teams);

        // Scores
        for s in scores.iter() {
            buf.extend_from_slice(&s.to_le_bytes());
        }

        // Cursors
        for (x, y, active) in cursors.iter() {
            buf.push(if *active { 1 } else { 0 });
            buf.extend_from_slice(&(*x as i16).to_le_bytes());
            buf.extend_from_slice(&(*y as i16).to_le_bytes());
        }
        // Pad if fewer cursors than NB_TEAMS
        for _ in cursors.len()..NB_TEAMS {
            buf.extend_from_slice(&[0, 0, 0, 0, 0]);
        }

        // Bitmap: XOR delta if possible, else raw
        let bitmap_data: Vec<u8> = if is_delta {
            bitmap.iter().zip(prev_bitmap.iter()).map(|(a, b)| a ^ b).collect()
        } else {
            bitmap.to_vec()
        };

        // Deflate compress
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&bitmap_data).unwrap();
        let compressed = encoder.finish().unwrap();
        buf.extend_from_slice(&compressed);

        buf
    }

    fn add_connection(&mut self, conn_id: usize, tx: mpsc::UnboundedSender<OutMsg>) {
        self.connections.insert(conn_id, ConnInfo {
            tx,
            name: String::new(),
            room_code: None,
        });
        // Send map list on connect (with connection ID so client knows itself)
        self.send_to(conn_id, ServerMsg::MapList { maps: self.map_index.clone(), your_id: conn_id });
    }

    fn remove_connection(&mut self, conn_id: usize) {
        let room_code = self.connections.get(&conn_id).and_then(|c| c.room_code.clone());
        if let Some(code) = room_code {
            self.leave_room(conn_id, &code);
        }
        self.connections.remove(&conn_id);
    }

    fn set_name(&mut self, conn_id: usize, name: String) {
        if let Some(conn) = self.connections.get_mut(&conn_id) {
            conn.name = name.clone();
        }
        // Update name in room too
        let room_code = self.connections.get(&conn_id).and_then(|c| c.room_code.clone());
        if let Some(code) = room_code {
            if let Some(room) = self.rooms.get_mut(&code) {
                if let Some(slot) = room.players.get_mut(&conn_id) {
                    slot.name = if name.is_empty() { format!("Player {}", slot.team_id + 1) } else { name };
                }
            }
            self.broadcast_lobby_update(&code);
        }
    }

    fn create_room(&mut self, conn_id: usize, is_public: bool, map_id: Option<String>, bot_count: Option<u8>, is_vanilla: Option<bool>) {
        // Leave current room if in one
        if let Some(code) = self.connections.get(&conn_id).and_then(|c| c.room_code.clone()) {
            self.leave_room(conn_id, &code);
        }

        let code = generate_room_code(&self.rooms);
        let mut room = Room::new(code.clone(), conn_id, is_public);

        let name = self.connections.get(&conn_id).map(|c| c.name.clone()).unwrap_or_default();
        room.add_player(conn_id, &name);

        // Apply optional settings from create-room screen
        if let Some(mid) = map_id {
            room.settings.map_id = mid;
        } else if let Some(first_map) = self.map_index.first() {
            room.settings.map_id = first_map.id.clone();
        }
        if let Some(bc) = bot_count {
            room.settings.bot_count = bc.min(31);
        }
        if let Some(v) = is_vanilla {
            room.settings.is_vanilla = v;
        }

        self.rooms.insert(code.clone(), room);
        if let Some(conn) = self.connections.get_mut(&conn_id) {
            conn.room_code = Some(code.clone());
        }

        println!("Room {code} created by conn {conn_id} (public: {is_public})");

        self.send_to(conn_id, ServerMsg::RoomCreated { code: code.clone() });
        self.broadcast_lobby_update(&code);
    }

    fn join_room(&mut self, conn_id: usize, code: &str) {
        let code = code.to_uppercase();

        // Check room exists
        let room_exists = self.rooms.contains_key(&code);
        if !room_exists {
            self.send_error(conn_id, "Room not found");
            return;
        }

        // Check not full and in waiting phase
        {
            let room = self.rooms.get(&code).unwrap();
            if room.is_full() {
                self.send_error(conn_id, "Room is full");
                return;
            }
            if room.phase != RoomPhase::Waiting {
                self.send_error(conn_id, "Game already in progress");
                return;
            }
        }

        // Leave current room if in one
        if let Some(old_code) = self.connections.get(&conn_id).and_then(|c| c.room_code.clone()) {
            self.leave_room(conn_id, &old_code);
        }

        let name = self.connections.get(&conn_id).map(|c| c.name.clone()).unwrap_or_default();
        let room = self.rooms.get_mut(&code).unwrap();
        if room.add_player(conn_id, &name).is_none() {
            self.send_error(conn_id, "Could not join room");
            return;
        }

        if let Some(conn) = self.connections.get_mut(&conn_id) {
            conn.room_code = Some(code.clone());
        }

        println!("Conn {conn_id} joined room {code}");
        self.broadcast_lobby_update(&code);
    }

    fn quick_play(&mut self, conn_id: usize) {
        // Find a public waiting room with space
        let existing_code = self.rooms.iter()
            .find(|(_, r)| {
                r.settings.is_public
                    && r.phase == RoomPhase::Waiting
                    && !r.is_full()
                    && !r.is_single_player
            })
            .map(|(code, _)| code.clone());

        if let Some(code) = existing_code {
            self.join_room(conn_id, &code);
        } else {
            // Create a new public room
            self.create_room(conn_id, true, None, None, None);
        }
    }

    fn list_rooms(&self, conn_id: usize) {
        let rooms: Vec<RoomListEntry> = self.rooms.values()
            .filter(|r| r.settings.is_public && r.phase == RoomPhase::Waiting && !r.is_single_player)
            .map(|r| {
                let host_name = r.players.get(&r.host_id)
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| "???".to_string());
                RoomListEntry {
                    code: r.code.clone(),
                    player_count: r.player_count(),
                    max_players: r.settings.max_players,
                    map_id: r.settings.map_id.clone(),
                    host_name,
                    is_vanilla: r.settings.is_vanilla,
                }
            })
            .collect();

        self.send_to(conn_id, ServerMsg::RoomList { rooms });
    }

    fn leave_room(&mut self, conn_id: usize, code: &str) {
        let should_delete;
        {
            let room = match self.rooms.get_mut(code) {
                Some(r) => r,
                None => return,
            };
            room.remove_player(conn_id);
            should_delete = room.players.is_empty();
        }

        if let Some(conn) = self.connections.get_mut(&conn_id) {
            conn.room_code = None;
        }
        self.send_to(conn_id, ServerMsg::LeftRoom);

        if should_delete {
            println!("Room {code} deleted (empty)");
            self.rooms.remove(code);
        } else {
            self.broadcast_lobby_update(code);
        }
    }

    fn handle_leave_room(&mut self, conn_id: usize) {
        if let Some(code) = self.connections.get(&conn_id).and_then(|c| c.room_code.clone()) {
            self.leave_room(conn_id, &code);
        }
    }

    fn toggle_ready(&mut self, conn_id: usize) {
        let code = match self.connections.get(&conn_id).and_then(|c| c.room_code.clone()) {
            Some(c) => c,
            None => return,
        };
        if let Some(room) = self.rooms.get_mut(&code) {
            if room.phase != RoomPhase::Waiting { return; }
            if let Some(slot) = room.players.get_mut(&conn_id) {
                slot.ready = !slot.ready;
            }
        }
        self.broadcast_lobby_update(&code);
    }

    fn set_map(&mut self, conn_id: usize, map_id: String) {
        let code = match self.connections.get(&conn_id).and_then(|c| c.room_code.clone()) {
            Some(c) => c,
            None => return,
        };
        if let Some(room) = self.rooms.get_mut(&code) {
            if room.host_id != conn_id { return; }  // host only
            if room.phase != RoomPhase::Waiting { return; }
            room.settings.map_id = map_id;
        }
        self.broadcast_lobby_update(&code);
    }

    fn set_bots(&mut self, conn_id: usize, count: u8) {
        let code = match self.connections.get(&conn_id).and_then(|c| c.room_code.clone()) {
            Some(c) => c,
            None => return,
        };
        if let Some(room) = self.rooms.get_mut(&code) {
            if room.host_id != conn_id { return; }
            if room.phase != RoomPhase::Waiting { return; }
            room.settings.bot_count = count.min(31);
        }
        self.broadcast_lobby_update(&code);
    }

    fn set_public(&mut self, conn_id: usize, is_public: bool) {
        let code = match self.connections.get(&conn_id).and_then(|c| c.room_code.clone()) {
            Some(c) => c,
            None => return,
        };
        if let Some(room) = self.rooms.get_mut(&code) {
            if room.host_id != conn_id { return; }
            if room.phase != RoomPhase::Waiting { return; }
            room.settings.is_public = is_public;
        }
        self.broadcast_lobby_update(&code);
    }

    fn set_vanilla(&mut self, conn_id: usize, is_vanilla: bool) {
        let code = match self.connections.get(&conn_id).and_then(|c| c.room_code.clone()) {
            Some(c) => c,
            None => return,
        };
        if let Some(room) = self.rooms.get_mut(&code) {
            if room.host_id != conn_id { return; }
            if room.phase != RoomPhase::Waiting { return; }
            room.settings.is_vanilla = is_vanilla;
        }
        self.broadcast_lobby_update(&code);
    }

    fn start_game(&mut self, conn_id: usize, config: Option<GameConfig>) {
        let code = match self.connections.get(&conn_id).and_then(|c| c.room_code.clone()) {
            Some(c) => c,
            None => return,
        };

        let maps_dir = self.maps_dir.clone();

        // Validate
        {
            let room = match self.rooms.get(&code) {
                Some(r) => r,
                None => return,
            };
            if room.host_id != conn_id { return; }
            if room.phase != RoomPhase::Waiting { return; }
            if room.settings.map_id.is_empty() {
                self.send_error(conn_id, "Select a map first");
                return;
            }
            // Need at least 2 teams total (players + bots)
            let total = room.player_count() + room.settings.bot_count as usize;
            if total < 2 {
                self.send_error(conn_id, "Need at least 2 teams (players + bots)");
                return;
            }
        }

        // Start the game
        let room = self.rooms.get_mut(&code).unwrap();
        room.start_game(&maps_dir, config.as_ref());

        // Send welcome to all players
        if let Some(ref game) = room.game {
            for (cid, slot) in &room.players {
                let welcome = ServerMsg::Welcome {
                    player_id: slot.team_id,
                    map_width: game.map_width(),
                    map_height: game.map_height(),
                    map_data: room.map_data.clone(),
                    map_id: room.settings.map_id.clone(),
                };
                if let Some(conn) = self.connections.get(cid) {
                    let _ = conn.tx.send(OutMsg::Json(welcome));
                }
            }
        }
    }

    fn start_single_player(&mut self, conn_id: usize, map_id: String, config: GameConfig) {
        // Leave current room if in one
        if let Some(code) = self.connections.get(&conn_id).and_then(|c| c.room_code.clone()) {
            self.leave_room(conn_id, &code);
        }

        let code = generate_room_code(&self.rooms);
        let mut room = Room::new(code.clone(), conn_id, false);
        room.is_single_player = true;
        room.settings.map_id = map_id;

        let name = self.connections.get(&conn_id).map(|c| c.name.clone()).unwrap_or_default();
        room.add_player(conn_id, &name);

        self.rooms.insert(code.clone(), room);
        if let Some(conn) = self.connections.get_mut(&conn_id) {
            conn.room_code = Some(code.clone());
        }

        println!("Single player room {code} created by conn {conn_id}");

        // Auto-start with config
        let maps_dir = self.maps_dir.clone();
        let room = self.rooms.get_mut(&code).unwrap();
        room.start_game(&maps_dir, Some(&config));

        // Send welcome
        if let Some(ref game) = room.game {
            if let Some(slot) = room.players.get(&conn_id) {
                let welcome = ServerMsg::Welcome {
                    player_id: slot.team_id,
                    map_width: game.map_width(),
                    map_height: game.map_height(),
                    map_data: room.map_data.clone(),
                    map_id: room.settings.map_id.clone(),
                };
                self.send_to(conn_id, welcome);
            }
        }
    }

    fn set_key_state(&mut self, conn_id: usize, keys: u8) {
        let code = match self.connections.get(&conn_id).and_then(|c| c.room_code.clone()) {
            Some(c) => c,
            None => return,
        };
        if let Some(room) = self.rooms.get_mut(&code) {
            if let Some(slot) = room.players.get(&conn_id) {
                let team_id = slot.team_id;
                if let Some(ref mut game) = room.game {
                    game.set_key_state(team_id, keys);
                }
            }
        }
    }

    fn set_cursor_speed(&mut self, conn_id: usize, speed: u8) {
        let code = match self.connections.get(&conn_id).and_then(|c| c.room_code.clone()) {
            Some(c) => c,
            None => return,
        };
        if let Some(room) = self.rooms.get_mut(&code) {
            if let Some(ref mut game) = room.game {
                game.cursor_speed = speed.clamp(1, 5) as i32;
            }
        }
    }

    /// Tick all active rooms
    fn tick_all(&mut self) {
        let codes: Vec<String> = self.rooms.keys().cloned().collect();
        let mut rooms_to_delete: Vec<String> = Vec::new();
        let mut end_sp_rooms: Vec<(String, usize)> = Vec::new(); // (code, conn_id)

        // Collect messages to send after room borrows are released
        let mut pending: Vec<(usize, OutMsg)> = Vec::new();

        for code in &codes {
            let room = match self.rooms.get_mut(code) {
                Some(r) => r,
                None => continue,
            };

            room.tick_count += 1;

            match room.phase.clone() {
                RoomPhase::Waiting => {}
                RoomPhase::Countdown(start_tick) => {
                    let elapsed = room.tick_count - start_tick;
                    let seconds_left = 3 - (elapsed / 20);
                    if elapsed % 20 == 0 && seconds_left >= 0 {
                        let msg = ServerMsg::Countdown { seconds: seconds_left as u8 };
                        for cid in room.players.keys() {
                            pending.push((*cid, OutMsg::Json(msg.clone())));
                        }
                    }
                    if elapsed >= 60 {
                        let maps_dir = self.maps_dir.clone();
                        room.start_game(&maps_dir, None);
                        if let Some(ref game) = room.game {
                            for (cid, slot) in &room.players {
                                let welcome = ServerMsg::Welcome {
                                    player_id: slot.team_id,
                                    map_width: game.map_width(),
                                    map_height: game.map_height(),
                                    map_data: room.map_data.clone(),
                                    map_id: room.settings.map_id.clone(),
                                };
                                pending.push((*cid, OutMsg::Json(welcome)));
                            }
                        }
                    }
                }
                RoomPhase::Playing => {
                    if let Some(ref mut game) = room.game {
                        let cpu_teams_clone: Vec<usize> = room.cpu_teams.clone();
                        for &cpu_team in &cpu_teams_clone {
                            let keys = room.cpu_ai.get_next_move(game, cpu_team);
                            game.set_key_state(cpu_team, keys);
                        }

                        game.tick();

                        let bitmap = game.get_bitmap();
                        let scores = game.get_scores();
                        let cursor_data = game.get_cursors();
                        let clock = game.global_clock;

                        let frame = Self::build_binary_state(
                            clock, &bitmap, &room.prev_bitmap, &scores, &cursor_data,
                        );

                        for cid in room.players.keys() {
                            pending.push((*cid, OutMsg::Binary(frame.clone())));
                        }
                        room.prev_bitmap = bitmap;

                        // Check winner (after 100 ticks)
                        if clock > 100 {
                            let active_teams: Vec<usize> = scores.iter().enumerate()
                                .filter(|(_, &s)| s > 0)
                                .map(|(i, _)| i)
                                .collect();

                            if active_teams.len() == 1 {
                                let winner_team = active_teams[0];
                                let winner_name = room.team_names.get(&winner_team)
                                    .cloned()
                                    .unwrap_or_else(|| format!("Team {}", winner_team + 1));

                                let game_over_msg = ServerMsg::GameOver {
                                    winner_team,
                                    winner_name,
                                };
                                for cid in room.players.keys() {
                                    pending.push((*cid, OutMsg::Json(game_over_msg.clone())));
                                }
                                room.phase = RoomPhase::GameOver(room.tick_count);
                            }
                        }
                    }
                }
                RoomPhase::GameOver(over_tick) => {
                    if let Some(ref mut game) = room.game {
                        game.tick();
                        let bitmap = game.get_bitmap();
                        let scores = game.get_scores();
                        let cursor_data = game.get_cursors();

                        let frame = Self::build_binary_state(
                            game.global_clock, &bitmap, &room.prev_bitmap, &scores, &cursor_data,
                        );
                        for cid in room.players.keys() {
                            pending.push((*cid, OutMsg::Binary(frame.clone())));
                        }
                        room.prev_bitmap = bitmap;
                    }

                    // 5 seconds cooldown = 100 ticks
                    if room.tick_count - over_tick > 100 {
                        if room.is_single_player {
                            let conn_id = *room.players.keys().next().unwrap();
                            end_sp_rooms.push((code.clone(), conn_id));
                        } else {
                            room.reset_to_waiting();
                            let msg = room.lobby_update_msg();
                            for cid in room.players.keys() {
                                pending.push((*cid, OutMsg::Json(msg.clone())));
                            }
                        }
                    }
                }
            }

            if room.players.is_empty() && room.phase == RoomPhase::Waiting {
                rooms_to_delete.push(code.clone());
            }
        }

        // Flush all pending messages
        for (cid, msg) in pending {
            if let Some(conn) = self.connections.get(&cid) {
                let _ = conn.tx.send(msg);
            }
        }

        for code in rooms_to_delete {
            println!("Room {code} deleted (empty)");
            self.rooms.remove(&code);
        }

        // Handle single player game-over: leave room and send back to menu
        for (code, conn_id) in end_sp_rooms {
            self.leave_room(conn_id, &code);
            // Also delete the room
            self.rooms.remove(&code);
            // Re-send map list so they see the menu
            self.send_to(conn_id, ServerMsg::MapList { maps: self.map_index.clone(), your_id: conn_id });
        }
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3001";

    // Resolve maps dir: env var > exe-relative > cwd fallback
    let maps_dir = std::env::var("MAPS_DIR")
        .map(PathBuf::from)
        .ok()
        .filter(|p| p.exists())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent()?.parent()?.parent()?.join("public/maps").canonicalize().ok())
        })
        .unwrap_or_else(|| PathBuf::from("public/maps"));

    let manager = RoomManager::new(maps_dir);

    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("LiquidWar.io server listening on ws://{addr}");

    let manager = Arc::new(Mutex::new(manager));
    let next_conn_id = Arc::new(Mutex::new(0usize));

    // Game loop — tick all rooms at 20Hz
    let mgr_tick = manager.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(50));
        loop {
            ticker.tick().await;
            let mut mgr = mgr_tick.lock().await;
            mgr.tick_all();
        }
    });

    // Accept connections
    while let Ok((stream, addr)) = listener.accept().await {
        let manager = manager.clone();
        let next_conn_id = next_conn_id.clone();

        tokio::spawn(async move {
            let ws_stream = match accept_async(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("WebSocket handshake failed for {addr}: {e}");
                    return;
                }
            };

            let (mut ws_tx, mut ws_rx) = ws_stream.split();
            let (tx, mut rx) = mpsc::unbounded_channel::<OutMsg>();

            // Assign connection ID
            let conn_id;
            {
                let mut id = next_conn_id.lock().await;
                conn_id = *id;
                *id += 1;
            }

            // Register connection
            {
                let mut mgr = manager.lock().await;
                mgr.add_connection(conn_id, tx.clone());
                println!("Conn {conn_id} connected from {addr}");
            }

            // Forward server messages to WebSocket
            let send_task = tokio::spawn(async move {
                while let Some(out) = rx.recv().await {
                    let ws_msg = match out {
                        OutMsg::Json(msg) => {
                            let text = serde_json::to_string(&msg).unwrap();
                            Message::Text(text.into())
                        }
                        OutMsg::Binary(data) => Message::Binary(data.into()),
                    };
                    if ws_tx.send(ws_msg).await.is_err() {
                        break;
                    }
                }
            });

            // Read client messages
            while let Some(Ok(msg)) = ws_rx.next().await {
                if let Message::Text(text) = msg {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&text) {
                        let mut mgr = manager.lock().await;
                        match client_msg {
                            ClientMsg::Join { name } => {
                                mgr.set_name(conn_id, name);
                            }
                            ClientMsg::KeyState { keys } => {
                                mgr.set_key_state(conn_id, keys);
                            }
                            ClientMsg::CursorSpeed { speed } => {
                                mgr.set_cursor_speed(conn_id, speed);
                            }
                            ClientMsg::CreateRoom { is_public, map_id, bot_count, is_vanilla } => {
                                mgr.create_room(conn_id, is_public, map_id, bot_count, is_vanilla);
                            }
                            ClientMsg::JoinRoom { code } => {
                                mgr.join_room(conn_id, &code);
                            }
                            ClientMsg::QuickPlay => {
                                mgr.quick_play(conn_id);
                            }
                            ClientMsg::ListRooms => {
                                mgr.list_rooms(conn_id);
                            }
                            ClientMsg::LeaveRoom => {
                                mgr.handle_leave_room(conn_id);
                            }
                            ClientMsg::ToggleReady => {
                                mgr.toggle_ready(conn_id);
                            }
                            ClientMsg::SetMap { map_id } => {
                                mgr.set_map(conn_id, map_id);
                            }
                            ClientMsg::SetBots { count } => {
                                mgr.set_bots(conn_id, count);
                            }
                            ClientMsg::SetPublic { is_public } => {
                                mgr.set_public(conn_id, is_public);
                            }
                            ClientMsg::SetVanilla { is_vanilla } => {
                                mgr.set_vanilla(conn_id, is_vanilla);
                            }
                            ClientMsg::StartGame { config } => {
                                mgr.start_game(conn_id, config);
                            }
                            ClientMsg::StartSinglePlayer { map_id, config } => {
                                mgr.start_single_player(conn_id, map_id, config);
                            }
                        }
                    }
                }
            }

            // Disconnect
            {
                let mut mgr = manager.lock().await;
                mgr.remove_connection(conn_id);
                println!("Conn {conn_id} disconnected");
            }
            send_task.abort();
        });
    }
}
