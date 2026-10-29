import test from "node:test";
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { resolve, dirname } from "node:path";
import {
  nodeSupported,
  readiness,
  digest,
  toolInputIdentity,
  output,
} from "./preflight.mjs";

import { fixture } from "./test-fixtures.mjs";

test("supported Node ranges come from metadata and reject unsupported runtimes", () => {
  for (const version of ["v22.18.0", "v22.24.0", "v24.0.0", "v24.22.1"])
    assert.equal(nodeSupported(version, "^22.18.0 || ^24.0.0"), true);
  for (const version of ["v22.17.9", "v23.0.0", "v26.12.1"])
    assert.equal(nodeSupported(version, "^22.18.0 || ^24.0.0"), false);
});

test("ordinary Rust readiness works without a populated dependency cache; tool provenance still requires its graph", async (t) => {
  const f = fixture(t);
  f.program(
    "cargo",
    `case " $* " in
  *" --no-deps "*) printf '{"target_directory":"%s/target","packages":[]}\\n' "$PWD";;
  *) echo 'offline dependency metadata unavailable' >&2; exit 9;;
esac`,
  );
  assert.deepEqual(
    (await readiness(f.root, { commands: [["cargo", "clippy"]] })).issues,
    [],
  );
  assert(
    (await readiness(f.root, { toolBuild: true })).issues.some((issue) =>
      issue.includes("offline dependency metadata unavailable"),
    ),
  );
});

test("preview reuses a verified bundle without npm dependencies; old seed is advisory", async (t) => {
  const f = fixture(t);
  const report = await readiness(f.root, { preview: true });
  assert.deepEqual(report.issues, []);
  assert(report.notes.some((note) => note.includes("schema 19 lags 22")));
  assert(report.notes.some((note) => note.includes("bundle can be reused")));
  assert.equal(existsSync(resolve(f.root, "web/atlas-ui/node_modules")), false);
});

test("capability and binary/source receipt failures give an explicit early repair", async (t) => {
  const f = fixture(t);
  f.program("atlas-dev", "echo unrecognized-subcommand >&2;exit 2");
  let report = await readiness(f.root, { preview: true });
  assert(report.issues.some((note) => note.includes("dev-tool-build")));
  f.program(
    "atlas-dev",
    `printf '{"version":1,"managed_preview":true,"workspace_schema_version":22}\\n'`,
  );
  f.put(
    "bin/atlas-dev.json",
    JSON.stringify({ version: 1, binary_sha256: "wrong" }),
  );
  report = await readiness(f.root, { preview: true });
  assert(report.issues.some((note) => note.includes("mismatch")));
  f.publish();
  f.put("crates/atlas-dev/src/main.rs", 'fn main() {println!("changed");}\n');
  report = await readiness(f.root, { preview: true });
  assert(report.issues.some((note) => note.includes("source inputs changed")));
});

test("commit-only changes preserve a retained tool receipt", async (t) => {
  const f = fixture(t);
  execFileSync("git", ["commit", "--allow-empty", "-qm", "test: checkpoint"], {
    cwd: f.root,
  });
  assert.deepEqual((await readiness(f.root, { preview: true })).issues, []);
});

test("wrong Rust or an external configured target fails before compilation", async (t) => {
  const f = fixture(t);
  f.program("rustc", "printf 'rustc 1.99.1 (fixture)\\n'");
  assert(
    (await readiness(f.root, { toolBuild: true })).issues.some((note) =>
      note.includes("pinned 1.95.0"),
    ),
  );
  f.program("rustc", "printf 'rustc 1.95.0 (fixture)\\n'");
  f.program(
    "cargo",
    `printf '{"target_directory":"/private/shared-target","packages":[]}\\n'`,
  );
  assert(
    (await readiness(f.root, { toolBuild: true })).issues.some((note) =>
      note.includes("private target/"),
    ),
  );
});

