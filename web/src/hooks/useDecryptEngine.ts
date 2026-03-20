import { useEffect, useRef, useState, useCallback } from "react";

export type CharState =
  | "aurebesh"
  | "scramble"
  | "flash"
  | "decoded"
  | "glow-fade";

export type Phase = "loading" | "aurebesh" | "decrypting" | "complete";

interface Section {
  title: string;
  content: string;
}

interface DecryptEngineOptions {
  sections: Section[];
  alreadyDecrypted: boolean;
  onComplete: () => void;
}

interface SectionState {
  title: CharState[];
  content: CharState[];
}

export interface DecryptEngineResult {
  phase: Phase;
  progress: number;
  sectionStates: SectionState[];
  scrambleChars: Map<string, string>;
  skip: () => void;
}

interface CharTransition {
  key: string;
  sectionIdx: number;
  field: "title" | "content";
  charIdx: number;
  scrambleStart: number;
  flashStart: number;
  decodedStart: number;
  glowEnd: number;
}

const AUREBESH_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const AUREBESH_HOLD_MS = 800;
const CHAR_DELAY_START = 4;
const CHAR_DELAY_END = 1;
const SCRAMBLE_DURATION = 50;
const FLASH_DURATION = 50;
const GLOW_FADE_DURATION = 300;
const FONT_LOAD_TIMEOUT = 3000;

function randomAurebeshChar(): string {
  return AUREBESH_CHARS[Math.floor(Math.random() * AUREBESH_CHARS.length)];
}

function isAnimatable(char: string): boolean {
  return char.trim().length > 0 && /[a-zA-Z0-9]/.test(char);
}

function buildSequentialTransitions(sections: Section[]): CharTransition[] {
  const transitions: CharTransition[] = [];

  // Count total animatable chars for delay interpolation
  let totalAnimatable = 0;
  for (const section of sections) {
    for (const ch of section.title) if (isAnimatable(ch)) totalAnimatable++;
    for (const ch of section.content) if (isAnimatable(ch)) totalAnimatable++;
  }

  let animatedCount = 0;
  let currentTime = 0;

  const addCharsFromText = (
    text: string,
    sectionIdx: number,
    field: "title" | "content",
  ) => {
    for (let charIdx = 0; charIdx < text.length; charIdx++) {
      if (!isAnimatable(text[charIdx])) continue;

      const progress = totalAnimatable > 1 ? animatedCount / (totalAnimatable - 1) : 0;
      const delay = CHAR_DELAY_START + progress * (CHAR_DELAY_END - CHAR_DELAY_START);

      const key = `${sectionIdx}-${field}-${charIdx}`;
      transitions.push({
        key,
        sectionIdx,
        field,
        charIdx,
        scrambleStart: currentTime,
        flashStart: currentTime + SCRAMBLE_DURATION,
        decodedStart: currentTime + SCRAMBLE_DURATION + FLASH_DURATION,
        glowEnd: currentTime + SCRAMBLE_DURATION + FLASH_DURATION + GLOW_FADE_DURATION,
      });

      currentTime += delay;
      animatedCount++;
    }
  };

  for (let sIdx = 0; sIdx < sections.length; sIdx++) {
    addCharsFromText(sections[sIdx].title, sIdx, "title");
    addCharsFromText(sections[sIdx].content, sIdx, "content");
  }

  return transitions;
}

function initSectionStates(sections: Section[], state: CharState): SectionState[] {
  return sections.map((s) => ({
    title: Array.from({ length: s.title.length }, () => state),
    content: Array.from({ length: s.content.length }, () => state),
  }));
}

