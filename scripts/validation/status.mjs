import {
  existsSync,
  readFileSync,
  readdirSync,
  realpathSync,
  statSync,
} from "node:fs";
import { resolve } from "node:path";
import {
  inputIdentity,
  commandIdentity,
  runnerIdentity,
  environmentIdentity,
  toolchainIdentity,
  environmentState,
  artifactIdentity,
} from "./inputs.mjs";
import { flatChecks } from "./plan.mjs";

const record = (path) =>
  Object.fromEntries(
    readFileSync(path, "utf8")
      .trim()
      .split("\n")
      .map((line) => {
        const at = line.indexOf("\t");
        return [line.slice(0, at), line.slice(at + 1)];
      }),
  );
export function inspectEvidence(plan, root = process.cwd()) {
  const inputs = inputIdentity("code");
  const runner = runnerIdentity();
  const environment = environmentIdentity();
  const directory =
    process.env.ATLAS_VALIDATION_OUTPUT_ROOT ||
    resolve(root, ".atlas/validation");
  const records = [];
  if (existsSync(directory))
    for (const run of readdirSync(directory).filter((name) =>
      name.startsWith("run."),
    )) {
      const parent = resolve(directory, run);
      try {
        const candidate = record(resolve(parent, "candidate.tsv"));
        for (const name of readdirSync(parent).filter((name) =>
          name.startsWith("check."),
        )) {
          const path = resolve(parent, name, "result.tsv");
          try {
            records.push({
              ...record(path),
              modified: statSync(path).mtimeMs,
              checkout: candidate.checkout,
              path,
            });
          } catch {}
        }
      } catch {}
    }
  const tools = new Map();
  return flatChecks(plan).map(([label, command, ...args]) => {
    if (!tools.has(command))
      tools.set(command, toolchainIdentity(command).identity);
    const expected = {
      runner_sha256: runner,
      environment_sha256: environment,
      toolchain_sha256: tools.get(command),
      artifacts_sha256: artifactIdentity(command, args),
      end_artifacts_sha256: artifactIdentity(command, args),
    };
    const candidates = records.filter(
      (r) => r.command_sha256 === commandIdentity([command, ...args]),
    );
    const failures = (r) => {
      const reasons = [];
      if (r.checkout !== realpathSync(root)) reasons.push("different checkout");
      if (r.kind !== "final" || r.status !== "passed" || r.exit !== "0")
        reasons.push("no successful final result");
      if (
        [
          r.start_inputs,
          r.end_inputs,
          r.candidate_start_inputs,
          r.candidate_end_inputs,
        ].some((value) => value !== inputs)
      )
        reasons.push("source inputs changed");
      for (const [key, value] of Object.entries(expected))
        if (value === "unavailable" || r[key] !== value)
          reasons.push(key.replace("_sha256", "") + " changed or unrecorded");
      if (r.environment_sha256 !== environment) {
        try {
          const before = JSON.parse(r.environment);
          const now = environmentState();
          const names = [
            ...new Set([...Object.keys(before), ...Object.keys(now)]),
          ].filter((name) => before[name] !== now[name]);
          if (names.length)
            reasons.push(`environment names: ${names.join(", ")}`);
        } catch {}
      }
      return reasons;
    };
    candidates.sort((a, b) => b.modified - a.modified);
    const latest = candidates[0];
    const match = latest && !failures(latest).length ? latest : undefined;
    return {
      label,
      command: [command, ...args],
      state: match
        ? "matching evidence"
        : candidates.length
          ? "invalidated"
          : "missing evidence",
      receipt: match?.path,
      reasons: match
        ? []
        : candidates.length
          ? failures(latest)
          : [
              "no receipt for this command; older receipts without execution identity are unverified",
            ],
    };
  });
}
