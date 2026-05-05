import type { SearchFilterDiscoveryMode } from "../../domain/search-field-domains.js";
import type { FilterExplorerSelectTargetOutcome } from "../filter-explorer/types.js";
import type { Pf2eTerminalSearchQuery } from "../search/service.js";
import type { SearchFilterExplorerFieldState } from "./filter-explorer-field-state.js";

export type SearchFilterExplorerSessionSnapshot = {
  query: Pf2eTerminalSearchQuery;
  fieldState: SearchFilterExplorerFieldState;
};

export type SearchFilterExplorerSessionEvent =
  | ({ kind: "change" } & SearchFilterExplorerSessionSnapshot)
  | ({ kind: "back" } & SearchFilterExplorerSessionSnapshot)
  | ({ kind: "exitRoot" } & SearchFilterExplorerSessionSnapshot)
  | ({ kind: "cancel" } & SearchFilterExplorerSessionSnapshot)
  | ({
      kind: "selectTarget";
      outcome: FilterExplorerSelectTargetOutcome;
      discoveryMode: SearchFilterDiscoveryMode;
    } & SearchFilterExplorerSessionSnapshot);
