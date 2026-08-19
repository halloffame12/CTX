"use client";

import { useState } from "react";

export default function VideoPlayer({
  src,
  title,
  caption,
  eyebrow,
  aspect = "aspect-video",
}: {
  src: string;
  title: string;
  caption?: string;
  eyebrow?: string;
  aspect?: string;
}) {
  const [playing, setPlaying] = useState(false);

  return (
    <div>
      <div
        className={`relative w-full overflow-hidden rounded-xl border border-line bg-ink shadow-lg ${aspect}`}
      >
        {playing ? (
          <video
            className="h-full w-full"
            controls
            autoPlay
            playsInline
            preload="metadata"
            aria-label={title}
          >
            <source src={src} type="video/mp4" />
          </video>
        ) : (
          <button
            type="button"
            onClick={() => setPlaying(true)}
            aria-label={`Play video: ${title}`}
            className="group relative block h-full w-full overflow-hidden text-left"
          >
            <span className="absolute inset-0 bg-[radial-gradient(120%_120%_at_50%_0%,rgba(13,148,136,0.35),rgba(13,148,136,0.08)_45%,transparent_70%),linear-gradient(180deg,rgba(28,25,23,0.55),rgba(28,25,23,0.9))]" />
            <span className="absolute inset-0 bg-[linear-gradient(rgba(255,255,255,0.04)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.04)_1px,transparent_1px)] bg-[size:44px_44px] opacity-40" />

            <span className="absolute inset-x-0 top-0 flex items-center justify-between p-4 font-mono text-[11px] uppercase tracking-widest text-white/60">
              <span>{eyebrow ?? "ctx"}</span>
              <span className="flex items-center gap-1.5">
                <span className="inline-block size-1.5 rounded-full bg-accent" />
                watch
              </span>
            </span>

            <span className="absolute inset-0 grid place-items-center">
              <span className="grid size-16 place-items-center rounded-full bg-white/95 shadow-xl ring-8 ring-white/10 transition-transform duration-200 group-hover:scale-105 group-active:scale-95 sm:size-20">
                <svg
                  className="ml-0.5 size-7 text-ink sm:size-8"
                  viewBox="0 0 24 24"
                  fill="currentColor"
                  aria-hidden
                >
                  <path d="M8 5v14l11-7z" />
                </svg>
              </span>
            </span>

            <span className="absolute inset-x-0 bottom-0 p-4 sm:p-5">
              <span className="block max-w-xl text-lg font-semibold leading-snug text-white sm:text-2xl">
                {title}
              </span>
              {caption && (
                <span className="mt-1 block max-w-xl text-sm leading-6 text-white/70">
                  {caption}
                </span>
              )}
            </span>
          </button>
        )}
      </div>
      <div className="mt-3 flex flex-wrap items-center justify-between gap-2 font-mono text-xs text-ink-faint">
        <span>{title}</span>
        {caption && <span>{caption}</span>}
      </div>
    </div>
  );
}