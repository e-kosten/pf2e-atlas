import type { MetadataFilterNode } from "../types.js";
import type { Pf2eTerminalFilterValuePolicy, Pf2eTerminalSearchQuery } from "./search-service.js";

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
  moveSelection: (delta: number, itemCount: number) => void;
  setSelectedIndex: (index: number) => void;
  selectCurrent: () => void;
  finish: () => void;
  cancel: () => void;
  replaceDraftQuery: (query: Pf2eTerminalSearchQuery) => void;
  setCategory: (category: Pf2eTerminalSearchQuery["filters"]["category"]) => void;
  setSubcategory: (subcategory: string | null) => void;
  setLevelRange: (range: { levelMin: number | null; levelMax: number | null }) => void;
  setRarityPolicy: (policy: Pf2eTerminalFilterValuePolicy<string>) => void;
  setActionCostPolicy: (policy: Pf2eTerminalFilterValuePolicy<number>) => void;
  setMetadataTree: (node: MetadataFilterNode | null) => void;
};

export function clampStructuredDraftSelection(index: number, itemCount: number): number {
  if (itemCount <= 0) {
    return 0;
  }
  return Math.max(0, Math.min(index, itemCount - 1));
}
