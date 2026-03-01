export interface MapInfo {
  id: string;
  name: string;
}

export interface MapListMsg {
  type: "map_list";
  maps: MapInfo[];
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
  bitmap: string; // base64
  scores: number[];
  cursors: Array<[number, number] | null>;
}

export interface PlayerJoinedMsg {
  type: "player_joined";
  playerId: number;
  name: string;
}

export interface PlayerLeftMsg {
  type: "player_left";
  playerId: number;
}

export type ServerMsg =
  | MapListMsg
  | WelcomeMsg
  | StateMsg
  | PlayerJoinedMsg
  | PlayerLeftMsg;

export class GameClient {
  private ws: WebSocket;
  public onMapList: ((msg: MapListMsg) => void) | null = null;
  public onWelcome: ((msg: WelcomeMsg) => void) | null = null;
  public onState: ((msg: StateMsg) => void) | null = null;
  public onPlayerJoined: ((msg: PlayerJoinedMsg) => void) | null = null;
  public onPlayerLeft: ((msg: PlayerLeftMsg) => void) | null = null;
  public onOpen: (() => void) | null = null;

  constructor(url: string) {
    this.ws = new WebSocket(url);

    this.ws.onopen = () => {
      this.onOpen?.();
    };

    this.ws.onmessage = (event) => {
      const msg: ServerMsg = JSON.parse(event.data);
      switch (msg.type) {
        case "map_list":
          this.onMapList?.(msg);
          break;
        case "welcome":
          this.onWelcome?.(msg);
          break;
        case "state":
          this.onState?.(msg);
          break;
        case "player_joined":
          this.onPlayerJoined?.(msg);
          break;
        case "player_left":
          this.onPlayerLeft?.(msg);
          break;
      }
    };
  }

  sendJoin(name: string) {
    this.send({ type: "join", name });
  }

  sendCursor(x: number, y: number) {
    this.send({ type: "cursor", x: Math.round(x), y: Math.round(y) });
  }

  sendKeyState(keys: number) {
    this.send({ type: "key_state", keys });
  }

  sendCursorSpeed(speed: number) {
    this.send({ type: "cursor_speed", speed });
  }

  sendTeamConfig(teams: Array<{ mode: string; name: string }>) {
    this.send({ type: "team_config", teams });
  }

  sendSelectMap(id: string) {
    this.send({ type: "select_map", id });
  }

  sendStartGame() {
    this.send({ type: "start_game" });
  }

  private send(msg: object) {
    if (this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(msg));
    }
  }
}
