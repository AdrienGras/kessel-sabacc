import { useState, useEffect, useCallback } from "react";
import { cn } from "@/lib/utils";

const CRAWL_TEXT = `The galaxy's most notorious card game
has been played in smuggler dens and
shadowy cantinas for centuries.

Fortunes won. Ships lost. Alliances
forged over two cards and a handful
of chips — then broken before the
next round.

From the spice mines of Kessel to
the back rooms of Mos Eisley, the
rules have always been simple: hold
your nerve, read the table, and
pray the odds fall your way.

But in the age of Shift Tokens,
nothing is certain. A single play
can turn a losing hand into a
devastating victory — or seal your
fate for good.

Now, the deck is shuffled.
The credits are on the line.
The seat across from you is taken
by someone who doesn't plan to lose.`;

const PHASE_TIMINGS = {
  intro: 5000,
  title: 4000,
  crawl: 18000,
};

type Phase = "intro" | "title" | "crawl" | "done";

interface StarWarsCrawlProps {
  onComplete: () => void;
}

export default function StarWarsCrawl({ onComplete }: StarWarsCrawlProps) {
  const [phase, setPhase] = useState<Phase>("intro");

  const reducedMotion =
    typeof window !== "undefined" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  useEffect(() => {
    if (reducedMotion) {
      setPhase("done");
      onComplete();
      return;
    }

    const timers: ReturnType<typeof setTimeout>[] = [];

    timers.push(
      setTimeout(() => setPhase("title"), PHASE_TIMINGS.intro),
    );
    timers.push(
      setTimeout(
        () => setPhase("crawl"),
        PHASE_TIMINGS.intro + PHASE_TIMINGS.title,
      ),
    );
    timers.push(
      setTimeout(() => {
        setPhase("done");
        onComplete();
      }, PHASE_TIMINGS.intro + PHASE_TIMINGS.title + PHASE_TIMINGS.crawl),
    );

    return () => timers.forEach(clearTimeout);
  }, [onComplete, reducedMotion]);

  const handleSkip = useCallback(() => {
    setPhase("done");
    onComplete();
  }, [onComplete]);

  if (reducedMotion) return null;

  return (
    <div
      className="pointer-events-none absolute inset-0 z-10 overflow-hidden"
      aria-live="polite"
    >
      {/* Phase 1: Intro text */}
      {phase === "intro" && (
        <div className="flex h-full items-center justify-center">
          <p
            className="text-center text-[10px] leading-relaxed sm:text-[12px]"
            style={{
              color: "#4fd1c5",
              fontFamily: "var(--font-pixel)",
              animation: "intro-fade 5s ease-in-out forwards",
            }}
          >
            A long time ago, in a cantina
            <br />
            far, far away...
          </p>
        </div>
      )}

      {/* Phase 2: Title */}
      {phase === "title" && (
        <div className="flex h-full flex-col items-center justify-center">
          <h1
            className="text-[24px] font-bold tracking-widest text-sand sm:text-[36px] md:text-[48px]"
            style={{
              fontFamily: "var(--font-pixel)",
              animation: "title-appear 4s ease-in-out forwards",
            }}
          >
            SABACC
          </h1>
          <p
            className="mt-2 text-[10px] tracking-[0.5em] text-gray-400 sm:text-[12px]"
            style={{
              fontFamily: "var(--font-pixel)",
              animation: "title-appear 4s ease-in-out forwards",
            }}
          >
            KESSEL
          </p>
        </div>
      )}

      {/* Phase 3: Crawl */}
      {phase === "crawl" && (
        <div className="crawl-perspective" onClick={handleSkip}>
          <div className="crawl-text">
            {CRAWL_TEXT.split("\n\n").map((paragraph, i) => (
              <p key={i} className="mb-8">
                {paragraph}
              </p>
            ))}
            <p
              className="mt-12 text-center"
              style={{ fontSize: "clamp(0.7rem, 1.8vw, 1.1rem)" }}
            >
              Are you in?
            </p>
          </div>
        </div>
      )}

      {/* Phase done: nothing rendered, button handled by parent */}
    </div>
  );
}

export function EnterButton({
  crawlDone,
  onClick,
  className,
}: {
  crawlDone: boolean;
  onClick: () => void;
  className?: string;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "z-20 cursor-pointer border-2 border-sand/60 px-6 py-3 text-sand transition-all duration-300",
        "hover:border-sand hover:bg-sand/10 hover:shadow-[0_0_20px_rgba(232,192,80,0.3)]",
        "focus:outline-none focus:ring-2 focus:ring-sand/50",
        crawlDone
          ? "text-[12px] sm:text-[14px]"
          : "text-[8px] opacity-40 hover:opacity-80",
        crawlDone && "animate-enter-pulse",
        className,
      )}
      style={{ fontFamily: "var(--font-pixel)" }}
    >
      ENTER THE CANTINA
    </button>
  );
}
