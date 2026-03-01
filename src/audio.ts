import MidiPlayer from "midi-player-js";
import Soundfont from "soundfont-player";

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

const SFX: Record<string, string> = {
  go: "war",
  loose: "foghorn",
  splash: "splash1",
  clock: "clock1",
  crowd: "crowd1",
  cuckoo: "cuckoo",
};

// GM program number -> soundfont-player instrument name
const GM_INSTRUMENTS: Record<number, string> = {
  0: "acoustic_grand_piano",
  1: "bright_acoustic_piano",
  2: "electric_grand_piano",
  3: "honkytonk_piano",
  4: "electric_piano_1",
  5: "electric_piano_2",
  6: "harpsichord",
  7: "clavinet",
  8: "celesta",
  9: "glockenspiel",
  10: "music_box",
  11: "vibraphone",
  12: "marimba",
  13: "xylophone",
  14: "tubular_bells",
  15: "dulcimer",
  16: "drawbar_organ",
  17: "percussive_organ",
  18: "rock_organ",
  19: "church_organ",
  20: "reed_organ",
  21: "accordion",
  22: "harmonica",
  23: "tango_accordion",
  24: "acoustic_guitar_nylon",
  25: "acoustic_guitar_steel",
  26: "electric_guitar_jazz",
  27: "electric_guitar_clean",
  28: "electric_guitar_muted",
  29: "overdriven_guitar",
  30: "distortion_guitar",
  31: "guitar_harmonics",
  32: "acoustic_bass",
  33: "electric_bass_finger",
  34: "electric_bass_pick",
  35: "fretless_bass",
  36: "slap_bass_1",
  37: "slap_bass_2",
  38: "synth_bass_1",
  39: "synth_bass_2",
  40: "violin",
  41: "viola",
  42: "cello",
  43: "contrabass",
  44: "tremolo_strings",
  45: "pizzicato_strings",
  46: "orchestral_harp",
  47: "timpani",
  48: "string_ensemble_1",
  49: "string_ensemble_2",
  50: "synth_strings_1",
  51: "synth_strings_2",
  52: "choir_aahs",
  53: "voice_oohs",
  54: "synth_choir",
  55: "orchestra_hit",
  56: "trumpet",
  57: "trombone",
  58: "tuba",
  59: "muted_trumpet",
  60: "french_horn",
  61: "brass_section",
  62: "synth_brass_1",
  63: "synth_brass_2",
  64: "soprano_sax",
  65: "alto_sax",
  66: "tenor_sax",
  67: "baritone_sax",
  68: "oboe",
  69: "english_horn",
  70: "bassoon",
  71: "clarinet",
  72: "piccolo",
  73: "flute",
  74: "recorder",
  75: "pan_flute",
  76: "blown_bottle",
  77: "shakuhachi",
  78: "whistle",
  79: "ocarina",
  80: "lead_1_square",
  81: "lead_2_sawtooth",
  82: "lead_3_calliope",
  83: "lead_4_chiff",
  84: "lead_5_charang",
  85: "lead_6_voice",
  86: "lead_7_fifths",
  87: "lead_8_bass_lead",
  88: "pad_1_new_age",
  89: "pad_2_warm",
  90: "pad_3_polysynth",
  91: "pad_4_choir",
  92: "pad_5_bowed",
  93: "pad_6_metallic",
  94: "pad_7_halo",
  95: "pad_8_sweep",
  96: "fx_1_rain",
  97: "fx_2_soundtrack",
  98: "fx_3_crystal",
  99: "fx_4_atmosphere",
  100: "fx_5_brightness",
  101: "fx_6_goblins",
  102: "fx_7_echoes",
  103: "fx_8_scifi",
  104: "sitar",
  105: "banjo",
  106: "shamisen",
  107: "koto",
  108: "kalimba",
  109: "bagpipe",
  110: "fiddle",
  111: "shanai",
  112: "tinkle_bell",
  113: "agogo",
  114: "steel_drums",
  115: "woodblock",
  116: "taiko_drum",
  117: "melodic_tom",
  118: "synth_drum",
  119: "reverse_cymbal",
};

type InstrumentPlayer = Soundfont.Player;

export class AudioManager {
  private ac: AudioContext | null = null;
  private musicGain: GainNode | null = null;
  private sfxGain: GainNode | null = null;
  private waterGain: GainNode | null = null;

  private waterEl: HTMLAudioElement | null = null;
  private currentTrack = -1;
  private musicVolume = 0.5;
  private sfxVolume = 0.6;
  private waterVolume = 0.3;
  private enabled = false;

  // MIDI playback state
  private midiPlayer: MidiPlayer.Player | null = null;
  private instruments: Map<string, InstrumentPlayer> = new Map();
  private channelProgram: Map<number, number> = new Map();
  private activeNotes: Map<string, InstrumentPlayer> = new Map();
  private loadingInstruments: Set<string> = new Set();

  constructor() {
    const enable = () => {
      if (!this.enabled) {
        this.enabled = true;
        this.ac = new AudioContext();

        this.musicGain = this.ac.createGain();
        this.musicGain.gain.value = this.musicVolume;
        this.musicGain.connect(this.ac.destination);

        this.sfxGain = this.ac.createGain();
        this.sfxGain.gain.value = this.sfxVolume;
        this.sfxGain.connect(this.ac.destination);

        this.waterGain = this.ac.createGain();
        this.waterGain.gain.value = this.waterVolume;
        this.waterGain.connect(this.ac.destination);

        this.playRandomMusic();
        this.playRandomWater();
      }
    };
    document.addEventListener("click", enable, { once: true });
    document.addEventListener("keydown", enable, { once: true });
  }

