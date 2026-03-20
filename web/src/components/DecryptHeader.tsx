import { useState, useEffect } from "react";
import type { Phase } from "@/hooks/useDecryptEngine";

interface DecryptHeaderProps {
  phase: Phase;
  progress: number;
}

const BAR_WIDTH = 20;

function buildProgressBar(progress: number): string {
  const filled = Math.round((progress / 100) * BAR_WIDTH);
  const empty = BAR_WIDTH - filled;
  return "█".repeat(filled) + "░".repeat(empty);
}

export function DecryptHeader({ phase, progress }: DecryptHeaderProps) {
  const [cursorVisible, setCursorVisible] = useState(true);

  // Blinking cursor for loading phase
  useEffect(() => {
    if (phase !== "loading") return;
    const interval = setInterval(() => setCursorVisible((v) => !v), 500);
    return () => clearInterval(interval);
  }, [phase]);

  return (
    <div
      className="font-decoded mb-4 text-[9px] leading-relaxed text-sand"
      role="status"
      aria-live="polite"
    >
      {phase === "loading" && (
        <p>
          {">"} CONNECTING...{cursorVisible ? "_" : " "}
        </p>
      )}

      {(phase === "aurebesh" || phase === "decrypting") && (
        <>
          <p>{">"} INTERCEPTED TRANSMISSION</p>
          <p>{">"} SOURCE: Kessel sector — cantina terminal</p>
          <p>
            {">"} STATUS: DECRYPTING... {buildProgressBar(progress)}{" "}
            {progress}%
          </p>
        </>
      )}

      {phase === "complete" && (
        <>
          <p>{">"} INTERCEPTED TRANSMISSION</p>
          <p>{">"} SOURCE: Kessel sector — cantina terminal</p>
          <p>{">"} STATUS: DECRYPTED ✓</p>
        </>
      )}
    </div>
  );
}
