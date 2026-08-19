const LINES: { text: string; dim?: boolean }[] = [
  { text: "$ ctx context \"find symbols for a search query\"" },
  { text: "" },
  { text: "PROJECT CONTEXT" },
  { text: "" },
  { text: "Task:" },
  { text: "  search" },
  { text: "" },
  { text: "Suggested files:" },
  { text: "  src/commands/search.rs  (score 10.00, ~213 tokens)" },
  { text: "      + exact symbol match `cmd_search`" },
  { text: "      + path matches keyword `search`" },
  { text: "  src/mcp/tools.rs  (score 8.40, ~439 tokens)" },
  { text: "      + exact symbol match `tool_search`" },
  { text: "      + imported by 4 files (hub)" },
  { text: "  src/graph/database.rs  (score 7.77, ~1923 tokens)" },
  { text: "      + exact symbol match `search`" },
  { text: "      + imported by 21 files (hub)" },
  { text: "" },
  { text: "Context budget: 11238 / 12000 tokens (estimate)", dim: true },
  { text: "Git changes considered: yes", dim: true },
  { text: "", dim: true },
  { text: "Every path comes from a real indexed graph — nothing is guessed.", dim: true },
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