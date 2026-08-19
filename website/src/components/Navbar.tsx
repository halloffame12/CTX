"use client";

import { useEffect, useState } from "react";
import Link from "next/link";

const LINKS = [
  { href: "/docs", label: "Docs" },
  { href: "/#film", label: "The film" },
  { href: "/#install", label: "Install" },
  { href: "https://github.com/halloffame12/CTX", label: "GitHub" },
];

export default function Navbar() {
  const [scrolled, setScrolled] = useState(false);
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 8);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <header
      className={`sticky top-0 z-50 border-b backdrop-blur-sm transition-colors ${
        scrolled ? "border-line bg-paper/90" : "border-line bg-paper/80"
      }`}
    >
      <nav className="ctx-container flex h-14 items-center justify-between sm:h-16">
        <Link href="/" className="flex items-center gap-2.5 font-mono text-lg font-semibold text-ink">
          <span className="grid size-7 place-items-center rounded bg-accent text-sm font-bold text-white">
            &gt;_
          </span>
          <span>ctx</span>
          <span className="hidden rounded border border-line bg-paper px-1.5 py-0.5 font-mono text-[10px] font-normal text-ink-faint min-[420px]:inline-block">
            v0.1.2
          </span>
        </Link>

        <div className="hidden items-center gap-8 text-sm text-ink-soft md:flex">
          {LINKS.map((l) =>
            l.href.startsWith("http") ? (
              <a
                key={l.label}
                href={l.href}
                target="_blank"
                rel="noreferrer"
                className="relative py-1 text-ink-soft transition-colors hover:text-ink after:absolute after:inset-x-0 after:bottom-0 after:h-px after:origin-left after:scale-x-0 after:bg-ink after:transition-transform after:duration-200 hover:after:scale-x-100"
              >
                {l.label}
              </a>
            ) : (
              <Link
                key={l.label}
                href={l.href}
                className="relative py-1 text-ink-soft transition-colors hover:text-ink after:absolute after:inset-x-0 after:bottom-0 after:h-px after:origin-left after:scale-x-0 after:bg-ink after:transition-transform after:duration-200 hover:after:scale-x-100"
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
          className="grid size-11 shrink-0 place-items-center rounded-lg border border-line text-ink transition-colors hover:bg-line/50 active:scale-[0.97] md:hidden"
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
        <div className="border-t border-line bg-paper px-4 py-3 md:hidden">
          {LINKS.map((l) =>
            l.href.startsWith("http") ? (
              <a
                key={l.label}
                href={l.href}
                target="_blank"
                rel="noreferrer"
                onClick={() => setOpen(false)}
                className="block rounded px-2 py-3 text-sm text-ink-soft hover:bg-line/50 hover:text-ink"
              >
                {l.label}
              </a>
            ) : (
              <Link
                key={l.label}
                href={l.href}
                onClick={() => setOpen(false)}
                className="block rounded px-2 py-3 text-sm text-ink-soft hover:bg-line/50 hover:text-ink"
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