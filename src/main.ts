import { AudioManager } from "./audio";
import { GameClient, GameConfig, TeamSlotConfig, MapInfo, LobbyUpdateMsg, RoomListEntry } from "./net";
import { Renderer } from "./renderer";
import { InputHandler } from "./input";

const SERVER_URL = location.port === "5173"
  ? `ws://${location.hostname}:3001`
  : `${location.protocol === "https:" ? "wss:" : "ws:"}//${location.host}/ws`;
const CURSOR_SEND_HZ = 20;

// Generate 32 team colors matching the renderer
function genColors(n: number): string[] {
  const base = ["#4287f5", "#f54242", "#42f560", "#f5d742", "#dc42f5", "#42f5e6"];
  const out = [...base];
  for (let i = 6; i < n; i++) {
    const hue = ((i - 6) * 360 / (n - 6) + 15) % 360;
    const s = 75, l = 55;
    out.push(`hsl(${Math.round(hue)},${s}%,${l}%)`);
  }
  return out;
}
const COLORS = genColors(32);

async function main() {
  const canvas = document.getElementById("game") as HTMLCanvasElement;
  const menuRoot = document.getElementById("menu-root")!;
  const gameOver = document.getElementById("game-over")!;
  const countdownOverlay = document.getElementById("countdown-overlay")!;
  const errorToast = document.getElementById("error-toast")!;
  const playerNameInput = document.getElementById("player-name") as HTMLInputElement;
  const gameMenu = document.getElementById("game-menu")!;

  let renderer: Renderer | null = null;
  let input: InputHandler | null = null;
  let cursorInterval: number | null = null;
  let allMaps: MapInfo[] = [];
  let gameRunning = false;
  let gameMenuOpen = false;
  let currentRoomCode: string | null = null;
  let isHost = false;
  let myConnId = -1;
  let lastLobbyUpdate: LobbyUpdateMsg | null = null;
  const audio = new AudioManager();

  // --- Settings Persistence ---
  interface LW5Settings {
    playerName: string;
    cursorSpeed: number;
    musicVol: number;
    sfxVol: number;
    waterVol: number;
  }
  const DEFAULT_SETTINGS: LW5Settings = {
    playerName: "Player", cursorSpeed: 1, musicVol: 70, sfxVol: 50, waterVol: 15,
  };
  function loadSettings(): LW5Settings {
    try {
      const raw = localStorage.getItem("lw5_settings");
      if (raw) return { ...DEFAULT_SETTINGS, ...JSON.parse(raw) };
    } catch {}
    return { ...DEFAULT_SETTINGS };
  }
  function saveSettings(s: LW5Settings) {
    try { localStorage.setItem("lw5_settings", JSON.stringify(s)); } catch {}
  }
  const settings = loadSettings();

  // --- In-Game Menu (Escape) --- (wired after client created below)
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && gameRunning && countdownOverlay.classList.contains("hidden")) {
      e.preventDefault();
      gameMenuOpen = !gameMenuOpen;
      gameMenu.classList.toggle("hidden", !gameMenuOpen);
    }
  });

  document.getElementById("btn-resume")!.addEventListener("click", () => {
    gameMenuOpen = false;
    gameMenu.classList.add("hidden");
  });

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

  menuRoot.querySelectorAll<HTMLElement>("[data-back]").forEach((btn) => {
    btn.addEventListener("click", () => {
      showScreen(btn.dataset.back!);
    });
  });

  // --- Error Toast ---
  let errorTimeout: number | null = null;
  function showError(message: string) {
    errorToast.textContent = message;
    errorToast.classList.remove("hidden");
    if (errorTimeout) clearTimeout(errorTimeout);
    errorTimeout = setTimeout(() => {
      errorToast.classList.add("hidden");
    }, 3000) as unknown as number;
  }

  // --- Options Sliders ---
  const optCursorSpeed = document.getElementById("opt-cursor-speed") as HTMLInputElement;
  const optMusicVol = document.getElementById("opt-music-vol") as HTMLInputElement;
  const optSfxVol = document.getElementById("opt-sfx-vol") as HTMLInputElement;
  const optWaterVol = document.getElementById("opt-water-vol") as HTMLInputElement;

  // Apply saved settings to UI and audio
  playerNameInput.value = settings.playerName;
  optCursorSpeed.value = String(settings.cursorSpeed);
  optMusicVol.value = String(settings.musicVol);
  optSfxVol.value = String(settings.sfxVol);
  optWaterVol.value = String(settings.waterVol);
  audio.setMusicVolume(settings.musicVol / 100);
  audio.setSfxVolume(settings.sfxVol / 100);
  audio.setWaterVolume(settings.waterVol / 100);

  optMusicVol.addEventListener("input", () => {
    const val = parseInt(optMusicVol.value);
    audio.setMusicVolume(val / 100);
    settings.musicVol = val;
    saveSettings(settings);
  });
  optSfxVol.addEventListener("input", () => {
    const val = parseInt(optSfxVol.value);
    audio.setSfxVolume(val / 100);
    settings.sfxVol = val;
    saveSettings(settings);
  });
  optWaterVol.addEventListener("input", () => {
    const val = parseInt(optWaterVol.value);
    audio.setWaterVolume(val / 100);
    settings.waterVol = val;
    saveSettings(settings);
  });
  optCursorSpeed.addEventListener("input", () => {
    const val = parseInt(optCursorSpeed.value);
    client.sendCursorSpeed(val);
    settings.cursorSpeed = val;
    saveSettings(settings);
  });

  // --- Map Grid Helpers ---
  function renderMapGrid(container: HTMLElement, maps: MapInfo[], onClick: (map: MapInfo) => void, selectedId?: string) {
    container.innerHTML = "";
    for (const map of maps) {
      const card = document.createElement("div");
      card.className = "map-card" + (selectedId === map.id ? " selected" : "");
      card.innerHTML = `
        <img src="/maps/${map.id}.png" alt="${map.name}" loading="lazy" />
        <div class="label" title="${map.name}">${map.name}</div>
      `;
      card.addEventListener("click", () => onClick(map));
      container.appendChild(card);
    }
  }

  function filterMaps(query: string): MapInfo[] {
    const q = query.toLowerCase();
    return allMaps.filter(
      (m) => m.name.toLowerCase().includes(q) || m.id.toLowerCase().includes(q),
    );
  }

  // --- Single Player Setup ---
  const DEFAULT_TEAM_NAMES = [
    "Napoleon", "Clovis", "Henri IV", "Cesar", "Geronimo", "Attila",
    "Genghis", "Cleopatra", "Alexander", "Hannibal", "Spartacus", "Boudicca",
    "Saladin", "Charlemagne", "Leonidas", "Ramses", "Montezuma", "Tokugawa",
    "Bismarck", "Victoria", "Shaka", "Suleiman", "Cyrus", "Pachacuti",
    "Ragnar", "Tamerlane", "Darius", "Barbarossa", "Ashoka", "Cortez",
    "Drake", "Bolivar",
  ];
  const spTeamSlots = document.getElementById("sp-team-slots")!;
  const spMapGrid = document.getElementById("sp-map-grid")!;
  const spMapSearch = document.getElementById("sp-map-search") as HTMLInputElement;

  // SP rule sliders
  const spRuleIds = ["sp-attack", "sp-defense", "sp-health", "sp-influence", "sp-army", "sp-cursor-speed"] as const;
  for (const id of spRuleIds) {
    const slider = document.getElementById(id) as HTMLInputElement;
    const valEl = document.getElementById(`${id}-val`)!;
    slider.addEventListener("input", () => { valEl.textContent = slider.value; });
  }

  // Dynamic team slots
  const TEAM_MODES: Array<"human" | "cpu" | "off"> = ["human", "cpu", "off"];
  interface SpSlot { mode: "human" | "cpu" | "off"; name: string; }
  const spSlots: SpSlot[] = [
    { mode: "human", name: DEFAULT_TEAM_NAMES[0] },
    { mode: "cpu", name: DEFAULT_TEAM_NAMES[1] },
  ];

  function cycleMode(btn: HTMLElement) {
    const cur = btn.getAttribute("data-mode") as "human" | "cpu" | "off";
    const next = TEAM_MODES[(TEAM_MODES.indexOf(cur) + 1) % 3];
    btn.setAttribute("data-mode", next);
    btn.textContent = next;
  }

  function renderSpSlots() {
    spTeamSlots.innerHTML = "";
    for (let i = 0; i < spSlots.length; i++) {
      const slot = spSlots[i];
      const el = document.createElement("div");
      el.className = "sp-team-slot";
      el.innerHTML = `
        <div class="sp-team-color" style="background:${COLORS[i]}"></div>
        <div class="sp-team-mode" data-team="${i}" data-mode="${slot.mode}">${slot.mode}</div>
        <input type="text" value="${slot.name}" maxlength="12" />
        ${spSlots.length > 2 ? '<div class="sp-remove-slot" title="Remove">&times;</div>' : ''}
      `;
      const modeBtn = el.querySelector(".sp-team-mode")!;
      modeBtn.addEventListener("click", (e) => {
        const btn = e.currentTarget as HTMLElement;
        cycleMode(btn);
        spSlots[i].mode = btn.getAttribute("data-mode") as "human" | "cpu" | "off";
      });
      const nameInput = el.querySelector("input") as HTMLInputElement;
      nameInput.addEventListener("input", () => { spSlots[i].name = nameInput.value; });
      const removeBtn = el.querySelector(".sp-remove-slot");
      if (removeBtn) {
        removeBtn.addEventListener("click", () => {
          spSlots.splice(i, 1);
          renderSpSlots();
        });
      }
      spTeamSlots.appendChild(el);
    }
    if (spSlots.length < 32) {
      const addBtn = document.createElement("div");
      addBtn.className = "sp-add-slot";
      addBtn.textContent = "+ Add Player";
      addBtn.addEventListener("click", () => {
        const idx = spSlots.length;
        spSlots.push({ mode: "cpu", name: DEFAULT_TEAM_NAMES[idx] || `Team ${idx + 1}` });
        renderSpSlots();
      });
      spTeamSlots.appendChild(addBtn);
    }
  }
  renderSpSlots();

  function buildSpGameConfig(): GameConfig {
    const pName = playerNameInput.value.trim() || "Player";
    let usedPlayerName = false;
    const teams: TeamSlotConfig[] = spSlots.map((s, i) => {
      let name = s.name.trim() || DEFAULT_TEAM_NAMES[i] || `Team ${i + 1}`;
      if (s.mode === "human" && !usedPlayerName) {
        name = pName;
        usedPlayerName = true;
      }
      return { mode: s.mode, name };
    });
    return {
      teams,
      fighter_attack: parseInt((document.getElementById("sp-attack") as HTMLInputElement).value),
      fighter_defense: parseInt((document.getElementById("sp-defense") as HTMLInputElement).value),
      fighter_new_health: parseInt((document.getElementById("sp-health") as HTMLInputElement).value),
      number_influence: parseInt((document.getElementById("sp-influence") as HTMLInputElement).value),
      fighter_number: parseInt((document.getElementById("sp-army") as HTMLInputElement).value),
      cursor_speed: parseInt((document.getElementById("sp-cursor-speed") as HTMLInputElement).value),
    };
  }

  let spSelectedMapId: string | null = null;

  function renderSpMapGrid(maps: MapInfo[]) {
    renderMapGrid(spMapGrid, maps, (map) => {
      spSelectedMapId = map.id;
      renderSpMapGrid(filterMaps(spMapSearch.value));
    }, spSelectedMapId || undefined);
  }

  spMapSearch.addEventListener("input", () => {
    renderSpMapGrid(filterMaps(spMapSearch.value));
  });

  const btnSpStart = document.getElementById("btn-sp-start")!;
  btnSpStart.addEventListener("click", () => {
    if (!spSelectedMapId && allMaps.length > 0) {
      spSelectedMapId = allMaps[0].id;
    }
    if (!spSelectedMapId) {
      showError("Select a map first");
      return;
    }
    client.sendStartSinglePlayer(spSelectedMapId, buildSpGameConfig());
  });

  // --- Multiplayer Buttons ---
  const btnQuickPlay = document.getElementById("btn-quick-play")!;
  const btnCreateRoom = document.getElementById("btn-create-room")!;
  const btnJoinCode = document.getElementById("btn-join-code")!;
  const joinCodeInput = document.getElementById("join-code-input") as HTMLInputElement;
  const btnRefreshRooms = document.getElementById("btn-refresh-rooms")!;
  const roomTableBody = document.getElementById("room-table-body")!;
  const noRoomsMsg = document.getElementById("no-rooms-msg")!;

  btnQuickPlay.addEventListener("click", () => client.sendQuickPlay());
  btnCreateRoom.addEventListener("click", () => {
    renderCrMapGrid(allMaps);
    showScreen("create-room");
  });

  btnJoinCode.addEventListener("click", () => {
    const code = joinCodeInput.value.trim().toUpperCase();
    if (code.length === 4) {
      client.sendJoinRoom(code);
    } else {
      showError("Enter a 4-character room code");
    }
  });
  joinCodeInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter") btnJoinCode.click();
  });

  btnRefreshRooms.addEventListener("click", () => client.sendListRooms());

  // Auto-request room list when entering browse screen
  document.querySelector("[data-screen='browse']")?.addEventListener("click", () => {
    client.sendListRooms();
  });

  function renderRoomList(rooms: RoomListEntry[]) {
    roomTableBody.innerHTML = "";
    if (rooms.length === 0) {
      noRoomsMsg.classList.remove("hidden");
      return;
    }
    noRoomsMsg.classList.add("hidden");
    for (const room of rooms) {
      const tr = document.createElement("tr");
      const modeBadge = room.isVanilla
        ? '<span class="mode-badge vanilla">Vanilla</span>'
        : '<span class="mode-badge custom">Custom</span>';
      tr.innerHTML = `
        <td>${room.code}</td>
        <td>${room.hostName}</td>
        <td>${room.playerCount}/${room.maxPlayers}</td>
        <td>${room.mapId || "\u2014"}</td>
        <td>${modeBadge}</td>
        <td><button class="join-btn" data-code="${room.code}">Join</button></td>
      `;
      tr.querySelector(".join-btn")!.addEventListener("click", () => {
        client.sendJoinRoom(room.code);
      });
      roomTableBody.appendChild(tr);
    }
  }

  // --- Create Room Screen ---
  const crMapGrid = document.getElementById("cr-map-grid")!;
  const crMapSearch = document.getElementById("cr-map-search") as HTMLInputElement;
  const crVanillaBtn = document.getElementById("cr-vanilla-btn")!;
  const crCustomBtn = document.getElementById("cr-custom-btn")!;
  const crBotsSlider = document.getElementById("cr-bots") as HTMLInputElement;
  const crBotsVal = document.getElementById("cr-bots-val")!;
  const crPublicToggle = document.getElementById("cr-public-toggle")!;
  const btnDoCreateRoom = document.getElementById("btn-do-create-room")!;
  let crIsVanilla = true;
  let crSelectedMapId: string | null = null;

  // CR rule sliders
  const crRuleIds = ["cr-attack", "cr-defense", "cr-health", "cr-influence", "cr-army"] as const;
  for (const id of crRuleIds) {
    const slider = document.getElementById(id) as HTMLInputElement;
    const valEl = document.getElementById(`${id}-val`)!;
    slider.addEventListener("input", () => { valEl.textContent = slider.value; });
  }

  crBotsSlider.addEventListener("input", () => { crBotsVal.textContent = crBotsSlider.value; });

  function setCrVanilla(vanilla: boolean) {
    crIsVanilla = vanilla;
    crVanillaBtn.classList.toggle("active", vanilla);
    crCustomBtn.classList.toggle("active", !vanilla);
    const rulesSection = document.getElementById("cr-rules-section")!;
    rulesSection.querySelectorAll(".rule-row").forEach(r => {
      r.classList.toggle("disabled", vanilla);
    });
    if (vanilla) {
      // Reset sliders to defaults
      for (const id of crRuleIds) {
        const slider = document.getElementById(id) as HTMLInputElement;
        const valEl = document.getElementById(`${id}-val`)!;
        const def = id === "cr-army" ? "16" : "8";
        slider.value = def;
        valEl.textContent = def;
      }
    }
  }
  crVanillaBtn.addEventListener("click", () => setCrVanilla(true));
  crCustomBtn.addEventListener("click", () => setCrVanilla(false));

  crPublicToggle.addEventListener("click", () => {
    const nowPublic = crPublicToggle.textContent === "OFF";
    crPublicToggle.textContent = nowPublic ? "ON" : "OFF";
    crPublicToggle.classList.toggle("active", nowPublic);
  });

  function renderCrMapGrid(maps: MapInfo[]) {
    renderMapGrid(crMapGrid, maps, (map) => {
      crSelectedMapId = map.id;
      renderCrMapGrid(filterMaps(crMapSearch.value));
    }, crSelectedMapId || undefined);
  }

  crMapSearch.addEventListener("input", () => {
    renderCrMapGrid(filterMaps(crMapSearch.value));
  });

  btnDoCreateRoom.addEventListener("click", () => {
    if (!crSelectedMapId && allMaps.length > 0) {
      crSelectedMapId = allMaps[0].id;
    }
    if (!crSelectedMapId) {
      showError("Select a map first");
      return;
    }
    const isPublic = crPublicToggle.textContent === "ON";
    const botCount = parseInt(crBotsSlider.value);
    client.sendCreateRoom({ isPublic, mapId: crSelectedMapId, botCount, isVanilla: crIsVanilla });
  });

  // --- Room Lobby ---
  const lobbyRoomCode = document.getElementById("lobby-room-code")!;
  const lobbyPlayerList = document.getElementById("lobby-player-list")!;
  const hostControls = document.getElementById("host-controls")!;
  const lobbyBotCount = document.getElementById("lobby-bot-count") as HTMLInputElement;
  const lobbyBotCountVal = document.getElementById("lobby-bot-count-val")!;
  const lobbyPublicToggle = document.getElementById("lobby-public-toggle")!;
  const lobbyMapGrid = document.getElementById("lobby-map-grid")!;
  const lobbyMapSearch = document.getElementById("lobby-map-search") as HTMLInputElement;
  const btnReady = document.getElementById("btn-ready")!;
  const btnStartGame = document.getElementById("btn-start-game")!;
  const btnLeaveRoom = document.getElementById("btn-leave-room")!;
  const lobbyVanillaBtn = document.getElementById("lobby-vanilla-btn")!;
  const lobbyCustomBtn = document.getElementById("lobby-custom-btn")!;
  let lobbyIsVanilla = true;

  const lobbyRuleRowIds = ["lobby-attack-row", "lobby-defense-row", "lobby-health-row", "lobby-influence-row", "lobby-army-row"];

  function setLobbyVanilla(vanilla: boolean) {
    lobbyIsVanilla = vanilla;
    lobbyVanillaBtn.classList.toggle("active", vanilla);
    lobbyCustomBtn.classList.toggle("active", !vanilla);
    for (const id of lobbyRuleRowIds) {
      document.getElementById(id)?.classList.toggle("disabled", vanilla);
    }
    if (vanilla) {
      const lobbyRuleDefaults: Record<string, string> = { "lobby-attack": "8", "lobby-defense": "8", "lobby-health": "8", "lobby-influence": "8", "lobby-army": "16" };
      for (const [id, def] of Object.entries(lobbyRuleDefaults)) {
        const slider = document.getElementById(id) as HTMLInputElement;
        const valEl = document.getElementById(`${id}-val`)!;
        slider.value = def;
        valEl.textContent = def;
      }
    }
    client.sendSetVanilla(vanilla);
  }
  lobbyVanillaBtn.addEventListener("click", () => { if (isHost) setLobbyVanilla(true); });
  lobbyCustomBtn.addEventListener("click", () => { if (isHost) setLobbyVanilla(false); });

  lobbyRoomCode.addEventListener("click", () => {
    if (currentRoomCode) {
      navigator.clipboard.writeText(currentRoomCode).catch(() => {});
    }
  });

  lobbyBotCount.addEventListener("input", () => {
    lobbyBotCountVal.textContent = lobbyBotCount.value;
    client.sendSetBots(parseInt(lobbyBotCount.value));
  });

  lobbyPublicToggle.addEventListener("click", () => {
    const nowPublic = lobbyPublicToggle.textContent === "OFF";
    client.sendSetPublic(nowPublic);
  });

  lobbyMapSearch.addEventListener("input", () => {
    const selectedMap = lastLobbyUpdate?.settings.map_id;
    renderMapGrid(lobbyMapGrid, filterMaps(lobbyMapSearch.value), (map) => {
      if (isHost) client.sendSetMap(map.id);
    }, selectedMap);
  });

  // Lobby rule sliders
  const lobbyRuleIds = ["lobby-attack", "lobby-defense", "lobby-health", "lobby-influence", "lobby-army"] as const;
  for (const id of lobbyRuleIds) {
    const slider = document.getElementById(id) as HTMLInputElement;
    const valEl = document.getElementById(`${id}-val`)!;
    slider.addEventListener("input", () => { valEl.textContent = slider.value; });
  }

  function buildLobbyGameConfig(): GameConfig {
    return {
      teams: [],
      fighter_attack: parseInt((document.getElementById("lobby-attack") as HTMLInputElement).value),
      fighter_defense: parseInt((document.getElementById("lobby-defense") as HTMLInputElement).value),
      fighter_new_health: parseInt((document.getElementById("lobby-health") as HTMLInputElement).value),
      number_influence: parseInt((document.getElementById("lobby-influence") as HTMLInputElement).value),
      fighter_number: parseInt((document.getElementById("lobby-army") as HTMLInputElement).value),
      cursor_speed: 1,
    };
  }

  btnReady.addEventListener("click", () => client.sendToggleReady());
  btnStartGame.addEventListener("click", () => client.sendStartGame(buildLobbyGameConfig()));
  btnLeaveRoom.addEventListener("click", () => client.sendLeaveRoom());

  function renderLobby(msg: LobbyUpdateMsg) {
    lastLobbyUpdate = msg;
    currentRoomCode = msg.code;
    isHost = msg.host_id === myConnId;

    lobbyRoomCode.innerHTML = `${msg.code}<span class="copy-hint">click to copy</span>`;

    let playersHtml = "<h3>Players</h3>";
    for (let i = 0; i < msg.players.length; i++) {
      const p = msg.players[i];
      const color = COLORS[i % 32];
      const readyIcon = p.ready
        ? '<span class="ready-icon">&#10003;</span>'
        : '<span class="not-ready-icon">&#9679;</span>';
      const hostTag = p.id === msg.host_id ? '<span class="host-tag">HOST</span>' : "";
      playersHtml += `
        <div class="lobby-player">
          <span class="dot" style="background:${color}"></span>
          ${readyIcon}
          <span style="color:${color}">${p.name}</span>
          ${hostTag}
        </div>
      `;
    }
    lobbyPlayerList.innerHTML = playersHtml;

    // Sync vanilla state
    lobbyIsVanilla = msg.settings.is_vanilla;
    lobbyVanillaBtn.classList.toggle("active", lobbyIsVanilla);
    lobbyCustomBtn.classList.toggle("active", !lobbyIsVanilla);
    for (const id of lobbyRuleRowIds) {
      document.getElementById(id)?.classList.toggle("disabled", lobbyIsVanilla);
    }

    if (isHost) {
      hostControls.style.display = "";
      btnStartGame.classList.remove("hidden");
      btnReady.classList.add("hidden");
      lobbyBotCount.value = String(msg.settings.bot_count);
      lobbyBotCountVal.textContent = String(msg.settings.bot_count);
      lobbyPublicToggle.textContent = msg.settings.is_public ? "ON" : "OFF";
      lobbyPublicToggle.classList.toggle("active", msg.settings.is_public);
    } else {
      hostControls.style.display = "none";
      btnStartGame.classList.add("hidden");
      btnReady.classList.remove("hidden");
      const me = msg.players.find((p) => p.id === myConnId);
      if (me) {
        btnReady.textContent = me.ready ? "Unready" : "Ready";
        btnReady.classList.toggle("is-ready", me.ready);
      }
    }

    renderMapGrid(lobbyMapGrid, allMaps, (map) => {
      if (isHost) client.sendSetMap(map.id);
    }, msg.settings.map_id);
  }

  // --- Networking ---
  const client = new GameClient(SERVER_URL);

  client.onOpen = () => {
    const name = playerNameInput.value.trim() || "Player";
    client.sendJoin(name);
  };

  playerNameInput.addEventListener("change", () => {
    const name = playerNameInput.value.trim() || "Player";
    client.sendJoin(name);
    settings.playerName = name;
    saveSettings(settings);
  });

  // Wire game menu buttons that need client
  document.getElementById("btn-leave-game")!.addEventListener("click", () => {
    gameMenuOpen = false;
    gameMenu.classList.add("hidden");
    client.sendLeaveRoom();
  });
  const gmCursorSpeed = document.getElementById("gm-cursor-speed") as HTMLInputElement;
  gmCursorSpeed.addEventListener("input", () => {
    client.sendCursorSpeed(parseInt(gmCursorSpeed.value));
  });

  client.onMapList = (msg) => {
    allMaps = msg.maps;
    myConnId = msg.yourId;

    menuRoot.classList.remove("hidden");
    gameOver.classList.add("hidden");
    countdownOverlay.classList.add("hidden");
    gameMenu.classList.add("hidden");
    canvas.classList.add("hidden");
    gameRunning = false;
    gameMenuOpen = false;

    renderSpMapGrid(allMaps);

    if (!currentRoomCode) {
      showScreen("main");
    }
  };

  client.onRoomCreated = (msg) => {
    currentRoomCode = msg.code;
    showScreen("lobby");
  };

  client.onRoomList = (msg) => {
    renderRoomList(msg.rooms);
  };

  client.onLobbyUpdate = (msg) => {
    currentRoomCode = msg.code;
    renderLobby(msg);

    const lobbyScreen = document.getElementById("menu-lobby")!;
    if (lobbyScreen.classList.contains("hidden")) {
      showScreen("lobby");
    }
  };

  client.onCountdown = (msg) => {
    if (msg.seconds > 0) {
      countdownOverlay.textContent = String(msg.seconds);
      countdownOverlay.classList.remove("hidden");
    } else {
      countdownOverlay.textContent = "GO!";
      setTimeout(() => countdownOverlay.classList.add("hidden"), 500);
    }
  };

  client.onError = (msg) => {
    showError(msg.message);
  };

  client.onLeftRoom = () => {
    currentRoomCode = null;
    isHost = false;
    lastLobbyUpdate = null;
    gameRunning = false;
    gameMenuOpen = false;

    canvas.classList.add("hidden");
    gameOver.classList.add("hidden");
    countdownOverlay.classList.add("hidden");
    gameMenu.classList.add("hidden");
    menuRoot.classList.remove("hidden");

    if (cursorInterval) {
      clearInterval(cursorInterval);
      cursorInterval = null;
    }
    renderer?.destroy();
    renderer = null;
    input = null;

    showScreen("main");
  };

  client.onWelcome = (msg) => {
    menuRoot.classList.add("hidden");
    gameOver.classList.add("hidden");
    countdownOverlay.classList.add("hidden");
    gameMenu.classList.add("hidden");
    gameMenuOpen = false;
    canvas.classList.remove("hidden");
    gameRunning = true;
    renderer = new Renderer(canvas, msg.mapWidth, msg.mapHeight);
    input = new InputHandler(canvas);
    audio.playSfx("go");

    // Pass team names to renderer (indexed by team ID)
    const teamNames: string[] = [];
    if (lastLobbyUpdate) {
      // MP: player names by lobby order (team IDs assigned sequentially)
      for (let i = 0; i < lastLobbyUpdate.players.length; i++) {
        teamNames[i] = lastLobbyUpdate.players[i].name;
      }
    } else {
      // SP: use spSlots names
      for (let i = 0; i < spSlots.length; i++) {
        if (spSlots[i].mode !== "off") teamNames[i] = spSlots[i].name;
      }
    }
    renderer.setTeamNames(teamNames);

    renderer.setMapData(msg.mapData);

    // Pick wall texture deterministically from map ID hash
    const allTextures = ["amethyst","bricks","crash1","electricblue","granite2",
      "greenmess","lumps","marble3","pebbles","pine","poolbottom","qbert",
      "redcubes","smallsquares","terra","wood2"];
    function hashStr(s: string): number {
      let h = 0;
      for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) | 0;
      return Math.abs(h);
    }
    const texHash = hashStr(msg.mapId);
    const wallTex = `/textures/${allTextures[texHash % allTextures.length]}.png`;

    const maptexUrl = `/maptex/${msg.mapId}.png`;
    const tryMaptex = new Image();
    tryMaptex.onload = () => {
      renderer!.setTextures(wallTex, maptexUrl);
    };
    tryMaptex.onerror = () => {
      const floorTex = `/textures/${allTextures[(texHash + 7) % allTextures.length]}.png`;
      renderer!.setTextures(wallTex, floorTex);
    };
    tryMaptex.src = maptexUrl;

    client.sendCursorSpeed(parseInt(optCursorSpeed.value));
    gmCursorSpeed.value = optCursorSpeed.value;

    if (cursorInterval) clearInterval(cursorInterval);
    cursorInterval = setInterval(() => {
      if (input) {
        client.sendKeyState(input.keyState);
      }
    }, 1000 / CURSOR_SEND_HZ) as unknown as number;
  };

  client.onState = (msg) => {
    if (!renderer) return;
    renderer.render(msg.bitmap, msg.cursors, msg.scores);
  };

  client.onGameOver = (msg) => {
    gameRunning = false;
    gameMenuOpen = false;
    gameMenu.classList.add("hidden");
    if (cursorInterval) {
      clearInterval(cursorInterval);
      cursorInterval = null;
    }
    const winnerText = gameOver.querySelector("#winner-text") as HTMLElement;
    winnerText.textContent = `${msg.winnerName} WINS!`;
    winnerText.style.color = COLORS[msg.winnerTeam % 32] || "#fff";
    gameOver.classList.remove("hidden");
  };
}

main().catch(console.error);
