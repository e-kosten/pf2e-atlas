import { fileURLToPath, URL } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

const testDeadlineScale = process.env.CI === "true" ? 2 : 1;

export default defineConfig({
  plugins: [react()],
  define: {
    __ATLAS_TEST_DEADLINE_SCALE__: JSON.stringify(testDeadlineScale),
  },
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
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    globals: true,
    testTimeout: 5_000 * testDeadlineScale,
  },
});
