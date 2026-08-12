import type { NextConfig } from "next";

const isDev = process.env.NODE_ENV === "development";

const nextConfig: NextConfig = {
  output: "export",
  // The GitHub Pages static export lives under /CTX. During `next dev` we
  // drop the basePath so the app serves at http://localhost:3000/.
  basePath: isDev ? "" : "/CTX",
  trailingSlash: true,
  images: { unoptimized: true },
};

export default nextConfig;