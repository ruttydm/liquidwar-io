import { inflateSync as decompressSync } from "fflate";

export interface MapInfo {
  id: string;
  name: string;
}

export interface MapListMsg {
  type: "map_list";
  maps: MapInfo[];
  yourId: number;
}

export interface RoomCreatedMsg {
  type: "room_created";
  code: string;
}

export interface RoomListEntry {
  code: string;
  playerCount: number;
  maxPlayers: number;
  mapId: string;
  hostName: string;
  isVanilla: boolean;
}

export interface RoomListMsg {
  type: "room_list";
  rooms: RoomListEntry[];
}

export interface LobbyPlayer {
  id: number;
  name: string;
  ready: boolean;
}

export interface LobbySettings {
  map_id: string;
  bot_count: number;
  is_public: boolean;
  max_players: number;
  is_vanilla: boolean;
}

export interface LobbyUpdateMsg {
  type: "lobby_update";
  code: string;
  host_id: number;
  players: LobbyPlayer[];
  settings: LobbySettings;
  phase: string;
}

export interface CountdownMsg {
  type: "countdown";
  seconds: number;
}

export interface ErrorMsg {
  type: "error";
  message: string;
}

export interface LeftRoomMsg {
  type: "left_room";
}

export interface WelcomeMsg {
  type: "welcome";
  playerId: number;
  mapWidth: number;
  mapHeight: number;
  mapData: number[];
  mapId: string;
}

export interface StateMsg {
  type: "state";
  tick: number;
  bitmap: Uint8Array;
  scores: number[];
  cursors: Array<[number, number] | null>;
}

export interface GameOverMsg {
  type: "game_over";
  winnerTeam: number;
  winnerName: string;
}

export interface TeamSlotConfig {
  mode: "human" | "cpu" | "off";
  name: string;
}

export interface GameConfig {
  teams: TeamSlotConfig[];
  fighter_attack: number;
  fighter_defense: number;
  fighter_new_health: number;
  number_influence: number;
  fighter_number: number;
  cursor_speed: number;
}

export type ServerMsg =
  | MapListMsg
  | RoomCreatedMsg
  | RoomListMsg
  | LobbyUpdateMsg
  | CountdownMsg
  | ErrorMsg
  | LeftRoomMsg
  | WelcomeMsg
  | StateMsg
  | GameOverMsg;

export class GameClient {
  private ws: WebSocket;
  private prevBitmap: Uint8Array | null = null;
  public onMapList: ((msg: MapListMsg) => void) | null = null;
  public onRoomCreated: ((msg: RoomCreatedMsg) => void) | null = null;
  public onRoomList: ((msg: RoomListMsg) => void) | null = null;
  public onLobbyUpdate: ((msg: LobbyUpdateMsg) => void) | null = null;
  public onCountdown: ((msg: CountdownMsg) => void) | null = null;
  public onError: ((msg: ErrorMsg) => void) | null = null;
  public onLeftRoom: ((msg: LeftRoomMsg) => void) | null = null;
  public onWelcome: ((msg: WelcomeMsg) => void) | null = null;
  public onState: ((msg: StateMsg) => void) | null = null;
  public onGameOver: ((msg: GameOverMsg) => void) | null = null;
  public onOpen: (() => void) | null = null;

  constructor(url: string) {
    this.ws = new WebSocket(url);
    this.ws.binaryType = "arraybuffer";

    this.ws.onopen = () => {
      this.onOpen?.();
    };

    this.ws.onmessage = (event) => {
      if (event.data instanceof ArrayBuffer) {
        this.handleBinaryState(event.data);
        return;
      }

      const msg: ServerMsg = JSON.parse(event.data);
      switch (msg.type) {
        case "map_list":
          this.onMapList?.(msg);
          break;
        case "room_created":
          this.onRoomCreated?.(msg);
          break;
        case "room_list":
          this.onRoomList?.(msg);
          break;
        case "lobby_update":
          this.onLobbyUpdate?.(msg);
          break;
        case "countdown":
          this.onCountdown?.(msg);
          break;
        case "error":
          this.onError?.(msg);
          break;
        case "left_room":
          this.onLeftRoom?.(msg);
          break;
        case "welcome":
          this.prevBitmap = null;
          this.onWelcome?.(msg);
          break;
        case "state":
          this.onState?.(msg);
          break;
        case "game_over":
          this.onGameOver?.(msg);
          break;
      }
    };
  }

  private handleBinaryState(buffer: ArrayBuffer) {
    const view = new DataView(buffer);
    const data = new Uint8Array(buffer);

    const flags = data[0];
    const isDelta = flags === 1;
    const tick = view.getInt32(1, true);
    const numTeams = data[5];

    let offset = 6;

    // Scores
    const scores: number[] = [];
    for (let i = 0; i < numTeams; i++) {
      scores.push(view.getUint32(offset, true));
      offset += 4;
    }

    // Cursors
    const cursors: Array<[number, number] | null> = [];
    for (let i = 0; i < numTeams; i++) {
      const active = data[offset];
      const x = view.getInt16(offset + 1, true);
      const y = view.getInt16(offset + 3, true);
      cursors.push(active ? [x, y] : null);
      offset += 5;
    }

    // Decompress bitmap
    const compressed = data.subarray(offset);
    const decompressed = decompressSync(compressed);

    // Apply XOR delta if needed
    let bitmap: Uint8Array;
    if (isDelta && this.prevBitmap && this.prevBitmap.length === decompressed.length) {
      bitmap = new Uint8Array(decompressed.length);
      for (let i = 0; i < decompressed.length; i++) {
        bitmap[i] = decompressed[i] ^ this.prevBitmap[i];
      }
    } else {
      bitmap = decompressed;
    }

    // Store for next delta
    this.prevBitmap = bitmap;

    this.onState?.({ type: "state", tick, bitmap, scores, cursors });
  }

  sendJoin(name: string) {
    this.send({ type: "join", name });
  }

  sendKeyState(keys: number) {
    this.send({ type: "key_state", keys });
  }

  sendCursorSpeed(speed: number) {
    this.send({ type: "cursor_speed", speed });
  }

  sendCreateRoom(opts: { isPublic: boolean; mapId?: string; botCount?: number; isVanilla?: boolean }) {
    this.send({ type: "create_room", is_public: opts.isPublic, map_id: opts.mapId, bot_count: opts.botCount, is_vanilla: opts.isVanilla });
  }

  sendSetVanilla(isVanilla: boolean) {
    this.send({ type: "set_vanilla", is_vanilla: isVanilla });
  }

  sendJoinRoom(code: string) {
    this.send({ type: "join_room", code });
  }

  sendQuickPlay() {
    this.send({ type: "quick_play" });
  }

  sendListRooms() {
    this.send({ type: "list_rooms" });
  }

  sendLeaveRoom() {
    this.send({ type: "leave_room" });
  }

  sendToggleReady() {
    this.send({ type: "toggle_ready" });
  }

  sendSetMap(mapId: string) {
    this.send({ type: "set_map", map_id: mapId });
  }

  sendSetBots(count: number) {
    this.send({ type: "set_bots", count });
  }

  sendSetPublic(isPublic: boolean) {
    this.send({ type: "set_public", is_public: isPublic });
  }

  sendStartGame(config?: GameConfig) {
    this.send({ type: "start_game", config: config || null });
  }

  sendStartSinglePlayer(mapId: string, config: GameConfig) {
    this.send({ type: "start_single_player", map_id: mapId, config });
  }

  private send(msg: object) {
    if (this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(msg));
    }
  }
}
