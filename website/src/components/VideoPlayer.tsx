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
        className={`relative w-full overflow-hidden rounded-lg border border-line-strong bg-ink shadow-sm ${aspect}`}
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
            className="group relative block h-full w-full overflow-hidden bg-ink text-left"
          >
            <span className="absolute inset-x-0 top-0 flex items-center justify-between p-3.5 font-mono text-[11px] uppercase tracking-widest text-white/50 sm:p-4">
              <span>{eyebrow ?? "ctx"}</span>
              <span className="flex items-center gap-1.5">
                <span className="inline-block size-1.5 rounded-full bg-accent" aria-hidden />
                watch
              </span>
            </span>
            <span className="absolute inset-0 grid place-items-center">
              <span className="grid size-14 place-items-center rounded-md border border-white/20 bg-white/5 text-white backdrop-blur-sm transition-colors duration-200 group-hover:border-white/40 group-hover:bg-white/10 group-active:scale-95 sm:size-16">
                <svg
                  className="ml-0.5 size-6 sm:size-7"
                  viewBox="0 0 24 24"
                  fill="currentColor"
                  aria-hidden
                >
                  <path d="M8 5v14l11-7z" />
                </svg>
              </span>
            </span>
            <span className="absolute inset-x-0 bottom-0 p-4 sm:p-5">
              <span className="block max-w-xl text-lg font-semibold leading-snug text-white sm:text-xl">
                {title}
              </span>
              {caption && (
                <span className="mt-1 block max-w-xl text-[13px] leading-6 text-white/60">
                  {caption}
                </span>
              )}
            </span>
          </button>
        )}
      </div>
      <div className="mt-2.5 flex flex-wrap items-center justify-between gap-2 font-mono text-[11px] text-ink-faint">
        <span>{title}</span>
        <span>mp4 · 1080p · click to play</span>
      </div>
    </div>
  );
}