import test from "node:test";
import assert from "node:assert/strict";
import { mainVerification } from "./main-ci.mjs";

const sha = "a".repeat(40);
const run = (values = {}) => ({
  id: 42,
  run_number: 10,
  run_attempt: 1,
  path: ".github/workflows/ci.yml",
  head_sha: sha,
  head_branch: "main",
  event: "push",
  status: "completed",
  conclusion: "success",
  ...values,
});
const jobs = () =>
  ["validation complete", "advisories"].map((name) => ({
    name,
    status: "completed",
    conclusion: "success",
  }));
const check = (runs, options = {}) =>
  mainVerification({
    sha,
    listRuns: async () => runs,
    getJobs: async () => jobs(),
    ...options,
  });

test("only successful exact-source main-push CI and its completed gates qualify", async () => {
  assert.equal((await check([run()])).verify, false);
  for (const mismatch of [
    { head_sha: "b".repeat(40) },
    { head_branch: "feature" },
    { event: "pull_request" },
    { event: "workflow_dispatch" },
    { path: ".github/workflows/other.yml" },
    { id: "42" },
  ])
    assert.equal((await check([run(mismatch)])).verify, true);
  for (const conclusion of ["failure", "cancelled", "skipped", "neutral", null])
    assert.equal((await check([run({ conclusion })])).verify, true);
  for (const name of ["validation complete", "advisories"])
    for (const change of [
      { conclusion: "failure" },
      { conclusion: "skipped" },
      { status: "queued" },
    ])
      assert.equal(
        (
          await check([run()], {
            getJobs: async () =>
              jobs().map((j) => (j.name === name ? { ...j, ...change } : j)),
          })
        ).verify,
        true,
      );
  assert.equal(
    (await check([run()], { getJobs: async () => [] })).verify,
    true,
  );
  assert.equal(
    (await check([], { getJobs: () => assert.fail("No run may be queried") }))
      .verify,
    true,
  );
});

test("pending latest run is awaited; superseded success and timeout do not qualify", async () => {
  let calls = 0,
    time = 0;
  const result = await check([], {
    listRuns: async () => [run({ status: calls++ ? "completed" : "queued" })],
    now: () => time,
    sleep: async (ms) => {
      time += ms;
    },
  });
  assert.equal(result.verify, false);
  assert.equal(calls, 2);
  calls = 0;
  time = 0;
  const timeout = await check([], {
    listRuns: async () => [
      run({ run_number: 9 }),
      run({ status: "in_progress" }),
    ],
    now: () => time,
    timeoutMs: 10,
    sleep: async (ms) => {
      time += ms;
      calls++;
    },
  });
  assert.equal(timeout.verify, true);
  assert.equal(calls, 1);
  assert.equal(
    (
      await check([
        run(),
        run({ id: 43, run_attempt: 2, conclusion: "cancelled" }),
      ])
    ).verify,
    true,
  );
});

test("manual force bypasses evidence; API errors cannot become successful reuse", async () => {
  assert.equal(
    (
      await check([], {
        force: true,
        listRuns: () => assert.fail("Force must not reuse CI"),
      })
    ).verify,
    true,
  );
  await assert.rejects(
    check([], {
      listRuns: async () => {
        throw new Error("API unavailable");
      },
    }),
    /API unavailable/,
  );
  await assert.rejects(check([], { sha: "main" }), /exact source SHA/);
  await assert.rejects(
    check([], { listRuns: async () => ({}) }),
    /Invalid main CI/,
  );
});
