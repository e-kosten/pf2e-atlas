import type { Pf2eTerminalSearchQuery } from "./service.js";

export type SearchStructuredDraftAnchor =
  | { kind: "addQueryPart" }
  | {
      kind: "queryPart";
      part: "category" | "subcategory" | "levelRange" | "rarity" | "actionCost";
    }
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
  | "metadata"
  | "finish"
  | "cancel";

export type SearchStructuredDraftEntry = {
  kind: SearchStructuredDraftEntryKind;
  key: string;
  label: string;
  value: string;
  description: string;
  disabled?: boolean;
  disabledReason?: string;
  metadataPath?: number[];
};

export type SearchStructuredDraftSession = {
  kind: "structuredDraft";
  anchor: SearchStructuredDraftAnchor;
  draftQuery: Pf2eTerminalSearchQuery;
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