test("prerequisite probes disable Rustup installation and reject alternate compiler selection", async (t) => {
  const f = fixture(t);
  f.program(
    "rustc",
    'test "$RUSTUP_AUTO_INSTALL" = 0 || exit 91; printf "rustc 1.95.0 (fixture)\\n"',
  );
  assert.match(output("rustc", ["--version"], f.root), /1.95.0/);
  for (const key of ["RUSTC", "CARGO_BUILD_RUSTC"]) {
    const old = process.env[key];
    try {
      process.env[key] = "/alternate/rustc";
      assert(
        (await readiness(f.root, { toolBuild: true })).issues.some((issue) =>
          issue.includes(`Unset ${key}`),
        ),
      );
    } finally {
      if (old === undefined) delete process.env[key];
      else process.env[key] = old;
    }
  }
  f.put(".cargo/config.toml", '[build]\nrustc="/alternate/rustc"\n');
  assert(
    (await readiness(f.root, { toolBuild: true })).issues.some((issue) =>
      issue.includes("custom rustc selector"),
    ),
  );
});

test("preview launcher rejects a legacy retained tool before invoking its builder", (t) => {
  const f = fixture(t);
  f.program("atlas-dev", "exit 2");
  f.put(
    "scripts/development/preview.sh",
    readFileSync(new URL("./preview.sh", import.meta.url)),
  );
  f.put(
    "scripts/development/preflight.mjs",
    readFileSync(new URL("./preflight.mjs", import.meta.url)),
  );
  f.put(
    "scripts/development/build-preview.mjs",
    "require('node:fs').writeFileSync('unexpected-build','bad');\n",
  );
  const result = spawnSync(
    "sh",
    ["scripts/development/preview.sh", "start", f.root],
    { cwd: f.root, env: process.env, encoding: "utf8" },
  );
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /dev-tool-build/);
  assert.equal(existsSync(resolve(f.root, "unexpected-build")), false);
});

test("tool freshness includes compiled assets, custom source paths and Cargo configuration", (t) => {
  const f = fixture(t);
  const before = toolInputIdentity(f.root);
  f.put("crates/atlas-dev/migrations/up.sql", "SELECT 1;");
  assert.notEqual(toolInputIdentity(f.root), before);
  const assets = toolInputIdentity(f.root);
  f.put("crates/atlas-dev/cli/custom.rs", "pub fn value() {}\n");
  assert.notEqual(toolInputIdentity(f.root), assets);
  const source = toolInputIdentity(f.root);
  f.put(".cargo/config.toml", '[build]\nrustflags=["--cfg","fixture"]\n');
  assert.notEqual(toolInputIdentity(f.root), source);
});

test("seed metadata problems do not become developer-tool failures", async (t) => {
  const f = fixture(t);
  rmSync(resolve(f.root, ".atlas/dev-baselines/current.json"));
  const report = await readiness(f.root, { preview: true });
  assert.deepEqual(report.issues, []);
  assert(
    report.notes.some((note) => note.includes("seed metadata unavailable")),
  );
});

test("symlinked workspace and dangling/shared Cargo targets fail closed", async (t) => {
  const f = fixture(t);
  rmSync(resolve(f.root, ".atlas/workspace.sqlite"));
  symlinkSync(
    resolve(f.root, "Cargo.lock"),
    resolve(f.root, ".atlas/workspace.sqlite"),
  );
  assert(
    (await readiness(f.root, { preview: true })).issues.some((note) =>
      note.includes("regular file"),
    ),
  );
  symlinkSync(
    resolve(f.root, "../missing-external-target"),
    resolve(f.root, "target"),
  );
  assert((await readiness(f.root, { toolBuild: true })).issues.length > 0);
});

test("Cargo build/target configuration environment participates in retained-tool freshness", (t) => {
  const f = fixture(t);
  const before = toolInputIdentity(f.root);
  const keys = [
    "CARGO_BUILD_RUSTFLAGS",
    "CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS",
  ];
  for (const key of keys) {
    const old = process.env[key];
    try {
      process.env[key] = "--cfg fixture";
      assert.notEqual(toolInputIdentity(f.root), before);
    } finally {
      if (old === undefined) delete process.env[key];
      else process.env[key] = old;
    }
  }
});
