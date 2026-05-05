import type { Pf2eTerminalAppServices } from "../../app-services.js";
import type { SearchTerminalPromptAdapters } from "../../interaction-context-adapters.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import type { DerivedTagTerminalApp } from "../../framework/types.js";
import type { FilterExplorerComposeTarget } from "../../filter-explorer/types.js";
import type { SearchFilterExplorerFieldState } from "../filter-explorer-field-state.js";
import type { SearchFilterDiscoveryMode } from "../../../domain/search-field-domains.js";
import type { SearchFilterExplorerSessionEvent } from "../filter-explorer-session-events.js";

export type SearchWorkspacePromptAdapters = Pick<
  SearchTerminalPromptAdapters,
  | "promptMultiSelectOption"
  | "promptOptionalSelectOption"
  | "promptSelectOption"
  | "promptTextInput"
>;

export type SearchWorkspaceTerminal = Pick<DerivedTagTerminalApp, "pauseForAnyKey" | "runPromptSession">;

export type SearchWorkspaceUser = Pick<Pf2eTerminalAppServices["user"], "search">;

export type OpenSearchFilterExplorer = (options: {
  title?: string;
  queryOverride?: Pf2eTerminalSearchQuery;
  initialDiscoveryMode?: SearchFilterDiscoveryMode;
  initialFieldState?: SearchFilterExplorerFieldState;
  preservedMetadata?: MetadataFilterNode | null;
  fieldOptions: Pf2eTerminalQueryFieldOption[];
  resolveSelectionTarget?: (node: import("../../../domain/ontology-types.js").OntologyNode | undefined) => FilterExplorerComposeTarget | undefined;
  onEvent?: (event: SearchFilterExplorerSessionEvent) => void;
  singleFieldBehavior?: "list" | "directValues";
}) => Promise<boolean>;
