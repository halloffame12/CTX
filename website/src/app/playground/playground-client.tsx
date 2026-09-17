"use client";

import { useMemo, useState } from "react";
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";

import searchLogin from "@/data/demo/search-login.json";
import searchAuth from "@/data/demo/search-auth.json";
import searchCart from "@/data/demo/search-cart.json";
import searchReceipt from "@/data/demo/search-receipt.json";
import symApplyPromo from "@/data/demo/symbol-applyPromo.json";
import symLogin from "@/data/demo/symbol-login.json";
import symSession from "@/data/demo/symbol-session.json";
import depsCart from "@/data/demo/deps-cart.json";
import depsLogin from "@/data/demo/deps-login.json";
import impactCheckout from "@/data/demo/impact-checkout.json";
import impactLogin from "@/data/demo/impact-login.json";
import ctxPromo from "@/data/demo/context-promo.json";
import ctxOauth from "@/data/demo/context-oauth.json";
import ctxPricing from "@/data/demo/context-pricing.json";
import skelCart from "@/data/demo/skeleton-cart.json";
import skelMain from "@/data/demo/skeleton-main.json";

type Tool = "search" | "symbol" | "deps" | "impact" | "context" | "skeleton";

interface Demo {
  key: string;
  tool: Tool;
  arg: string;
  description: string;
  data: unknown;
}

const DEMOS: Demo[] = [
  { key: "search-login", tool: "search", arg: "login", description: "Find the login symbol", data: searchLogin },
  { key: "search-auth", tool: "search", arg: "auth", description: "Search for anything auth-related", data: searchAuth },
  { key: "search-cart", tool: "search", arg: "cart", description: "Search for cart symbols", data: searchCart },
  { key: "search-receipt", tool: "search", arg: "receipt", description: "Search for receipt symbols", data: searchReceipt },
  { key: "symbol-applyPromo", tool: "symbol", arg: "applyPromo", description: "Definition, refs and deps of applyPromo", data: symApplyPromo },
  { key: "symbol-login", tool: "symbol", arg: "login", description: "Definition, refs and deps of login", data: symLogin },
  { key: "symbol-session", tool: "symbol", arg: "Session", description: "Definition, refs and deps of Session", data: symSession },
  { key: "deps-cart", tool: "deps", arg: "src/cart/cart.ts", description: "What cart.ts imports and who imports it", data: depsCart },
  { key: "deps-login", tool: "deps", arg: "src/auth/login.ts", description: "What login.ts imports and who imports it", data: depsLogin },
  { key: "impact-checkout", tool: "impact", arg: "checkout", description: "What breaks if you change checkout", data: impactCheckout },
  { key: "impact-login", tool: "impact", arg: "login", description: "What breaks if you change login", data: impactLogin },
  { key: "context-promo", tool: "context", arg: "add a SAVE10 promo code discount at cart checkout", description: "Ranked context package for a promo task", data: ctxPromo },
  { key: "context-oauth", tool: "context", arg: "add Google OAuth login so returning shoppers don't have to type a password", description: "Ranked context package for an OAuth task", data: ctxOauth },
  { key: "context-pricing", tool: "context", arg: "show prices in EUR and format them with two decimals", description: "Ranked context package for a currency task", data: ctxPricing },
  { key: "skeleton-cart", tool: "skeleton", arg: "src/cart/cart.ts", description: "Body-less structural skeleton of cart.ts", data: skelCart },
  { key: "skeleton-main", tool: "skeleton", arg: "src/main.ts", description: "Body-less structural skeleton of main.ts", data: skelMain },
];

function cmdFor(d: Demo): string {
  return `ctx ${d.tool} "${d.arg}"`;
}

function score(query: string, d: Demo): number {
  const q = query.trim().toLowerCase();
  if (!q) return 0;
  const cmd = cmdFor(d).toLowerCase();
  if (cmd === q) return 100000;
  if (cmd.startsWith(q)) return 50000 - cmd.length;
  if (cmd.includes(q)) return 20000 - cmd.indexOf(q);
  const arg = d.arg.toLowerCase();
  const tool = d.tool;
  let s = 0;
  if (tool.includes(q)) s += 15000;
  if (arg.includes(q)) s += 10000 - arg.indexOf(q);
  if (d.description.toLowerCase().includes(q)) s += 5000;
  // subsequence match as a weak fallback
  let i = 0;
  for (const ch of cmd) {
    if (ch === q[i]) i += 1;
    if (i === q.length) {
      s += 2000;
      break;
    }
  }
  return s;
}

const TOOL_COLORS: Record<Tool, string> = {
  search: "text-accent-deep",
  symbol: "text-ink",
  deps: "text-ink-soft",
  impact: "text-ink",
  context: "text-accent-deep",
  skeleton: "text-ink-soft",
};

