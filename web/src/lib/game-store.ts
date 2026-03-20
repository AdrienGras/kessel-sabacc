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
  howToPlayDecrypted: boolean;
  setWasmReady: (version: string) => void;
  setConfig: (config: GameConfig) => void;
  setHowToPlayDecrypted: () => void;
  resetHowToPlayDecrypted: () => void;
}

export const useGameStore = create<GameStore>((set) => ({
  wasmReady: false,
  wasmVersion: null,
  config: null,
  howToPlayDecrypted: false,
  setWasmReady: (version) => set({ wasmReady: true, wasmVersion: version }),
  setConfig: (config) => set({ config }),
  setHowToPlayDecrypted: () => set({ howToPlayDecrypted: true }),
  resetHowToPlayDecrypted: () => set({ howToPlayDecrypted: false }),
}));
