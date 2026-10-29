import { readiness, requireReadiness } from "../development/preflight.mjs";
import { runChecks } from "./run-check.mjs";
import { validationPlan, flatChecks } from "./plan.mjs";
import { inspectEvidence } from "./status.mjs";

const [mode, ...args] = process.argv.slice(2);
try {
  if (!["--rust", "--selected"].includes(mode))
    throw new Error("A Rust gate or selected surfaces are required");
  const selected =
    mode === "--selected"
      ? new Set((args.shift() ?? "").split(" ").filter(Boolean))
      : undefined;
  const surfaces = new Set([
    "rust",
    "web",
    "browser",
    "web-format",
    "dev",
    "source",
    "release",
    "research",
    "skill",
    "tooling",
  ]);
  if (selected && [...selected].some((name) => !surfaces.has(name)))
    throw new Error("Unknown validation surface");
  const allowed = [
    "--verbose",
    "-v",
    "--help",
    "-h",
    "--plan",
    "--status",
    "--serial",
    ...(selected ? ["--full"] : []),
  ];
  for (const argument of args)
    if (!allowed.includes(argument))
      throw new Error(`Unknown verify option: ${argument}`);
  if (args.includes("--help") || args.includes("-h"))
    console.log(
      "Usage: scripts/verify.sh [--verbose] [--plan | --status] [--serial]\nRun or inspect the full Rust fmt, clippy, test, and build gate.",
    );
  else {
    const full = mode === "--rust" || args.includes("--full");
    const plan = validationPlan(selected, {
      full,
      parallel: !args.includes("--serial"),
    });
    if (args.includes("--plan") || args.includes("--status")) {
      for (const task of plan) {
        if (task.parallel)
          console.log("Parallel lanes: frontend tests and Rust checks");
        for (const check of flatChecks([task]))
          console.log(
            `${check[0]}: ${check
              .slice(1)
              .map((value) => JSON.stringify(value))
              .join(" ")}`,
          );
      }
      console.log(
        "Prerequisites: inspection probes versions/offline metadata; it never installs or runs checks.",
      );
      const commands = flatChecks(plan).map((task) => task.slice(1));
      for (const prefix of new Set(
        commands
          .filter((task) => task[0] === "npm" && task.includes("--prefix"))
          .map((task) => task[task.indexOf("--prefix") + 1]),
      ))
        console.log(
          `Dependencies: npm --prefix ${prefix} ci (explicit preparation when missing/stale).`,
        );
      if (full) {
        const report = await readiness(process.cwd(), { commands });
        for (const issue of report.issues) console.log(`Not ready: ${issue}`);
        if (!report.issues.length)
          console.log("Selected operation prerequisites are available.");
      }
      if (selected?.has("browser"))
        console.log(
          "Browser obligation: select relevant local specs/projects with just validate-browser; CI owns the full applicable matrix.",
        );
      if (args.includes("--status")) {
        for (const result of inspectEvidence(plan))
          console.log(
            `${result.state}: ${result.label}${result.receipt ? `; ${result.receipt}` : `; ${result.reasons.join(", ")}`}`,
          );
        console.log(
          "Evidence inspection does not skip checks or prove human acceptance/independent review.",
        );
      }
    } else {
      if (full)
        await requireReadiness(process.cwd(), {
          commands: flatChecks(plan).map((task) => task.slice(1)),
        });
      process.exitCode = await runChecks(plan, {
        kind: full ? "final" : "iteration",
        verbose: args.includes("--verbose") || args.includes("-v"),
      });
    }
  }
} catch (error) {
  console.error(error.message);
  process.exitCode = 2;
}
