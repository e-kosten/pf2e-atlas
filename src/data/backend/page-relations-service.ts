import type { DatabaseSync } from "node:sqlite";

import type { PageReferenceCollectionResult, PageReferenceEdge, PageReferenceResult } from "../../domain/page-relations-types.js";
import type { RecordKey } from "../../domain/record-types.js";
import { fetchPageReferenceEdgeRows } from "../record-queries.js";
import { edgeRowToPageReferenceEdge } from "../rows.js";
import type { Pf2eRecordCatalog } from "./record-catalog.js";

function sortReferenceEdges(edges: PageReferenceEdge[]): PageReferenceEdge[] {
  return [...edges].sort((left, right) => {
    const leftLabel =
      left.displayText ?? (left.direction === "outgoing" ? left.toRecordKey : left.fromRecordKey);
    const rightLabel =
      right.displayText ?? (right.direction === "outgoing" ? right.toRecordKey : right.fromRecordKey);
    return leftLabel.localeCompare(rightLabel) || left.referenceText.localeCompare(right.referenceText);
  });
}

export class Pf2ePageRelationsBackendService {
  constructor(
    private readonly db: DatabaseSync,
    private readonly catalog: Pick<Pf2eRecordCatalog, "getRecordsByKeys">,
  ) {}

  getReferenceEdges(
    recordKeys: readonly RecordKey[],
    options: {
      includeOutgoing?: boolean;
      includeIncoming?: boolean;
    } = {},
  ): PageReferenceCollectionResult {
    const uniqueRecordKeys = [...new Set(recordKeys)];
    const directionsSpecified = options.includeOutgoing !== undefined || options.includeIncoming !== undefined;
    const includeOutgoing = directionsSpecified ? options.includeOutgoing === true : true;
    const includeIncoming = directionsSpecified ? options.includeIncoming === true : true;
    const emptyResult: PageReferenceResult = { records: [], edges: [] };

    const outgoing = includeOutgoing ? this.collectDirection("outgoing", uniqueRecordKeys) : emptyResult;
    const incoming = includeIncoming ? this.collectDirection("incoming", uniqueRecordKeys) : emptyResult;

    return {
      outgoing,
      incoming,
      edges: [...outgoing.edges, ...incoming.edges],
    };
  }

  private collectDirection(
    direction: PageReferenceEdge["direction"],
    recordKeys: readonly RecordKey[],
  ): PageReferenceResult {
    if (recordKeys.length === 0) {
      return { records: [], edges: [] };
    }

    const rows = fetchPageReferenceEdgeRows(this.db, direction, recordKeys);
    const edges = sortReferenceEdges(rows.map((row) => edgeRowToPageReferenceEdge(row, direction)));
    const relatedRecordKeys = [
      ...new Set(edges.map((edge) => (direction === "outgoing" ? edge.toRecordKey : edge.fromRecordKey))),
    ];

    return {
      records: this.catalog.getRecordsByKeys(relatedRecordKeys),
      edges,
    };
  }
}
