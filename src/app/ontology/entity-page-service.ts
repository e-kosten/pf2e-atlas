import type { OntologyTextLine } from "../../domain/ontology-types.js";
import type { NormalizedRecord } from "../../domain/record-types.js";
import type { PageRelationsResult } from "../../domain/page-relations-types.js";
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
  loadPageRelations: (recordKey: string) => PageRelationsResult;
};

export type Pf2eApplicationEntityPageService = {
  buildDocument: (record: EntityPageSourceRecord) => EntityPageDocument;
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

  return {
    buildDocument,
    buildDetailLines: (record, options) => renderEntityPageDocument(buildDocument(record), options),
  };
}
