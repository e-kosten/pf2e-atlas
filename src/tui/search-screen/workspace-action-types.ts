import type { Pf2eTerminalAppServices } from "../app-services.js";
import type { SearchTerminalPromptAdapters } from "../interaction-context-adapters.js";
import type {
  Pf2eTerminalFilterExplorerDraft,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../search/service.js";
import type { DerivedTagTerminalApp } from "../framework/types.js";

export type SearchWorkspacePromptAdapters = Pick<
  SearchTerminalPromptAdapters,
  | "promptCommandPalette"
  | "promptOptionalSelectOption"
  | "promptPolicySelectOption"
  | "promptSelectOption"
  | "promptTextInput"
>;

export type SearchWorkspaceTerminal = Pick<DerivedTagTerminalApp, "pauseForAnyKey">;

export type SearchWorkspaceUser = Pick<Pf2eTerminalAppServices["user"], "search">;

export type OpenSearchFilterExplorer = (options: {
  queryOverride?: Pf2eTerminalSearchQuery;
  fieldOptions: Pf2eTerminalQueryFieldOption[];
  initialDraft?: Pf2eTerminalFilterExplorerDraft;
  onApply: (draft: Pf2eTerminalFilterExplorerDraft) => void;
  onReturn?: () => void;
  singleFieldBehavior?: "list" | "directValues";
}) => Promise<boolean>;
