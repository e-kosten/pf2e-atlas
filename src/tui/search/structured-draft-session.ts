import type { Pf2eTerminalSearchQuery } from "./service.js";
import type { Pf2eTerminalSearchQueryBase } from "./query-projection.js";

export type SearchStructuredDraftAnchor =
  | { kind: "addQueryPart" }
  | { kind: "queryTreeRoot" }
  | {
      kind: "queryNode";
      path: number[];
    };

export type SearchStructuredDraftEntryKind =
  | "category"
  | "subcategory"
  | "levelRange"
  | "rarity"
  | "actionCost"
  | "queryTreeRoot"
  | "queryNode"
  | "queryInsertionSlot"
  | "finish"
  | "cancel";

export type SearchStructuredDraftEntry = {
  kind: SearchStructuredDraftEntryKind;
  key: string;
  label: string;
  value?: string;
  description: string;
  disabled?: boolean;
  disabledReason?: string;
  treePath?: number[];
  insertionPath?: number[];
  indent?: number;
  menuLabel?: string;
  metadataPath?: number[];
};

export type SearchStructuredDraftSession = {
  kind: "structuredDraft";
  anchor: SearchStructuredDraftAnchor;
  baseQuery: Pf2eTerminalSearchQueryBase;
  draftFilter: Pf2eTerminalSearchQuery["filter"];
  entries: SearchStructuredDraftEntry[];
  selectedIndex: number;
  metadataFocusPath: number[] | null;
};

export function clampStructuredDraftSelection(index: number, itemCount: number): number {
  if (itemCount <= 0) {
    return 0;
  }
  return Math.max(0, Math.min(index, itemCount - 1));
}
