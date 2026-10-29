import { execFileSync } from "node:child_process";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { resolve, dirname } from "node:path";
import { digest, toolInputIdentity } from "./preflight.mjs";
export function fixture(t) {
  const root = mkdtempSync(resolve(tmpdir(), "atlas-preflight-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const put = (name, value) => {
    const path = resolve(root, name);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, value);
    return path;
  };
  put(
    "web/atlas-ui/package.json",
    '{"type":"module","engines":{"node":"^22.18.0 || ^24.0.0"}}',
  );
  put("rust-toolchain.toml", '[toolchain]\nchannel="1.95.0"\n');
  put("Cargo.toml", "[workspace]\n");
  put("Cargo.lock", "fixture lock\n");
  put("crates/atlas-dev/Cargo.toml", "[package]\nname='atlas-dev'\n");
  put("crates/atlas-dev/src/main.rs", "fn main() {}\n");
  put(".atlas/workspace.sqlite", "private migrated database");
  put(
    "web/atlas-ui/apiContract.ts",
    "export function verifyFrontendBuild() {return {verified:true};}\n",
  );
  put(".gitignore", ".atlas/\nbin/\ntarget/\n");
  put(
    "scripts/validation/inputs.mjs",
    readFileSync(new URL("../validation/inputs.mjs", import.meta.url)),
  );
  execFileSync("git", ["init", "--quiet"], { cwd: root });
  for (const [key, value] of [
    ["user.name", "Fixture"],
    ["user.email", "fixture@example.invalid"],
    ["commit.gpgsign", "false"],
    ["core.hooksPath", "/dev/null"],
  ])
    execFileSync("git", ["config", key, value], { cwd: root });
  execFileSync("git", ["add", "."], { cwd: root });
  execFileSync("git", ["commit", "-qm", "test: fixture"], { cwd: root });
  const bin = resolve(root, "bin");
  mkdirSync(bin);
  const program = (name, body) => {
    const path = put(`bin/${name}`, `#!/bin/sh\n${body}\n`);
    chmodSync(path, 0o755);
    return path;
  };
  program("rustc", "printf 'rustc 1.95.0 (fixture)\\n'");
  program(
    "cargo",
    `printf '{"target_directory":"%s/target","packages":[{"name":"atlas-dev","manifest_path":"%s/crates/atlas-dev/Cargo.toml","dependencies":[]}]}\\n' "$PWD" "$PWD"`,
  );
  const tool = program(
    "atlas-dev",
    `printf '{"version":1,"managed_preview":true,"workspace_schema_version":22}\\n'`,
  );
  const oldPath = process.env.PATH,
    oldTool = process.env.ATLAS_DEV_BINARY;
  process.env.PATH = bin + ":" + oldPath;
  process.env.ATLAS_DEV_BINARY = tool;
  t.after(() => {
    process.env.PATH = oldPath;
    if (oldTool === undefined) delete process.env.ATLAS_DEV_BINARY;
    else process.env.ATLAS_DEV_BINARY = oldTool;
  });
  const publish = () =>
    put(
      "bin/atlas-dev.json",
      JSON.stringify({
        version: 1,
        binary_sha256: digest(readFileSync(tool)),
        input_sha256: toolInputIdentity(root),
      }),
    );
  publish();
  put(".atlas/dev-baselines/current.json", '{"baseline":"seed"}');
  put(
    ".atlas/dev-baselines/versions/seed/manifest.json",
    '{"database":{"schema_version":19}}',
  );
  return { root, put, program, publish, tool };
}
