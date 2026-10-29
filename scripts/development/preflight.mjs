import { environmentState } from "../validation/inputs.mjs";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  existsSync,
  lstatSync,
  readFileSync,
  readlinkSync,
  readdirSync,
  realpathSync,
} from "node:fs";
import { dirname, relative, resolve } from "node:path";
import { pathToFileURL } from "node:url";

export const digest = (bytes) =>
  createHash("sha256").update(bytes).digest("hex");
export function output(command, args, root) {
  const result =
    process.platform === "win32"
      ? spawnSync("sh", ["-c", 'exec "$@"', "sh", command, ...args], {
          cwd: root,
          encoding: "utf8",
          timeout: 10000,
          maxBuffer: 32 * 1024 * 1024,
          env: { ...process.env, RUSTUP_AUTO_INSTALL: "0" },
        })
      : spawnSync(command, args, {
          cwd: root,
          encoding: "utf8",
          timeout: 10000,
          maxBuffer: 32 * 1024 * 1024,
          env: { ...process.env, RUSTUP_AUTO_INSTALL: "0" },
        });
  if (result.error || result.status !== 0)
    throw new Error(
      result.error?.message ||
        result.stderr.trim() ||
        `${command} exited ${result.status}`,
    );
  return result.stdout.trim();
}
export function nodeSupported(version, range) {
  const actual = /^v?(\d+)\.(\d+)\.(\d+)/.exec(version)?.slice(1).map(Number);
  return (
    !!actual &&
    range.split("||").some((part) => {
      const floor = /^\^(\d+)\.(\d+)\.(\d+)$/
        .exec(part.trim())
        ?.slice(1)
        .map(Number);
      if (!floor)
        throw new Error(`Unsupported Node engine expression: ${range}`);
      return (
        actual[0] === floor[0] &&
        (actual[1] > floor[1] ||
          (actual[1] === floor[1] && actual[2] >= floor[2]))
      );
    })
  );
}
export function canonical(path) {
  const tail = [];
  let current = resolve(path);
  while (!lstatSafe(current)) {
    const parent = dirname(current);
    if (parent === current) throw new Error(`Cannot resolve ${path}`);
    tail.unshift(relative(parent, current));
    current = parent;
  }
  return resolve(realpathSync(current), ...tail);
}
export function metadata(root) {
  return JSON.parse(
    output(
      "cargo",
      ["metadata", "--format-version", "1", "--locked", "--offline"],
      root,
    ),
  );
}

