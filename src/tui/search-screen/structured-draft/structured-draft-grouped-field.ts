import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import {
  getSearchFilterNodeAtPath,
  isSearchFilterBooleanGroup,
  updateSearchFilterNodeAtPath,
} from "../../search/query-core.js";
import { metadataFilterNodeToCanonicalFilter } from "../../search/query-parts.js";
import {
  getSearchQueryMetadataTree,
  setSearchQueryMetadataTree,
} from "../../search/query-state.js";
import type {
  Pf2eTerminalFilterExplorerDraft,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import {
  buildSearchFilterExplorerComposeDraft,
  buildSearchFilterExplorerFieldState,
  type SearchFilterExplorerFieldState,
} from "../filter-explorer-field-state.js";
import { getGroupedFieldChildIndexes } from "./structured-draft-grouped-paths.js";

export type StructuredDraftGroupedFieldSearchAdapter = {
  applyDiscoverableQueryFieldSelections(
    query: Pf2eTerminalSearchQuery,
    selections: Record<string, { include: string[]; exclude: string[] }>,
    fieldOrder: string[],
  ): Pf2eTerminalSearchQuery;
};

export function buildGroupedFieldSeedDiscreteClauses(
  node: SearchFilterNode | undefined,
  field: string | undefined,
  operator: "include" | "exclude" = "include",
): Pf2eTerminalFilterExplorerDraft["discreteClauses"] {
  if (!node || !field) {
    return [];
  }

  if (field === "rarity" || field === "actionCost") {
    if (node.kind === field && node.match.kind === "eq") {
      return [{ field, value: String(node.match.value), operator }];
    }
    if (node.kind === "anyOf") {
      return node.children.flatMap((child) => buildGroupedFieldSeedDiscreteClauses(child, field, operator));
    }
    if (node.kind === "not") {
      return buildGroupedFieldSeedDiscreteClauses(node.child, field, "exclude");
    }
    return [];
  }

  if (node.kind === "metadataPredicate" && node.predicate.field === field && "value" in node.predicate) {
    return [{ field, value: String(node.predicate.value), operator }];
  }
  if (node.kind === "anyOf") {
    return node.children.flatMap((child) => buildGroupedFieldSeedDiscreteClauses(child, field, operator));
  }
  if (node.kind === "not") {
    return buildGroupedFieldSeedDiscreteClauses(node.child, field, "exclude");
  }

  return [];
}

function buildGroupedFieldSeedGroupNode(
  groupNode: Extract<SearchFilterNode, { kind: "allOf" } | { kind: "anyOf" }>,
  removedChildIndexes: ReadonlySet<number>,
): SearchFilterNode | undefined {
  const remainingChildren = groupNode.children.filter((_child, childIndex) => !removedChildIndexes.has(childIndex));
  if (remainingChildren.length === 0) {
    return undefined;
  }
  if (remainingChildren.length === 1) {
    return remainingChildren[0];
  }
  return {
    kind: groupNode.kind,
    children: remainingChildren,
  };
}

export function buildGroupedFieldReplacementNodes(
  searchUser: StructuredDraftGroupedFieldSearchAdapter,
  query: Pf2eTerminalSearchQuery,
  fieldState: SearchFilterExplorerFieldState,
  fieldOption: Pf2eTerminalQueryFieldOption,
): SearchFilterNode[] {
  const field = fieldOption.value;
  const selection = fieldState.discreteSelections[field] ?? { include: [], exclude: [] };
  const draft = buildSearchFilterExplorerComposeDraft(fieldState);
  const discreteClauses = draft.discreteClauses.filter((clause) => clause.field === field);

  if (field === "rarity") {
    return discreteClauses.map((clause) =>
      clause.operator === "include"
        ? ({ kind: "rarity", match: { kind: "eq", value: clause.value } } satisfies SearchFilterNode)
        : ({
            kind: "not",
            child: { kind: "rarity", match: { kind: "eq", value: clause.value } },
          } satisfies SearchFilterNode),
    );
  }

  if (field === "actionCost") {
    const replacementNodes: SearchFilterNode[] = [];
    for (const clause of discreteClauses) {
      const numericValue = Number.parseInt(clause.value, 10);
      if (!Number.isFinite(numericValue)) {
        continue;
      }
      replacementNodes.push(
        clause.operator === "include"
          ? ({ kind: "actionCost", match: { kind: "eq", value: numericValue } } satisfies SearchFilterNode)
          : ({
              kind: "not",
              child: { kind: "actionCost", match: { kind: "eq", value: numericValue } },
            } satisfies SearchFilterNode),
      );
    }
    return replacementNodes;
  }

  if (fieldOption.fieldType === "set") {
    return discreteClauses.flatMap((clause) => {
      const selection =
        clause.operator === "include"
          ? { include: [clause.value], exclude: [] }
          : { include: [], exclude: [clause.value] };
      const replacementNode = metadataFilterNodeToCanonicalFilter(
        getSearchQueryMetadataTree(
          searchUser.applyDiscoverableQueryFieldSelections(
            setSearchQueryMetadataTree(query, null),
            { [field]: selection },
            [field],
          ),
        ),
      );
      return replacementNode ? [replacementNode] : [];
    });
  }

  const replacementNode = metadataFilterNodeToCanonicalFilter(
    getSearchQueryMetadataTree(
      searchUser.applyDiscoverableQueryFieldSelections(
        setSearchQueryMetadataTree(query, null),
        { [field]: selection },
        [field],
      ),
    ),
  );
  return replacementNode ? [replacementNode] : [];
}

export function buildGroupedFieldSeedState(
  query: Pf2eTerminalSearchQuery,
  groupPath: number[],
  options?: {
    field?: string;
    fieldMemberPaths?: number[][];
  },
): {
  seedGroupPath: number[];
  seedQuery: Pf2eTerminalSearchQuery;
  initialFieldState: SearchFilterExplorerFieldState;
  preservedMetadata: MetadataFilterNode | null;
} {
  const groupNode =
    groupPath.length === 0 ? query.filter : (getSearchFilterNodeAtPath(query.filter, groupPath) ?? undefined);
  if (!groupNode) {
    return {
      seedGroupPath: [],
      seedQuery: query,
      initialFieldState: buildSearchFilterExplorerFieldState({
        discreteClauses: [],
        scalarClauses: {},
      }),
      preservedMetadata: getSearchQueryMetadataTree(query),
    };
  }

  const fieldChildIndexes = new Set<number>(getGroupedFieldChildIndexes(groupPath, options?.fieldMemberPaths ?? []));

  const initialDraft: Pf2eTerminalFilterExplorerDraft = {
    discreteClauses: [...fieldChildIndexes]
      .sort((left, right) => left - right)
      .flatMap((childIndex) =>
        buildGroupedFieldSeedDiscreteClauses(
          isSearchFilterBooleanGroup(groupNode) ? groupNode.children[childIndex] : undefined,
          options?.field,
        ),
      ),
    scalarClauses: {},
  };

  const seedGroupNode =
    isSearchFilterBooleanGroup(groupNode) && fieldChildIndexes.size > 0
      ? buildGroupedFieldSeedGroupNode(groupNode, fieldChildIndexes)
      : groupNode;

  const seedQuery = {
    ...query,
    filter:
      groupPath.length === 0
        ? seedGroupNode
        : updateSearchFilterNodeAtPath(query.filter, groupPath, () => seedGroupNode),
  };
  return {
    seedGroupPath: groupPath,
    seedQuery,
    initialFieldState: buildSearchFilterExplorerFieldState(initialDraft),
    preservedMetadata: getSearchQueryMetadataTree(seedQuery),
  };
}
