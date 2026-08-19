"use client";

import { useEffect, useState } from "react";
import Link from "next/link";

const LINKS = [
  { href: "/docs", label: "Docs" },
  { href: "/#film", label: "The film" },
  { href: "/#install", label: "Install" },
  { href: "https://github.com/halloffame12/CTX", label: "GitHub" },
];

export default function Navbar({ dark = false }: { dark?: boolean }) {
  const [scrolled, setScrolled] = useState(false);
  const [open, setOpen] = useState(false);
  const light = dark && !scrolled;

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 8);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <header
      className={`sticky top-0 z-50 border-b transition-colors ${
        light
          ? "border-white/10 bg-transparent"
          : scrolled
            ? "border-line bg-paper/90 backdrop-blur-sm"
            : "border-transparent bg-transparent"
      }`}
    >
      <nav className="ctx-container flex h-14 items-center justify-between sm:h-16">
        <Link href="/" className={`flex items-center gap-2.5 font-mono text-lg font-semibold ${light ? "text-white" : "text-ink"}`}>
          <span className="grid size-7 place-items-center rounded bg-accent text-sm font-bold text-white">
            &gt;_
          </span>
          <span>ctx</span>
          <span
            className={`hidden rounded border px-1.5 py-0.5 font-mono text-[10px] font-normal min-[420px]:inline-block ${
              light ? "border-white/20 text-white/70" : "border-line bg-paper text-ink-faint"
            }`}
          >
            v0.1.2
          </span>
        </Link>

        <div
          className={`hidden items-center gap-8 text-sm md:flex ${
            light ? "text-white/70" : "text-ink-soft"
          }`}
        >
          {LINKS.map((l) =>
            l.href.startsWith("http") ? (
              <a
                key={l.label}
                href={l.href}
                target="_blank"
                rel="noreferrer"
                className={`relative py-1 transition-colors ${
                  light ? "hover:text-white after:bg-white" : "hover:text-ink after:bg-ink"
                } after:absolute after:inset-x-0 after:bottom-0 after:h-px after:origin-left after:scale-x-0 after:transition-transform after:duration-200 hover:after:scale-x-100`}
              >
                {l.label}
              </a>
            ) : (
              <Link
                key={l.label}
                href={l.href}
                className={`relative py-1 transition-colors ${
                  light ? "hover:text-white after:bg-white" : "hover:text-ink after:bg-ink"
                } after:absolute after:inset-x-0 after:bottom-0 after:h-px after:origin-left after:scale-x-0 after:transition-transform after:duration-200 hover:after:scale-x-100`}
              >
                {l.label}
              </Link>
            )
          )}
        </div>

        <button
          type="button"
          onClick={() => setOpen((o) => !o)}
          aria-label={open ? "Close menu" : "Open menu"}
          className={`grid size-11 shrink-0 place-items-center rounded-lg border transition-colors active:scale-[0.97] md:hidden ${
            light
              ? "border-white/25 text-white hover:bg-white/10"
              : "border-line text-ink hover:bg-line/50"
          }`}
        >
          <svg className="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" aria-hidden>
            {open ? (
              <path d="M6 6l12 12M18 6L6 18" />
            ) : (
              <>
                <path d="M4 7h16M4 12h16M4 17h16" />
              </>
            )}
          </svg>
        </button>
      </nav>

      {open && (
        <div
          className={`border-t px-4 py-3 md:hidden ${
            light ? "border-white/10 bg-ink text-white" : "border-line bg-paper"
          }`}
        >
          {LINKS.map((l) =>
            l.href.startsWith("http") ? (
              <a
                key={l.label}
                href={l.href}
                target="_blank"
                rel="noreferrer"
                onClick={() => setOpen(false)}
                className={`block rounded px-2 py-3 text-sm ${
                  light ? "text-white/80 hover:bg-white/10 hover:text-white" : "text-ink-soft hover:bg-line/50 hover:text-ink"
                }`}
              >
                {l.label}
              </a>
            ) : (
              <Link
                key={l.label}
                href={l.href}
                onClick={() => setOpen(false)}
                className={`block rounded px-2 py-3 text-sm ${
                  light ? "text-white/80 hover:bg-white/10 hover:text-white" : "text-ink-soft hover:bg-line/50 hover:text-ink"
                }`}
              >
                {l.label}
              </Link>
            )
          )}
        </div>
      )}
    </header>
  );
}