function PrettyJson({ data }: { data: unknown }) {
  const text = useMemo(() => JSON.stringify(data, null, 2), [data]);
  return (
    <pre className="ctx-scroll min-w-0 overflow-x-auto p-4 font-mono text-[12.5px] leading-relaxed text-ink-soft">
      {text}
    </pre>
  );
}

export default function Playground() {
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState<Demo>(DEMOS[5]);

  const results = useMemo(() => {
    if (!query.trim()) return [];
    return [...DEMOS]
      .map((d) => ({ d, s: score(query, d) }))
      .filter((r) => r.s > 0)
      .sort((a, b) => b.s - a.s)
      .slice(0, 5);
  }, [query]);

  const active = results[0]?.s > 0 && results[0]?.s >= 5000 && query.trim() ? results[0].d : selected;

  return (
    <>
      <Navbar />
      <main className="pb-24">
        <header className="border-b border-line bg-surface">
          <div className="ctx-container py-12 sm:py-16">
            <p className="font-mono text-[11px] uppercase tracking-widest text-accent-deep">
              Playground
            </p>
            <h1 className="mt-3 max-w-2xl text-3xl font-bold tracking-tight text-ink sm:text-4xl">
              Try CTX — no install required.
            </h1>
            <p className="mt-4 max-w-2xl text-[15px] leading-relaxed text-ink-soft">
              Every result below is real output from the CTX engine, generated against a bundled
              sample TypeScript shop. Type a query, pick an example, and explore how CTX answers
              the questions coding agents actually ask.
            </p>
            <p className="mt-3 text-[13px] text-ink-faint">
              Run these exact commands on your own repo in one line:{" "}
              <code className="rounded bg-paper px-1.5 py-0.5 font-mono text-[12px] text-ink">
                npx -y ctxai-cli mcp
              </code>
            </p>
          </div>
        </header>

        <section className="ctx-container mt-10">
          <label htmlFor="cmd" className="mb-2 block font-mono text-xs uppercase tracking-widest text-ink-faint">
            Query the demo repo
          </label>
          <div className="relative">
            <span className="pointer-events-none absolute left-4 top-1/2 -translate-y-1/2 select-none font-mono text-sm text-accent-deep">
              $
            </span>
            <input
              id="cmd"
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={'ctx context "add google oauth" …'}
              spellCheck={false}
              autoComplete="off"
              className="w-full rounded-lg border border-line bg-surface py-3 pl-8 pr-4 font-mono text-sm text-ink placeholder:text-ink-faint focus:border-ink/40 focus:outline-none"
            />
          </div>

          {results.length > 0 && (
            <ul className="mt-2 overflow-hidden rounded-lg border border-line bg-surface">
              {results.map(({ d, s }) => (
                <li key={d.key}>
                  <button
                    type="button"
                    onClick={() => {
                      setSelected(d);
                      setQuery(cmdFor(d));
                    }}
                    className="flex w-full items-center justify-between gap-4 px-4 py-2.5 text-left text-sm hover:bg-line/40"
                  >
                    <span className="truncate font-mono text-ink">
                      <span className={`font-semibold ${TOOL_COLORS[d.tool]}`}>{d.tool}</span> {d.arg}
                    </span>
                    <span className="shrink-0 text-xs text-ink-faint">{s}</span>
                  </button>
                </li>
              ))}
            </ul>
          )}

          <div className="mt-6 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {DEMOS.map((d) => (
              <button
                key={d.key}
                type="button"
                onClick={() => {
                  setSelected(d);
                  setQuery(cmdFor(d));
                }}
                className={`group rounded-lg border bg-surface p-4 text-left transition-colors ${
                  selected.key === d.key
                    ? "border-accent/50 bg-accent-pale"
                    : "border-line hover:border-ink/30"
                }`}
              >
                <span className={`block font-mono text-[12px] font-semibold ${TOOL_COLORS[d.tool]}`}>
                  ctx {d.tool} &ldquo;{d.arg}&rdquo;
                </span>
                <span className="mt-1 block text-[13px] leading-snug text-ink-soft">
                  {d.description}
                </span>
              </button>
            ))}
          </div>
        </section>

        <section className="ctx-container mt-10">
          <div className="overflow-hidden rounded-xl border border-line bg-surface shadow-sm">
            <div className="flex items-center gap-2 border-b border-line bg-paper px-4 py-2.5">
              <span className="size-2.5 rounded-full bg-line-strong" />
              <span className="size-2.5 rounded-full bg-line-strong" />
              <span className="size-2.5 rounded-full bg-line-strong" />
              <code className="ml-2 truncate font-mono text-[12.5px] text-ink">
                {cmdFor(active)}
              </code>
              <span className="ml-auto hidden font-mono text-[11px] text-ink-faint sm:block">
                real output · sample repo
              </span>
            </div>
            <PrettyJson data={active.data} />
          </div>
        </section>
      </main>
      <Footer />
    </>
  );
}