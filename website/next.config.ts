import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  basePath: "/CTX",
  trailingSlash: true,
  images: { unoptimized: true },
};

export default nextConfig;