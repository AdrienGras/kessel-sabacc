import { create } from "zustand";
import { persist } from "zustand/middleware";

interface AudioStore {
  volume: number;
  muted: boolean;
  setVolume: (v: number) => void;
  toggleMute: () => void;
}

export const useAudioStore = create<AudioStore>()(
  persist(
    (set) => ({
      volume: 0.5,
      muted: false,
      setVolume: (volume) => set({ volume }),
      toggleMute: () => set((s) => ({ muted: !s.muted })),
    }),
    { name: "sabacc-audio" },
  ),
);
