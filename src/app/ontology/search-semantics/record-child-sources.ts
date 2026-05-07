import type { OntologyNode } from "../../../domain/ontology-types.js";
import { buildNormalizedRecordNode } from "../node-helpers.js";
import type { SearchSemanticsRecordsDataService } from "./types.js";

export function buildQueryRecordChildren(
  dataService: SearchSemanticsRecordsDataService,
  query: OntologyNode["query"] | undefined,
): readonly OntologyNode[] {
  if (!query) {
    return [];
  }

  const request = query.request;
  if (request.mode !== "browse") {
    return [];
  }

  return dataService.listRecords(request).records.map(buildNormalizedRecordNode);
}

async function buildQueryRecordChildrenAsync(
  dataService: SearchSemanticsRecordsDataService,
  query: OntologyNode["query"] | undefined,
): Promise<readonly OntologyNode[]> {
  if (!query) {
    return [];
  }

  if (query.request.mode === "browse") {
    return buildQueryRecordChildren(dataService, query);
  }

  const search = dataService.search;
  if (!search) {
    return [];
  }

  return (await search(query.request)).records.map(buildNormalizedRecordNode);
}

export function buildQueryRecordChildSource(
  dataService: SearchSemanticsRecordsDataService,
  query: NonNullable<OntologyNode["query"]>,
): OntologyNode["childSource"] {
  return { kind: "lazy", load: () => buildQueryRecordChildrenAsync(dataService, query) };
}
