export type DocNavItem = {
  href: string;
  label: string;
  desc: string;
};

export const DOC_NAV: DocNavItem[] = [
  { href: "/docs", label: "Overview", desc: "What ctx is, what it isn't, and the core loop." },
  { href: "/docs/install", label: "Installation", desc: "npm, cargo, and prebuilt binaries." },
  {
    href: "/docs/commands",
    label: "Command reference",
    desc: "Every command and flag, with examples.",
  },
  { href: "/docs/config", label: "Configuration", desc: "The .ctx/config.toml file and its defaults." },
  { href: "/docs/mcp", label: "MCP server", desc: "Expose the code graph to any MCP client." },
  {
    href: "/docs/architecture",
    label: "How it works",
    desc: "Indexing, resolution, ranking, and git awareness.",
  },
  { href: "/docs/faq", label: "FAQ & limitations", desc: "Privacy, supported languages, and honest limits." },
];