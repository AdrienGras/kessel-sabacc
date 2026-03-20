import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/8bit/button";
import { Input } from "@/components/ui/8bit/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/8bit/select";
import { Switch } from "@/components/ui/8bit/switch";
import { Slider } from "@/components/ui/8bit/slider";
import { useGameStore } from "@/lib/game-store";

export default function Setup() {
  const navigate = useNavigate();
  const [playerName, setPlayerName] = useState("Player");
  const [botCount, setBotCount] = useState(3);
  const [buyIn, setBuyIn] = useState("100");
  const [tokensEnabled, setTokensEnabled] = useState(true);
  const [tokensPerPlayer, setTokensPerPlayer] = useState(3);
  const [difficulty, setDifficulty] = useState("basic");

  const handleStart = () => {
    useGameStore.getState().setConfig({
      playerName,
      botCount,
      buyIn: Number(buyIn),
      tokensEnabled,
      tokensPerPlayer: tokensEnabled ? tokensPerPlayer : 0,
      difficulty,
    });
    navigate("/play", { state: { preset: "custom" } });
  };

  return (
    <div className="mx-auto flex min-h-screen max-w-md flex-col items-center justify-center gap-6 px-4">
      <h1 className="text-sand">Custom Game</h1>

      {/* Player Name */}
      <div className="flex w-full flex-col gap-1">
        <label className="text-[9px] text-gray-400">Player Name</label>
        <Input
          value={playerName}
          onChange={(e) => setPlayerName(e.target.value)}
          maxLength={20}
        />
      </div>

      {/* Bot Count */}
      <div className="flex w-full flex-col gap-1">
        <label className="text-[9px] text-gray-400">
          Opponents: {botCount}
        </label>
        <Slider
          min={1}
          max={7}
          step={1}
          value={[botCount]}
          onValueChange={([v]) => setBotCount(v)}
        />
      </div>

      {/* Buy-in */}
      <div className="flex w-full flex-col gap-1">
        <label className="text-[9px] text-gray-400">Buy-in (credits)</label>
        <Select value={buyIn} onValueChange={(v) => setBuyIn(String(v))}>
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="50">50</SelectItem>
            <SelectItem value="100">100</SelectItem>
            <SelectItem value="150">150</SelectItem>
            <SelectItem value="200">200</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Shift Tokens toggle */}
      <div className="flex w-full items-center justify-between">
        <label className="text-[9px] text-gray-400">Shift Tokens</label>
        <Switch checked={tokensEnabled} onCheckedChange={setTokensEnabled} />
      </div>

      {/* Tokens per player (conditional) */}
      {tokensEnabled && (
        <div className="flex w-full flex-col gap-1">
          <label className="text-[9px] text-gray-400">
            Tokens per player: {tokensPerPlayer}
          </label>
          <Slider
            min={1}
            max={8}
            step={1}
            value={[tokensPerPlayer]}
            onValueChange={([v]) => setTokensPerPlayer(v)}
          />
        </div>
      )}

      {/* Difficulty */}
      <div className="flex w-full flex-col gap-1">
        <label className="text-[9px] text-gray-400">Bot Difficulty</label>
        <Select value={difficulty} onValueChange={(v) => setDifficulty(String(v))}>
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="basic">Basic</SelectItem>
            <SelectItem value="expert">Expert</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Start */}
      <Button className="mt-4 w-full" onClick={handleStart}>
        START GAME
      </Button>

      <Button
        variant="outline"
        className="w-48 text-[9px]"
        onClick={() => navigate("/")}
      >
        Back to Menu
      </Button>
    </div>
  );
}
