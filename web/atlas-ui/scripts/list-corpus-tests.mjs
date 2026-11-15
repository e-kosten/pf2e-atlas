import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readdirSync } from "node:fs";
import process from "node:process";
import { URL, fileURLToPath } from "node:url";

const root = new URL("../", import.meta.url);
const cli = fileURLToPath(new URL("node_modules/@playwright/test/cli.js", root));
const listing = execFileSync(
  process.execPath,
  [cli, "test", "--config", "playwright.config.ts", "--list", "--reporter=json"],
  { cwd: root, encoding: "utf8" },
);
const report = JSON.parse(listing);
const expected = readdirSync(new URL("e2e/", root), { recursive: true })
  .map((file) => file.replaceAll("\\", "/"))
  .filter((file) => file.endsWith(".spec.ts") && !file.startsWith("devices/"))
  .sort();

assert.equal(report.config.rootDir, fileURLToPath(new URL("e2e", root)));
assert(expected.length > 0, "No full-corpus specs found");
const discovered = [...new Set(report.suites.map((suite) => suite.file))].sort();
assert.deepEqual(discovered, expected, "Full-corpus Playwright spec discovery");
const count = report.stats.expected + report.stats.skipped;
assert(count > 0, "Playwright did not discover full-corpus tests");
process.stdout.write(
  `Discovered ${count} tests across ${discovered.length} full-corpus specs.\n`,
);
