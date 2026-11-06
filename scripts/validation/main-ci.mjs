import { execFileSync } from "node:child_process";
import { appendFileSync, realpathSync } from "node:fs";
import { fileURLToPath } from "node:url";

const workflow = ".github/workflows/ci.yml";
const pending = new Set([
  "queued",
  "in_progress",
  "waiting",
  "pending",
  "requested",
]);

export async function mainVerification({
  sha,
  force = false,
  listRuns,
  getJobs,
  sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms)),
  now = Date.now,
  timeoutMs = 30 * 60_000,
}) {
  if (!/^[0-9a-f]{40}$/.test(sha))
    throw new Error("Expected an exact source SHA");
  if (force) return { verify: true, reason: "Fresh verification requested" };
  const deadline = now() + timeoutMs;
  while (true) {
    const runs = await listRuns(sha);
    if (!Array.isArray(runs)) throw new Error("Invalid main CI run response");
    const run = runs
      .filter(
        (item) =>
          item.head_sha === sha &&
          item.head_branch === "main" &&
          item.event === "push" &&
          item.path === workflow &&
          Number.isSafeInteger(item.id) &&
          item.id > 0 &&
          Number.isSafeInteger(item.run_number) &&
          item.run_number > 0 &&
          Number.isSafeInteger(item.run_attempt) &&
          item.run_attempt > 0,
      )
      .sort(
        (a, b) => b.run_number - a.run_number || b.run_attempt - a.run_attempt,
      )[0];
    if (!run) return { verify: true, reason: "No matching main-push CI run" };
    if (run.status === "completed") {
      if (run.conclusion !== "success")
        return { verify: true, reason: `Main CI concluded ${run.conclusion}` };
      const jobs = await getJobs(run.id);
      if (!Array.isArray(jobs)) throw new Error("Invalid main CI job response");
      const complete = ["validation complete", "advisories"].every((name) => {
        const matching = jobs.filter((job) => job.name === name);
        return (
          matching.length === 1 &&
          matching[0].status === "completed" &&
          matching[0].conclusion === "success"
        );
      });
      return complete
        ? { verify: false, reason: "Reusing successful main CI", runId: run.id }
        : {
            verify: true,
            reason: "Main CI lacks successful aggregate/advisory evidence",
          };
    }
    if (!pending.has(run.status) || now() >= deadline)
      return {
        verify: true,
        reason: "Main CI is incomplete; run fresh verification",
      };
    await sleep(Math.min(15_000, deadline - now()));
  }
}

if (
  process.argv[1] &&
  realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  const [sha, ...args] = process.argv.slice(2);
  if (args.length > 1 || args.some((arg) => arg !== "--force"))
    throw new Error("Usage: main-ci.mjs SHA [--force]");
  const api = (endpoint) =>
    JSON.parse(
      execFileSync("gh", ["api", "--paginate", "--slurp", endpoint], {
        encoding: "utf8",
        maxBuffer: 16 * 1024 * 1024,
      }),
    ).flat();
  const result = await mainVerification({
    sha,
    force: args.includes("--force"),
    listRuns: (source) =>
      api(
        `repos/{owner}/{repo}/actions/workflows/ci.yml/runs?event=push&branch=main&head_sha=${source}&per_page=100`,
      ).flatMap((page) => page.workflow_runs),
    getJobs: (id) =>
      api(
        `repos/{owner}/{repo}/actions/runs/${id}/jobs?filter=latest&per_page=100`,
      ).flatMap((page) => page.jobs),
  });
  process.stdout.write(`verify=${result.verify}\n`);
  const message = `${result.reason}${result.runId ? ` (run ${result.runId}, commit ${sha})` : ` (commit ${sha})`}\n`;
  process.stderr.write(message);
  if (process.env.GITHUB_STEP_SUMMARY)
    appendFileSync(process.env.GITHUB_STEP_SUMMARY, message);
}
