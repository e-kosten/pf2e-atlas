import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  mkdtempSync,
  mkdirSync,
  writeFileSync,
  readFileSync,
  readdirSync,
  rmSync,
  copyFileSync,
  chmodSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
export const helper = fileURLToPath(new URL("./run-check.sh", import.meta.url));
export const shell = `exec sh "$ATLAS_CHECK_HELPER" $([ "\${ATLAS_TEST_VERBOSE:-0}" = 0 ] || printf -- --verbose) "$@"`;
export function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), "atlas-check-test-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const cwd = join(root, "checkout");
  mkdirSync(cwd);
  const git = (...args) => {
    const result = spawnSync("git", args, { cwd, encoding: "utf8" });
    assert.equal(result.status, 0, result.stderr);
    return result.stdout.trim();
  };
  mkdirSync(join(cwd, "scripts", "validation"), { recursive: true });
  for (const name of [
    "run-check.sh",
    "launch.sh",
    "run-check.mjs",
    "run-process.mjs",
    "verify.mjs",
    "inputs.mjs",
    "plan.mjs",
    "tooling.mjs",
    "status.mjs",
  ]) {
    copyFileSync(
      fileURLToPath(new URL(`./${name}`, import.meta.url)),
      join(cwd, "scripts", "validation", name),
    );
  }
  mkdirSync(join(cwd, "scripts", "development"), { recursive: true });
  copyFileSync(
    new URL("../development/preflight.mjs", import.meta.url),
    join(cwd, "scripts/development/preflight.mjs"),
  );
  mkdirSync(join(cwd, "web/atlas-ui/node_modules"), { recursive: true });
  writeFileSync(
    join(cwd, "web/atlas-ui/node_modules/.package-lock.json"),
    "{}",
  );
  copyFileSync(
    new URL("../../web/atlas-ui/package.json", import.meta.url),
    join(cwd, "web/atlas-ui/package.json"),
  );
  copyFileSync(
    new URL("../../rust-toolchain.toml", import.meta.url),
    join(cwd, "rust-toolchain.toml"),
  );
  mkdirSync(join(cwd, "scripts", "git-hooks"), { recursive: true });
  copyFileSync(
    fileURLToPath(new URL("../verify.sh", import.meta.url)),
    join(cwd, "scripts", "verify.sh"),
  );
  chmodSync(join(cwd, "scripts", "verify.sh"), 0o755);
  copyFileSync(
    fileURLToPath(new URL("../verify-changed.sh", import.meta.url)),
    join(cwd, "scripts", "verify-changed.sh"),
  );
  copyFileSync(
    fileURLToPath(new URL("../git-hooks/common.sh", import.meta.url)),
    join(cwd, "scripts", "git-hooks", "common.sh"),
  );
  writeFileSync(
    join(cwd, ".gitignore"),
    ".atlas/\nnode_modules/\nweb/atlas-ui/dist/\ntarget/\n",
  );
  git("init", "--quiet");
  git("config", "user.name", "Validation fixture");
  git("config", "user.email", "validation@example.invalid");
  git("config", "commit.gpgsign", "false");
  git("config", "core.hooksPath", "/dev/null");
  writeFileSync(join(cwd, "code.rs"), "pub fn value() {}\n");
  writeFileSync(join(cwd, "README.md"), "Fixture docs\n");
  git("add", ".");
  git("commit", "--quiet", "-m", "test: fixture");
  const env = {
    ...process.env,
    ATLAS_CHECK_HELPER: join(cwd, "scripts", "validation", "run-check.sh"),
    ATLAS_CHECK_MODULE: new URL("./run-check.mjs", import.meta.url).href,
    ATLAS_VALIDATION_OUTPUT_ROOT: join(root, "evidence"),
    ATLAS_VALIDATION_HEARTBEAT_SECONDS: "1",
  };
  delete env.GITHUB_STEP_SUMMARY;
  const run = (command, extra = {}) =>
    spawnSync(
      "sh",
      ["-c", shell, "fixture", "fixture check", "sh", "-c", command],
      { cwd, env: { ...env, ...extra }, encoding: "utf8", timeout: 8000 },
    );
  const records = () =>
    readdirSync(env.ATLAS_VALIDATION_OUTPUT_ROOT).flatMap((run) => {
      const dir = join(env.ATLAS_VALIDATION_OUTPUT_ROOT, run);
      return readdirSync(dir)
        .filter((name) => name.startsWith("check."))
        .map((name) => {
          const check = join(dir, name);
          const result = Object.fromEntries(
            readFileSync(join(check, "result.tsv"), "utf8")
              .trim()
              .split("\n")
              .map((line) => line.split("\t")),
          );
          return { dir, check, ...result };
        });
    });
  return { cwd, env, run, records };
}
