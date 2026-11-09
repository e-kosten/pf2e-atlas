import { execFile, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  appendFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const releaseUrl = "https://github.com/axodotdev/cargo-dist/releases/download";
const config = new URL("../../dist-workspace.toml", import.meta.url);
const checksums = new URL("./cargo-dist-checksums.json", import.meta.url);
const execFileAsync = promisify(execFile);

function archiveDigest(path) {
  try {
    return createHash("sha256").update(readFileSync(path)).digest("hex");
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
    return undefined;
  }
}

function archiveName(target) {
  const extension = target.endsWith("-windows-msvc") ? "zip" : "tar.xz";
  return `cargo-dist-${target}.${extension}`;
}

function lock() {
  const { version, archives } = JSON.parse(readFileSync(checksums, "utf8"));
  const workspace = readFileSync(config, "utf8");
  const workspaceVersion = workspace.match(/^cargo-dist-version\s*=\s*"([^"]+)"/m)?.[1];
  const targets = JSON.parse(workspace.match(/^targets\s*=\s*(\[.*\])/m)?.[1] ?? "[]");
  if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version) ||
      version !== workspaceVersion ||
      targets.length !== Object.keys(archives).length ||
      targets.some((target) => !/^[a-f0-9]{64}$/.test(archives[target]))) {
    throw new Error("cargo-dist checksum pins must match the workspace version and targets");
  }
  return { version, archives };
}

async function downloadArchive(url, path, digest) {
  const partial = `${path}.partial`;
  try {
    const { stderr } = await execFileAsync("curl", [
      "--disable", "--fail", "--no-progress-meter", "--location",
      "--proto", "=https", "--proto-redir", "=https",
      "--connect-timeout", "10", "--max-time", "25",
      "--retry", "3", "--retry-max-time", "90", "--retry-all-errors",
      "--output", partial, url,
    ], { timeout: 120_000 });
    if (stderr) console.error(`${url}: ${stderr.trimEnd()}`);
    if (archiveDigest(partial) !== digest) throw new Error("SHA-256 mismatch");
    renameSync(partial, path);
  } catch (error) {
    throw new Error(`cargo-dist download failed for ${url}: ${error.message}`, { cause: error });
  } finally {
    rmSync(partial, { force: true });
  }
}

async function acquire(directory, { version, archives }) {
  mkdirSync(directory, { recursive: true });
  const results = await Promise.allSettled(Object.entries(archives).map(async ([target, digest]) => {
    const filename = archiveName(target);
    const path = join(directory, filename);
    const cached = archiveDigest(path);
    if (cached === digest) {
      console.log(`${target}: using verified cached cargo-dist ${version}`);
      return;
    }
    if (cached !== undefined) {
      console.warn(`${target}: cached cargo-dist checksum mismatch; fetching a fresh archive`);
      rmSync(path);
    }
    console.log(`${target}: downloading cargo-dist ${version} (up to four attempts)`);
    await downloadArchive(`${releaseUrl}/v${version}/${filename}`, path, digest);
  }));
  const failures = results.filter((result) => result.status === "rejected");
  if (failures.length > 0) {
    throw new Error(failures.map((failure) => failure.reason.message).join("\n"));
  }
}

function install(directory, target, { version, archives }) {
  if (!Object.hasOwn(archives, target)) throw new Error(`unsupported cargo-dist target ${target}`);
  const archive = join(directory, archiveName(target));
  if (archiveDigest(archive) !== archives[target]) {
    throw new Error(`missing or invalid cargo-dist archive for ${target}`);
  }
  const extracted = mkdtempSync(join(process.env.RUNNER_TEMP, "atlas-cargo-dist-"));
  const windows = target.endsWith("-windows-msvc");
  // Git Bash can put GNU tar first on PATH, which cannot extract ZIP archives.
  const tar = windows ? join(process.env.SystemRoot, "System32", "tar.exe") : "tar";
  execFileSync(tar, ["-xf", archive, "-C", extracted], { stdio: "inherit" });
  const bin = windows ? extracted : join(extracted, `cargo-dist-${target}`);
  const actual = execFileSync(join(bin, windows ? "dist.exe" : "dist"), ["--version"], {
    encoding: "utf8",
  }).trim();
  if (actual !== `cargo-dist ${version}`) {
    throw new Error(`expected cargo-dist ${version}, got ${actual}`);
  }
  console.log(`Verified ${actual} for ${target}`);
  appendFileSync(process.env.GITHUB_PATH, `${bin}\n`);
}

try {
  const [, , command, directory, target] = process.argv;
  const pinned = lock();
  switch (command) {
    case "check":
      break;
    case "acquire":
      await acquire(directory, pinned);
      break;
    case "install":
      install(directory, target, pinned);
      break;
    default:
      throw new Error(`unknown cargo-dist command ${command}`);
  }
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
