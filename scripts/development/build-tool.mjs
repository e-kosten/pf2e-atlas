import { runChecks } from "../validation/run-check.mjs";
import {
  chmodSync,
  copyFileSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, resolve } from "node:path";
import {
  digest,
  output,
  requireReadiness,
  sharedStore,
  toolCapabilities,
  toolInputIdentity,
} from "./preflight.mjs";
const root = output("git", ["rev-parse", "--show-toplevel"], process.cwd());
try {
  await requireReadiness(root, { toolBuild: true });
  const before = toolInputIdentity(root);
  const status = await runChecks([
    [
      "build retained developer tool",
      "cargo",
      "build",
      "--locked",
      "-p",
      "atlas-dev",
      "--target-dir",
      resolve(root, "target"),
    ],
  ]);
  if (status !== 0) {
    process.exitCode = status;
    throw new Error(
      `atlas-dev build exited ${status}; retained tool preserved.`,
    );
  }
  if (before !== toolInputIdentity(root))
    throw new Error(
      "Developer tool inputs changed during compilation; retained tool preserved.",
    );
  const binary = resolve(
    root,
    "target/debug",
    process.platform === "win32" ? "atlas-dev.exe" : "atlas-dev",
  );
  const destination = resolve(sharedStore(root), "tools/atlas-dev");
  const staged = destination + `.${process.pid}.tmp`;
  mkdirSync(dirname(destination), { recursive: true });
  try {
    copyFileSync(binary, staged);
    chmodSync(staged, 0o755);
    const receipt = {
      version: 1,
      binary_sha256: digest(readFileSync(staged)),
      input_sha256: before,
      toolchain: output("rustc", ["--version"], root),
      source_commit: output("git", ["rev-parse", "HEAD"], root),
      source_dirty:
        output(
          "git",
          ["status", "--porcelain", "--untracked-files=normal"],
          root,
        ).length !== 0,
      capabilities: toolCapabilities(staged, root),
    };
    writeFileSync(staged + ".json", JSON.stringify(receipt, null, 2) + "\n");
    renameSync(staged, destination);
    renameSync(staged + ".json", destination + ".json");
    const published = JSON.parse(readFileSync(destination + ".json", "utf8"));
    if (published.binary_sha256 !== digest(readFileSync(destination)))
      throw new Error(
        "Concurrent developer-tool publication changed the binary/receipt; retry explicitly.",
      );
    console.error(`Retained atlas-dev: ${destination}`);
  } finally {
    rmSync(staged, { force: true });
    rmSync(staged + ".json", { force: true });
  }
} catch (error) {
  console.error(error.message);
  process.exitCode ||= 1;
}