function checkCompilerSelection(root) {
  for (const name of ["RUSTC", "CARGO_BUILD_RUSTC"])
    if (process.env[name])
      throw new Error(
        `Unset ${name}; readiness supports the pinned PATH compiler.`,
      );
  const directories = new Set([
    process.env.CARGO_HOME || resolve(process.env.HOME || root, ".cargo"),
  ]);
  for (let directory = root; ; directory = dirname(directory)) {
    directories.add(resolve(directory, ".cargo"));
    if (directory === dirname(directory)) break;
  }
  // Conservatively reject compiler selectors rather than interpret Cargo TOML
  // precedence. Wrappers and rustflags remain supported and fingerprinted.
  for (const directory of directories)
    for (const name of ["config", "config.toml"]) {
      const path = resolve(directory, name);
      if (
        existsSync(path) &&
        /(?:[.\s{"']|^)rustc["']?\s*=/.test(
          readFileSync(path, "utf8").replace(/^\s*#.*$/gm, ""),
        )
      )
        throw new Error(
          `Remove the custom rustc selector in ${path}; readiness supports the pinned PATH compiler.`,
        );
    }
}
export function toolInputIdentity(root, data = metadata(root)) {
  root = realpathSync(root);
  const packages = new Map(
    data.packages
      .filter((pkg) => !pkg.source)
      .map((pkg) => [canonical(dirname(pkg.manifest_path)), pkg]),
  );
  const tool = [...packages.values()].find((pkg) => pkg.name === "atlas-dev");
  if (!tool) throw new Error("atlas-dev package is missing");
  const entries = new Map();
  const excluded = new Set([
    "target",
    "node_modules",
    ".git",
    ".atlas",
    ".cache",
  ]);
  function collect(path, key, ancestors = new Set()) {
    const stat = lstatSafe(path);
    if (!stat) return;
    if (stat.isSymbolicLink()) {
      entries.set(key + ":link", readlinkSync(path, { encoding: "buffer" }));
      const actual = realpathSync(path);
      if (ancestors.has(actual))
        throw new Error(`Cyclic developer tool input: ${path}`);
      collect(actual, key, new Set([...ancestors, actual]));
      return;
    }
    if (stat.isDirectory())
      for (const name of readdirSync(path).sort()) {
        if (!excluded.has(name))
          collect(resolve(path, name), `${key}/${name}`, ancestors);
      }
    else if (stat.isFile())
      entries.set(
        key,
        Buffer.concat([
          Buffer.from(`${stat.mode & 0o111 ? "executable" : "file"}\0`),
          readFileSync(path),
        ]),
      );
  }
  const visited = new Set();
  function visit(pkg) {
    const directory = canonical(dirname(pkg.manifest_path));
    if (visited.has(directory)) return;
    visited.add(directory);
    const local = relative(root, directory);
    collect(
      directory,
      local.startsWith("..") ? `external:${directory}` : `workspace:${local}`,
    );
    for (const dependency of pkg.dependencies)
      if (dependency.path) {
        const next = packages.get(canonical(dependency.path));
        if (!next)
          throw new Error(
            `Local developer-tool dependency missing from Cargo metadata: ${dependency.path}`,
          );
        visit(next);
      }
  }
  visit(tool);
  for (const name of [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    ".cargo",
  ])
    collect(resolve(root, name), `workspace:${name}`);
  for (let ancestor = dirname(root); ; ancestor = dirname(ancestor)) {
    for (const name of ["config", "config.toml"])
      collect(
        resolve(ancestor, ".cargo", name),
        `ancestor:${ancestor}/.cargo/${name}`,
      );
    if (ancestor === dirname(ancestor)) break;
  }
  const cargoHome =
    process.env.CARGO_HOME || resolve(process.env.HOME || root, ".cargo");
  for (const name of ["config", "config.toml"])
    collect(resolve(cargoHome, name), `global-cargo:${name}`);
  entries.set(
    "compiler environment",
    Buffer.from(
      JSON.stringify(
        Object.entries(environmentState()).filter(([name]) =>
          /^(CARGO_|RUST|CC(?:_|$)|CXX(?:_|$)|AR(?:_|$)|CFLAGS|CXXFLAGS|CPPFLAGS|LDFLAGS|LIBRARY_PATH|CPATH|MACOSX_DEPLOYMENT_TARGET|SDKROOT)/.test(
            name,
          ),
        ),
      ),
    ),
  );
  const hash = createHash("sha256");
  for (const [key, value] of [...entries].sort(([a], [b]) =>
    a.localeCompare(b),
  ))
    hash
      .update(`${key.length}:`)
      .update(key)
      .update(`${value.length}:`)
      .update(value);
  return hash.digest("hex");
}

export function sharedStore(root) {
  const common = output(
    "git",
    ["rev-parse", "--path-format=absolute", "--git-common-dir"],
    root,
  );
  return resolve(dirname(common), ".atlas/dev-baselines");
}
export function retainedTool(root) {
  return (
    process.env.ATLAS_DEV_BINARY ||
    resolve(sharedStore(root), "tools/atlas-dev")
  );
}
export function toolCapabilities(tool, root) {
  const caps = JSON.parse(output(tool, ["baseline", "capabilities"], root));
  if (
    caps.version !== 1 ||
    caps.managed_preview !== true ||
    !Number.isInteger(caps.workspace_schema_version)
  )
    throw new Error("Unsupported developer-tool capabilities");
  return caps;
}
export async function readiness(
  root,
  { commands = [], preview = false, toolBuild = false, shared = false } = {},
) {
  root = realpathSync(root);
  const issues = [];
  const notes = [];
  if (lstatSafe(resolve(root, ".atlas"))?.isSymbolicLink())
    issues.push("Checkout .atlas must be private, not a symlink.");
  const needCargo =
    preview ||
    shared ||
    toolBuild ||
    commands.some((args) => args[0] === "cargo");
  const engines = JSON.parse(
    readFileSync(resolve(root, "web/atlas-ui/package.json"), "utf8"),
  ).engines.node;
  if (!nodeSupported(process.version, engines))
    issues.push(
      `Node ${process.version} selected at ${process.execPath}; use ${engines} from web/atlas-ui/package.json.`,
    );
  let data;
  if (needCargo) {
    const pinned = /channel\s*=\s*"([^"]+)"/.exec(
      readFileSync(resolve(root, "rust-toolchain.toml"), "utf8"),
    )?.[1];
    try {
      checkCompilerSelection(root);
      const actual = output("rustc", ["--version"], root).split(" ")[1];
      if (actual !== pinned)
        issues.push(
          `Rust ${actual} selected; use pinned ${pinned} (rustup toolchain install ${pinned}, or put that installation on PATH).`,
        );
      else {
        data = metadata(root);
        if (canonical(data.target_directory) !== resolve(root, "target"))
          issues.push(
            `Cargo target is ${data.target_directory}; use this checkout's private target/ and remove shared target overrides.`,
          );
      }
    } catch (error) {
      issues.push(
        `Rust/Cargo prerequisite failed: ${error.message}. Install the pinned toolchain or run cargo fetch --locked explicitly if dependency metadata is missing.`,
      );
    }
  }
  const packages = new Set(
    commands
      .filter((args) => args[0] === "npm" && args.includes("--prefix"))
      .map((args) => args[args.indexOf("--prefix") + 1]),
  );
  if (preview) {
    const database = lstatSafe(resolve(root, ".atlas/workspace.sqlite"));
    if (database && (!database.isFile() || database.isSymbolicLink()))
      issues.push("Private workspace must be a regular file, not a symlink.");
    if (!database)
      issues.push(
        "Private workspace missing; run just dev-workspace-init explicitly.",
      );
  }
  if (preview || shared) {
    let capabilities;
    try {
      const tool = retainedTool(root);
      const caps = toolCapabilities(tool, root);
      capabilities = caps;
      const receipt = JSON.parse(readFileSync(tool + ".json", "utf8"));
      if (
        receipt.version !== 1 ||
        receipt.binary_sha256 !== digest(readFileSync(tool))
      )
        throw new Error("developer-tool binary/receipt mismatch");
      if (data && receipt.input_sha256 !== toolInputIdentity(root, data))
        throw new Error("developer-tool source inputs changed");
      notes.push(
        `Retained tool supports managed previews; compiled schema ${caps.workspace_schema_version}.`,
      );
    } catch (error) {
      issues.push(
        `Retained developer tool unavailable/stale: ${error.message}. Run just dev-tool-build explicitly before compilation/refresh; unset ATLAS_DEV_BINARY if it selects an unrecorded candidate.`,
      );
    }
    if (capabilities)
      try {
        const receiptPath = resolve(root, ".atlas/dev-workspace.json");
        const pinned = existsSync(receiptPath)
          ? JSON.parse(readFileSync(receiptPath, "utf8"))
          : undefined;
        const store = pinned?.store || sharedStore(root);
        const id =
          pinned?.baseline ||
          JSON.parse(readFileSync(resolve(store, "current.json"), "utf8"))
            .baseline;
        if (!/^[a-zA-Z0-9][a-zA-Z0-9._-]*$/.test(id))
          throw new Error("invalid baseline identity");
        const seed = JSON.parse(
          readFileSync(resolve(store, "versions", id, "manifest.json"), "utf8"),
        );
        if (
          seed.database.schema_version < capabilities.workspace_schema_version
        )
          notes.push(
            `Shared seed schema ${seed.database.schema_version} lags ${capabilities.workspace_schema_version}; run explicit dev-shared-refresh after validating a current build. Existing task databases may already be migrated.`,
          );
      } catch (error) {
        notes.push(
          `Shared seed metadata unavailable: ${error.message}. Inspect dev-status with the selected store; this does not establish private database compatibility.`,
        );
      }
  }
  if (preview) {
    try {
      const { verifyFrontendBuild } = await import(
        pathToFileURL(resolve(root, "web/atlas-ui/apiContract.ts"))
      );
      verifyFrontendBuild(root);
      notes.push("Verified frontend bundle can be reused.");
    } catch {
      packages.add("web/atlas-ui");
    }
  }
  for (const prefix of packages)
    if (!existsSync(resolve(root, prefix, "node_modules/.package-lock.json")))
      issues.push(
        `Dependencies missing for ${prefix}; run npm --prefix ${prefix} ci explicitly.`,
      );
  return { issues, notes };
}
function lstatSafe(path) {
  try {
    return lstatSync(path);
  } catch {
    return undefined;
  }
}
export async function requireReadiness(root, options) {
  const report = await readiness(root, options);
  for (const note of report.notes) console.error(note);
  if (report.issues.length) throw new Error(report.issues.join("\n"));
  return report;
}
if (
  process.argv[1] &&
  existsSync(process.argv[1]) &&
  import.meta.url === pathToFileURL(realpathSync(process.argv[1])).href
) {
  const mode = process.argv[2];
  if (!["--preview", "--validation", "--build-tool"].includes(mode)) {
    console.error(
      "Usage: node scripts/development/preflight.mjs --preview|--validation|--build-tool [CHECKOUT]",
    );
    process.exitCode = 2;
  } else
    try {
      await requireReadiness(resolve(process.argv[3] || "."), {
        preview: mode === "--preview",
        toolBuild: mode === "--build-tool",
        commands: mode === "--validation" ? [["cargo"]] : [],
      });
      console.error("Operation prerequisites ready.");
    } catch (error) {
      console.error(error.message);
      process.exitCode = 1;
    }
}
