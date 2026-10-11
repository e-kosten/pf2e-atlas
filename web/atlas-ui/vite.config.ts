import { fileURLToPath, URL } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    strictPort: false,
    fs: {
      allow: [fileURLToPath(new URL("../..", import.meta.url))],
    },
    proxy: {
      "/api": {
        target: process.env.ATLAS_API_PROXY ?? "http://127.0.0.1:4727",
        changeOrigin: true,
      },
    },
  },
  test: {
    // Match CI's bounded worker count for the Ant-heavy DOM suites.
    maxWorkers: 1,
    include: ["src/**/*.test.{ts,tsx}"],
    server: { deps: { inline: [/@react-querybuilder/, /antd/] } },
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    globals: true,
  },
});
