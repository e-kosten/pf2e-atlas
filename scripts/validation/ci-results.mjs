import assert from "node:assert/strict";
import { realpathSync } from "node:fs";
import { fileURLToPath } from "node:url";

function selected(flags, ...names) {
  return names.some((name) => flags[name] === "true");
}

export function requireSelectedSteps(group, flags, steps) {
  const expected = {
    "repository-tooling": {
      "embedding-receipt": selected(flags, "audit", "tooling"),
      audit: selected(flags, "audit"),
      skill: selected(flags, "skill"),
      research: selected(flags, "research"),
      tooling: selected(flags, "tooling"),
      corpus: selected(flags, "corpus"),
      "source-tests": selected(flags, "source"),
      "source-generate": selected(flags, "source"),
      release: selected(flags, "release"),
    },
    windows: {
      storage: selected(flags, "windows"),
      publication: selected(flags, "windows"),
      cancellation: selected(flags, "tooling"),
      release: selected(flags, "release"),
    },
  };
  assert(Object.hasOwn(expected, group), group);
  for (const [step, required] of Object.entries(expected[group])) {
    assert.equal(steps[step]?.outcome, required ? "success" : "skipped", step);
  }
}

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
    "tooling",
    "embedding",
    "workspace",
    "windows",
    "audit",
    "corpus",
    "desktop",
    "windows-desktop",
  ]) {
    assert(["true", "false"].includes(flags[key]), key);
  }
  const expected = {
    "web-bundle":
      flags.rust === "true" || flags.web === "true" || flags.browser === "true",
    "browser-fixture": flags.browser === "true",
    browsers: flags.browser === "true",
    rust: flags.rust === "true",
    windows: selected(flags, "windows", "tooling", "release"),
    "atlas-dev": flags.dev === "true",
    "web-ui": flags.web === "true",
    "repository-tooling": selected(
      flags,
      "audit",
      "skill",
      "research",
      "source",
      "release",
      "tooling",
      "corpus",
    ),
    "release-tools": flags.release === "true",
    desktop: flags.desktop === "true",
    "windows-desktop": flags["windows-desktop"] === "true",
    embedding: flags.embedding === "true",
    workspace: flags.workspace === "true",
  };
  for (const [job, required] of Object.entries(expected)) {
    assert.equal(jobs[job].result, required ? "success" : "skipped", job);
  }
}

if (
  process.argv[1] &&
  realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  const group = process.argv[2];
  if (group)
    requireSelectedSteps(
      group,
      JSON.parse(process.env.SELECTION_JSON),
      JSON.parse(process.env.STEPS_JSON),
    );
  else requireSelectedJobs(JSON.parse(process.env.NEEDS_JSON));
}
