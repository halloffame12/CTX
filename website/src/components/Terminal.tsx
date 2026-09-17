"use client";

import { useEffect, useMemo, useRef, useState } from "react";

type Line = { text: string; prompt?: boolean; dim?: boolean; accent?: boolean };

export default function Terminal({
  title = "ctx",
  prompt = "$",
  lines,
  startDelay = 700,
  charMs = 16,
}: {
  title?: string;
  prompt?: string;
  lines: Line[];
  startDelay?: number;
  charMs?: number;
}) {
  const [shown, setShown] = useState(0);
  const reduced = useRef(false);

  useEffect(() => {
    reduced.current =
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduced.current) {
      setShown(lines.reduce((n, l) => n + l.text.length + 1, 0));
      return;
    }
    const full = lines.reduce((n, l) => n + l.text.length + 1, 0);
    let i = 0;
    let interval: ReturnType<typeof setInterval> | undefined;
    const t0 = setTimeout(() => {
      interval = setInterval(() => {
        i += 1;
        setShown(i);
        if (i >= full) {
          if (interval) clearInterval(interval);
        }
      }, charMs);
    }, startDelay);
    return () => {
      clearTimeout(t0);
      if (interval) clearInterval(interval);
    };
  }, [lines, startDelay, charMs]);

  // Flatten into a single stream so the typewriter advances across the whole buffer.
  // `starts[li]` = index at which line `li` begins in the concatenated stream.
  const starts = useMemo(() => {
    const out: number[] = [];
    let acc = 0;
    for (const l of lines) {
      out.push(acc);
      acc += l.text.length + 1;
    }
    return out;
  }, [lines]);
  const total = lines.reduce((n, l) => n + l.text.length + 1, 0);

  const isDone = shown >= total;

  return (
    <div className="overflow-hidden rounded-lg border border-line bg-ink text-left shadow-sm">
      <div className="flex items-center gap-2 border-b border-white/10 px-4 py-2.5">
        <span className="size-2.5 rounded-full bg-white/15" aria-hidden />
        <span className="size-2.5 rounded-full bg-white/15" aria-hidden />
        <span className="ml-1 truncate font-mono text-xs text-white/50">{title}</span>
      </div>
      <pre
        className="ctx-scroll overflow-x-auto px-5 py-4 font-mono text-[13px] leading-6 sm:text-sm"
        style={{ minHeight: 0 }}
      >
        {lines.map((line, li) => {
          const start = starts[li] ?? 0;
          const len = line.text.length + 1;
          const partial = shown - start;
          const reveal = Math.max(0, Math.min(len, partial));
          if (isDone) {
            return (
              <div
                key={li}
                className={`whitespace-pre ${
                  line.prompt
                    ? "font-medium text-white"
                    : line.accent
                      ? "text-accent"
                      : line.dim
                        ? "text-white/45"
                        : "text-white/70"
                }`}
              >
                {line.prompt ? `${prompt} ${line.text}` : line.text}
              </div>
            );
          }
          const text = line.text.slice(0, reveal);
          const showCursor = reveal < len && reveal >= 0;
          return (
            <div
              key={li}
              className={`whitespace-pre ${
                reveal === 0
                  ? "h-6"
                  : line.prompt
                    ? "font-medium text-white"
                    : line.accent
                      ? "text-accent"
                      : line.dim
                        ? "text-white/45"
                        : "text-white/70"
              }`}
            >
              {line.prompt ? `${prompt} ${text}` : text}
              {showCursor && text.length === line.text.length ? (
                <span className="ctx-cursor" aria-hidden />
              ) : null}
            </div>
          );
        })}
        <div className="mt-1.5 flex items-center gap-2 border-t border-white/10 pt-3 text-[11px] text-white/40">
          <span className="inline-block size-1.5 rounded-full bg-accent" aria-hidden />
          local · deterministic · offline
        </div>
      </pre>
    </div>
  );
}