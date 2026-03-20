import { Howl, Howler } from "howler";

interface TrackOptions {
  loop?: boolean;
  volume?: number;
}

const tracks = new Map<string, Howl>();
const targetVolumes = new Map<string, number>();
let currentMusicKey: string | null = null;

export const audio = {
  register(key: string, src: string, options: TrackOptions = {}) {
    if (tracks.has(key)) return;
    const vol = options.volume ?? 1;
    targetVolumes.set(key, vol);
    tracks.set(
      key,
      new Howl({
        src: [src],
        loop: options.loop ?? false,
        volume: vol,
        preload: true,
        onloaderror: (_id, err) =>
          console.warn(`[audio] Failed to load "${key}":`, err),
      }),
    );
  },

  playMusic(key: string, fadeIn = 1000) {
    if (currentMusicKey === key) return;

    const next = tracks.get(key);
    if (!next) {
      console.warn(`[audio] Track "${key}" not registered`);
      return;
    }

    // Ensure AudioContext is resumed (may still be suspended)
    this.unlock();

    // Fade out current
    if (currentMusicKey) {
      const prev = tracks.get(currentMusicKey);
      if (prev) {
        prev.fade(prev.volume(), 0, 500);
        prev.once("fade", () => prev.stop());
      }
    }

    currentMusicKey = key;
    next.volume(0);
    next.play();
    next.fade(0, targetVolumes.get(key) ?? 0.4, fadeIn);
  },

  stopMusic(fadeOut = 500) {
    if (!currentMusicKey) return;
    const cur = tracks.get(currentMusicKey);
    if (cur) {
      cur.fade(cur.volume(), 0, fadeOut);
      cur.once("fade", () => cur.stop());
    }
    currentMusicKey = null;
  },

  playSfx(key: string) {
    const sfx = tracks.get(key);
    if (!sfx) {
      console.warn(`[audio] SFX "${key}" not registered`);
      return;
    }
    sfx.play();
  },

  setVolume(v: number) {
    Howler.volume(v);
  },

  setMuted(muted: boolean) {
    Howler.mute(muted);
  },

  unlock() {
    const ctx = Howler.ctx;
    if (ctx && ctx.state === "suspended") {
      ctx.resume();
    }
  },

  getCurrentMusicKey() {
    return currentMusicKey;
  },
};
