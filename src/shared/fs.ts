import { constants } from "node:fs";
import { access, stat } from "node:fs/promises";

export async function pathExists(targetPath: string): Promise<boolean> {
  try {
    await stat(targetPath);
    return true;
  } catch {
    return false;
  }
}

export async function pathIsReadable(targetPath: string): Promise<boolean> {
  try {
    await access(targetPath, constants.R_OK);
    return true;
  } catch {
    return false;
  }
}

export async function fileExists(targetPath: string): Promise<boolean> {
  return pathExists(targetPath);
}
