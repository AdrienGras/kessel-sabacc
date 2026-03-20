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

interface Cluster {
  sectionIdx: number;
  field: "title" | "content";
  charIndices: number[];
  triggerTime: number;
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
const TITLE_DECODE_START = 200;
const TITLE_DECODE_END = 800;
const BODY_SLOW_START = 800;
const BODY_SLOW_END = 2300;
const BODY_MEDIUM_START = 2300;
const BODY_MEDIUM_END = 3800;
const BODY_FAST_START = 3800;
const BODY_FAST_END = 5300;
const SCRAMBLE_DURATION = 50;
const FLASH_DURATION = 50;
const GLOW_FADE_DURATION = 300;
const FONT_LOAD_TIMEOUT = 3000;

function randomAurebeshChar(): string {
  return AUREBESH_CHARS[Math.floor(Math.random() * AUREBESH_CHARS.length)];
}

function shuffleArray<T>(arr: T[]): T[] {
  const shuffled = [...arr];
  for (let i = shuffled.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
  }
  return shuffled;
}

function isAnimatable(char: string): boolean {
  return char.trim().length > 0 && /[a-zA-Z0-9]/.test(char);
}

function buildClusters(sections: Section[]): Cluster[] {
  const clusters: Cluster[] = [];

  for (let sIdx = 0; sIdx < sections.length; sIdx++) {
    const section = sections[sIdx];

    // Title clusters: 3-4 chars, between TITLE_DECODE_START and TITLE_DECODE_END
    const titleIndices = shuffleArray(
      [...section.title]
        .map((ch, i) => ({ ch, i }))
        .filter(({ ch }) => isAnimatable(ch))
        .map(({ i }) => i),
    );
    const titleClusterSize = Math.min(4, Math.max(3, Math.ceil(titleIndices.length / 3)));
    for (let i = 0; i < titleIndices.length; i += titleClusterSize) {
      const chunk = titleIndices.slice(i, i + titleClusterSize);
      const t =
        TITLE_DECODE_START +
        (i / titleIndices.length) * (TITLE_DECODE_END - TITLE_DECODE_START);
      clusters.push({
        sectionIdx: sIdx,
        field: "title",
        charIndices: chunk,
        triggerTime: t,
      });
    }

    // Content clusters: distributed across slow/medium/fast phases
    const contentIndices = shuffleArray(
      [...section.content]
        .map((ch, i) => ({ ch, i }))
        .filter(({ ch }) => isAnimatable(ch))
        .map(({ i }) => i),
    );

    const total = contentIndices.length;
    const slowCount = Math.floor(total * 0.2);
    const mediumCount = Math.floor(total * 0.3);

    // Slow phase: 2-4 chars per cluster
    let offset = 0;
    const slowIndices = contentIndices.slice(offset, offset + slowCount);
    offset += slowCount;
    for (let i = 0; i < slowIndices.length; i += 3) {
      const chunk = slowIndices.slice(i, i + Math.min(4, 2 + Math.floor(Math.random() * 3)));
      const t =
        BODY_SLOW_START +
        (i / Math.max(1, slowIndices.length)) * (BODY_SLOW_END - BODY_SLOW_START);
      clusters.push({
        sectionIdx: sIdx,
        field: "content",
        charIndices: chunk,
        triggerTime: t,
      });
    }

    // Medium phase: 4-8 chars per cluster
    const mediumIndices = contentIndices.slice(offset, offset + mediumCount);
    offset += mediumCount;
    for (let i = 0; i < mediumIndices.length; i += 6) {
      const chunk = mediumIndices.slice(i, i + Math.min(8, 4 + Math.floor(Math.random() * 5)));
      const t =
        BODY_MEDIUM_START +
        (i / Math.max(1, mediumIndices.length)) * (BODY_MEDIUM_END - BODY_MEDIUM_START);
      clusters.push({
        sectionIdx: sIdx,
        field: "content",
        charIndices: chunk,
        triggerTime: t,
      });
    }

    // Fast phase: 10-20 chars per cluster
    const fastIndices = contentIndices.slice(offset);
    for (let i = 0; i < fastIndices.length; i += 15) {
      const chunk = fastIndices.slice(i, i + Math.min(20, 10 + Math.floor(Math.random() * 11)));
      const t =
        BODY_FAST_START +
        (i / Math.max(1, fastIndices.length)) * (BODY_FAST_END - BODY_FAST_START);
      clusters.push({
        sectionIdx: sIdx,
        field: "content",
        charIndices: chunk,
        triggerTime: t,
      });
    }
  }

  return clusters.sort((a, b) => a.triggerTime - b.triggerTime);
}

function buildTransitions(clusters: Cluster[]): CharTransition[] {
  const transitions: CharTransition[] = [];

  for (const cluster of clusters) {
    for (const charIdx of cluster.charIndices) {
      const key = `${cluster.sectionIdx}-${cluster.field}-${charIdx}`;
      transitions.push({
        key,
        sectionIdx: cluster.sectionIdx,
        field: cluster.field,
        charIdx,
        scrambleStart: cluster.triggerTime,
        flashStart: cluster.triggerTime + SCRAMBLE_DURATION,
        decodedStart: cluster.triggerTime + SCRAMBLE_DURATION + FLASH_DURATION,
        glowEnd:
          cluster.triggerTime +
          SCRAMBLE_DURATION +
          FLASH_DURATION +
          GLOW_FADE_DURATION,
      });
    }
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

      const clusters = buildClusters(sections);
      const transitions = buildTransitions(clusters);
      transitionsRef.current = transitions;

      const totalDuration =
        transitions.length > 0
          ? Math.max(...transitions.map((t) => t.glowEnd))
          : BODY_FAST_END;

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
