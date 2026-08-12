"use client";

import { useEffect, useRef, useState } from "react";

type Scene = {
  prompt: string;
  lines: (string | { text: string; dim?: boolean; ok?: boolean | null })[];
};

const SCENES: Scene[] = [
  {
    prompt: "ctx context \"add Google Auth\"",
    lines: [
      { text: "# context package · 7,842 / 8,000 tokens (estimate)" },
      { text: "Recommended files:", ok: null },
      { text: "  src/auth/oauth.ts      (score 0.91)  path matches `auth`", ok: true },
      { text: "  src/models/user.ts      (score 0.83)  imported by 12 files (hub)" },
      { text: "  src/api/routes.ts       (score 0.77)  has symbol `signInWithOAuth`", ok: true },
      { text: "  src/db/session.ts       (score 0.61)  modified recently", ok: true },
      { text: "Omitted: 3 lower-relevance files", dim: true },
    ],
  },
  {
    prompt: "ctx impact UserService.update",
    lines: [
      { text: "# impact analysis · depth 3" },
      { text: "Direct dependents  2 files, 4 symbols", ok: null },
      { text: "  src/controllers/user.rs", ok: true },
      { text: "  src/handlers/admin.rs", ok: true },
      { text: "Indirect dependents  1 file, 2 symbols", ok: null },
      { text: "  tests/integration.rs", ok: true },
      { text: "UNKNOWN 1 unresolved import in affected files", dim: true },
    ],
  },
  {
    prompt: "ctx search \"rate limit\"",
    lines: [
      { text: "Symbols", ok: null },
      { text: "  rate_limiter.create      src/middleware/ratelimit.ex  (method)", ok: true },
      { text: "  RateLimiter              src/core/rate.rs  (struct)", ok: true },
      { text: "  is_rate_limited          src/api/guard.go  (function)", ok: true },
      { text: "Files", ok: null },
      { text: "  src/middleware/ratelimit.ex", ok: true },
      { text: "  3 results · 0.4ms", dim: true },
    ],
  },
{
    prompt: "ctx doctor --json",
    lines: [
      { text: "{" },
      { text: "  \"status\": \"READY\"," },
      { text: "  \"git\": true," },
      { text: "  \"framework\": \"serde\"," },
      { text: "  \"index\": { \"files\": 52, \"symbols\": 468 }," },
      { text: "  \"warnings\": []" },
      { text: "}" },
    ],
  },
];

export default function TerminalDemo() {
  const [sceneIdx, setSceneIdx] = useState(0);
  const [shown, setShown] = useState(0);
  const [typed, setTyped] = useState(0);
  const [done, setDone] = useState(false);

  const scene = SCENES[sceneIdx];
  const promptRef = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    let cancelled = false;

    let timer: number | ReturnType<typeof setTimeout>;
    const t0 = performance.now();
    const step = (now: number) => {
      if (cancelled) return;
      const elapsed = now - t0;
      const charsPerSec = 55;
      const newTyped = Math.min(scene.prompt.length, Math.floor((elapsed / 1000) * charsPerSec));
      setTyped(newTyped);
      if (newTyped < scene.prompt.length) {
        timer = requestAnimationFrame(step);
      } else {
        timer = setTimeout(() => {
          if (cancelled) return;
          setDone(true);
          timer = setTimeout(() => {
            if (!cancelled) {
              setTyped(0);
              setShown(0);
              setDone(false);
              setSceneIdx((s) => (s + 1) % SCENES.length);
            }
          }, 5200);
        }, 420);
      }
    };
    timer = requestAnimationFrame(step);

    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [sceneIdx, scene.prompt.length]);

  useEffect(() => {
    if (!done) return;
    let i = 0;
    const ival = setInterval(() => {
      i += 1;
      setShown(i);
      if (i >= scene.lines.length) clearInterval(ival);
    }, 150);
    return () => clearInterval(ival);
  }, [done, sceneIdx, scene.lines.length]);

  return (
    <div className="w-full rounded-2xl border border-white/10 bg-ink-900/90 shadow-2xl shadow-black/50 backdrop-blur">
      <div className="flex items-center gap-2 border-b border-white/10 px-4 py-3">
        <span className="size-3 rounded-full bg-red-400/80" />
        <span className="size-3 rounded-full bg-amber-400/80" />
        <span className="size-3 rounded-full bg-emerald-400/80" />
        <span className="ml-3 font-mono text-xs text-slate-500">ctx — zsh</span>
      </div>
      <div className="ctx-scroll max-h-[300px] overflow-y-auto p-4 font-mono text-[13px] leading-6 sm:max-h-[340px] sm:text-sm">
        <p className="text-slate-500">
          <span className="text-accent-400">➜</span>{" "}
          <span className="text-slate-300">~</span>{" "}
          <span ref={promptRef}>
            {scene.prompt.slice(0, typed)}
            <span className={`ctx-caret text-accent-400 ${done ? "hidden" : ""}`}>▍</span>
          </span>
        </p>
        {scene.lines.slice(0, shown).map((line, i) => {
          const text = typeof line === "string" ? line : line.text;
          const dim = typeof line === "object" && line.dim;
          const ok = typeof line === "object" ? line.ok : null;
          return (
            <p
              key={i}
              className={
                dim
                  ? "text-slate-600"
                  : ok === true
                    ? "text-emerald-300/90"
                    : ok === false
                      ? "text-red-300/80"
                      : "text-slate-300"
              }
            >
              {text}
            </p>
          );
        })}
        <p className={`mt-1 text-accent-400 ${done ? "" : "hidden"}`}>
          <span className="text-slate-300">~</span> <span className="ctx-caret">▍</span>
        </p>
      </div>
    </div>
  );
}