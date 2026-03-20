import { useNavigate } from "react-router-dom";
import { ColorButton } from "@/components/ColorButton";

interface CreditEntry {
  label: string;
  value: string;
  url?: string;
}

interface CreditSection {
  title: string;
  entries?: CreditEntry[];
  freeText?: string;
}

const CREDITS: CreditSection[] = [
  {
    title: "Game Design & Development",
    entries: [
      { label: "Created by", value: "Adrien Gras" },
      { label: "Game engine", value: "sabacc-core (Rust)" },
      { label: "TUI client", value: "sabacc-cli (Ratatui)" },
      { label: "Web client", value: "React + Vite + WebAssembly" },
      { label: "AI assistant", value: "Claude (Anthropic)" },
    ],
  },
  {
    title: "Music",
    entries: [
      {
        label: "Cantina Theme (Chill)",
        value: "Astronaut",
      },
    ],
  },
  {
    title: "Technology",
    entries: [
      { label: "Language", value: "Rust + TypeScript" },
      { label: "WASM bindings", value: "wasm-bindgen + wasm-pack" },
      { label: "UI framework", value: "React 19" },
      { label: "Styling", value: "Tailwind CSS v4 + 8bitcn" },
      { label: "State", value: "Zustand" },
      { label: "Audio", value: "Howler.js" },
      { label: "Fonts", value: "Press Start 2P, Aurebesh AF" },
    ],
  },
  {
    title: "Special Thanks",
    freeText: `The Star Wars universe
created by George Lucas — Lucasfilm / Disney

The Ratatui community
The shadcn and 8bitcn contributors
The Rust + WebAssembly working group`,
  },
  {
    title: "A Note from the Developer",
    freeText: `Star Wars is the greatest franchise ever created.

Yes, even with the sequels. Even with the
questionable choices. Even with Jar Jar.
(Especially with Jar Jar.)

Because Star Wars isn't about any single film.
It's about a galaxy that taught us to dream bigger
than our own backyard. It's about a kid on a desert
planet staring at twin suns and believing something
extraordinary was out there, waiting.

That feeling doesn't expire. It doesn't get
retconned. No boardroom decision can take it away.

So keep dreaming. Keep building. Keep imagining
impossible things — whether it's a card game in
a cantina, a ship that makes the Kessel Run in
twelve parsecs, or your own little corner of the
galaxy.

The Force will be with you. Always.`,
  },
];

export default function Credits() {
  const navigate = useNavigate();

  return (
    <div className="mx-auto flex min-h-screen max-w-xl flex-col items-center gap-8 px-4 py-12">
      <h1
        className="text-[14px] tracking-widest text-sand sm:text-[16px]"
        style={{ fontFamily: "var(--font-pixel)" }}
      >
        CREDITS
      </h1>

      <div className="h-px w-48 bg-sand/40" />

      {CREDITS.map((section) => (
        <div key={section.title} className="w-full">
          <h2
            className="mb-4 text-[10px] tracking-wider text-amber-400"
            style={{ fontFamily: "var(--font-pixel)" }}
          >
            {section.title}
          </h2>

          {section.entries?.map((entry) => (
            <div
              key={entry.label}
              className="mb-2 flex justify-between gap-4 text-[9px]"
            >
              <span className="text-gray-500">{entry.label}</span>
              {entry.url ? (
                <a
                  href={entry.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-cyan-400 hover:underline"
                >
                  {entry.value}
                </a>
              ) : (
                <span className="text-right text-cyan-400">{entry.value}</span>
              )}
            </div>
          ))}

          {section.freeText && (
            <p className="whitespace-pre-line text-[8px] leading-relaxed text-gray-400">
              {section.freeText}
              {section.title === "A Note from the Developer" && (
                <span className="mt-6 block text-right text-sand">
                  — Adrien
                </span>
              )}
            </p>
          )}
        </div>
      ))}

      <div className="h-px w-48 bg-sand/40" />

      <ColorButton
        className="w-48 text-[9px]"
        onClick={() => navigate("/menu")}
      >
        Back to Menu
      </ColorButton>
    </div>
  );
}
