import { create } from "zustand";

interface GameStore {
  wasmReady: boolean;
  wasmVersion: string | null;
  setWasmReady: (version: string) => void;
}

export const useGameStore = create<GameStore>((set) => ({
  wasmReady: false,
  wasmVersion: null,
  setWasmReady: (version) => set({ wasmReady: true, wasmVersion: version }),
}));
