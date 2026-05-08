import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import {
  getSearchFilterNodeAtPath,
  isSearchFilterBooleanGroup,
  updateSearchFilterNodeAtPath,
} from "../../search/query-core.js";
import {
  getSearchQueryPredicateFilter,
  setSearchQueryPredicateFilter,
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
import { FILTER_EXPLORER_VOCABULARY } from "../../filter-explorer/types.js";
import { getGroupedFieldChildIndexes } from "./structured-draft-grouped-paths.js";
import { SEARCH_REQUEST_VOCABULARY } from "../../../domain/search-request-types.js";

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
  operator: (typeof FILTER_EXPLORER_VOCABULARY.DISCRETE_CLAUSE_OPERATOR)[keyof typeof FILTER_EXPLORER_VOCABULARY.DISCRETE_CLAUSE_OPERATOR] = FILTER_EXPLORER_VOCABULARY.DISCRETE_CLAUSE_OPERATOR.INCLUDE,
): Pf2eTerminalFilterExplorerDraft["discreteClauses"] {
  if (!node || !field) {
    return [];
  }

  if (field === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.PACK) {
    if (node.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.PACK) {
      return [{ field, value: node.value, operator }];
    }
    if (node.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF) {
      return node.children.flatMap((child) => buildGroupedFieldSeedDiscreteClauses(child, field, operator));
    }
    if (node.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT) {
      return buildGroupedFieldSeedDiscreteClauses(
        node.child,
        field,
        FILTER_EXPLORER_VOCABULARY.DISCRETE_CLAUSE_OPERATOR.EXCLUDE,
      );
    }
    return [];
  }

  if (
    field === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY ||
    field === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ACTION_COST
  ) {
    if (node.kind === field && node.match.kind === SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.EQ) {
      return [{ field, value: String(node.match.value), operator }];
    }
    if (
      field === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY &&
      node.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY &&
      node.match.kind === SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.IN
    ) {
      return node.match.values.map((value) => ({ field, value, operator }));
    }
    if (
      field === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY &&
      node.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY &&
      node.match.kind === SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.NOT_IN
    ) {
      return node.match.values.map((value) => ({
        field,
        value,
        operator: FILTER_EXPLORER_VOCABULARY.DISCRETE_CLAUSE_OPERATOR.EXCLUDE,
      }));
    }
    if (node.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF) {
      return node.children.flatMap((child) => buildGroupedFieldSeedDiscreteClauses(child, field, operator));
    }
    if (node.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT) {
      return buildGroupedFieldSeedDiscreteClauses(
        node.child,
        field,
        FILTER_EXPLORER_VOCABULARY.DISCRETE_CLAUSE_OPERATOR.EXCLUDE,
      );
    }
    return [];
  }

  if (
    node.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE &&
    node.predicate.field === field &&
    "value" in node.predicate
  ) {
    return [{ field, value: String(node.predicate.value), operator }];
  }
  if (node.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF) {
    return node.children.flatMap((child) => buildGroupedFieldSeedDiscreteClauses(child, field, operator));
  }
  if (node.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT) {
    return buildGroupedFieldSeedDiscreteClauses(
      node.child,
      field,
      FILTER_EXPLORER_VOCABULARY.DISCRETE_CLAUSE_OPERATOR.EXCLUDE,
    );
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

  if (field === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY) {
    return discreteClauses.map((clause) =>
      clause.operator === FILTER_EXPLORER_VOCABULARY.DISCRETE_CLAUSE_OPERATOR.INCLUDE
        ? ({
            kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY,
            match: { kind: SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.EQ, value: clause.value },
          } satisfies SearchFilterNode)
        : ({
            kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT,
            child: {
              kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY,
              match: { kind: SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.EQ, value: clause.value },
            },
          } satisfies SearchFilterNode),
    );
  }

  if (field === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.PACK) {
    return discreteClauses.map((clause) =>
      clause.operator === FILTER_EXPLORER_VOCABULARY.DISCRETE_CLAUSE_OPERATOR.INCLUDE
        ? ({ kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.PACK, value: clause.value } satisfies SearchFilterNode)
        : ({
            kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT,
            child: { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.PACK, value: clause.value },
          } satisfies SearchFilterNode),
    );
  }

  if (field === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ACTION_COST) {
    const replacementNodes: SearchFilterNode[] = [];
    for (const clause of discreteClauses) {
      const numericValue = Number.parseInt(clause.value, 10);
      if (!Number.isFinite(numericValue)) {
        continue;
      }
      replacementNodes.push(
        clause.operator === FILTER_EXPLORER_VOCABULARY.DISCRETE_CLAUSE_OPERATOR.INCLUDE
          ? ({
              kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ACTION_COST,
              match: { kind: SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.EQ, value: numericValue },
            } satisfies SearchFilterNode)
          : ({
              kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT,
              child: {
                kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ACTION_COST,
                match: { kind: SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.EQ, value: numericValue },
              },
            } satisfies SearchFilterNode),
      );
    }
    return replacementNodes;
  }

  if (fieldOption.fieldType === "set") {
    return discreteClauses.flatMap((clause) => {
      const selection =
        clause.operator === FILTER_EXPLORER_VOCABULARY.DISCRETE_CLAUSE_OPERATOR.INCLUDE
          ? { include: [clause.value], exclude: [] }
          : { include: [], exclude: [clause.value] };
      const replacementNode = getSearchQueryPredicateFilter(
        searchUser.applyDiscoverableQueryFieldSelections(
          setSearchQueryPredicateFilter(query, null),
          { [field]: selection },
          [field],
        ),
      );
      return replacementNode ? [replacementNode] : [];
    });
  }

  const replacementNode = getSearchQueryPredicateFilter(
    searchUser.applyDiscoverableQueryFieldSelections(
      setSearchQueryPredicateFilter(query, null),
      { [field]: selection },
      [field],
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
  preservedFilter: SearchFilterNode | null;
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
      preservedFilter: getSearchQueryPredicateFilter(query),
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
    preservedFilter: getSearchQueryPredicateFilter(seedQuery),
  };
}
