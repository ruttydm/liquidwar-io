import { GameClient } from "./net";
import { Renderer } from "./renderer";
import { InputHandler } from "./input";

const SERVER_URL = `ws://${location.hostname}:3001`;
const CURSOR_SEND_HZ = 20;

async function main() {
  const canvas = document.getElementById("game") as HTMLCanvasElement;
  const scoreboard = document.getElementById("scoreboard")!;

  const name = prompt("Enter your name:") || "Player";
  const client = new GameClient(SERVER_URL);

  let renderer: Renderer | null = null;
  let input: InputHandler | null = null;

  client.onOpen = () => {
    client.sendJoin(name);
  };

  client.onWelcome = (msg) => {
    renderer = new Renderer(canvas, msg.mapWidth, msg.mapHeight);
    input = new InputHandler(canvas, msg.mapWidth, msg.mapHeight);

    setInterval(() => {
      if (input) {
        client.sendCursor(input.cursorX, input.cursorY);
      }
    }, 1000 / CURSOR_SEND_HZ);
  };

  client.onState = (msg) => {
    if (!renderer) return;

    // Decode base64 bitmap
    const binary = atob(msg.bitmap);
    const bitmap = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bitmap[i] = binary.charCodeAt(i);
    }

    renderer.render(bitmap, msg.cursors);
    updateScoreboard(scoreboard, msg.scores);
  };
}

const COLORS = ["#4287f5", "#f54242", "#42f560", "#f5d742"];

function updateScoreboard(el: HTMLElement, scores: number[]) {
  el.innerHTML = scores
    .map(
      (s, i) =>
        `<span style="color:${COLORS[i]}">P${i + 1}: ${s}</span>`,
    )
    .filter((_, i) => scores[i] > 0)
    .join(" &nbsp; ");
}

main().catch(console.error);
