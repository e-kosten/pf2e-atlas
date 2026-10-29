import assert from "node:assert/strict";
import { realpathSync } from "node:fs";
import { fileURLToPath } from "node:url";

export function requireSelectedJobs(jobs) {
  assert.equal(
    jobs.changes.result,
    "success",
    "Validation selection must succeed",
  );
  const flags = jobs.changes.outputs;
  for (const key of [
    "rust",
    "web",
    "source",
    "release",
    "research",
    "skill",
    "browser",
    "dev",
  ]) {
    assert(["true", "false"].includes(flags[key]), key);
  }
  const expected = {
    "web-bundle":
      flags.rust === "true" || flags.web === "true" || flags.browser === "true",
    "browser-fixture": flags.browser === "true",
    browsers: flags.browser === "true",
    rust: flags.rust === "true",
    "windows-artifact": flags.rust === "true",
    "atlas-dev": flags.rust === "true" || flags.dev === "true",
    "web-ui": flags.web === "true",
    "source-contracts": flags.source === "true",
    "release-tools": flags.release === "true",
    research: flags.research === "true",
    skill: flags.skill === "true",
  };
  for (const [job, required] of Object.entries(expected)) {
    assert.equal(jobs[job].result, required ? "success" : "skipped", job);
  }
}

if (
  process.argv[1] &&
  realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)
)
  requireSelectedJobs(JSON.parse(process.env.NEEDS_JSON));
