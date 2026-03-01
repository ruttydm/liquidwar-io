use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use game::constants::*;
use game::game::GameState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Duration};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMsg {
    #[serde(rename = "join")]
    Join { name: String },
    #[serde(rename = "cursor")]
    Cursor { x: u32, y: u32 },
}

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
enum ServerMsg {
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
    },
    #[serde(rename = "state")]
    State {
        tick: u32,
        bitmap: String, // base64-encoded
        scores: [u32; MAX_PLAYERS],
        cursors: Vec<Option<(u32, u32)>>,
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
    game: GameState,
    players: HashMap<usize, PlayerSlot>,
    next_id: usize,
    map_data: Vec<u8>,
}

impl Room {
    fn new() -> Self {
        let game = GameState::new();
        let map_data = game.map.to_bytes();
        Room {
            game,
            players: HashMap::new(),
            next_id: 0,
            map_data,
        }
    }

    fn add_player(&mut self, tx: mpsc::UnboundedSender<ServerMsg>) -> Option<(usize, ServerMsg)> {
        if self.next_id >= MAX_PLAYERS {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;

        self.game.add_player(id);

        let welcome = ServerMsg::Welcome {
            player_id: id,
            map_width: MAP_WIDTH,
            map_height: MAP_HEIGHT,
            map_data: self.map_data.clone(),
        };

        self.players.insert(
            id,
            PlayerSlot {
                id,
                name: String::new(),
                tx,
            },
        );

        Some((id, welcome))
    }

    fn remove_player(&mut self, id: usize) {
        self.players.remove(&id);
        let msg = ServerMsg::PlayerLeft { player_id: id };
        for slot in self.players.values() {
            let _ = slot.tx.send(msg.clone());
        }
    }

    fn set_cursor(&mut self, player_id: usize, x: u32, y: u32) {
        self.game.set_cursor(player_id, x, y);
    }

    fn tick(&mut self) {
        self.game.tick();

        let bitmap = self.game.get_bitmap();
        let bitmap_b64 = base64::engine::general_purpose::STANDARD.encode(&bitmap);
        let scores = self.game.get_scores();
        let cursor_data = self.game.get_cursors();

        let cursors: Vec<Option<(u32, u32)>> = cursor_data
            .iter()
            .map(|(x, y, active)| if *active { Some((*x, *y)) } else { None })
            .collect();

        let msg = ServerMsg::State {
            tick: self.game.tick,
            bitmap: bitmap_b64,
            scores,
            cursors,
        };

        for slot in self.players.values() {
            let _ = slot.tx.send(msg.clone());
        }
    }
}

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3001";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("Liquid War server listening on ws://{addr}");

    let room = Arc::new(Mutex::new(Room::new()));

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
                    Some((id, welcome)) => {
                        player_id = id;
                        let msg = serde_json::to_string(&welcome).unwrap();
                        if ws_tx.send(Message::Text(msg.into())).await.is_err() {
                            return;
                        }
                        println!("Player {id} connected from {addr}");
                    }
                    None => {
                        let _ = ws_tx
                            .send(Message::Close(None))
                            .await;
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
