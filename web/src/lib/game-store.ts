import { create } from "zustand";

interface GameConfig {
  playerName: string;
  botCount: number;
  buyIn: number;
  tokensEnabled: boolean;
  tokensPerPlayer: number;
  difficulty: string;
}

interface GameStore {
  wasmReady: boolean;
  wasmVersion: string | null;
  config: GameConfig | null;
  setWasmReady: (version: string) => void;
  setConfig: (config: GameConfig) => void;
}

export const useGameStore = create<GameStore>((set) => ({
  wasmReady: false,
  wasmVersion: null,
  config: null,
  setWasmReady: (version) => set({ wasmReady: true, wasmVersion: version }),
  setConfig: (config) => set({ config }),
}));
