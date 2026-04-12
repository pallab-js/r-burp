import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: process.env.NODE_ENV === "production" ? "export" : undefined,
  distDir: process.env.NODE_ENV === "production" ? "out" : undefined,
  basePath: "",
  images: {
    unoptimized: true,
  },
};

export default nextConfig;
