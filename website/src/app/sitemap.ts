import type { MetadataRoute } from "next";

export const dynamic = "force-static";

const BASE = "https://ctx.sumitchauhan.me";

const STATIC_PAGES = [
  { path: "", priority: 1, changeFrequency: "monthly" as const },
  { path: "/docs", priority: 0.9, changeFrequency: "monthly" as const },
  { path: "/docs/install", priority: 0.8, changeFrequency: "monthly" as const },
  { path: "/docs/commands", priority: 0.8, changeFrequency: "monthly" as const },
  { path: "/docs/mcp", priority: 0.8, changeFrequency: "monthly" as const },
  { path: "/docs/architecture", priority: 0.7, changeFrequency: "yearly" as const },
  { path: "/docs/faq", priority: 0.7, changeFrequency: "monthly" as const },
];

export default function sitemap(): MetadataRoute.Sitemap {
  return STATIC_PAGES.map((p) => ({
    url: `${BASE}${p.path}/`,
    lastModified: new Date(),
    changeFrequency: p.changeFrequency,
    priority: p.priority,
  }));
}