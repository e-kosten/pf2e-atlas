import type { SearchRequest, SearchRequestPart } from "../domain/search-request-types.js";
import type { MetadataFilterNode } from "../domain/metadata-filter-types.js";
import type { SearchExecutionFilters } from "./contracts.js";
import { searchRequestPartsToMetadataFilterNode } from "../domain/search-request-types.js";

function buildDiscretePolicyNodes(request: SearchRequest): MetadataFilterNode[] {
  const nodes: MetadataFilterNode[] = [];

  for (const part of request.parts ?? []) {
    if (part.kind === "rarityPolicy") {
      if (part.policy.any.length === 1) {
        nodes.push({ field: "rarity", op: "eq", value: part.policy.any[0]! });
      } else if (part.policy.any.length > 1) {
        nodes.push({ field: "rarity", op: "in", values: part.policy.any });
      }

      if (part.policy.exclude.length > 0) {
        nodes.push({ field: "rarity", op: "notIn", values: part.policy.exclude });
      }
      continue;
    }

    if (part.kind === "actionCostPolicy") {
      if (part.policy.any.length === 1) {
        nodes.push({ field: "actionCost", op: "eq", value: part.policy.any[0]! });
      } else if (part.policy.any.length > 1) {
        nodes.push({
          or: part.policy.any.map((value) => ({
            field: "actionCost",
            op: "eq",
            value,
          })),
        });
      }

      if (part.policy.exclude.length === 1) {
        nodes.push({
          not: {
            field: "actionCost",
            op: "eq",
            value: part.policy.exclude[0]!,
          },
        });
      } else if (part.policy.exclude.length > 1) {
        nodes.push({
          not: {
            or: part.policy.exclude.map((value) => ({
              field: "actionCost",
              op: "eq",
              value,
            })),
          },
        });
      }
    }
  }

  return nodes;
}

function getSubcategoryPart(parts: readonly SearchRequestPart[]): SearchExecutionFilters["subcategory"] {
  return parts.find((part): part is Extract<SearchRequestPart, { kind: "subcategory" }> => part.kind === "subcategory")
    ?.subcategory;
}

function getLevelRange(
  parts: readonly SearchRequestPart[],
): Pick<SearchExecutionFilters, "levelMin" | "levelMax"> {
  const part = parts.find(
    (candidate): candidate is Extract<SearchRequestPart, { kind: "levelRange" }> => candidate.kind === "levelRange",
  );
  if (!part) {
    return {};
  }
  return {
    levelMin: part.levelMin ?? undefined,
    levelMax: part.levelMax ?? undefined,
  };
}

export function compileSearchRequest(request: SearchRequest): SearchExecutionFilters {
  const parts = request.parts ?? [];
  const metadataTree = searchRequestPartsToMetadataFilterNode(parts);
  const metadataClauses = [...buildDiscretePolicyNodes(request), ...(metadataTree ? [metadataTree] : [])];
  const metadata =
    metadataClauses.length === 0
      ? undefined
      : metadataClauses.length === 1
        ? metadataClauses[0]
        : { and: metadataClauses };
  const text = request.text?.trim();

  return {
    searchProfile: request.intent === "search" ? request.searchProfile : undefined,
    sort: request.sort,
    sortSeed: request.sortSeed,
    explain: request.explain,
    nameQuery: request.intent === "lookup" ? text : undefined,
    query: request.intent === "search" ? text : undefined,
    excludeQuery: request.excludeQuery,
    linksTo: request.linksTo,
    linksToMode: request.linksToMode,
    excludeLinksTo: request.excludeLinksTo,
    pack: request.pack,
    category: request.category,
    subcategory: getSubcategoryPart(parts),
    scopes: request.scopes,
    ...getLevelRange(parts),
    metadata,
    priceMin: request.priceMin,
    priceMax: request.priceMax,
    offset: request.offset,
    limit: request.limit,
  };
}
