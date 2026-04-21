export {
  buildFilterExplorerMetadataNodeFromPolicy as buildMetadataNodeFromPolicy,
  buildFilterExplorerPolicyFromMetadataNode as buildPolicyFromMetadataNode,
  buildFilterExplorerPolicyFromPredicate as buildPolicyFromPredicate,
} from "../filter-explorer/search-draft.js";

import type { Pf2eTerminalFilterValuePolicy, Pf2eTerminalQueryFieldSelectionMap } from "../search/service.js";

export function buildQueryFieldSelectionMap(
  field: string,
  policy: Pf2eTerminalFilterValuePolicy<string>,
): Pf2eTerminalQueryFieldSelectionMap {
  return {
    [field]: {
      any: [...policy.any],
      all: [...policy.all],
      exclude: [...policy.exclude],
    },
  };
}