  private async loadInstrument(name: string): Promise<InstrumentPlayer | null> {
    if (this.instruments.has(name)) return this.instruments.get(name)!;
    if (this.loadingInstruments.has(name)) return null;
    if (!this.ac || !this.musicGain) return null;

    this.loadingInstruments.add(name);
    try {
      const inst = await Soundfont.instrument(this.ac, name as any, {
        soundfont: "MusyngKite",
        destination: this.musicGain,
      });
      this.instruments.set(name, inst);
      return inst;
    } catch {
      return null;
    } finally {
      this.loadingInstruments.delete(name);
    }
  }

  private async preloadMidiInstruments(url: string): Promise<Set<string>> {
    const names = new Set<string>();
    try {
      const resp = await fetch(url);
      const buf = await resp.arrayBuffer();
      const arr = new Uint8Array(buf);

      // Scan for program change events (0xC0-0xCF)
      for (let i = 0; i < arr.length - 1; i++) {
        const status = arr[i];
        if (status >= 0xc0 && status <= 0xcf) {
          const prog = arr[i + 1];
          const instName = GM_INSTRUMENTS[prog];
          if (instName) names.add(instName);
        }
      }
    } catch { /* ignore */ }

    // Always include piano as fallback
    names.add("acoustic_grand_piano");
    return names;
  }

  private async playRandomMusic() {
    if (!this.enabled || !this.ac) return;

    let next = Math.floor(Math.random() * MUSIC_TRACKS.length);
    if (next === this.currentTrack && MUSIC_TRACKS.length > 1) {
      next = (next + 1) % MUSIC_TRACKS.length;
    }
    this.currentTrack = next;

    // Stop previous playback
    if (this.midiPlayer) {
      this.midiPlayer.stop();
      this.midiPlayer = null;
    }
    this.stopAllNotes();
    this.channelProgram.clear();

    const trackName = MUSIC_TRACKS[next];
    const url = `/music/${trackName}.mid`;

    // Preload instruments used in this MIDI file
    const instNames = await this.preloadMidiInstruments(url);
    await Promise.all([...instNames].map((n) => this.loadInstrument(n)));

    // Fetch and play the MIDI file
    try {
      const resp = await fetch(url);
      const buf = await resp.arrayBuffer();

      this.midiPlayer = new MidiPlayer.Player((event: any) => {
        this.handleMidiEvent(event);
      });

      this.midiPlayer.on("endOfFile", () => {
        this.stopAllNotes();
        setTimeout(() => this.playRandomMusic(), 1000);
      });

      this.midiPlayer.loadArrayBuffer(buf);
      this.midiPlayer.play();
    } catch {
      setTimeout(() => this.playRandomMusic(), 3000);
    }
  }

  private handleMidiEvent(event: any) {
    if (!this.ac) return;

    if (event.name === "Program Change") {
      this.channelProgram.set(event.channel, event.value);
      // Preload the instrument
      const instName = GM_INSTRUMENTS[event.value] || "acoustic_grand_piano";
      this.loadInstrument(instName);
      return;
    }

    // Skip percussion channel (10)
    if (event.channel === 10) return;

    if (event.name === "Note on" && event.velocity > 0) {
      const prog = this.channelProgram.get(event.channel) ?? 0;
      const instName = GM_INSTRUMENTS[prog] || "acoustic_grand_piano";
      const inst = this.instruments.get(instName);
      if (!inst) return;

      const noteKey = `${event.channel}-${event.noteNumber}`;
      // Stop previous note on same channel/note
      const prev = this.activeNotes.get(noteKey);
      if (prev) prev.stop();

      const vel = event.velocity / 100; // midi-player-js normalizes to 0-100
      inst.play(event.noteNumber.toString(), this.ac.currentTime, {
        gain: vel,
        duration: 4,
      });
      this.activeNotes.set(noteKey, inst);
    }

    if (
      event.name === "Note off" ||
      (event.name === "Note on" && event.velocity === 0)
    ) {
      const noteKey = `${event.channel}-${event.noteNumber}`;
      const inst = this.activeNotes.get(noteKey);
      if (inst) {
        inst.stop(this.ac!.currentTime);
        this.activeNotes.delete(noteKey);
      }
    }
  }

  private stopAllNotes() {
    for (const inst of this.activeNotes.values()) {
      try { inst.stop(); } catch { /* ignore */ }
    }
    this.activeNotes.clear();
  }

  private playRandomWater() {
    if (!this.enabled) return;

    const idx = Math.floor(Math.random() * WATER_SOUNDS.length);
    if (this.waterEl) {
      this.waterEl.pause();
    }

    this.waterEl = new Audio(`/water/${WATER_SOUNDS[idx]}.wav`);
    this.waterEl.volume = this.waterVolume;
    this.waterEl.loop = true;
    this.waterEl.play().catch(() => {});
  }

  playSfx(name: string) {
    if (!this.enabled) return;
    const file = SFX[name];
    if (!file) return;

    const audio = new Audio(`/sfx/${file}.wav`);
    audio.volume = this.sfxVolume;
    audio.play().catch(() => {});
  }

  setMusicVolume(v: number) {
    this.musicVolume = v;
    if (this.musicGain) this.musicGain.gain.value = v;
  }

  setSfxVolume(v: number) {
    this.sfxVolume = v;
  }

  setWaterVolume(v: number) {
    this.waterVolume = v;
    if (this.waterEl) this.waterEl.volume = v;
  }

  stopAll() {
    if (this.midiPlayer) {
      this.midiPlayer.stop();
      this.midiPlayer = null;
    }
    this.stopAllNotes();
    if (this.waterEl) {
      this.waterEl.pause();
      this.waterEl = null;
    }
  }
}
