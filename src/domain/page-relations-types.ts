import type { NormalizedRecord, RecordKey, SourceCategory } from "./record-types.js";
import type { SearchRequest } from "./search-request-types.js";
import type { SearchCategory, SearchSubcategory } from "./search-types.js";

export interface PageReferenceEdge {
  fromRecordKey: RecordKey;
  toRecordKey: RecordKey;
  displayText: string | null;
  referenceText: string;
  direction: "outgoing" | "incoming";
  relationshipType: "references" | "referenced_by";
  sourcePackName: string;
  sourceRecordType: string;
  sourceDocumentType: string;
  sourceCategory: SourceCategory;
}

export interface PageReferenceResult {
  records: NormalizedRecord[];
  edges: PageReferenceEdge[];
}

export interface PageReferenceCollectionResult {
  outgoing: PageReferenceResult;
  incoming: PageReferenceResult;
  edges: PageReferenceEdge[];
}

export interface PageRelationGroup {
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  count: number;
  request: SearchRequest;
}

export interface PageRelationsResult extends PageReferenceCollectionResult {
  recordKey: RecordKey;
  incomingGroups: PageRelationGroup[];
}
