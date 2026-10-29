#!/usr/bin/env node
// Build orchestration stays outside the build-independent Rust developer tool.
import { buildProfile, selectedProfile } from "./build-profile.mjs";
import { requireReadiness } from "./preflight.mjs";
import { createHash } from "node:crypto";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  renameSync,
  writeFileSync,
} from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { execFileSync, spawnSync } from "node:child_process";

const checkout = execFileSync(
  "git",
  ["-C", process.argv[2] ?? ".", "rev-parse", "--show-toplevel"],
  { encoding: "utf8" },
).trim();
const profile = buildProfile(checkout, selectedProfile(process.argv.slice(3)));
const run = (program, args) => {
  const result = spawnSync(program, args, { cwd: checkout, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0)
    throw new Error(`${program} exited with ${result.status ?? result.signal}`);
};
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const git = (...args) => execFileSync("git", ["-C", checkout, ...args]);
const files = () =>
  [
    ...new Set(
      git("ls-files", "-z", "--cached", "--others", "--exclude-standard")
        .toString()
        .split("\0")
        .filter(Boolean),
    ),
  ].sort();
// The shared frontend owner defines UI inputs; this owner adds Rust/build inputs.
const relevant = (name) =>
  /^(Cargo\.(toml|lock)|rust-toolchain\.toml)$/.test(name) ||
  name.startsWith(".cargo/") ||
  name.startsWith("crates/") ||
  name.startsWith("scripts/development/");
const fingerprint = (names) => {
  const digest = createHash("sha256");
  for (const name of names) {
    digest.update(name).update("\0");
    const file = resolve(checkout, name);
    digest
      .update(existsSync(file) ? readFileSync(file) : "<deleted>")
      .update("\0");
  }
  return digest.digest("hex");
};
await requireReadiness(checkout, { commands: [["cargo"]] });
const { frontendInputIdentity, verifyFrontendBuild } = await import(
  pathToFileURL(resolve(checkout, "web/atlas-ui/apiContract.ts"))
);
const state = () => ({
  commit: git("rev-parse", "HEAD").toString().trim(),
  dirty: git("status", "--porcelain", "--untracked-files=normal").length !== 0,
  input_sha256: hash(
    fingerprint(files().filter(relevant)) +
      "\0" +
      frontendInputIdentity(checkout),
  ),
});
const directory = resolve(checkout, ".atlas");
if (existsSync(directory) && lstatSync(directory).isSymbolicLink())
  throw new Error("Checkout .atlas must be private, not a symlink");
mkdirSync(directory, { recursive: true });
const before = state();
const rustVersion = execFileSync("rustc", ["--version"], {
  encoding: "utf8",
}).trim();
let frontend;
try {
  frontend = verifyFrontendBuild(checkout);
} catch {
  await requireReadiness(checkout, {
    commands: [["npm", "--prefix", "web/atlas-ui"]],
  });
  run("npm", ["--prefix", "web/atlas-ui", "run", "build"]);
  frontend = verifyFrontendBuild(checkout);
}
console.error("Using checksum-verified current frontend bundle");
run("cargo", [
  "build",
  ...profile.cargoArgs,
  "--locked",
  "-p",
  "atlas-cli",
  "--target-dir",
  resolve(checkout, "target"),
]);
const after = state();
const finalFrontend = verifyFrontendBuild(checkout);
if (
  before.input_sha256 !== after.input_sha256 ||
  JSON.stringify(frontend) !== JSON.stringify(finalFrontend)
) {
  throw new Error(
    "Source or frontend changed during the build; existing preview preserved. Finish those changes and retry.",
  );
}
const binary = profile.binary;
const receipt = {
  version: 1,
  profile: profile.name,
  binary_sha256: hash(readFileSync(binary)),
  frontend_sha256: frontend.sha256["index.html"],
  api_contract: frontend.api_contract,
  source: { ...after, observed_start: before, observed_finish: after },
  toolchain: rustVersion,
  node: process.version,
  created_at: new Date().toISOString(),
};
const destination = profile.receipt;
const temporary = `${destination}.${process.pid}.tmp`;
writeFileSync(temporary, `${JSON.stringify(receipt, null, 2)}\n`, {
  mode: 0o600,
});
renameSync(temporary, destination);
console.error(`Candidate built: ${binary}\nBuild receipt: ${destination}`);
