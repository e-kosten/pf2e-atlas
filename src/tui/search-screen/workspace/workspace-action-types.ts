import type { Pf2eTerminalAppServices } from "../../app-services.js";
import type { SearchTerminalPromptAdapters } from "../../interaction-context-adapters.js";
import type {
  Pf2eTerminalFilterExplorerDraft,
  Pf2eTerminalPreparedFilterExplorerContext,
  Pf2eTerminalPreparedFilterExplorerDraft,
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
  fieldOptions: Pf2eTerminalQueryFieldOption[];
  initialPreparedDraft?: Pf2eTerminalPreparedFilterExplorerDraft;
  onDraftChange?: (
    draft: Pf2eTerminalFilterExplorerDraft,
    context: Pf2eTerminalPreparedFilterExplorerContext,
  ) => void;
  onApply: (
    draft: Pf2eTerminalFilterExplorerDraft,
    context: Pf2eTerminalPreparedFilterExplorerContext,
  ) => void;
  onReturn?: () => void;
  singleFieldBehavior?: "list" | "directValues";
}) => Promise<boolean>;
