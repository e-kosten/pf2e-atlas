import type { MetadataFilterNode, SearchFilters, SearchProfile, SearchSort } from "../../domain/index.js";
import {
  getSearchQueryActionCostPolicy,
  getSearchQueryCategory,
  getSearchQueryLevelRange,
  getSearchQueryMetadataTree,
  getSearchQueryRarityPolicy,
  getSearchQuerySubcategory,
} from "./query-state.js";
import type { Pf2eTerminalSearchQuery } from "./service-types.js";

function buildDiscreteFilterNodes(query: Pf2eTerminalSearchQuery): MetadataFilterNode[] {
  const nodes: MetadataFilterNode[] = [];
  const rarityPolicy = getSearchQueryRarityPolicy(query);
  const actionCostPolicy = getSearchQueryActionCostPolicy(query);

  if (rarityPolicy.any.length === 1) {
    nodes.push({
      field: "rarity",
      op: "eq",
      value: rarityPolicy.any[0]!,
    });
  } else if (rarityPolicy.any.length > 1) {
    nodes.push({
      field: "rarity",
      op: "in",
      values: rarityPolicy.any,
    });
  }

  if (rarityPolicy.exclude.length > 0) {
    nodes.push({
      field: "rarity",
      op: "notIn",
      values: rarityPolicy.exclude,
    });
  }

  if (actionCostPolicy.any.length === 1) {
    nodes.push({
      field: "actionCost",
      op: "eq",
      value: actionCostPolicy.any[0]!,
    });
  } else if (actionCostPolicy.any.length > 1) {
    nodes.push({
      or: actionCostPolicy.any.map((value) => ({
        field: "actionCost",
        op: "eq",
        value,
      })),
    });
  }

  if (actionCostPolicy.exclude.length === 1) {
    nodes.push({
      not: {
        field: "actionCost",
        op: "eq",
        value: actionCostPolicy.exclude[0]!,
      },
    });
  } else if (actionCostPolicy.exclude.length > 1) {
    nodes.push({
      not: {
        or: actionCostPolicy.exclude.map((value) => ({
          field: "actionCost",
          op: "eq",
          value,
        })),
      },
    });
  }

  return nodes;
}

export function buildSearchFilters(
  query: Pf2eTerminalSearchQuery,
  options: {
    limit?: number;
    offset?: number;
    query?: string;
    nameQuery?: string;
    searchProfile?: SearchProfile;
    sort?: SearchSort;
    sortSeed?: number | null;
  } = {},
): SearchFilters {
  const metadataTree = getSearchQueryMetadataTree(query);
  const metadataClauses = [...buildDiscreteFilterNodes(query), ...(metadataTree ? [metadataTree] : [])];
  const metadata =
    metadataClauses.length === 0
      ? undefined
      : metadataClauses.length === 1
        ? metadataClauses[0]
        : { and: metadataClauses };
  return {
    category: getSearchQueryCategory(query) ?? undefined,
    subcategory: getSearchQuerySubcategory(query) ?? undefined,
    levelMin: getSearchQueryLevelRange(query).levelMin ?? undefined,
    levelMax: getSearchQueryLevelRange(query).levelMax ?? undefined,
    rarity: undefined,
    actionCost: undefined,
    metadata,
    limit: options.limit ?? query.limit,
    offset: options.offset ?? 0,
    query: options.query,
    nameQuery: options.nameQuery,
    searchProfile: options.searchProfile,
    sort: options.sort,
    sortSeed: options.sortSeed ?? undefined,
  };
}
