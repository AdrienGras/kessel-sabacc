import { useEffect } from "react";
import { Outlet } from "react-router-dom";
import Starfield from "./Starfield";
import { initWasm } from "@/lib/wasm";

export default function Layout() {
  // Silently preload WASM on mount — error is ignored (retry on /play)
  useEffect(() => {
    initWasm().catch(() => {});
  }, []);

  return (
    <div className="relative min-h-screen overflow-hidden bg-[#0a0a0a] text-white">
      <Starfield />
      <div className="relative z-10">
        <Outlet />
      </div>
    </div>
  );
}
