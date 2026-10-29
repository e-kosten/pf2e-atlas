import { execFileSync } from "node:child_process";
import {
  inputIdentity,
  commandIdentity,
  runnerIdentity,
  environmentIdentity,
  toolchainIdentity,
  environmentState,
  artifactIdentity,
} from "./inputs.mjs";
import {
  appendFileSync,
  existsSync,
  closeSync,
  mkdirSync,
  mkdtempSync,
  openSync,
  readFileSync,
  realpathSync,
  writeFileSync,
  writeSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { runProcess } from "./run-process.mjs";

const git = (...args) => execFileSync("git", args, { encoding: "utf8" }).trim();
const tsv = (values) =>
  Object.entries(values)
    .map(([key, value]) => `${key}\t${value}\n`)
    .join("");

export async function runChecks(
  checks,
  { kind = "final", verbose = false } = {},
) {
  if (checks.length === 0) return 0;
  const checkout = git("rev-parse", "--show-toplevel");
  process.chdir(checkout);
  const root =
    process.env.ATLAS_VALIDATION_OUTPUT_ROOT ||
    join(checkout, ".atlas/validation");
  mkdirSync(root, { recursive: true });
  const dir = realpathSync(mkdtempSync(join(root, "run.")));
  const candidateInputs =
    kind === "final" ? inputIdentity("code") : "per-check";
  writeFileSync(
    join(dir, "candidate.tsv"),
    tsv({
      kind,
      checkout,
      start_commit: git("rev-parse", "HEAD"),
      start_inputs: candidateInputs,
      started_utc: new Date().toISOString(),
    }),
  );
  console.error(`Validation evidence: ${dir}`);
  const controller = new AbortController();
  const signals = new Map([
    ["SIGHUP", 129],
    ["SIGINT", 130],
    ["SIGTERM", 143],
  ]);
  const handlers = [...signals].map(([name, status]) => {
    const handler = () => controller.abort(status);
    process.on(name, handler);
    return [name, handler];
  });
  const cancellation = process.env.ATLAS_VALIDATION_CANCEL_FILE;
  const poll =
    cancellation &&
    setInterval(() => {
      try {
        controller.abort(
          Number(readFileSync(cancellation, "utf8").trim()) || 125,
        );
      } catch (error) {
        if (error.code !== "ENOENT") controller.abort(125);
      }
    }, 50);
  async function sequence(tasks, signal = controller.signal) {
    for (const task of tasks) {
      if (task.parallel) {
        const pair = new AbortController();
        const cancel = () => pair.abort(signal.reason);
        signal.addEventListener("abort", cancel, { once: true });
        if (signal.aborted) cancel();
        let failure;
        let error;
        const lane = async (tasks) => {
          try {
            const status = await sequence(tasks, pair.signal);
            if (status && failure === undefined && !pair.signal.aborted) {
              failure = status;
              pair.abort(143);
            }
            return status;
          } catch (caught) {
            error ??= caught;
            pair.abort(143);
            throw caught;
          }
        };
        await Promise.allSettled([
          lane(task.parallel.frontend),
          lane(task.parallel.rust),
        ]);
        signal.removeEventListener("abort", cancel);
        if (signal.aborted) return signal.reason;
        if (failure) {
          if (error)
            console.error(`Peer cleanup/evidence error: ${error.message}`);
          return failure;
        }
        if (error) throw error;
        continue;
      }
      const [label, command, ...args] = task;
      if (signal.aborted) return signal.reason;
      const check = mkdtempSync(join(dir, "check."));
      const log = join(check, "output.log");
      const started = Date.now();
      const scope =
        kind === "iteration"
          ? { "web UI format": "web-format", "cargo fmt": "rust-format" }[
              label
            ] || "code"
          : "code";
      const startCommit = git("rev-parse", "HEAD");
      const startInputs = inputIdentity(scope);
      writeFileSync(
        join(check, "command.args"),
        [command, ...args].join("\n") + "\n",
      );
      writeFileSync(join(check, "phase.pid"), `${process.pid}\n`);
      const toolchain = toolchainIdentity(command);
      const startArtifacts = artifactIdentity(command, args);
      const identity = {
        command_sha256: commandIdentity([command, ...args]),
        runner_sha256: runnerIdentity(),
        environment_sha256: environmentIdentity(),
        toolchain_sha256: toolchain.identity,
      };
      writeFileSync(join(check, "toolchain.txt"), toolchain.text);
      const fd = openSync(log, "w");
      const seconds = Number(
        process.env.ATLAS_VALIDATION_HEARTBEAT_SECONDS || 20,
      );
      const heartbeat = setInterval(
        () =>
          console.error(
            `Still running ${label} (${Math.floor((Date.now() - started) / 1000)}s); log: ${log}`,
          ),
        (Number.isInteger(seconds) && seconds >= 1 && seconds <= 30
          ? seconds
          : 20) * 1000,
      );
      let status;
      let state;
      try {
        if (kind === "final" && candidateInputs !== startInputs) {
          writeSync(
            fd,
            "Candidate inputs changed between checks; start a fresh validation run.\n",
          );
          status = 78;
          state = "stale";
        } else {
          console.error(`Running ${label}...`);
          status = await runProcess(command, args, {
            signal,
            onOutput: (chunk) => {
              writeSync(fd, chunk);
              if (verbose) process.stderr.write(chunk);
            },
          });
        }
      } finally {
        clearInterval(heartbeat);
        closeSync(fd);
      }
      const endCommit = git("rev-parse", "HEAD");
      let endInputs;
      try {
        endInputs = inputIdentity(scope);
      } catch {
        endInputs = "unavailable";
      }
      let endArtifacts;
      try {
        endArtifacts = artifactIdentity(command, args);
      } catch {
        endArtifacts = "unavailable";
      }
      const runEndInputs = kind === "final" ? endInputs : "per-check";
      if (signal.aborted) {
        status = signal.reason;
        state = "interrupted";
      } else if (!state) {
        state = status ? "failed" : "passed";
        if (
          !status &&
          (startArtifacts !== endArtifacts ||
            startInputs !== endInputs ||
            (kind === "final" && candidateInputs !== runEndInputs))
        ) {
          state = "stale";
          status = 78;
        }
      }
      const elapsed = Math.floor((Date.now() - started) / 1000);
      const result = join(check, "result.tsv");
      writeFileSync(
        result,
        tsv({
          kind,
          label,
          ...identity,
          environment: JSON.stringify(environmentState()),
          artifacts_sha256: startArtifacts,
          end_artifacts_sha256: endArtifacts,
          status: state,
          exit: status,
          duration_seconds: elapsed,
          start_commit: startCommit,
          end_commit: endCommit,
          input_scope: scope,
          start_inputs: startInputs,
          end_inputs: endInputs,
          candidate_start_inputs: candidateInputs,
          candidate_end_inputs: runEndInputs,
          log,
          command: join(check, "command.args"),
        }),
      );
      appendFileSync(
        join(dir, "checks.tsv"),
        `${state}\t${label}\t${elapsed}\t${result}\n`,
      );
      console.error(
        status
          ? `Failed ${label} (${elapsed}s, ${state}); evidence: ${result}`
          : `Passed ${label} (${elapsed}s); log: ${log}`,
      );
      if (status && !verbose) process.stderr.write(readFileSync(log));
      if (process.env.GITHUB_STEP_SUMMARY)
        appendFileSync(
          process.env.GITHUB_STEP_SUMMARY,
          `- ${label}: ${elapsed}s (exit ${status}), ${state}; log: ${log}\n`,
        );
      if (status) return status;
    }
    return 0;
  }
  try {
    return await sequence(checks);
  } finally {
    if (poll) clearInterval(poll);
    for (const [name, handler] of handlers) process.off(name, handler);
  }
}

if (
  process.argv[1] &&
  existsSync(process.argv[1]) &&
  import.meta.url === pathToFileURL(realpathSync(process.argv[1])).href
) {
  const args = process.argv.slice(2);
  const verbose = args[0] === "--verbose";
  if (verbose) args.shift();
  if (args.length < 2) {
    console.error(
      "Usage: node scripts/validation/run-check.mjs [--verbose] LABEL COMMAND [ARG...]",
    );
    process.exitCode = 2;
  } else {
    try {
      process.exitCode = await runChecks([args], { verbose });
    } catch (error) {
      console.error(error.message);
      process.exitCode = 125;
    }
  }
}
