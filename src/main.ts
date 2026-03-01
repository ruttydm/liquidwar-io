import { AudioManager } from "./audio";
import { GameClient, MapInfo } from "./net";
import { Renderer } from "./renderer";
import { InputHandler } from "./input";

const SERVER_URL = `ws://${location.hostname}:3001`;
const CURSOR_SEND_HZ = 20;

const COLORS = ["#4287f5", "#f54242", "#42f560", "#f5d742", "#dc42f5", "#42f5e6"];

async function main() {
  const canvas = document.getElementById("game") as HTMLCanvasElement;
  const scoreboard = document.getElementById("scoreboard")!;
  const lobby = document.getElementById("lobby")!;
  const mapGrid = document.getElementById("map-grid")!;
  const mapSearch = document.getElementById("map-search") as HTMLInputElement;

  let renderer: Renderer | null = null;
  let input: InputHandler | null = null;
  let cursorInterval: number | null = null;
  let allMaps: MapInfo[] = [];
  const audio = new AudioManager();

  const client = new GameClient(SERVER_URL);

  client.onOpen = () => {
    client.sendJoin("Player");
  };

  // Server sent a map list — show map selection lobby
  client.onMapList = (msg) => {
    allMaps = msg.maps;
    lobby.classList.remove("hidden");
    renderMapGrid(allMaps);
  };

  function renderMapGrid(maps: MapInfo[]) {
    mapGrid.innerHTML = "";
    for (const map of maps) {
      const card = document.createElement("div");
      card.className = "map-card";
      card.innerHTML = `
        <img src="/maps/${map.id}.png" alt="${map.name}" loading="lazy" />
        <div class="label" title="${map.name}">${map.name}</div>
      `;
      card.addEventListener("click", () => {
        client.sendSelectMap(map.id);
      });
      mapGrid.appendChild(card);
    }
  }

  // Search filter
  mapSearch.addEventListener("input", () => {
    const q = mapSearch.value.toLowerCase();
    const filtered = allMaps.filter(
      (m) => m.name.toLowerCase().includes(q) || m.id.toLowerCase().includes(q),
    );
    renderMapGrid(filtered);
  });

  // Server says game is starting — switch to game view
  client.onWelcome = (msg) => {
    lobby.classList.add("hidden");
    renderer = new Renderer(canvas, msg.mapWidth, msg.mapHeight);
    input = new InputHandler(canvas, msg.mapWidth, msg.mapHeight);
    audio.playSfx("go");

    // Clear old interval if re-joining
    if (cursorInterval) clearInterval(cursorInterval);
    cursorInterval = setInterval(() => {
      if (input) {
        client.sendCursor(input.cursorX, input.cursorY);
      }
    }, 1000 / CURSOR_SEND_HZ) as unknown as number;
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
