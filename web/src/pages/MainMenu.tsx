import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/8bit/button";

const TITLE_ART = [
  " ███████  █████  ██████   █████   ██████  ██████",
  " ██      ██   ██ ██   ██ ██   ██ ██      ██     ",
  " ███████ ███████ ██████  ███████ ██      ██     ",
  "      ██ ██   ██ ██   ██ ██   ██ ██      ██     ",
  " ███████ ██   ██ ██████  ██   ██  ██████  ██████",
];

interface MenuItem {
  label: string;
  description: string;
  action: () => void;
}

export default function MainMenu() {
  const navigate = useNavigate();
  const [hovered, setHovered] = useState(0);

  const menuItems: MenuItem[] = [
    {
      label: "Hyperspace Sabacc",
      description:
        "The full Outlaw experience — Shift Tokens, bluffs, and chaos.\n3 opponents, 100 credits, 3 tokens each.",
      action: () => navigate("/play", { state: { preset: "hyperspace" } }),
    },
    {
      label: "Classic Sabacc",
      description:
        "Pure cards, no tricks. The way Han won the Falcon from Lando.\n3 opponents, 100 credits. Just you and the deck.",
      action: () => navigate("/play", { state: { preset: "classic" } }),
    },
    {
      label: "Lando's Challenge",
      description:
        "Face the galaxy's smoothest smuggler in a 1v1 duel.\n100 credits, tokens ON. Expert difficulty.",
      action: () => navigate("/play", { state: { preset: "lando" } }),
    },
    {
      label: "Custom Game",
      description:
        "Set up your own table in the back of the cantina.\nPick your opponents, stakes, and house rules.",
      action: () => navigate("/setup"),
    },
    {
      label: "How to Play",
      description:
        "Every smuggler starts somewhere.\nLearn the cards, the bets, and how not to lose your ship.",
      action: () => navigate("/how-to-play"),
    },
  ];

  return (
    <div className="flex min-h-screen flex-col items-center justify-center gap-4 px-4">
      {/* ASCII Title */}
      <div className="relative mb-2">
        {/* Shadow layer */}
        <pre
          className="absolute top-[2px] left-[2px] select-none text-[8px] leading-tight sm:text-[10px] md:text-[12px]"
          style={{ color: "rgb(80, 65, 25)" }}
          aria-hidden="true"
        >
          {TITLE_ART.join("\n")}
        </pre>
        {/* Main layer */}
        <pre className="relative text-[8px] font-bold leading-tight text-sand sm:text-[10px] md:text-[12px]">
          {TITLE_ART.join("\n")}
        </pre>
      </div>

      {/* Subtitle */}
      <p className="mb-1 text-[10px] tracking-[0.5em] text-gray-500">
        K E S S E L
      </p>

      {/* Separator */}
      <div className="mb-2 h-px w-60 bg-sand/40" />

      {/* Description */}
      <div className="mb-2 h-10 text-center text-[9px] leading-relaxed whitespace-pre-line text-gray-400">
        {menuItems[hovered].description}
      </div>

      {/* Menu items */}
      <div className="flex flex-col gap-2">
        {menuItems.map((item, i) => (
          <Button
            key={item.label}
            variant={hovered === i ? "default" : "outline"}
            className="w-64 text-[9px]"
            onMouseEnter={() => setHovered(i)}
            onClick={item.action}
          >
            {item.label}
          </Button>
        ))}
      </div>
    </div>
  );
}
