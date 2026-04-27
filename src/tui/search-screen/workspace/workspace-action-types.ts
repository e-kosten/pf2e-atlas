import type { Pf2eTerminalAppServices } from "../../app-services.js";
import type { SearchTerminalPromptAdapters } from "../../interaction-context-adapters.js";
import type {
  Pf2eTerminalFilterExplorerDraft,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import type { DerivedTagTerminalApp } from "../../framework/types.js";

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
  queryOverride?: Pf2eTerminalSearchQuery;
  initialDraft?: Pf2eTerminalFilterExplorerDraft;
  fieldOptions: Pf2eTerminalQueryFieldOption[];
  onQueryChange?: (query: Pf2eTerminalSearchQuery) => void;
  onReturn?: () => void;
  onCancel?: (query: Pf2eTerminalSearchQuery) => void;
  onBack?: (query: Pf2eTerminalSearchQuery) => void;
  onExitRoot?: (query: Pf2eTerminalSearchQuery) => void;
  singleFieldBehavior?: "list" | "directValues";
}) => Promise<boolean>;
