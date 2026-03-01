const MUSIC_TRACKS = [
  "clarity",
  "fodder",
  "headless",
  "marauder",
  "return",
  "thmoov",
  "torqued",
];

const WATER_SOUNDS = [
  "water",
  "bubble",
  "tidal",
  "amb3",
  "amb4",
  "bath1",
  "bath2",
  "flush",
  "forest1",
  "kitch4",
  "lavaflow",
  "niagara",
  "shower1",
  "sodapor",
  "thundr2",
  "thundr3",
];

const SFX = {
  go: "war",
  loose: "foghorn",
  splash: "splash1",
  clock: "clock1",
  crowd: "crowd1",
  cuckoo: "cuckoo",
};

export class AudioManager {
  private musicEl: HTMLAudioElement | null = null;
  private waterEl: HTMLAudioElement | null = null;
  private currentTrack = -1;
  private musicVolume = 0.5;
  private sfxVolume = 0.6;
  private waterVolume = 0.3;
  private enabled = false;

  constructor() {
    // Audio needs user interaction to start
    const enable = () => {
      if (!this.enabled) {
        this.enabled = true;
        this.playRandomMusic();
        this.playRandomWater();
      }
    };
    document.addEventListener("click", enable, { once: true });
    document.addEventListener("keydown", enable, { once: true });
  }

  private playRandomMusic() {
    if (!this.enabled) return;

    let next = Math.floor(Math.random() * MUSIC_TRACKS.length);
    if (next === this.currentTrack && MUSIC_TRACKS.length > 1) {
      next = (next + 1) % MUSIC_TRACKS.length;
    }
    this.currentTrack = next;

    if (this.musicEl) {
      this.musicEl.pause();
    }

    this.musicEl = new Audio(`/music/${MUSIC_TRACKS[next]}.ogg`);
    this.musicEl.volume = this.musicVolume;
    this.musicEl.addEventListener("ended", () => this.playRandomMusic());
    this.musicEl.addEventListener("error", () => {
      // If OGG not available, try next track after delay
      setTimeout(() => this.playRandomMusic(), 2000);
    });
    this.musicEl.play().catch(() => {});
  }

  private playRandomWater() {
    if (!this.enabled) return;

    const idx = Math.floor(Math.random() * WATER_SOUNDS.length);
    if (this.waterEl) {
      this.waterEl.pause();
    }

    this.waterEl = new Audio(`/water/${WATER_SOUNDS[idx]}.ogg`);
    this.waterEl.volume = this.waterVolume;
    this.waterEl.loop = true;
    this.waterEl.play().catch(() => {});
  }

  playSfx(name: keyof typeof SFX) {
    if (!this.enabled) return;
    const file = SFX[name];
    if (!file) return;

    const audio = new Audio(`/sfx/${file}.ogg`);
    audio.volume = this.sfxVolume;
    audio.play().catch(() => {});
  }

  setMusicVolume(v: number) {
    this.musicVolume = v;
    if (this.musicEl) this.musicEl.volume = v;
  }

  setWaterVolume(v: number) {
    this.waterVolume = v;
    if (this.waterEl) this.waterEl.volume = v;
  }

  stopAll() {
    if (this.musicEl) {
      this.musicEl.pause();
      this.musicEl = null;
    }
    if (this.waterEl) {
      this.waterEl.pause();
      this.waterEl = null;
    }
  }
}
