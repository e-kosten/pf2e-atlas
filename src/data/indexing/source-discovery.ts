import { execFile } from "node:child_process";
import { readdir, rm, stat } from "node:fs/promises";
import path from "node:path";
import { promisify } from "node:util";

import { pathExists } from "../../shared/fs.js";
import { PackManifestEntry } from "../../domain/record-types.js";

const execFileAsync = promisify(execFile);

function hashText(value: string): number {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }

  return hash >>> 0;
}

function isJsonRecord(filename: string): boolean {
  return filename.endsWith(".json") && filename !== "_folders.json";
}

export async function walkJsonFiles(root: string): Promise<string[]> {
  const entries = await readdir(root, { withFileTypes: true });
  const files = await Promise.all(
    entries.map(async (entry) => {
      const entryPath = path.join(root, entry.name);
      if (entry.isDirectory()) {
        return walkJsonFiles(entryPath);
      }

      if (entry.isFile() && isJsonRecord(entry.name)) {
        return [entryPath];
      }

      return [];
    }),
  );

  return files.flat();
}

async function directoryExists(targetPath: string): Promise<boolean> {
  try {
    const details = await stat(targetPath);
    return details.isDirectory();
  } catch {
    return false;
  }
}

async function isGitCheckout(rootPath: string): Promise<boolean> {
  return pathExists(path.join(rootPath, ".git"));
}

async function computeFileSignature(rootPath: string, filePaths: string[]): Promise<string> {
  const files = [...new Set(filePaths)].sort((left, right) => left.localeCompare(right));
  let hash = 2166136261;
  for (const filePath of files) {
    const details = await stat(filePath);
    const value = `${path.relative(rootPath, filePath)}:${details.size}:${Math.trunc(details.mtimeMs)}`;
    hash ^= hashText(value);
    hash = Math.imul(hash, 16777619);
  }

  return String(hash >>> 0);
}

export async function computeSourceSignature(rootPath: string, manifestPath: string): Promise<string> {
  if (await isGitCheckout(rootPath)) {
    try {
      const [{ stdout: headStdout }, { stdout: statusStdout }, { stdout: untrackedStdout }] = await Promise.all([
        execFileAsync("git", ["-C", rootPath, "rev-parse", "HEAD"], { timeout: 10_000 }),
        execFileAsync("git", ["-C", rootPath, "status", "--porcelain", "--untracked-files=no"], { timeout: 10_000 }),
        execFileAsync(
          "git",
          [
            "-C",
            rootPath,
            "ls-files",
            "--others",
            "--exclude-standard",
            "--full-name",
            "--",
            "*.json",
            ":(glob)**/*.json",
          ],
          { timeout: 10_000 },
        ),
      ]);
      const head = headStdout.trim();
      const dirty = statusStdout.trim();
      const untrackedJsonFiles = untrackedStdout
        .split(/\r?\n/)
        .map((filePath) => filePath.trim())
        .filter(Boolean)
        .map((filePath) => path.join(rootPath, filePath));
      const untrackedJsonSignature = await computeFileSignature(rootPath, untrackedJsonFiles);
      return `git:${head}:${dirty}:${untrackedJsonSignature}`;
    } catch {
      // Fall through to filesystem signature.
    }
  }

  const files = [manifestPath, ...(await walkJsonFiles(rootPath))];
  return `fs:${await computeFileSignature(rootPath, files)}`;
}

export async function removeIndexFiles(indexPath: string): Promise<void> {
  await rm(indexPath, { force: true });
  await rm(`${indexPath}-wal`, { force: true });
  await rm(`${indexPath}-shm`, { force: true });
}

export async function resolvePackPath(rootPath: string, pack: PackManifestEntry): Promise<string | null> {
  const candidates = [
    path.join(rootPath, pack.path),
    path.join(rootPath, "packs", "pf2e", pack.name),
    path.join(rootPath, pack.path.replace(/^packs\//, "packs/pf2e/")),
  ];

  for (const candidate of candidates) {
    const normalized = path.normalize(candidate);
    if (await directoryExists(normalized)) {
      return normalized;
    }
  }

  return null;
}
