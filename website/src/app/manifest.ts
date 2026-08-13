import type { MetadataRoute } from "next";

export const dynamic = "force-static";

export default function manifest(): MetadataRoute.Manifest {
  return {
    name: "ctx — codebase context for AI coding agents",
    short_name: "ctx",
    description:
      "A local, deterministic code graph for AI agents: symbol search, impact analysis, and ranked context over stdio.",
    start_url: "/",
    display: "standalone",
    background_color: "#fafaf9",
    theme_color: "#0d9488",
    icons: [
      {
        src: "/icon.svg",
        sizes: "any",
        type: "image/svg+xml",
      },
    ],
  };
}