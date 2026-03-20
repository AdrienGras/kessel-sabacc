import { useEffect } from "react";
import { Outlet, useLocation } from "react-router-dom";
import Starfield from "./Starfield";
import MuteButton from "./MuteButton";
import { initWasm } from "@/lib/wasm";
import { audio } from "@/lib/audio";
import { useAudioStore } from "@/lib/audio-store";

export default function Layout() {
  const location = useLocation();
  const { volume, muted } = useAudioStore();

  // Silently preload WASM on mount — error is ignored (retry on /play)
  useEffect(() => {
    initWasm().catch(() => {});
  }, []);

  // Sync audio store → AudioManager on mount and changes
  useEffect(() => {
    audio.setVolume(volume);
    audio.setMuted(muted);
  }, [volume, muted]);

  // Play music based on current route
  useEffect(() => {
    if (location.pathname === "/play") {
      // Future: audio.playMusic('game-theme')
      audio.playMusic("menu-theme");
    } else {
      audio.playMusic("menu-theme");
    }
  }, [location.pathname]);

  return (
    <div className="relative min-h-screen overflow-hidden bg-[#0a0a0a] text-white">
      <Starfield />

      {/* Header bar with mute button */}
      <div className="fixed top-0 right-0 z-30 p-3">
        <MuteButton />
      </div>

      <div className="relative z-10">
        <Outlet />
      </div>
    </div>
  );
}
