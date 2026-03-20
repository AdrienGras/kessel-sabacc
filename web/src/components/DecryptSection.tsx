import { memo } from "react";
import type { CharState } from "@/hooks/useDecryptEngine";
import { cn } from "@/lib/utils";

interface DecryptSectionProps {
  title: string;
  content: string;
  titleStates: CharState[];
  contentStates: CharState[];
  scrambleChars: Map<string, string>;
  sectionIdx: number;
}

const stateClasses: Record<CharState, string> = {
  aurebesh: "font-aurebesh text-gray-300",
  scramble: "font-aurebesh text-sand decrypt-char-stable",
  flash: "font-decoded decrypt-flash decrypt-char-stable",
  decoded: "font-decoded text-gray-300",
  "glow-fade": "font-decoded decrypt-glow-fade text-gray-300",
};

interface SpanRange {
  state: CharState;
  text: string;
  keys?: string[];
  scrambleTexts?: string[];
}

function coalesceRanges(
  text: string,
  states: CharState[],
  scrambleChars: Map<string, string>,
  sectionIdx: number,
  field: string,
): SpanRange[] {
  const ranges: SpanRange[] = [];
  let i = 0;

  while (i < text.length) {
    const ch = text[i];
    const state = states[i] ?? "decoded";

    // Spaces and punctuation: collect as raw text in decoded state
    if (ch.trim().length === 0 || !/[a-zA-Z0-9]/.test(ch)) {
      let raw = ch;
      i++;
      while (i < text.length && (text[i].trim().length === 0 || !/[a-zA-Z0-9]/.test(text[i]))) {
        raw += text[i];
        i++;
      }
      // Merge into previous range if same state, else push new
      if (ranges.length > 0 && ranges[ranges.length - 1].state === "decoded") {
        ranges[ranges.length - 1].text += raw;
      } else {
        ranges.push({ state: "decoded", text: raw });
      }
      continue;
    }

    // Scramble chars must be individual spans
    if (state === "scramble") {
      const key = `${sectionIdx}-${field}-${i}`;
      const scrambleChar = scrambleChars.get(key) ?? ch;
      ranges.push({
        state: "scramble",
        text: scrambleChar,
        keys: [key],
      });
      i++;
      continue;
    }

    // Coalesce consecutive chars of same state
    let run = ch;
    i++;
    while (i < text.length && states[i] === state && /[a-zA-Z0-9]/.test(text[i])) {
      run += text[i];
      i++;
    }
    ranges.push({ state, text: run });
  }

  return ranges;
}

function renderRanges(ranges: SpanRange[], prefix: string) {
  return ranges.map((range, idx) => {
    if (range.state === "decoded" && !range.keys) {
      return (
        <span key={`${prefix}-${idx}`} className="font-decoded text-gray-300">
          {range.text}
        </span>
      );
    }
    return (
      <span
        key={range.keys?.[0] ?? `${prefix}-${idx}`}
        className={cn(stateClasses[range.state])}
      >
        {range.text}
      </span>
    );
  });
}

export const DecryptSection = memo(function DecryptSection({
  title,
  content,
  titleStates,
  contentStates,
  scrambleChars,
  sectionIdx,
}: DecryptSectionProps) {
  const titleRanges = coalesceRanges(title, titleStates, scrambleChars, sectionIdx, "title");
  const contentRanges = coalesceRanges(content, contentStates, scrambleChars, sectionIdx, "content");

  return (
    <div>
      <h3 className="mb-2 text-[11px] font-bold text-sand">
        {renderRanges(titleRanges, `t-${sectionIdx}`)}
      </h3>
      <p className="text-[9px] leading-relaxed whitespace-pre-line text-gray-300">
        {renderRanges(contentRanges, `c-${sectionIdx}`)}
      </p>
    </div>
  );
});
