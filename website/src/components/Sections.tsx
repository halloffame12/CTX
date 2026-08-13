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
    <div className="overflow-hidden rounded-lg border border-line bg-surface">
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

export function H2({ children }: { children: ReactNode }) {
  return (
    <h2 className="mt-12 mb-4 text-xl font-semibold text-ink">{children}</h2>
  );
}

export function P({ children }: { children: ReactNode }) {
  return <p className="my-4 leading-7 text-ink-soft">{children}</p>;
}

export function Note({ children }: { children: ReactNode }) {
  return (
    <div className="my-5 rounded-lg border border-line bg-paper px-4 py-3 text-sm leading-6 text-ink-soft">
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