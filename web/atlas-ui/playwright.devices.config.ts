import { defineConfig, devices } from "@playwright/test";
import { resolve } from "node:path";
import { browserFixture } from "../../scripts/browser-fixture.mjs";

const touchSpecs = [
  "selection-search.spec.ts",
  "journeys.spec.ts",
  "reader-typography.spec.ts",
  "modal-layout.spec.ts",
  "list-metadata.spec.ts",
  "saved-edit-conflicts.spec.ts",
  "feedback.spec.ts",
  "spell-effects.spec.ts",
  "conditions.spec.ts",
  "condition-refinement.spec.ts",
  "encounter-forms.spec.ts",
  "encounter-name-edit.spec.ts",
  "turn-reminders.spec.ts",
];

const port = Number(process.env.ATLAS_DEVICE_PORT ?? 8796);
if (!Number.isInteger(port) || port < 1 || port > 65535)
  throw new Error("ATLAS_DEVICE_PORT must be a port from 1 through 65535.");
const fixture = browserFixture();
const baseURL = `http://127.0.0.1:${port}`;
const keywordOnly = process.env.ATLAS_BROWSER_FIXTURE_KEYWORD === "1";
const keywordSpec = "keyword-search.spec.ts";

export default defineConfig({
  testDir: "./e2e/devices",
  testIgnore: keywordOnly ? undefined : keywordSpec,
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 45_000,
  expect: { timeout: 10_000 },
  snapshotPathTemplate: "{testDir}/visual/{platform}/{projectName}/{arg}{ext}",
  updateSnapshots: "none",
  outputDir: "../../scratch/responsive-validation/browser/results",
  reporter: [
    ["list"],
    [
      "html",
      {
        outputFolder: "../../scratch/responsive-validation/browser/report",
        open: "never",
      },
    ],
  ],
  use: { baseURL, trace: "retain-on-failure", screenshot: "only-on-failure" },
  projects: [
    {
      name: "desktop-chromium",
      testMatch: keywordOnly ? keywordSpec : undefined,
      use: {
        ...devices["Desktop Chrome"],
        channel: "chromium",
        viewport: { width: 1440, height: 900 },
        colorScheme: "light",
      },
    },
    {
      name: "phone-chromium",
      grepInvert: /@engine/,
      testMatch: keywordOnly ? keywordSpec : touchSpecs,
      use: { ...devices["Pixel 7"], channel: "chromium", colorScheme: "dark" },
    },
    {
      name: "phone-webkit",
      testMatch: keywordOnly ? keywordSpec : touchSpecs,
      use: { ...devices["iPhone 13"], colorScheme: "light", reducedMotion: "reduce" },
    },
    {
      name: "tablet-webkit",
      grepInvert: /@engine/,
      testMatch: keywordOnly ? keywordSpec : touchSpecs,
      use: { ...devices["iPad Mini"], colorScheme: "dark" },
    },
  ],
  webServer: {
    command: `${fixture.command} 127.0.0.1:${port}`,
    cwd: resolve(import.meta.dirname, "../.."),
    url: `${baseURL}/api/readiness`,
    reuseExistingServer: false,
    timeout: 600_000,
    gracefulShutdown: { signal: "SIGINT", timeout: 10_000 },
  },
});
