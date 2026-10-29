import { runChecks } from "./run-check.mjs";

const cargo = (label, ...args) => [label, "cargo", ...args];
const npm = (label, script, prefix = "web/atlas-ui") => [
  label,
  "npm",
  "--prefix",
  prefix,
  "run",
  script,
];
const shell = (label, script) => [label, "sh", script];
const fmt = cargo("cargo fmt", "fmt", "--check");
const strictClippy = [
  "-D",
  "warnings",
  ...[
    "unwrap_used",
    "expect_used",
    "panic",
    "unimplemented",
    "todo",
    "unreachable",
  ].flatMap((rule) => ["-D", `clippy::${rule}`]),
];
const clippy = (scope) => [
  cargo(
    "broad clippy",
    "clippy",
    ...scope,
    "--all-targets",
    "--",
    "-D",
    "warnings",
    "-D",
    "clippy::dbg_macro",
  ),
  cargo(
    "strict runtime clippy",
    "clippy",
    ...scope,
    "--lib",
    "--bins",
    "--",
    ...strictClippy,
  ),
];
const rust = [
  cargo("nextest availability", "nextest", "--version"),
  cargo("nextest configuration", "nextest", "show-config", "version"),
  fmt,
  ...clippy(["--workspace"]),
  cargo("workspace tests", "nextest", "run", "--workspace"),
  cargo("workspace doctests", "test", "--workspace", "--doc"),
  cargo("workspace build", "build", "--workspace"),
];
const developer = [
  fmt,
  ...clippy(["-p", "atlas-dev", "-p", "atlas-cli-support"]),
  cargo(
    "developer tests",
    "test",
    "-p",
    "atlas-dev",
    "-p",
    "atlas-cli-support",
    "--locked",
  ),
  cargo("developer build", "build", "-p", "atlas-dev", "--locked"),
];

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
  ]);
  if (selected && [...selected].some((name) => !surfaces.has(name)))
    throw new Error("Unknown validation surface");
  const allowed = [
    "--verbose",
    "-v",
    "--help",
    "-h",
    ...(selected ? ["--full"] : []),
  ];
  for (const argument of args)
    if (!allowed.includes(argument))
      throw new Error(`Unknown verify option: ${argument}`);
  if (args.includes("--help") || args.includes("-h")) {
    console.log(
      "Usage: scripts/verify.sh [--verbose]\nRun the full Rust fmt, clippy, nextest, doctest, and build gate.",
    );
  } else {
    const full = args.includes("--full") || mode === "--rust";
    const checks = [];
    if (selected) {
      const touched = (name) => selected.has(name);
      if (
        touched("web") ||
        touched("web-format") ||
        (full && touched("browser") && !touched("rust"))
      ) {
        checks.push(
          !full
            ? npm("web UI format", "format:check")
            : touched("web")
              ? npm("web UI verify", "verify")
              : npm("browser checks and bundle", "verify:bundle"),
        );
      }
      if (full && touched("rust") && !touched("web"))
        checks.push(npm("web bundle for Rust", "build"));
      if (touched("rust")) checks.push(...(full ? rust : [fmt]));
      if (touched("dev") && !touched("rust"))
        checks.push(...(full ? developer : [fmt]));
      if (full) {
        if (touched("skill"))
          checks.push(
            [
              "bundled skill",
              "node",
              "--test",
              "scripts/validation/skill.test.ts",
            ],
            ["skill package", "node", "scripts/validation/skill.ts"],
          );
        if (touched("source"))
          checks.push(
            npm(
              "source contract fixtures",
              "verify",
              "dev-tools/source-contracts",
            ),
            npm(
              "pinned source generation",
              "verify-generated",
              "dev-tools/source-contracts",
            ),
          );
        if (touched("release"))
          checks.push(
            shell(
              "release tooling and static checks",
              "scripts/release/validate-release-tooling.sh",
            ),
            shell(
              "release smoke tests",
              "scripts/release/test-release-tools.sh",
            ),
            shell(
              "release preparation tests",
              "scripts/release/test-prepare-release.sh",
            ),
          );
        if (touched("research"))
          checks.push(
            shell("research evidence", "scripts/validation/research.sh"),
          );
      }
    } else checks.push(...rust);
    process.exitCode = await runChecks(checks, {
      kind: full ? "final" : "iteration",
      verbose: args.includes("--verbose") || args.includes("-v"),
    });
  }
} catch (error) {
  console.error(error.message);
  process.exitCode = 2;
}
