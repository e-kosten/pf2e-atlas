import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  realpathSync,
} from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import {
  browserOptions,
  runBrowserCandidate,
} from "./validate-browser-candidate.mjs";
import { runChecks } from "./validation/run-check.mjs";
import { output, requireReadiness } from "./development/preflight.mjs";

export async function validateBrowser(args = process.argv.slice(2)) {
  const root = output("git", ["rev-parse", "--show-toplevel"], process.cwd());
  const parsed = browserOptions(["--artifact", "managed-package", ...args]);
  if (parsed.help) {
    console.log(
      "Usage: just validate-browser [--offline] [--output DIR] -- [specs/project selectors]\nVerify/reuse the frontend, package once, and run committed browser tests in isolation. Commit source and install the selected browsers first.",
    );
    return;
  }
  if (process.platform === "win32")
    throw new Error(
      "Packaged browser candidates run on macOS/Linux; use CI for the platform matrix.",
    );
  if (parsed.artifact !== "managed-package" || parsed.commit)
    throw new Error(
      "Use validate-browser-candidate to replay an explicitly selected artifact/commit.",
    );
  if (output("git", ["status", "--porcelain", "--untracked-files=all"], root))
    throw new Error("Commit source changes before final browser validation.");
  await requireReadiness(root, { commands: [["cargo"]] });
  const { verifyFrontendBuild } = await import(
    pathToFileURL(resolve(root, "web/atlas-ui/apiContract.ts"))
  );
  const checks = [];
  try {
    verifyFrontendBuild(root);
    console.error("Reusing verified frontend bundle.");
  } catch {
    await requireReadiness(root, {
      commands: [["npm", "--prefix", "web/atlas-ui"]],
    });
    checks.push([
      "browser frontend build",
      "npm",
      "--prefix",
      "web/atlas-ui",
      "run",
      "build",
    ]);
  }
  const directory = resolve(root, "scratch/browser-packages");
  mkdirSync(directory, { recursive: true });
  const artifact = mkdtempSync(resolve(directory, "candidate-"));
  try {
    checks.push([
      "package browser candidate",
      process.execPath,
      resolve(root, "scripts/browser-fixture.mjs"),
      artifact,
    ]);
    const status = await runChecks(checks);
    if (status) {
      process.exitCode = status;
      return;
    }
    console.error(
      "Browser artifact frozen; isolated tests/configuration/dependencies will now be pinned. Development may resume.",
    );
    await runBrowserCandidate(["--artifact", artifact, ...args]);
  } finally {
    rmSync(artifact, { recursive: true, force: true });
  }
}
if (
  process.argv[1] &&
  existsSync(process.argv[1]) &&
  import.meta.url === pathToFileURL(realpathSync(process.argv[1])).href
)
  validateBrowser().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
