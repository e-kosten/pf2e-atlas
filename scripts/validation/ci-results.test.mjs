import { test } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { requireSelectedJobs, requireSelectedSteps } from "./ci-results.mjs";

const names = [
  "web-bundle",
  "browser-fixture",
  "browsers",
  "rust",
  "windows",
  "atlas-dev",
  "web-ui",
  "desktop",
  "windows-desktop",
  "release-tools",
  "embedding",
  "workspace",
  "repository-tooling",
];
const flags = [
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
];
const selected = () =>
  Object.fromEntries([
    [
      "changes",
      {
        result: "success",
        outputs: Object.fromEntries(flags.map((key) => [key, "true"])),
      },
    ],
    ...names.map((name) => [name, { result: "success" }]),
  ]);

test("all selected surfaces must succeed, including every browser prerequisite", () => {
  requireSelectedJobs(selected());
  for (const name of ["changes", ...names]) {
    for (const result of ["failure", "cancelled", "skipped"]) {
      const jobs = selected();
      jobs[name].result = result;
      assert.throws(
        () => requireSelectedJobs(jobs),
        undefined,
        `${name}: ${result}`,
      );
    }
  }
});

test("UI and browser-only changes keep real-app browser gates without full Rust", () => {
  for (const surfaces of [["web", "browser"], ["browser"]]) {
    const jobs = selected();
    jobs.changes.outputs = Object.fromEntries(
      flags.map((key) => [key, String(surfaces.includes(key))]),
    );
    for (const name of [
      "rust",
      "windows",
      "atlas-dev",
      "desktop",
      "windows-desktop",
      "release-tools",
      "embedding",
      "workspace",
      "repository-tooling",
    ])
      jobs[name].result = "skipped";
    if (!surfaces.includes("web")) jobs["web-ui"].result = "skipped";
    requireSelectedJobs(jobs);
    jobs.browsers.result = "skipped";
    assert.throws(() => requireSelectedJobs(jobs), /browsers/);
  }
});

test("standalone developer changes require no frontend or browser job", () => {
  const jobs = selected();
  jobs.changes.outputs = Object.fromEntries(
    flags.map((key) => [key, String(key === "dev")]),
  );
  for (const name of names)
    jobs[name].result = name === "atlas-dev" ? "success" : "skipped";
  requireSelectedJobs(jobs);
  jobs["atlas-dev"].result = "skipped";
  assert.throws(() => requireSelectedJobs(jobs), /atlas-dev/);
});

test("tooling-only changes require native Windows cancellation without application gates", () => {
  const jobs = selected();
  jobs.changes.outputs = Object.fromEntries(
    flags.map((key) => [key, String(key === "tooling")]),
  );
  for (const name of names)
    jobs[name].result = ["windows", "repository-tooling"].includes(name)
      ? "success"
      : "skipped";
  requireSelectedJobs(jobs);
  jobs["windows"].result = "skipped";
  assert.throws(() => requireSelectedJobs(jobs), /windows/);
});

test("core Rust changes require browser prerequisites, but CLI-only Rust does not", () => {
  for (const browser of [true, false]) {
    const jobs = selected();
    jobs.changes.outputs = Object.fromEntries(
      flags.map((key) => [
        key,
        String(key === "rust" || (browser && key === "browser")),
      ]),
    );
    for (const name of [
      "web-ui",
      "atlas-dev",
      "windows",
      "desktop",
      "windows-desktop",
      "release-tools",
      "embedding",
      "workspace",
      "repository-tooling",
    ])
      jobs[name].result = "skipped";
    if (!browser)
      for (const name of ["browser-fixture", "browsers"])
        jobs[name].result = "skipped";
    requireSelectedJobs(jobs);
    jobs.rust.result = "skipped";
    assert.throws(() => requireSelectedJobs(jobs), /rust/);
  }
});

test("docs-only selection permits only skipped gates and rejects malformed flags", () => {
  const jobs = selected();
  jobs.changes.outputs = Object.fromEntries(flags.map((key) => [key, "false"]));
  for (const name of names) jobs[name].result = "skipped";
  requireSelectedJobs(jobs);
  jobs.rust.result = "success";
  assert.throws(() => requireSelectedJobs(jobs), /rust/);
  jobs.rust.result = "skipped";
  delete jobs.changes.outputs.web;
  assert.throws(() => requireSelectedJobs(jobs), /web/);
});

test("the production CLI enforces the selected gate results", () => {
  const jobs = selected();
  const run = () =>
    spawnSync(
      process.execPath,
      [fileURLToPath(new URL("./ci-results.mjs", import.meta.url))],
      {
        env: { ...process.env, NEEDS_JSON: JSON.stringify(jobs) },
        encoding: "utf8",
      },
    );
  assert.equal(run().status, 0);
  jobs.browsers.result = "skipped";
  const failed = run();
  assert.equal(failed.status, 1);
  assert.match(failed.stderr, /browsers/);
});

test("dependency and native selection are independently enforced", () => {
  for (const [surface, job] of [
    ["audit", "repository-tooling"],
    ["corpus", "repository-tooling"],
    ["desktop", "desktop"],
    ["windows-desktop", "windows-desktop"],
    ["windows", "windows"],
  ]) {
    const jobs = selected();
    jobs.changes.outputs = Object.fromEntries(
      flags.map((key) => [key, String(key === surface)]),
    );
    for (const name of names)
      jobs[name].result = name === job ? "success" : "skipped";
    requireSelectedJobs(jobs);
    for (const result of ["failure", "cancelled", "skipped"]) {
      jobs[job].result = result;
      assert.throws(() => requireSelectedJobs(jobs), new RegExp(job));
    }
  }
});

test("grouped checks must run exactly their selected steps", () => {
  for (const [name, ids, selection, chosen] of [
    [
      "repository-tooling",
      [
        "embedding-receipt",
        "audit",
        "skill",
        "research",
        "tooling",
        "corpus",
        "source-tests",
        "source-generate",
        "release",
      ],
      { audit: "true" },
      ["embedding-receipt", "audit"],
    ],
    [
      "windows",
      ["storage", "publication", "cancellation", "release"],
      { windows: "true", release: "true" },
      ["storage", "publication", "release"],
    ],
  ]) {
    const steps = Object.fromEntries(
      ids.map((id) => [
        id,
        { outcome: chosen.includes(id) ? "success" : "skipped" },
      ]),
    );
    requireSelectedSteps(name, selection, steps);
    for (const id of chosen) {
      for (const outcome of ["failure", "skipped", "cancelled", undefined]) {
        assert.throws(
          () =>
            requireSelectedSteps(name, selection, {
              ...steps,
              [id]: outcome ? { outcome } : undefined,
            }),
          new RegExp(id),
        );
      }
    }
  }
});
