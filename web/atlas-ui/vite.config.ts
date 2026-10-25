import { fileURLToPath, URL } from "node:url";
import react from "@vitejs/plugin-react";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [tanstackRouter({ target: "react", autoCodeSplitting: true }), react()],
  build: {
    rolldownOptions: {
      output: {
        // Keep Ant's generated initializers together while bounding its dependencies
        // with the rest of the vendor graph.
        strictExecutionOrder: true,
        codeSplitting: {
          includeDependenciesRecursively: false,
          groups: [
            {
              name: "ant",
              test: /node_modules[\\/]antd[\\/]/,
              priority: 1,
            },
            { name: "vendor", test: /node_modules/, maxSize: 350_000 },
          ],
        },
      },
    },
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
    // Match CI's bounded worker count for the Ant-heavy DOM suites.
    maxWorkers: 1,
    include: ["src/**/*.test.{ts,tsx}"],
    server: { deps: { inline: [/@react-querybuilder/, /antd/] } },
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    globals: true,
  },
});
