import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ColorButton } from "@/components/ColorButton";
import { Card, CardContent } from "@/components/ui/8bit/card";
import { DecryptHeader } from "@/components/DecryptHeader";
import { DecryptSection } from "@/components/DecryptSection";
import { useDecryptEngine } from "@/hooks/useDecryptEngine";
import { useGameStore } from "@/lib/game-store";

const SECTIONS = [
  {
    title: "Welcome to the Table",
    content: `In the shadow of Kessel's spice mines, smugglers and scoundrels gather to play the galaxy's most notorious card game. Fortunes are won and lost in minutes. Lando Calrissian once bet — and lost — the Millennium Falcon at a table just like this one.

Your goal is simple: outlast every other player at the table. When your chips are gone, so are you. The last one standing takes the pot.`,
  },
  {
    title: "The Deck",
    content: `The Sabacc deck is split into two families — Sand and Blood — each marked with its own colour and symbol.

Each family contains:
  · Number cards valued 1 through 6 (3 copies each)
  · 2 Sylop cards — wildcards that copy the value of your other card (powerful, but rare)
  · 2 Impostor cards — their value is unknown until the reveal, when you roll dice to determine it

That makes 44 cards in total: 22 Sand, 22 Blood.`,
  },
  {
    title: "Your Hand",
    content: `You always hold exactly two cards: one Sand, one Blood. Think of them as two halves of a wager — you want them to match as closely as possible.

  Example: Sand 3 + Blood 3 = Sabacc! (difference 0)
  Example: Sand 5 + Blood 2 = difference of 3 (bad)`,
  },
  {
    title: "Hand Rankings",
    content: `From best to worst:

  1. PURE SABACC — Both cards are Sylops. The rarest hand.
  2. PRIME SABACC — Via the PrimeSabacc shift token. Roll dice + match.
  3. SYLOP SABACC — One Sylop + one number card. The Sylop copies the number.
  4. SABACC — Two number cards with the same value. Ties broken by lowest value.
  5. NON-SABACC — Values differ. Smaller difference is better.`,
  },
  {
    title: "How a Round Plays Out",
    content: `Each round consists of 3 turns. On your turn:

  DRAW — Pick a card from one of four sources (Sand Deck, Sand Discard, Blood Deck, Blood Discard). Discard one of the same family. Costs 1 chip.

  STAND — Do nothing. It's free. But some Shift Tokens punish those who Stand.

Before choosing, you may optionally play one Shift Token.`,
  },
  {
    title: "Impostors",
    content: `Impostors are wild cards with a twist. At the reveal, any player holding an Impostor rolls two dice and picks one of the two values.

  Example: Sand Impostor + Blood 2. You roll 3 and 5. Pick 3 → hand becomes Sand 3 + Blood 2.`,
  },
  {
    title: "Scoring & Penalties",
    content: `After 3 turns, all players reveal. Best hand wins.

  · WINNER recovers all invested chips.
  · Losers with SABACC lose 1 chip penalty.
  · Losers with NON-SABACC lose chips equal to their difference.
  · Tied best hands: all recover their chips.

Penalty chips are destroyed, not given to the winner.`,
  },
  {
    title: "Shift Tokens",
    content: `Each player gets random tokens at game start. Use once per game, before Draw/Stand.

Helpful: FreeDraw, Refund, ExtraRefund, Immunity
Harmful: GeneralTariff, TargetTariff, Embargo, Embezzlement, GeneralAudit, TargetAudit, Exhaustion
Rule-changers: Markdown, MajorFraud, CookTheBooks, DirectTransaction, PrimeSabacc`,
  },
];

function HowToPlayContent({ remountKey }: { remountKey: number }) {
  const navigate = useNavigate();
  const howToPlayDecrypted = useGameStore((s) => s.howToPlayDecrypted);
  const setHowToPlayDecrypted = useGameStore((s) => s.setHowToPlayDecrypted);
  const resetHowToPlayDecrypted = useGameStore(
    (s) => s.resetHowToPlayDecrypted,
  );

  const handleComplete = useCallback(() => {
    setHowToPlayDecrypted();
  }, [setHowToPlayDecrypted]);

  const { phase, progress, sectionStates, scrambleChars, skip } =
    useDecryptEngine({
      sections: SECTIONS,
      alreadyDecrypted: howToPlayDecrypted,
      onComplete: handleComplete,
    });

  const isAnimating = phase !== "complete";

  // Scroll lock during animation
  useEffect(() => {
    if (isAnimating) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "";
    }
    return () => {
      document.body.style.overflow = "";
    };
  }, [isAnimating]);

  // Global skip: Space/Esc/click
  useEffect(() => {
    if (!isAnimating) return;

    const handleKey = (e: KeyboardEvent) => {
      if (e.key === " " || e.key === "Escape") {
        e.preventDefault();
        skip();
      }
    };
    const handleClick = () => skip();

    window.addEventListener("keydown", handleKey);
    window.addEventListener("click", handleClick);
    return () => {
      window.removeEventListener("keydown", handleKey);
      window.removeEventListener("click", handleClick);
    };
  }, [isAnimating, skip]);

  const handleReplay = (e: React.MouseEvent) => {
    e.stopPropagation();
    window.scrollTo({ top: 0 });
    resetHowToPlayDecrypted();
  };

  // Use remountKey to suppress lint warning — it forces a fresh hook instance
  void remountKey;

  return (
    <div className="relative mx-auto flex max-w-2xl flex-col gap-4 px-4 py-8" aria-live="polite">
      <DecryptHeader phase={phase} progress={progress} />

      {/* Scanline CRT overlay */}
      {isAnimating && <div className="scanline-overlay" aria-hidden="true" />}

      <Card className="border-sand dark:border-sand">
        <CardContent className="flex flex-col gap-6 pt-6">
          {SECTIONS.map((section, idx) => (
            <DecryptSection
              key={section.title}
              title={section.title}
              content={section.content}
              titleStates={sectionStates[idx]?.title ?? []}
              contentStates={sectionStates[idx]?.content ?? []}
              scrambleChars={scrambleChars}
              sectionIdx={idx}
            />
          ))}
        </CardContent>
      </Card>

      <div className="mx-auto mt-4 flex gap-4">
        <ColorButton className="w-48" onClick={() => navigate("/menu")}>
          Back to Menu
        </ColorButton>

        {!isAnimating && (
          <ColorButton
            className="w-48 opacity-50 hover:opacity-100"
            onClick={handleReplay}
          >
            Replay Decrypt
          </ColorButton>
        )}
      </div>
    </div>
  );
}

export default function HowToPlay() {
  const [remountKey, setRemountKey] = useState(0);
  const howToPlayDecrypted = useGameStore((s) => s.howToPlayDecrypted);

  // When howToPlayDecrypted resets to false, force remount
  useEffect(() => {
    if (!howToPlayDecrypted) {
      setRemountKey((k) => k + 1);
    }
  }, [howToPlayDecrypted]);

  return <HowToPlayContent key={remountKey} remountKey={remountKey} />;
}
