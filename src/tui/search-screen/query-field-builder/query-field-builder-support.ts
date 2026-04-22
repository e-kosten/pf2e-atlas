import type { MetadataFilterNode } from "../../../domain/metadata-filter-types.js";
import {
  appendMetadataNodeAtPath,
  normalizeMetadataNode,
} from "../../search/query-core.js";
import { getSearchQueryMetadataTree, setSearchQueryMetadataTree } from "../../search/query-state.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import type { SearchQueryFieldBuilderSession } from "./query-field-builder-session.js";

export type QueryFieldBuilderState = {
  draftQuery: Pf2eTerminalSearchQuery;
  path: number[];
  items: SearchQueryFieldBuilderSession["items"];
  selectedIndex: number;
  fieldDrafts: Record<string, MetadataFilterNode | null>;
};

export function buildQueryFieldBuilderItems(
  fieldOptions: Pf2eTerminalQueryFieldOption[],
): SearchQueryFieldBuilderSession["items"] {
  return [
    ...fieldOptions.map((fieldOption) => ({
      kind: "field" as const,
      fieldOption,
      label: fieldOption.label,
    })),
    { kind: "finish" as const, label: "Return to Staged Query" },
    { kind: "cancel" as const, label: "Discard Field Edits" },
  ];
}

export function compileQueryFieldBuilderDrafts(
  fieldDrafts: Record<string, MetadataFilterNode | null>,
): MetadataFilterNode | null {
  const nodes = Object.values(fieldDrafts)
    .map((node) => normalizeMetadataNode(node))
    .filter((node): node is MetadataFilterNode => Boolean(node));

  if (nodes.length === 0) {
    return null;
  }
  if (nodes.length === 1) {
    return nodes[0]!;
  }
  return { and: nodes };
}

export function buildQueryFieldBuilderPreviewQuery(state: QueryFieldBuilderState): Pf2eTerminalSearchQuery {
  const stagedNode = compileQueryFieldBuilderDrafts(state.fieldDrafts);
  if (!stagedNode) {
    return state.draftQuery;
  }

  return setSearchQueryMetadataTree(
    state.draftQuery,
    appendMetadataNodeAtPath(getSearchQueryMetadataTree(state.draftQuery), state.path, stagedNode),
  );
}

export function buildQueryFieldBuilderSessionItems(
  state: QueryFieldBuilderState,
): SearchQueryFieldBuilderSession["items"] {
  return state.items.map((item) =>
    item.kind === "field"
      ? {
          ...item,
          label: state.fieldDrafts[item.fieldOption.value] ? `${item.fieldOption.label} | staged` : item.fieldOption.label,
        }
      : item,
  );
}
