import { spawn, execFileSync } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import {
  constants,
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  openSync,
  closeSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, resolve } from "node:path";
import { createServer } from "node:net";
import { fileURLToPath, pathToFileURL } from "node:url";
import { verifyBrowserArtifact } from "./browser-fixture.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const hash = (path) =>
  createHash("sha256").update(readFileSync(path)).digest("hex");
const usage =
  "Usage: node scripts/validate-browser-candidate.mjs --artifact DIR [--commit REF] [--output DIR] [--offline] -- [Playwright test options]\nRuns an authenticated built app with isolated committed tests/configuration and npm ci dependencies; no Rust build, corpus copy or browser download. Install required Playwright browsers first.";

function options(args) {
  const result = { playwright: [] };
  for (let i = 0; i < args.length; i++) {
    const name = args[i];
    if (name === "--") {
      result.playwright = args.slice(i + 1);
      break;
    }
    if (name === "--offline") {
      result.offline = true;
      continue;
    }
    if (name === "--help" || name === "-h") return { help: true };
    if (
      !["--artifact", "--commit", "--output"].includes(name) ||
      !args[i + 1] ||
      args[i + 1].startsWith("--")
    )
      throw new Error(usage);
    result[name.slice(2)] = args[++i];
  }
  if (!result.artifact) throw new Error(usage);
  const selectors = [];
  const valued = new Set([
    "--project",
    "--grep",
    "--grep-invert",
    "--repeat-each",
  ]);
  for (let i = 0; i < result.playwright.length; i++) {
    const argument = result.playwright[i];
    if (!argument.startsWith("-")) {
      selectors.push(argument);
      continue;
    }
    const separator = argument.indexOf("=");
    const key = separator < 0 ? argument : argument.slice(0, separator);
    const normalized = key === "-g" ? "--grep" : key;
    if (!valued.has(normalized))
      throw new Error(
        `Unsupported final-validation option: ${argument}. Select specs, --project, --grep, --grep-invert or --repeat-each; validation config and assertions stay fixed.`,
      );
    const value =
      separator < 0 ? result.playwright[++i] : argument.slice(separator + 1);
    if (!value || (separator < 0 && value.startsWith("-")))
      throw new Error(`Missing value for ${key}`);
    if (normalized === "--repeat-each" && !/^[1-9][0-9]*$/.test(value))
      throw new Error("--repeat-each must execute at least one iteration.");
    selectors.push(`${normalized}=${value}`);
  }
  result.playwright = selectors;
  return result;
}

