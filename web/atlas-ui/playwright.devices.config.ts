import { defineConfig, devices } from "@playwright/test";
import { resolve } from "node:path";
import { browserFixture } from "../../scripts/browser-fixture.mjs";

const touchSpecs = [
  "journeys.spec.ts",
  "modal-layout.spec.ts",
  "list-metadata.spec.ts",
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

export default defineConfig({
  testDir: "./e2e/devices",
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
      use: {
        ...devices["Desktop Chrome"],
        viewport: { width: 1440, height: 900 },
        colorScheme: "light",
      },
    },
    {
      name: "phone-chromium",
      grepInvert: /@engine/,
      testMatch: touchSpecs,
      use: { ...devices["Pixel 7"], colorScheme: "dark" },
    },
    {
      name: "phone-webkit",
      testMatch: touchSpecs,
      use: { ...devices["iPhone 13"], colorScheme: "light", reducedMotion: "reduce" },
    },
    {
      name: "tablet-webkit",
      grepInvert: /@engine/,
      testMatch: touchSpecs,
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
