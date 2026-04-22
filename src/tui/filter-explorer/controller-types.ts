import type { DerivedTagTerminalListNavigationAction } from "../framework/input.js";
import type { DerivedTagTerminalInputEvent } from "../framework/types.js";
import type {
  TerminalInteractionAction,
  TerminalTextEntryIntent,
} from "../interaction-bindings.js";
import type { FilterExplorerBrowserContext } from "./types.js";

export type FilterExplorerKeyContext = FilterExplorerBrowserContext & {
  event: DerivedTagTerminalInputEvent;
};

export type FilterExplorerInteractionRoute = {
  event: DerivedTagTerminalInputEvent;
  interactionAction?: TerminalInteractionAction;
  searchModeAction?: { id: "cancel" };
  textEntryIntent?: TerminalTextEntryIntent;
  listNavigationAction?: DerivedTagTerminalListNavigationAction;
  detailNavigationAction?: DerivedTagTerminalListNavigationAction;
};
