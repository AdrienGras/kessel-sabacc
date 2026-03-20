import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ColorButton } from "@/components/ColorButton";
import { Badge } from "@/components/ui/8bit/badge";
import { initWasm } from "@/lib/wasm";
import { useGameStore } from "@/lib/game-store";

export default function Game() {
  const navigate = useNavigate();
  const { wasmReady, wasmVersion } = useGameStore();
  const [pingResult, setPingResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    initWasm()
      .then((wasm) => {
        const version = wasm.version();
        useGameStore.getState().setWasmReady(version);
        setPingResult(wasm.ping());
      })
      .catch((err: unknown) => {
        setError(err instanceof Error ? err.message : "Failed to load WASM");
      });
  }, []);

  if (error) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center gap-4">
        <p className="text-blood">WASM Error: {error}</p>
        <ColorButton onClick={() => navigate("/")}>
          Back to Menu
        </ColorButton>
      </div>
    );
  }

  if (!wasmReady) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center gap-4">
        <p className="animate-pulse text-sand">Loading WASM engine...</p>
      </div>
    );
  }

  return (
    <div className="flex min-h-screen flex-col items-center justify-center gap-6">
      <h1 className="text-sand">Game Board</h1>

      <div className="flex gap-2">
        <Badge variant="outline">{wasmVersion}</Badge>
        <Badge variant="outline">ping: {pingResult}</Badge>
      </div>

      <p className="text-[10px] text-gray-500">
        Game board coming soon...
      </p>

      <ColorButton onClick={() => navigate("/")}>
        Back to Menu
      </ColorButton>
    </div>
  );
}
