import type { ReactNode } from "react";
import CopyButton from "./CopyButton";

export function Section({
  id,
  children,
  className = "",
}: {
  id?: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <section id={id} className={`scroll-mt-20 px-4 py-16 sm:px-6 sm:py-24 ${className}`}>
      <div className="mx-auto max-w-6xl">{children}</div>
    </section>
  );
}

export function Eyebrow({ children }: { children: ReactNode }) {
  return (
    <p className="mb-3 flex items-center gap-2 font-mono text-xs uppercase tracking-widest text-accent-400 sm:text-sm">
      <span aria-hidden>▍</span>
      {children}
    </p>
  );
}

export function SectionTitle({ children }: { children: ReactNode }) {
  return (
    <h2 className="text-3xl font-bold tracking-tight text-white sm:text-4xl">{children}</h2>
  );
}

export function CodeBlock({ title, code }: { title: string; code: string }) {
  return (
    <div className="overflow-hidden rounded-xl border border-white/10 bg-ink-900">
      <div className="flex items-center justify-between border-b border-white/10 px-4 py-2">
        <span className="font-mono text-xs text-slate-400">{title}</span>
        <CopyButton text={code} />
      </div>
      <pre className="ctx-scroll overflow-x-auto p-4 font-mono text-[13px] leading-6 text-slate-200">
        <code>{code}</code>
      </pre>
    </div>
  );
}

export function FeatureCard({
  icon,
  title,
  children,
}: {
  icon: ReactNode;
  title: string;
  children: ReactNode;
}) {
  return (
    <div className="group rounded-2xl border border-white/10 bg-ink-900/60 p-6 transition-colors hover:border-accent-500/40">
      <div className="mb-4 grid size-11 place-items-center rounded-xl border border-white/10 bg-gradient-to-br from-accent-400/15 to-sky-500/15 text-accent-300 transition-transform group-hover:scale-105">
        {icon}
      </div>
      <h3 className="mb-2 text-lg font-semibold text-white">{title}</h3>
      <p className="text-sm leading-6 text-slate-400">{children}</p>
    </div>
  );
}