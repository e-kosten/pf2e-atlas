import { constants } from "node:fs";
import { access } from "node:fs/promises";
import { DatabaseSync } from "node:sqlite";

import { loadConfig } from "./config.js";
import type { AppConfig } from "../domain/config-types.js";

export type Pf2eApplicationIndexHandle = {
  close: () => void;
  config: AppConfig;
  db: DatabaseSync;
};

export type Pf2eApplicationStorageService = {
  config: AppConfig;
  openIndex: () => Promise<Pf2eApplicationIndexHandle>;
};

async function assertReadableIndexPath(indexPath: string): Promise<void> {
  await access(indexPath, constants.R_OK);
}

function createPf2eApplicationIndexHandle(config: AppConfig): Pf2eApplicationIndexHandle {
  const db = new DatabaseSync(config.indexPath);
  let isClosed = false;
  return {
    close: () => {
      if (isClosed) {
        return;
      }
      isClosed = true;
      db.close();
    },
    config,
    db,
  };
}

export async function openPf2eApplicationIndex(config: AppConfig): Promise<Pf2eApplicationIndexHandle> {
  await assertReadableIndexPath(config.indexPath);
  return createPf2eApplicationIndexHandle(config);
}

export async function openConfiguredPf2eApplicationIndex(argv: string[]): Promise<Pf2eApplicationIndexHandle> {
  return openPf2eApplicationIndex(await loadConfig(argv));
}

export function createPf2eApplicationStorageService(config: AppConfig): Pf2eApplicationStorageService {
  return {
    config,
    openIndex: () => openPf2eApplicationIndex(config),
  };
}
