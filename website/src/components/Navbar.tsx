"use client";

import { useEffect, useState } from "react";
import Link from "next/link";

const LINKS = [
  { href: "/docs", label: "Docs" },
  { href: "/playground", label: "Playground" },
  { href: "https://github.com/halloffame12/CTX", label: "GitHub", external: true },
  { href: "/docs/architecture", label: "Architecture" },
  {
    href: "https://github.com/halloffame12/CTX/blob/main/CHANGELOG.md",
    label: "Changelog",
    external: true,
  },
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
        scrolled ? "border-line-strong bg-paper/90" : "border-line bg-paper/80"
      }`}
    >
      <nav className="ctx-container flex h-14 items-center justify-between sm:h-16">
        <Link href="/" className="flex items-center gap-2.5 font-mono text-lg font-semibold text-ink">
          <span className="grid size-7 place-items-center rounded bg-ink text-sm font-bold text-white">
            &gt;_
          </span>
          <span>ctx</span>
          <span className="hidden rounded border border-line bg-surface px-1.5 py-0.5 font-mono text-[10px] font-normal text-ink-faint min-[420px]:inline-block">
            v0.1.6
          </span>
        </Link>

        <div className="hidden items-center gap-8 text-sm text-ink-soft md:flex">
          {LINKS.map((l) =>
            l.external ? (
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

        <div className="flex items-center gap-3">
          <Link
            href="/docs/install"
            className="hidden items-center rounded-md bg-ink px-3.5 py-2 text-sm font-semibold text-white transition-colors hover:bg-ink/85 sm:inline-flex"
          >
            Get started
          </Link>
          <a
            href="https://github.com/halloffame12/CTX"
            target="_blank"
            rel="noreferrer"
            className="hidden items-center gap-1.5 rounded-md border border-line bg-surface px-3 py-2 text-sm font-medium text-ink transition-colors hover:border-ink/25 sm:inline-flex"
          >
            <svg className="size-4" viewBox="0 0 16 16" fill="currentColor" aria-hidden>
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z" />
            </svg>
            GitHub
          </a>
          <button
            type="button"
            onClick={() => setOpen((o) => !o)}
            aria-label={open ? "Close menu" : "Open menu"}
            aria-expanded={open}
            className="grid size-11 shrink-0 place-items-center rounded-md border border-line text-ink transition-colors hover:bg-line/50 active:scale-[0.97] md:hidden"
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
        </div>
      </nav>

      {open && (
        <div className="border-t border-line bg-paper px-4 py-3 md:hidden">
          <div className="grid gap-1">
            {LINKS.map((l) =>
              l.external ? (
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
            <Link
              href="/docs/install"
              onClick={() => setOpen(false)}
              className="mt-2 block rounded-md bg-ink px-4 py-3 text-center text-sm font-semibold text-white"
            >
              Get started
            </Link>
          </div>
        </div>
      )}
    </header>
  );
}