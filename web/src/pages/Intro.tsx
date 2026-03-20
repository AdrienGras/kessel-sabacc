import { useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import Starfield from "@/components/Starfield";
import StarWarsCrawl, { EnterButton } from "@/components/StarWarsCrawl";
import { audio } from "@/lib/audio";
import { initWasm } from "@/lib/wasm";

export default function Intro() {
  const navigate = useNavigate();
  const [crawlDone, setCrawlDone] = useState(false);
  const [exiting, setExiting] = useState(false);

  // Preload WASM silently during the crawl
  useState(() => {
    initWasm().catch(() => {});
  });

  const handleCrawlComplete = useCallback(() => {
    setCrawlDone(true);
  }, []);

  const handleEnter = useCallback(() => {
    if (exiting) return;
    setExiting(true);

    // Unlock AudioContext + start music from user gesture (critical for autoplay)
    audio.unlock();
    audio.playMusic("menu-theme");

    // Fade out then navigate
    setTimeout(() => {
      navigate("/menu");
    }, 800);
  }, [exiting, navigate]);

  return (
    <div
      className="relative flex min-h-screen items-end justify-center overflow-hidden bg-black pb-16 transition-opacity duration-800"
      style={{ opacity: exiting ? 0 : 1 }}
    >
      <Starfield />

      <StarWarsCrawl onComplete={handleCrawlComplete} />

      {/* Button always visible — 2 visual states */}
      <div
        className="z-20 flex flex-col items-center transition-all duration-1000"
        style={{
          position: crawlDone ? "fixed" : "fixed",
          bottom: crawlDone ? "50%" : "2rem",
          transform: crawlDone ? "translateY(50%)" : "none",
        }}
      >
        <EnterButton crawlDone={crawlDone} onClick={handleEnter} />
      </div>
    </div>
  );
}