async function main() {
  const selected = options(process.argv.slice(2));
  if (selected.help) {
    console.log(usage);
    return;
  }
  if (process.platform === "win32")
    throw new Error(
      "Packaged browser candidates run on macOS/Linux; use CI for the platform matrix.",
    );
  const original = resolve(selected.artifact);
  const supplied = JSON.parse(
    readFileSync(resolve(original, "manifest.json"), "utf8"),
  );
  if (!/^[a-f0-9]{40,64}$/.test(supplied.commit))
    throw new Error("Artifact must identify a full source commit.");
  const commit = execFileSync(
    "git",
    [
      "rev-parse",
      "--verify",
      "--end-of-options",
      `${selected.commit || supplied.commit}^{commit}`,
    ],
    { cwd: root, encoding: "utf8" },
  ).trim();
  const manifest = verifyBrowserArtifact(original, commit);
  const output = resolve(
    selected.output ||
      resolve(
        root,
        "scratch/browser-candidates",
        `${Date.now()}-${randomUUID().slice(0, 8)}`,
      ),
  );
  if (existsSync(output))
    throw new Error(
      "Output directory already exists; choose a new run directory.",
    );
  mkdirSync(output, { recursive: true });
  const checkout = resolve(output, "checkout");
  const artifact = resolve(output, "artifact");
  const report = {
    version: 1,
    commit,
    tree: execFileSync("git", ["rev-parse", `${commit}^{tree}`], {
      cwd: root,
      encoding: "utf8",
    }).trim(),
    started_at: new Date().toISOString(),
    platform: process.platform,
    arch: process.arch,
    node: process.version,
    runner_sha256: hash(fileURLToPath(import.meta.url)),
    artifact: manifest,
    status: "running",
    stages: [],
  };
  const save = () =>
    writeFileSync(
      resolve(output, "result.json"),
      JSON.stringify(report, null, 2) + "\n",
    );
  let active;
  let interrupted;
  const onSignal = (signal) => {
    interrupted = signal;
    if (active?.pid) {
      try {
        if (process.platform === "win32") active.kill(signal);
        else process.kill(-active.pid, signal);
      } catch (error) {
        if (error.code !== "ESRCH") console.error(String(error));
      }
    }
  };
  process.on("SIGINT", onSignal);
  process.on("SIGTERM", onSignal);
  save();
  console.log(
    `Candidate: ${commit}\nResults: ${resolve(output, "result.json")}`,
  );
  async function stage(name, command, args, cwd, env = process.env) {
    if (interrupted) throw new Error(`Interrupted by ${interrupted}`);
    const entry = {
      name,
      command: [command, ...args],
      started_at: new Date().toISOString(),
      status: "running",
      log: resolve(output, `${name}.log`),
    };
    report.stages.push(entry);
    save();
    console.log(`Running ${name}; log: ${entry.log}`);
    const fd = openSync(entry.log, "w");
    const started = Date.now();
    try {
      const exit = await new Promise((resolveExit, reject) => {
        active = spawn(command, args, {
          cwd,
          env,
          detached: process.platform !== "win32",
          stdio: ["ignore", fd, fd],
        });
        active.once("error", reject);
        active.once("exit", (code, signal) => resolveExit({ code, signal }));
        const timer = setInterval(
          () =>
            console.log(
              `Still running ${name} (${Math.round((Date.now() - started) / 1000)}s); ${entry.log}`,
            ),
          30000,
        );
        active.once("close", () => clearInterval(timer));
      });
      entry.exit_code = exit.code;
      entry.signal = exit.signal;
      if (exit.code !== 0 || interrupted)
        throw new Error(
          `${name} failed (${exit.signal || exit.code}); see ${entry.log}`,
        );
      entry.status = "passed";
    } catch (error) {
      entry.status = interrupted ? "interrupted" : "failed";
      entry.error = String(error);
      throw error;
    } finally {
      active = undefined;
      closeSync(fd);
      entry.duration_ms = Date.now() - started;
      save();
    }
  }
  try {
    mkdirSync(artifact);
    for (const name of ["browser_fixture", "index.html", "manifest.json"])
      copyFileSync(
        resolve(original, name),
        resolve(artifact, name),
        constants.COPYFILE_FICLONE,
      );
    report.artifact = verifyBrowserArtifact(artifact, commit);
    save();
    // Separate Git metadata avoids coupling this run's HEAD to the development branch.
    await stage(
      "checkout",
      "git",
      ["clone", "--quiet", "--shared", "--no-checkout", root, checkout],
      root,
    );
    await stage(
      "candidate",
      "git",
      ["-C", checkout, "checkout", "--quiet", "--detach", commit],
      root,
    );
    const ui = resolve(checkout, "web/atlas-ui");
    const { frontendInputIdentity } = await import(
      pathToFileURL(resolve(ui, "apiContract.ts"))
    );
    if (typeof frontendInputIdentity !== "function")
      throw new Error(
        "Candidate lacks the frontend build identity helper; rebuild with current tooling.",
      );
    const assertFrontendInputs = () => {
      const identity = frontendInputIdentity(checkout);
      if (identity !== report.artifact.frontend.input_sha256)
        throw new Error(
          "Packaged frontend inputs do not match the isolated candidate. Rebuild from committed inputs and matching Vite environment; ignored local env files are not copied.",
        );
      report.frontend_input_sha256 = identity;
    };
    assertFrontendInputs();
    report.package_lock_sha256 = hash(resolve(ui, "package-lock.json"));
    const npm = process.platform === "win32" ? "npm.cmd" : "npm";
    await stage(
      "dependencies",
      npm,
      ["ci", ...(selected.offline ? ["--offline"] : [])],
      ui,
    );
    report.playwright = JSON.parse(
      readFileSync(
        resolve(ui, "node_modules/@playwright/test/package.json"),
        "utf8",
      ),
    ).version;
    const assertFixedSource = () => {
      const actual = execFileSync("git", ["rev-parse", "HEAD"], {
        cwd: checkout,
        encoding: "utf8",
      }).trim();
      const changes = execFileSync(
        "git",
        ["status", "--porcelain", "--untracked-files=all"],
        { cwd: checkout, encoding: "utf8" },
      ).trim();
      if (actual !== commit || changes)
        throw new Error("Isolated candidate source changed during validation.");
    };
    assertFixedSource();
    assertFrontendInputs();
    const port =
      process.env.ATLAS_DEVICE_PORT ||
      (await new Promise((resolvePort, reject) => {
        const server = createServer();
        server.once("error", reject);
        server.listen(0, "127.0.0.1", () => {
          const address = server.address();
          server.close((error) =>
            error ? reject(error) : resolvePort(String(address.port)),
          );
        });
      }));
    report.device_port = port;
    const env = {
      ...process.env,
      ATLAS_DEVICE_PORT: port,
      ATLAS_BROWSER_ARTIFACT: artifact,
      PLAYWRIGHT_HTML_OUTPUT_DIR: resolve(output, "report"),
    };
    await stage(
      "browsers",
      npm,
      [
        "exec",
        "--",
        "playwright",
        "test",
        "--config",
        "playwright.devices.config.ts",
        "--output",
        resolve(output, "results"),
        ...selected.playwright,
      ],
      ui,
      env,
    );
    if (hash(resolve(ui, "package-lock.json")) !== report.package_lock_sha256)
      throw new Error("Candidate lockfile changed during validation.");
    assertFixedSource();
    assertFrontendInputs();
    verifyBrowserArtifact(artifact, commit);
    report.status = "passed";
  } catch (error) {
    report.status = interrupted ? "interrupted" : "failed";
    report.error = String(error);
    process.exitCode = interrupted ? 130 : 1;
    console.error(String(error));
  } finally {
    process.removeListener("SIGINT", onSignal);
    process.removeListener("SIGTERM", onSignal);
    try {
      const evidence = resolve(
        checkout,
        "scratch/responsive-validation/browser",
      );
      if (existsSync(evidence))
        cpSync(evidence, resolve(output, "browser-evidence"), {
          recursive: true,
        });
      // Dependencies are private to the run; retain reports and immutable app, not a second checkout/cache.
      rmSync(checkout, { recursive: true, force: true });
    } catch (error) {
      report.status = "failed";
      report.cleanup_error = String(error);
      process.exitCode = 1;
    }
    report.finished_at = new Date().toISOString();
    save();
    console.log(
      `${report.status}: ${commit}\nResults: ${resolve(output, "result.json")}`,
    );
  }
}

main().catch((error) => {
  console.error(String(error));
  process.exitCode = 1;
});
