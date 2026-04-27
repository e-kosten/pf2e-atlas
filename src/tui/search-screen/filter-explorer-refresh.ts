import type { OntologyDomainModel } from "../../domain/ontology-types.js";
import type { SearchFilterDiscoveryMode } from "../../domain/search-field-domains.js";

export type SearchFilterExplorerRefreshPlan =
  | {
      kind: "retainCurrent";
      requestId: number;
      mode: SearchFilterDiscoveryMode;
    }
  | {
      kind: "useCached";
      requestId: number;
      mode: SearchFilterDiscoveryMode;
      model: OntologyDomainModel;
    }
  | {
      kind: "load";
      requestId: number;
      pendingMode: SearchFilterDiscoveryMode;
    };

export function planSearchFilterExplorerRefresh(args: {
  nextMode: SearchFilterDiscoveryMode;
  displayedMode: SearchFilterDiscoveryMode;
  cache: ReadonlyMap<SearchFilterDiscoveryMode, OntologyDomainModel>;
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
