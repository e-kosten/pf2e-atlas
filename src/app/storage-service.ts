import { DatabaseSync } from "node:sqlite";

import type { AppConfig } from "../domain/config-types.js";
import {
  buildDerivedTagOntologyExplorerModel,
  type DerivedTagOntologyExplorerModel,
} from "./ontology/derived-tag-explorer.js";

export type Pf2eApplicationStorageService = {
  openIndex: (_argv: string[]) => Promise<{ db: DatabaseSync; config: AppConfig }>;
  loadDerivedTagOntologyExplorerModel: () => DerivedTagOntologyExplorerModel;
};

function withOpenIndex<T>(config: AppConfig, operation: (db: DatabaseSync) => T): T {
  const db = new DatabaseSync(config.indexPath);
  try {
    return operation(db);
  } finally {
    db.close();
  }
}

export function createPf2eApplicationStorageService(config: AppConfig): Pf2eApplicationStorageService {
  return {
    openIndex: (_argv) =>
      Promise.resolve({
        config,
        db: new DatabaseSync(config.indexPath),
      }),
    loadDerivedTagOntologyExplorerModel: () =>
      withOpenIndex(config, (db) =>
        buildDerivedTagOntologyExplorerModel(db, {
          cacheKey: config.indexPath,
        }),
      ),
  };
}
