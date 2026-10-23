import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import {
  chmodSync,
  copyFileSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const artifactFiles = ["browser_fixture", "index.html"];
const hash = (path) =>
  createHash("sha256").update(readFileSync(path)).digest("hex");
const head = () =>
  execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8",
  }).trim();
const quote = (value) => `'${value.replaceAll("'", "'\\''")}'`;

export function browserArtifact(
  directory = process.env.ATLAS_BROWSER_ARTIFACT,
) {
  if (!directory) return undefined;
  const path = resolve(directory);
  const manifest = JSON.parse(
    readFileSync(resolve(path, "manifest.json"), "utf8"),
  );
  if (manifest.commit !== head())
    throw new Error("Browser artifact belongs to another commit.");
  if (manifest.platform !== process.platform || manifest.arch !== process.arch)
    throw new Error(
      "Browser artifact belongs to another platform/architecture.",
    );
  for (const name of artifactFiles) {
    if (manifest.sha256?.[name] !== hash(resolve(path, name)))
      throw new Error(`Browser artifact checksum mismatch: ${name}`);
  }
  return {
    command: quote(resolve(path, "browser_fixture")),
    index: resolve(path, "index.html"),
  };
}

if (
  process.argv[1] &&
  realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  const directory = process.argv[2];
  if (!directory)
    throw new Error("Usage: node scripts/browser-fixture.mjs OUTPUT_DIRECTORY");
  if (
    execFileSync("git", ["status", "--porcelain", "--untracked-files=no"], {
      cwd: root,
      encoding: "utf8",
    }).trim()
  )
    throw new Error(
      "Commit source changes before packaging a browser fixture.",
    );
  execFileSync(
    "cargo",
    ["build", "--locked", "-p", "atlas-web", "--example", "browser_fixture"],
    { cwd: root, stdio: "inherit" },
  );
  if (process.platform === "linux")
    execFileSync(
      "strip",
      ["--strip-debug", "target/debug/examples/browser_fixture"],
      { cwd: root, stdio: "inherit" },
    );
  mkdirSync(directory, { recursive: true });
  copyFileSync(
    resolve(root, "target/debug/examples/browser_fixture"),
    resolve(directory, "browser_fixture"),
  );
  chmodSync(resolve(directory, "browser_fixture"), 0o755);
  copyFileSync(
    resolve(root, "web/atlas-ui/dist/index.html"),
    resolve(directory, "index.html"),
  );
  writeFileSync(
    resolve(directory, "manifest.json"),
    JSON.stringify(
      {
        commit: head(),
        platform: process.platform,
        arch: process.arch,
        sha256: Object.fromEntries(
          artifactFiles.map((name) => [name, hash(resolve(directory, name))]),
        ),
      },
      null,
      2,
    ) + "\n",
  );
  browserArtifact(directory);
}
