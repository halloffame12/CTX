"use client";

import { useMemo, useState } from "react";

export type GraphNode = {
  id: string;
  label: string; // file path, rendered in mono
  x: number;
  y: number;
  target?: boolean; // emphasized node (e.g. the change)
};

export type GraphEdge = {
  from: string;
  to: string;
  label?: string; // e.g. "imports", "extends"
};

export default function CodeGraph({
  title,
  caption,
  nodes,
  edges,
  width = 640,
  height = 360,
}: {
  title: string;
  caption: string;
  nodes: GraphNode[];
  edges: GraphEdge[];
  width?: number;
  height?: number;
}) {
  const [active, setActive] = useState<string | null>(null);

  const connected = useMemo(() => {
    const s = new Set<string>();
    if (!active) return s;
    for (const e of edges) {
      if (e.from === active) s.add(e.to);
      if (e.to === active) s.add(e.from);
    }
    return s;
  }, [active, edges]);

  const dim = (id: string) => active !== null && id !== active && !connected.has(id);

  return (
    <figure className="overflow-hidden rounded-lg border border-line bg-surface">
      <figcaption className="flex flex-wrap items-center justify-between gap-2 border-b border-line px-4 py-2.5">
        <span className="font-mono text-xs uppercase tracking-widest text-ink-faint">{title}</span>
        <span className="hidden font-mono text-[11px] text-ink-faint sm:inline">
          {active ? (
            <span className="text-accent-deep">focus: {active}</span>
          ) : (
            "hover a node to trace its edges"
          )}
        </span>
      </figcaption>
      <svg
        role="img"
        aria-label={title}
        viewBox={`0 0 ${width} ${height}`}
        className="block h-auto w-full bg-[radial-gradient(circle_at_50%_45%,rgba(13,148,136,0.06),transparent_60%)]"
        onMouseLeave={() => setActive(null)}
      >
        {/* edges */}
        {edges.map((e, i) => {
          const from = nodes.find((n) => n.id === e.from);
          const to = nodes.find((n) => n.id === e.to);
          if (!from || !to) return null;
          const hot = active !== null && (from.id === active || to.id === active);
          const faded = active !== null && !hot;
          const mx = (from.x + to.x) / 2;
          const my = (from.y + to.y) / 2;
          return (
            <g key={i} className="transition-opacity duration-200" style={{ opacity: faded ? 0.18 : 1 }}>
              <line
                x1={from.x}
                y1={from.y}
                x2={to.x}
                y2={to.y}
                stroke={hot ? "#0d9488" : "#d2d2cc"}
                strokeWidth={hot ? 1.75 : 1}
              />
              {e.label && (
                <g transform={`translate(${mx} ${my})`}>
                  <rect
                    x={-e.label.length * 3.4 - 5}
                    y={-8}
                    width={e.label.length * 6.8 + 10}
                    height={16}
                    rx={3}
                    fill="#ffffff"
                    stroke={hot ? "#0d9488" : "#d2d2cc"}
                    strokeWidth={0.75}
                  />
                  <text
                    textAnchor="middle"
                    dominantBaseline="central"
                    fontSize={9}
                    fontFamily="var(--font-geist-mono), ui-monospace, monospace"
                    fill={hot ? "#0b6b62" : "#70707a"}
                  >
                    {e.label}
                  </text>
                </g>
              )}
            </g>
          );
        })}

        {/* nodes */}
        {nodes.map((n) => {
          const hot = active !== null && (n.id === active || connected.has(n.id));
          return (
            <g
              key={n.id}
              tabIndex={0}
              role="button"
              aria-label={`${n.id} — ${n.label}`}
              className="cursor-pointer outline-none transition-opacity duration-200"
              style={{ opacity: dim(n.id) ? 0.18 : 1 }}
              onMouseEnter={() => setActive(n.id)}
              onFocus={() => setActive(n.id)}
              onBlur={() => setActive(null)}
              onKeyDown={(ev) => {
                if (ev.key === "Enter" || ev.key === " ") {
                  ev.preventDefault();
                  setActive(active === n.id ? null : n.id);
                }
              }}
            >
              <circle
                cx={n.x}
                cy={n.y}
                r={n.target ? 6 : 4.5}
                fill={n.target ? "#0d9488" : hot ? "#0b6b62" : "#12121a"}
              />
              <text
                x={n.x + 12}
                y={n.y + 2.5}
                fontSize={n.target ? 12 : 11}
                fontFamily="var(--font-geist-mono), ui-monospace, monospace"
                fill={n.target ? "#0b6b62" : hot ? "#0b6b62" : "#4b4b52"}
              >
                {n.label}
              </text>
            </g>
          );
        })}
      </svg>
      <p className="border-t border-line bg-paper px-4 py-2.5 text-[13px] leading-6 text-ink-faint">
        {caption}
      </p>
    </figure>
  );
}