import { DatabaseSync } from "node:sqlite";

import type { AppConfig } from "../../types.js";
import {
  buildDerivedTagOntologyExplorerModel,
  type DerivedTagOntologyExplorerModel,
} from "./data.js";

export type Pf2eTerminalOntologyExplorerService = {
  loadModel: () => DerivedTagOntologyExplorerModel;
};

export function createPf2eTerminalOntologyExplorerService(
  config: AppConfig,
): Pf2eTerminalOntologyExplorerService {
  return {
    loadModel: () => {
      const db = new DatabaseSync(config.indexPath);
      try {
        return buildDerivedTagOntologyExplorerModel(db, { cacheKey: config.indexPath });
      } finally {
        db.close();
      }
    },
  };
}
