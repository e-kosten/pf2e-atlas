import { access } from "node:fs/promises";
import { constants } from "node:fs";
import path from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

export interface RefreshResult {
  warning: string | null;
  summary: string;
}

async function exists(targetPath: string): Promise<boolean> {
  try {
    await access(targetPath, constants.R_OK);
    return true;
  } catch {
    return false;
  }
}

export async function refreshPf2eCheckout(rootPath: string): Promise<RefreshResult> {
  const gitDir = path.join(rootPath, ".git");
  if (!(await exists(gitDir))) {
    return {
      warning: null,
      summary: "Skipped PF2E data refresh because the vendored data path is not a git checkout.",
    };
  }

  try {
    const { stdout, stderr } = await execFileAsync("git", ["-C", rootPath, "pull", "--ff-only"], {
      timeout: 20_000,
    });

    const output = [stdout, stderr].filter(Boolean).join("\n").trim();
    return {
      warning: null,
      summary: output.length > 0 ? output : "PF2E data refresh completed.",
    };
  } catch (error) {
    const details = error as Error & { stdout?: string; stderr?: string };
    const output = [details.stdout, details.stderr, details.message].filter(Boolean).join("\n").trim();
    return {
      warning: `PF2E data refresh failed: ${output}`,
      summary: "PF2E data refresh failed; continuing with the existing local checkout.",
    };
  }
}
