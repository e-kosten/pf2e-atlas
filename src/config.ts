import { access } from "node:fs/promises";
import path from "node:path";
import { constants } from "node:fs";

import { AppConfig } from "./types.js";
import { expandHome } from "./utils.js";

function parseCliArgs(argv: string[]): Record<string, string> {
  const parsed: Record<string, string> = {};

  for (let index = 0; index < argv.length; index += 1) {
    const current = argv[index];
    if (!current || !current.startsWith("--")) {
      continue;
    }

    const [rawKey, inlineValue] = current.slice(2).split("=", 2);
    if (!rawKey) {
      continue;
    }

    const nextValue = inlineValue ?? argv[index + 1];
    if (!nextValue || nextValue.startsWith("--")) {
      continue;
    }

    parsed[rawKey] = nextValue;
    if (inlineValue === undefined) {
      index += 1;
    }
  }

  return parsed;
}

async function canRead(targetPath: string): Promise<boolean> {
  try {
    await access(targetPath, constants.R_OK);
    return true;
  } catch {
    return false;
  }
}

export async function loadConfig(argv = process.argv.slice(2), env = process.env): Promise<AppConfig> {
  const args = parseCliArgs(argv);
  const configuredPath = args["data-path"] ?? env.PF2E_DATA_PATH ?? path.join(process.cwd(), "vendor", "pf2e");

  const rootPath = path.resolve(expandHome(configuredPath));
  const manifestCandidates = [
    path.join(rootPath, "system.pf2e.json"),
    path.join(rootPath, "static", "system.json"),
  ];

  for (const candidate of manifestCandidates) {
    if (await canRead(candidate)) {
      return {
        dataPath: configuredPath,
        rootPath,
        manifestPath: candidate,
      };
    }
  }

  throw new Error(
    `Could not find a readable PF2E system manifest under ${rootPath}. Clone the PF2E repo into vendor/pf2e or pass --data-path /path/to/pf2e.`,
  );
}
