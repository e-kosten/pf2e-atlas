import { test } from "node:test";
import assert from "node:assert/strict";
import { requireSelectedJobs } from "./ci-results.mjs";

const names = [
  "web-bundle",
  "browser-fixture",
  "browsers",
  "rust",
  "windows-artifact",
  "atlas-dev",
  "web-ui",
  "source-contracts",
  "release-tools",
  "research",
  "skill",
];
const flags = ["rust", "web", "source", "release", "research", "skill"];
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

test("frontend-only and Rust-only changes retain real-app browser gates", () => {
  for (const surface of ["web", "rust"]) {
    const jobs = selected();
    jobs.changes.outputs = Object.fromEntries(
      flags.map((key) => [key, String(key === surface)]),
    );
    for (const name of [
      "source-contracts",
      "release-tools",
      "research",
      "skill",
    ])
      jobs[name].result = "skipped";
    for (const name of surface === "web"
      ? ["windows-artifact", "atlas-dev"]
      : ["web-ui"])
      jobs[name].result = "skipped";
    requireSelectedJobs(jobs);
    jobs.browsers.result = "skipped";
    assert.throws(() => requireSelectedJobs(jobs), /browsers/);
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
