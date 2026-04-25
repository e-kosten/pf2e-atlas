import type { OntologyDomainModel } from "../../domain/ontology-types.js";
import type { FilterExplorerDiscoveryMode } from "../filter-explorer/types.js";

export type SearchFilterExplorerRefreshPlan =
  | {
      kind: "retainCurrent";
      requestId: number;
      mode: FilterExplorerDiscoveryMode;
    }
  | {
      kind: "useCached";
      requestId: number;
      mode: FilterExplorerDiscoveryMode;
      model: OntologyDomainModel;
    }
  | {
      kind: "load";
      requestId: number;
      pendingMode: FilterExplorerDiscoveryMode;
    };

export function planSearchFilterExplorerRefresh(args: {
  nextMode: FilterExplorerDiscoveryMode;
  displayedMode: FilterExplorerDiscoveryMode;
  cache: ReadonlyMap<FilterExplorerDiscoveryMode, OntologyDomainModel>;
  currentRequestId: number;
  force?: boolean;
}): SearchFilterExplorerRefreshPlan {
  const requestId = args.currentRequestId + 1;
  if (!args.force) {
    if (args.nextMode === args.displayedMode) {
      return {
        kind: "retainCurrent",
        requestId,
        mode: args.nextMode,
      };
    }

    const cachedModel = args.cache.get(args.nextMode);
    if (cachedModel) {
      return {
        kind: "useCached",
        requestId,
        mode: args.nextMode,
        model: cachedModel,
      };
    }
  }

  return {
    kind: "load",
    requestId,
    pendingMode: args.nextMode,
  };
}

export function shouldApplySearchFilterExplorerRefresh(args: {
  currentRequestId: number;
  completedRequestId: number;
}): boolean {
  return args.currentRequestId === args.completedRequestId;
}
