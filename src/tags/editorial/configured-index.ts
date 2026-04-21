import { constants } from "node:fs";
import { access } from "node:fs/promises";
import { DatabaseSync } from "node:sqlite";

import { loadConfig } from "../../app/config.js";

export async function openEditorialConfiguredIndex(
  argv: string[],
): Promise<{ db: DatabaseSync; config: Awaited<ReturnType<typeof loadConfig>> }> {
  const config = await loadConfig(argv);
  await access(config.indexPath, constants.R_OK);
  return {
    config,
    db: new DatabaseSync(config.indexPath),
  };
}

