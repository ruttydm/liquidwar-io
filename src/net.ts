export interface WelcomeMsg {
  type: "welcome";
  playerId: number;
  mapWidth: number;
  mapHeight: number;
  mapData: number[];
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

export type ServerMsg = WelcomeMsg | StateMsg | PlayerJoinedMsg | PlayerLeftMsg;

export class GameClient {
  private ws: WebSocket;
  public onWelcome: ((msg: WelcomeMsg) => void) | null = null;
  public onState: ((msg: StateMsg) => void) | null = null;
  public onOpen: (() => void) | null = null;

  constructor(url: string) {
    this.ws = new WebSocket(url);

    this.ws.onopen = () => {
      this.onOpen?.();
    };

    this.ws.onmessage = (event) => {
      const msg: ServerMsg = JSON.parse(event.data);
      switch (msg.type) {
        case "welcome":
          this.onWelcome?.(msg);
          break;
        case "state":
          this.onState?.(msg);
          break;
      }
    };
  }

  sendJoin(name: string) {
    this.send({ type: "join", name });
  }

  sendCursor(x: number, y: number) {
    this.send({ type: "cursor", x, y });
  }

  private send(msg: object) {
    if (this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(msg));
    }
  }
}
