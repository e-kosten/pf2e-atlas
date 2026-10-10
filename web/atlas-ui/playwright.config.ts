import { defineConfig } from "@playwright/test";
// A caller starts an actual Atlas server against its chosen checked artifact.
// This suite deliberately does not fabricate readiness or launch a mock API.
const baseURL = process.env.ATLAS_BROWSER_BASE_URL || "http://127.0.0.1:8765";
export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  workers: 1,
  timeout: 30000,
  expect: { timeout: 10000 },
  outputDir: "../../scratch/browser-validation/results",
  reporter: [
    ["list"],
    [
      "html",
      { outputFolder: "../../scratch/browser-validation/report", open: "never" },
    ],
  ],
  use: {
    baseURL,
    browserName: "chromium",
    viewport: { width: 1440, height: 1000 },
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
});
