"use client";

import { useEffect, useState } from "react";

type TocItem = { id: string; text: string; level: number };

function collect(): TocItem[] {
  const root = document.querySelector("[data-doc-article]");
  if (!root) return [];
  return Array.from(root.querySelectorAll("h2[id], h3[id]")).map((el) => ({
    id: el.id,
    text: el.textContent ?? "",
    level: el.tagName === "H2" ? 2 : 3,
  }));
}

export default function DocsToc() {
  const [items, setItems] = useState<TocItem[]>([]);
  const [active, setActive] = useState<string>("");

  useEffect(() => {
    const frame = requestAnimationFrame(() => {
      setItems(collect());
    });

    const onScroll = () => {
      let current = "";
      for (const el of document.querySelectorAll("[data-doc-article] h2[id], [data-doc-article] h3[id]")) {
        if (el.getBoundingClientRect().top <= 96) current = el.id;
      }
      setActive(current);
    };
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener("scroll", onScroll);
    };
  }, []);

  if (items.length === 0) return null;

  return (
    <nav aria-label="On this page" className="sticky top-20 max-h-[calc(100vh-6rem)] overflow-y-auto">
      <p className="mb-3 font-mono text-xs uppercase tracking-widest text-ink-faint">On this page</p>
      <ul className="space-y-1 border-l border-line">
        {items.map((item) => (
          <li key={item.id}>
            <a
              href={`#${item.id}`}
              className={`block border-l-2 py-1 pr-2 text-[13px] leading-5 transition-colors ${
                item.level === 3 ? "pl-6" : "pl-3"
              } ${
                active === item.id
                  ? "-ml-px border-accent text-accent-deep"
                  : "border-transparent text-ink-faint hover:text-ink"
              }`}
            >
              {item.text}
            </a>
          </li>
        ))}
      </ul>
    </nav>
  );
}