export function useDecryptEngine({
  sections,
  alreadyDecrypted,
  onComplete,
}: DecryptEngineOptions): DecryptEngineResult {
  const [phase, setPhase] = useState<Phase>(
    alreadyDecrypted ? "complete" : "loading",
  );
  const [progress, setProgress] = useState(alreadyDecrypted ? 100 : 0);
  const [sectionStates, setSectionStates] = useState<SectionState[]>(() =>
    initSectionStates(sections, alreadyDecrypted ? "decoded" : "aurebesh"),
  );
  const [scrambleChars, setScrambleChars] = useState<Map<string, string>>(
    () => new Map(),
  );

  const rafRef = useRef<number | null>(null);
  const startTimeRef = useRef<number>(0);
  const transitionsRef = useRef<CharTransition[]>([]);
  const mutableStatesRef = useRef<SectionState[]>([]);
  const mutableScrambleRef = useRef<Map<string, string>>(new Map());
  const skippedRef = useRef(false);
  const completedRef = useRef(alreadyDecrypted);
  const onCompleteRef = useRef(onComplete);
  onCompleteRef.current = onComplete;

  const skip = useCallback(() => {
    if (completedRef.current) return;
    skippedRef.current = true;
    if (rafRef.current !== null) {
      cancelAnimationFrame(rafRef.current);
      rafRef.current = null;
    }
    const decoded = initSectionStates(sections, "decoded");
    setSectionStates(decoded);
    setScrambleChars(new Map());
    setPhase("complete");
    setProgress(100);
    completedRef.current = true;
    onCompleteRef.current();
  }, [sections]);

  useEffect(() => {
    if (alreadyDecrypted) return;

    // Respect prefers-reduced-motion
    const motionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    if (motionQuery.matches) {
      skip();
      return;
    }

    // Phase 0: Load fonts
    const loadFonts = async () => {
      try {
        await Promise.race([
          Promise.all([
            document.fonts.load('10px "Aurebesh AF"'),
            document.fonts.load('10px "Press Start 2P"'),
          ]),
          new Promise((resolve) => setTimeout(resolve, FONT_LOAD_TIMEOUT)),
        ]);
      } catch {
        // Graceful degradation — proceed anyway
      }

      if (skippedRef.current) return;

      // Phase 1: Aurebesh hold
      setPhase("aurebesh");

      const transitions = buildSequentialTransitions(sections);
      transitionsRef.current = transitions;

      const totalDuration =
        transitions.length > 0
          ? Math.max(...transitions.map((t) => t.glowEnd))
          : 0;

      // Initialize mutable state
      mutableStatesRef.current = initSectionStates(sections, "aurebesh");
      mutableScrambleRef.current = new Map();

      // Wait for aurebesh hold, then start decrypting
      setTimeout(() => {
        if (skippedRef.current) return;
        setPhase("decrypting");
        startTimeRef.current = performance.now();

        let nextTransitionIdx = 0;

        const tick = (now: number) => {
          if (skippedRef.current) return;

          const elapsed = now - startTimeRef.current;
          const states = mutableStatesRef.current;
          const scramble = mutableScrambleRef.current;
          let changed = false;

          // Process transitions up to current time
          while (nextTransitionIdx < transitions.length) {
            const t = transitions[nextTransitionIdx];
            if (t.scrambleStart > elapsed) break;
            nextTransitionIdx++;

            // Determine current state for this char
            const arr = states[t.sectionIdx][t.field];
            if (elapsed < t.flashStart) {
              arr[t.charIdx] = "scramble";
              scramble.set(t.key, randomAurebeshChar());
            } else if (elapsed < t.decodedStart) {
              arr[t.charIdx] = "flash";
              scramble.delete(t.key);
            } else {
              arr[t.charIdx] = "decoded";
              scramble.delete(t.key);
            }
            changed = true;
          }

          // Update already-triggered chars that advanced state
          for (let i = 0; i < nextTransitionIdx; i++) {
            const t = transitions[i];
            const arr = states[t.sectionIdx][t.field];
            const current = arr[t.charIdx];
            if (current === "scramble" && elapsed >= t.flashStart) {
              arr[t.charIdx] = "flash";
              scramble.delete(t.key);
              changed = true;
            }
            if (current === "flash" && elapsed >= t.decodedStart) {
              arr[t.charIdx] = "decoded";
              changed = true;
            }
            if (current === "decoded" && elapsed < t.glowEnd) {
              arr[t.charIdx] = "glow-fade";
              changed = true;
            }
            if (current === "glow-fade" && elapsed >= t.glowEnd) {
              arr[t.charIdx] = "decoded";
              changed = true;
            }
          }

          if (changed) {
            setSectionStates(states.map((s) => ({ ...s, title: [...s.title], content: [...s.content] })));
            setScrambleChars(new Map(scramble));
          }

          const prog = Math.min(100, Math.round((elapsed / totalDuration) * 100));
          setProgress(prog);

          if (elapsed >= totalDuration) {
            // Flush: set everything to decoded
            const decoded = initSectionStates(sections, "decoded");
            setSectionStates(decoded);
            setScrambleChars(new Map());
            setPhase("complete");
            setProgress(100);
            completedRef.current = true;
            onCompleteRef.current();
            rafRef.current = null;
            return;
          }

          rafRef.current = requestAnimationFrame(tick);
        };

        rafRef.current = requestAnimationFrame(tick);
      }, AUREBESH_HOLD_MS);
    };

    loadFonts();

    return () => {
      if (rafRef.current !== null) {
        cancelAnimationFrame(rafRef.current);
        rafRef.current = null;
      }
    };
  }, [alreadyDecrypted, sections, skip]);

  return { phase, progress, sectionStates, scrambleChars, skip };
}
