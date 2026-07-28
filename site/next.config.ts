import path from "node:path";
import type { NextConfig } from "next";

const isStaticExport = process.env.npm_lifecycle_event !== "build:sites";

const nextConfig: NextConfig = {
  ...(isStaticExport ? { output: "export" as const } : {}),
  trailingSlash: true,
  turbopack: {
    root: path.resolve(process.cwd(), "..")
  }
};

export default nextConfig;
