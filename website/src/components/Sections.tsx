import type { ReactNode } from "react";
import CopyButton from "./CopyButton";

export function CodeBlock({
  title,
  code,
}: {
  title?: string;
  code: string;
}) {
  return (
    <div className="min-w-0 overflow-hidden rounded-lg border border-line bg-surface">
      {title && (
        <div className="flex items-center justify-between border-b border-line px-4 py-2">
          <span className="font-mono text-xs text-ink-faint">{title}</span>
          <CopyButton text={code} />
        </div>
      )}
      <pre className="ctx-scroll overflow-x-auto p-4 font-mono text-[13px] leading-6 text-ink">
        <code>{code}</code>
      </pre>
    </div>
  );
}

export function DocsShell({ children }: { children: ReactNode }) {
  return <div className="mx-auto max-w-3xl">{children}</div>;
}

export function H1({ children }: { children: ReactNode }) {
  return (
    <h1 className="text-3xl font-bold tracking-tight text-ink sm:text-4xl">
      {children}
    </h1>
  );
}

export function H2({ children, id }: { children: ReactNode; id?: string }) {
  return (
    <h2 id={id} className="mt-12 mb-4 text-xl font-semibold text-ink">
      {children}
    </h2>
  );
}

export function P({ children }: { children: ReactNode }) {
  return <p className="my-4 leading-7 text-ink-soft">{children}</p>;
}

export function Code({ children }: { children: ReactNode }) {
  return (
    <code className="rounded bg-line/60 px-1.5 py-0.5 font-mono text-[0.85em] text-ink">
      {children}
    </code>
  );
}

export function Ul({ children }: { children: ReactNode }) {
  return <ul className="my-4 space-y-2 pl-5 leading-7 text-ink-soft [&>li]:list-disc">{children}</ul>;
}

export function Ol({ children }: { children: ReactNode }) {
  return <ol className="my-4 list-decimal space-y-2 pl-5 leading-7 text-ink-soft">{children}</ol>;
}

export function Li({ children }: { children: ReactNode }) {
  return <li>{children}</li>;
}

export function Note({ children }: { children: ReactNode }) {
  return (
    <div className="my-5 rounded-lg border border-line bg-paper px-4 py-3 text-sm leading-6 text-ink-soft">
      {children}
    </div>
  );
}

export function Warn({ children }: { children: ReactNode }) {
  return (
    <div className="my-5 rounded-lg border border-amber-300 bg-amber-50 px-4 py-3 text-sm leading-6 text-amber-900">
      <span className="mb-0.5 block font-mono text-xs font-bold uppercase tracking-wider text-amber-700">
        Caution
      </span>
      {children}
    </div>
  );
}

export function CommandTable({
  rows,
}: {
  rows: { cmd: string; desc: string }[];
}) {
  return (
    <div className="my-6 overflow-x-auto rounded-lg border border-line">
      <table className="w-full border-collapse text-sm">
        <thead>
          <tr className="border-b border-line bg-paper text-left">
            <th className="px-4 py-2.5 font-mono text-xs font-semibold text-ink">
              Command
            </th>
            <th className="px-4 py-2.5 text-xs font-semibold text-ink">
              What it does
            </th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.cmd} className="border-b border-line last:border-0">
              <td className="px-4 py-2.5 align-top font-mono text-[13px] text-accent-deep">
                {r.cmd}
              </td>
              <td className="px-4 py-2.5 align-top leading-6 text-ink-soft">
                {r.desc}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}