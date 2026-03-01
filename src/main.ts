import { AudioManager } from "./audio";
import { GameClient, MapInfo } from "./net";
import { Renderer } from "./renderer";
import { InputHandler } from "./input";

const SERVER_URL = `ws://${location.hostname}:3001`;
const CURSOR_SEND_HZ = 20;

const COLORS = ["#4287f5", "#f54242", "#42f560", "#f5d742", "#dc42f5", "#42f5e6"];

interface TeamSlot {
  mode: string;
  name: string;
}

async function main() {
  const canvas = document.getElementById("game") as HTMLCanvasElement;
  const scoreboard = document.getElementById("scoreboard")!;
  const menuRoot = document.getElementById("menu-root")!;
  const mapGrid = document.getElementById("map-grid")!;
  const mapSearch = document.getElementById("map-search") as HTMLInputElement;
  const gameOver = document.getElementById("game-over")!;

  let renderer: Renderer | null = null;
  let input: InputHandler | null = null;
  let cursorInterval: number | null = null;
  let allMaps: MapInfo[] = [];
  let gameRunning = false;
  const audio = new AudioManager();

  // Team config (6 slots)
  const teamConfig: TeamSlot[] = [
    { mode: "human", name: "Player 1" },
    { mode: "cpu", name: "CPU 2" },
    { mode: "off", name: "Player 3" },
    { mode: "off", name: "Player 4" },
    { mode: "off", name: "Player 5" },
    { mode: "off", name: "Player 6" },
  ];

  // --- Menu Navigation ---
  const menuScreens = menuRoot.querySelectorAll<HTMLElement>(".menu-screen");
  function showScreen(name: string) {
    menuScreens.forEach((el) => {
      el.classList.toggle("hidden", el.id !== `menu-${name}`);
    });
  }

  menuRoot.querySelectorAll<HTMLElement>("[data-screen]").forEach((btn) => {
    btn.addEventListener("click", () => {
      showScreen(btn.dataset.screen!);
    });
  });
  menuRoot.querySelectorAll<HTMLElement>(".back-btn").forEach((btn) => {
    btn.addEventListener("click", () => showScreen("main"));
  });

  // --- Options Sliders ---
  const optCursorSpeed = document.getElementById("opt-cursor-speed") as HTMLInputElement;
  const optMusicVol = document.getElementById("opt-music-vol") as HTMLInputElement;
  const optSfxVol = document.getElementById("opt-sfx-vol") as HTMLInputElement;
  const optWaterVol = document.getElementById("opt-water-vol") as HTMLInputElement;

  optMusicVol.addEventListener("input", () => {
    audio.setMusicVolume(parseInt(optMusicVol.value) / 100);
  });
  optSfxVol.addEventListener("input", () => {
    audio.setSfxVolume(parseInt(optSfxVol.value) / 100);
  });
  optWaterVol.addEventListener("input", () => {
    audio.setWaterVolume(parseInt(optWaterVol.value) / 100);
  });
  optCursorSpeed.addEventListener("input", () => {
    client.sendCursorSpeed(parseInt(optCursorSpeed.value));
  });

  // --- Team Setup UI ---
  const teamSlotsEl = document.getElementById("team-slots")!;
  function populateTeamSlots() {
    teamSlotsEl.innerHTML = "";
    for (let i = 0; i < 6; i++) {
      const slot = document.createElement("div");
      slot.className = "team-slot";
      slot.innerHTML = `
        <div class="team-color" style="background:${COLORS[i]}"></div>
        <select data-team="${i}">
          <option value="human"${teamConfig[i].mode === "human" ? " selected" : ""}>Human</option>
          <option value="cpu"${teamConfig[i].mode === "cpu" ? " selected" : ""}>CPU</option>
          <option value="off"${teamConfig[i].mode === "off" ? " selected" : ""}>Off</option>
        </select>
        <input type="text" value="${teamConfig[i].name}" data-team-name="${i}" />
      `;
      teamSlotsEl.appendChild(slot);

      const select = slot.querySelector("select")!;
      select.addEventListener("change", () => {
        teamConfig[i].mode = select.value;
      });
      const nameInput = slot.querySelector("input")!;
      nameInput.addEventListener("input", () => {
        teamConfig[i].name = nameInput.value;
      });
    }
  }
  populateTeamSlots();

  // --- Networking ---
  const client = new GameClient(SERVER_URL);

  client.onOpen = () => {
    client.sendJoin("Player");
  };

  // Server sent a map list — show map selection
  client.onMapList = (msg) => {
    allMaps = msg.maps;
    menuRoot.classList.remove("hidden");
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
        // Send team config before starting
        client.sendTeamConfig(teamConfig.map((t) => ({ mode: t.mode, name: t.name })));
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
    menuRoot.classList.add("hidden");
    gameOver.classList.add("hidden");
    canvas.classList.remove("hidden");
    scoreboard.classList.remove("hidden");
    gameRunning = true;
    renderer = new Renderer(canvas, msg.mapWidth, msg.mapHeight);
    input = new InputHandler(canvas);
    audio.playSfx("go");

    // Upload wall map data for textured rendering
    renderer.setMapData(msg.mapData);

    // Try loading map-specific texture, fall back to defaults
    const defaultBg = "/textures/marble3.png";
    const defaultFg = "/textures/wood2.png";
    const maptexUrl = `/maptex/${msg.mapId}.png`;

    // Try maptex first, if it fails use defaults
    const tryMaptex = new Image();
    tryMaptex.onload = () => {
      renderer!.setTextures(defaultBg, maptexUrl);
    };
    tryMaptex.onerror = () => {
      renderer!.setTextures(defaultBg, defaultFg);
    };
    tryMaptex.src = maptexUrl;

    // Send cursor speed from options
    client.sendCursorSpeed(parseInt(optCursorSpeed.value));

    // Clear old interval if re-joining
    if (cursorInterval) clearInterval(cursorInterval);
    cursorInterval = setInterval(() => {
      if (input) {
        client.sendKeyState(input.keyState);
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

    renderer.render(bitmap, msg.cursors, msg.scores);
    updateScoreboard(scoreboard, msg.scores);

    // Winner detection
    if (gameRunning && msg.tick > 100) {
      const nonZero = msg.scores.filter((s: number) => s > 0);
      if (nonZero.length === 1) {
        const winnerTeam = msg.scores.findIndex((s: number) => s > 0);
        showWinner(winnerTeam);
      }
    }
  };

  function showWinner(teamIdx: number) {
    gameRunning = false;
    if (cursorInterval) {
      clearInterval(cursorInterval);
      cursorInterval = null;
    }
    const winnerText = gameOver.querySelector("#winner-text") as HTMLElement;
    winnerText.textContent = `${teamConfig[teamIdx]?.name || `Team ${teamIdx + 1}`} WINS!`;
    winnerText.style.color = COLORS[teamIdx] || "#fff";
    gameOver.classList.remove("hidden");
  }

  // Return to menu button
  const returnBtn = document.getElementById("return-to-menu");
  if (returnBtn) {
    returnBtn.addEventListener("click", () => {
      gameOver.classList.add("hidden");
      canvas.classList.add("hidden");
      scoreboard.classList.add("hidden");
      menuRoot.classList.remove("hidden");
      showScreen("main");
      renderer = null;
      input = null;
    });
  }
}

function updateScoreboard(el: HTMLElement, scores: number[]) {
  const total = scores.reduce((a, b) => a + b, 0);
  if (total === 0) return;

  el.innerHTML = scores
    .map((s, i) => {
      if (s === 0) return "";
      const pct = ((s / total) * 100).toFixed(1);
      return `<span style="color:${COLORS[i]}">P${i + 1}: ${pct}%</span>`;
    })
    .filter(Boolean)
    .join(" &nbsp; ");
}

main().catch(console.error);
