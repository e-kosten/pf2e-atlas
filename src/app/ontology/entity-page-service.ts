import type { OntologyTextLine } from "../../domain/ontology-types.js";
import type { NormalizedRecord, RecordKey } from "../../domain/record-types.js";
import type { PageRelationsResult } from "../../domain/page-relations-types.js";
import {
  buildEntityPageDocument,
  renderEntityPageDocument,
  type EntityPageDocumentBuildOptions,
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
  buildDocument: (record: EntityPageSourceRecord, options?: EntityPageDocumentBuildOptions) => EntityPageDocument;
  buildDocumentByRecordKey: (
    recordKey: RecordKey,
    options?: EntityPageDocumentBuildOptions,
  ) => EntityPageDocument | null;
  // Compatibility output for non-qualifying plain-text/detail consumers only.
  // Structured TUI page hosts should consume buildDocument* and page-document.
  buildDetailLines: (
    record: EntityPageSourceRecord,
    options?: { includeHeader?: boolean; recordTargetAction?: EntityPageDocumentBuildOptions["recordTargetAction"] },
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
  const buildDocument = (
    record: EntityPageSourceRecord,
    options: EntityPageDocumentBuildOptions = {},
  ): EntityPageDocument => {
    const entityRecord = toEntityRecord(record);
    const relations = relationsService.loadPageRelations(entityRecord.recordKey);
    return buildEntityPageDocument(entityRecord, relations, options);
  };
  const buildDocumentByRecordKey = (
    recordKey: RecordKey,
    options: EntityPageDocumentBuildOptions = {},
  ): EntityPageDocument | null => {
    const record = relationsService.getRecord?.(recordKey);
    return record ? buildDocument(record, options) : null;
  };

  return {
    buildDocument,
    buildDocumentByRecordKey,
    buildDetailLines: (record, options) =>
      renderEntityPageDocument(buildDocument(record, options), { includeHeader: options?.includeHeader }),
  };
}
