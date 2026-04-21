import { DatabaseSync } from "node:sqlite";

import type { AppConfig } from "../domain/config-types.js";

export type Pf2eApplicationStorageService = {
  openIndex: (_argv: string[]) => Promise<{ db: DatabaseSync; config: AppConfig }>;
};

export function createPf2eApplicationStorageService(config: AppConfig): Pf2eApplicationStorageService {
  return {
    openIndex: (_argv) =>
      Promise.resolve({
        config,
        db: new DatabaseSync(config.indexPath),
      }),
  };
}
