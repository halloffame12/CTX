const LINES: { text: string; dim?: boolean }[] = [
  { text: "$ ctx context \"add Google OAuth\"" },
  { text: "" },
  { text: "Indexed 1,204 files · 8,391 symbols · 34 ms" },
  { text: "" },
  { text: "Suggested files (4 of 7 within budget):" },
  { text: "  src/auth/oauth.ts        score 0.91  path matches `auth`" },
  { text: "  src/models/user.ts       score 0.83  imported by 12 files" },
  { text: "  src/api/routes.ts        score 0.77  defines signInWithOAuth" },
  { text: "  src/db/session.ts        score 0.61  modified this branch" },
  { text: "" },
  { text: "Omitted 3 files below relevance threshold.", dim: true },
  { text: "Context budget used: 1,842 / 12,000 tokens (estimate)", dim: true },
];

export default function TerminalDemo() {
  return (
    <div className="w-full rounded-lg border border-line bg-ink text-left shadow-sm">
      <div className="flex items-center gap-2 border-b border-white/10 px-4 py-2.5">
        <span className="size-2.5 rounded-full bg-[#3f3f46]" />
        <span className="size-2.5 rounded-full bg-[#3f3f46]" />
        <span className="size-2.5 rounded-full bg-[#3f3f46]" />
        <span className="ml-3 font-mono text-xs text-white/40">ctx — zsh</span>
      </div>
      <div className="ctx-scroll overflow-x-auto p-4 font-mono text-[13px] leading-6">
        {LINES.map((line, i) => (
          <p key={i} className={line.dim ? "text-white/35" : "text-white/85"}>
            {line.text}
          </p>
        ))}
      </div>
    </div>
  );
}