import type { OntologyTextLine } from "../../domain/ontology-types.js";
import type { NormalizedRecord, RecordKey } from "../../domain/record-types.js";
import type { PageRelationsResult } from "../../domain/page-relations-types.js";
import { buildScopeFilter, type SearchRequest } from "../../domain/search-request-types.js";
import {
  buildEntityPageDocument,
  renderEntityPageDocument,
  type EntityPageDocument,
} from "./entity-page.js";
import {
  mapNormalizedRecordToOntologyExplorerEntityRecord,
  type OntologyExplorerEntityRecord,
} from "./entity-record.js";

type EntityPageSourceRecord = NormalizedRecord | OntologyExplorerEntityRecord;

type EntityPageRelationsService = {
  loadPageRelations: (recordKey: RecordKey) => PageRelationsResult;
  getRecord?: (recordKey: RecordKey) => NormalizedRecord | undefined;
};

export type Pf2eApplicationEntityPageService = {
  buildDocument: (record: EntityPageSourceRecord) => EntityPageDocument;
  buildDocumentByRecordKey: (recordKey: RecordKey) => EntityPageDocument | null;
  buildLookupRequestByRecordKey: (recordKey: RecordKey) => SearchRequest | null;
  buildDetailLines: (
    record: EntityPageSourceRecord,
    options?: { includeHeader?: boolean },
  ) => OntologyTextLine[];
};

function isNormalizedRecord(record: EntityPageSourceRecord): record is NormalizedRecord {
  return "normalizedName" in record;
}

function toEntityRecord(record: EntityPageSourceRecord): OntologyExplorerEntityRecord {
  return isNormalizedRecord(record) ? mapNormalizedRecordToOntologyExplorerEntityRecord(record) : record;
}

export function createPf2eApplicationEntityPageService(
  relationsService: EntityPageRelationsService,
): Pf2eApplicationEntityPageService {
  const buildDocument = (record: EntityPageSourceRecord): EntityPageDocument => {
    const entityRecord = toEntityRecord(record);
    const relations = relationsService.loadPageRelations(entityRecord.recordKey);
    return buildEntityPageDocument(entityRecord, relations);
  };
  const buildDocumentByRecordKey = (recordKey: RecordKey): EntityPageDocument | null => {
    const record = relationsService.getRecord?.(recordKey);
    return record ? buildDocument(record) : null;
  };
  const buildLookupRequestByRecordKey = (recordKey: RecordKey): SearchRequest | null => {
    const record = relationsService.getRecord?.(recordKey);
    if (!record) {
      return null;
    }

    return {
      mode: "lookup",
      search: {
        query: record.name,
      },
      filter: buildScopeFilter(record.category, record.subcategory),
      limit: 5,
    };
  };

  return {
    buildDocument,
    buildDocumentByRecordKey,
    buildLookupRequestByRecordKey,
    buildDetailLines: (record, options) => renderEntityPageDocument(buildDocument(record), options),
  };
}
