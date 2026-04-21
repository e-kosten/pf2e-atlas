import { countMetadataPredicates, getMetadataNodeAtPath } from "../../search/query-core.js";
import { clampStructuredDraftSelection } from "../../search/structured-draft-session.js";
import type {
  SearchStructuredDraftAnchor,
  SearchStructuredDraftEntry,
  SearchStructuredDraftEntryKind,
} from "../../search/structured-draft-session.js";
import {
  getSearchQueryActionCostPolicy,
  getSearchQueryCategory,
  getSearchQueryMetadataTree,
  getSearchQueryRarityPolicy,
  getSearchQuerySubcategory,
} from "../../search/query-state.js";
import type {
  Pf2eTerminalFacetValueOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import {
  formatFilterPolicy,
  formatLevelRange,
  formatSearchCategory,
  formatSearchSubcategory,
} from "../model.js";

export type SearchStructuredDraftState = {
  anchor: SearchStructuredDraftAnchor;
  draftQuery: Pf2eTerminalSearchQuery;
  metadataFocusPath: number[] | null;
  selectedIndex: number;
};

export function buildStructuredDraftEntries(
  draftQuery: Pf2eTerminalSearchQuery,
  metadataFocusPath: number[] | null,
  options: {
    hasSelectableSubcategories: (category: Pf2eTerminalSearchQuery["filters"]["category"]) => boolean;
    getActionCostOptions: (
      category: Pf2eTerminalSearchQuery["filters"]["category"],
      subcategory: ReturnType<typeof getSearchQuerySubcategory>,
    ) => Pf2eTerminalFacetValueOption[];
  },
): SearchStructuredDraftEntry[] {
  const draftCategory = getSearchQueryCategory(draftQuery);
  const draftSubcategory = getSearchQuerySubcategory(draftQuery);
  const draftRarityPolicy = getSearchQueryRarityPolicy(draftQuery);
  const draftActionCostPolicy = getSearchQueryActionCostPolicy(draftQuery);
  const draftMetadataTree = getSearchQueryMetadataTree(draftQuery);
  const entries: SearchStructuredDraftEntry[] = [
    {
      kind: "category",
      key: "category",
      label: "Category",
      value: formatSearchCategory(draftCategory),
      description: "Choose the category boundary for the staged structured query.",
    },
  ];

  if (options.hasSelectableSubcategories(draftCategory)) {
    entries.push({
      kind: "subcategory",
      key: "subcategory",
      label: "Subcategory",
      value: formatSearchSubcategory(draftSubcategory),
      description: "Choose the staged subcategory boundary within the current category.",
    });
  }

  entries.push(
    {
      kind: "levelRange",
      key: "levelRange",
      label: "Level Range",
      value: formatLevelRange(draftQuery),
      description: "Constrain the staged query by minimum or maximum level.",
    },
    {
      kind: "rarity",
      key: "rarity",
      label: "Rarity",
      value: formatFilterPolicy(draftRarityPolicy),
      description: "Stage include or exclude rarity filters.",
    },
  );

  if (options.getActionCostOptions(draftCategory, draftSubcategory).length > 0) {
    entries.push({
      kind: "actionCost",
      key: "actionCost",
      label: "Action Cost",
      value: formatFilterPolicy(draftActionCostPolicy),
      description: "Stage include or exclude action-cost filters.",
    });
  }

  if (draftCategory) {
    const focusedNode = metadataFocusPath ? getMetadataNodeAtPath(draftMetadataTree, metadataFocusPath) : null;
    const predicateCount = countMetadataPredicates(draftMetadataTree);
    entries.push({
      kind: "metadata",
      key: "metadata",
      label: "Query Logic",
      value:
        predicateCount > 0
          ? `${predicateCount} staged clause${predicateCount === 1 ? "" : "s"}`
          : "No staged clauses",
      description: focusedNode
        ? `Resume staged metadata editing at ${metadataFocusPath?.length === 0 ? "the root query node" : `path ${metadataFocusPath?.join(".")}`}.`
        : "Stage metadata clauses and logic groups for the structured query.",
      metadataPath: metadataFocusPath ?? [],
    });
  }

  entries.push(
    {
      kind: "finish",
      key: "finish",
      label: "Apply Structured Edit",
      value: "Apply staged query",
      description: "Commit the staged structured query back into the live editor.",
    },
    {
      kind: "cancel",
      key: "cancel",
      label: "Discard Structured Edit",
      value: "Discard staged query",
      description: "Discard the staged structured query and keep the live query unchanged.",
    },
  );

  return entries;
}

function getStructuredDraftAnchorKind(
  anchor: SearchStructuredDraftAnchor,
  entries: SearchStructuredDraftEntry[],
): SearchStructuredDraftEntryKind {
  if (anchor.kind === "queryPart") {
    return anchor.part;
  }
  if (anchor.kind === "queryNode") {
    return "metadata";
  }

  const preferredKinds: SearchStructuredDraftEntryKind[] = [
    "category",
    "subcategory",
    "levelRange",
    "rarity",
    "actionCost",
    "metadata",
  ];
  const emptyKinds = new Set<SearchStructuredDraftEntryKind>();

  for (const entry of entries) {
    if (
      entry.kind === "category" ||
      entry.kind === "subcategory" ||
      entry.kind === "levelRange" ||
      entry.kind === "rarity" ||
      entry.kind === "actionCost" ||
      entry.kind === "metadata"
    ) {
      if (
        entry.value === "Any Category" ||
        entry.value === "Any Subcategory" ||
        entry.value === "(any)" ||
        entry.value === "No staged clauses"
      ) {
        emptyKinds.add(entry.kind);
      }
    }
  }

  return (
    preferredKinds.find((kind) => emptyKinds.has(kind) && entries.some((entry) => entry.kind === kind)) ?? "metadata"
  );
}

export function getStructuredDraftSelectionIndex(
  anchor: SearchStructuredDraftAnchor,
  entries: SearchStructuredDraftEntry[],
): number {
  const preferredKind = getStructuredDraftAnchorKind(anchor, entries);
  const entryIndex = entries.findIndex((entry) => entry.kind === preferredKind);
  return clampStructuredDraftSelection(entryIndex >= 0 ? entryIndex : 0, entries.length);
}
