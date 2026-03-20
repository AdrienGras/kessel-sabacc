import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./globals.css";
import App from "./App";
import { audio } from "./lib/audio";
import { useAudioStore } from "./lib/audio-store";

// Register audio tracks at boot
audio.register("menu-theme", import.meta.env.BASE_URL + "audio/star_wars_cantina_band_chill_astronaut.mp3", {
  loop: true,
  volume: 0.4,
});

// Sync persisted audio preferences
const { volume, muted } = useAudioStore.getState();
audio.setVolume(volume);
audio.setMuted(muted);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
