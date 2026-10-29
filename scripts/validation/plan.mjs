import { toolingChecks } from "./tooling.mjs";

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
const rustPrerequisites = [
  cargo("nextest availability", "nextest", "--version"),
  cargo("nextest configuration", "nextest", "show-config", "version"),
];
const rust = [
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

export function validationPlan(
  selected,
  { full = true, parallel = true } = {},
) {
  const checks = [];
  if (full && (!selected || selected.has("rust")))
    checks.push(...rustPrerequisites);
  const concurrent =
    full &&
    parallel &&
    selected?.has("web") &&
    (selected.has("rust") || selected.has("dev"));
  if (full && selected?.has("tooling")) checks.push(...toolingChecks());
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
            ? npm(
                concurrent ? "web checks and bundle" : "web UI verify",
                concurrent ? "verify:bundle" : "verify",
              )
            : npm("browser checks and bundle", "verify:bundle"),
      );
    }
    if (full && touched("rust") && !touched("web"))
      checks.push(npm("web bundle for Rust", "build"));
    if (concurrent)
      checks.push({
        parallel: {
          frontend: [npm("web UI tests", "test")],
          rust: touched("rust") ? rust : developer,
        },
      });
    if (touched("rust") && !concurrent) checks.push(...(full ? rust : [fmt]));
    if (touched("dev") && !touched("rust") && !concurrent)
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
          shell("release smoke tests", "scripts/release/test-release-tools.sh"),
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
  return checks;
}

export function flatChecks(plan) {
  return plan.flatMap((task) =>
    task.parallel ? [...task.parallel.frontend, ...task.parallel.rust] : [task],
  );
}
