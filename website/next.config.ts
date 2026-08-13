import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  // The site is served from the custom domain ctx.sumitchauhan.me, so no
  // basePath is needed in production. During `next dev` it also serves at
  // http://localhost:3000/.
  basePath: "",
  trailingSlash: true,
  images: { unoptimized: true },
};

export default nextConfig;