import { buildAllOfFilter, buildScopeFilter, type SearchRequest } from "../domain/search-request-types.js";
import type { PageRelationGroup, PageRelationsResult, PageReferenceCollectionResult } from "../domain/page-relations-types.js";
import type { RecordKey } from "../domain/record-types.js";
import type { SearchCategory, SearchSubcategory } from "../domain/search-types.js";
import type { Pf2eDataService } from "../data/service.js";

type PageRelationsDataService = Pick<Pf2eDataService, "getReferenceEdges">;

export type Pf2eApplicationPageRelationsService = {
  loadPageRelations: (recordKey: RecordKey) => PageRelationsResult;
};

type GroupBucket = {
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  count: number;
};

function buildIncomingGroupRequest(
  recordKey: RecordKey,
  group: Pick<GroupBucket, "category" | "subcategory">,
): SearchRequest {
  return {
    mode: "browse",
    filter: buildAllOfFilter([
      buildScopeFilter(group.category, group.subcategory),
      { kind: "linkedFrom", source: recordKey },
    ]),
    sort: { kind: "alphabetical" },
    limit: 50,
  };
}

function buildIncomingGroups(recordKey: RecordKey, relations: PageReferenceCollectionResult): PageRelationGroup[] {
  const groups = new Map<string, GroupBucket>();
  for (const record of relations.incoming.records) {
    const key = `${record.category}|${record.subcategory ?? ""}`;
    const bucket = groups.get(key) ?? {
      category: record.category,
      subcategory: record.subcategory,
      count: 0,
    };
    bucket.count += 1;
    groups.set(key, bucket);
  }

  return [...groups.values()]
    .sort((left, right) => right.count - left.count || left.category.localeCompare(right.category))
    .map((group) => ({
      ...group,
      request: buildIncomingGroupRequest(recordKey, group),
    }));
}

export function createPf2eApplicationPageRelationsService(
  dataService: PageRelationsDataService,
): Pf2eApplicationPageRelationsService {
  return {
    loadPageRelations: (recordKey) => {
      const relations = dataService.getReferenceEdges([recordKey], {
        includeOutgoing: true,
        includeIncoming: true,
      });

      return {
        recordKey,
        ...relations,
        incomingGroups: buildIncomingGroups(recordKey, relations),
      };
    },
  };
}
