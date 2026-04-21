import type { OntologyTextLine } from "../../../domain/derived-tag-types.js";
import { buildOntologyExplorerEntityDetailLines } from "../../../app/ontology/entity-record.js";
import type { DerivedTagReviewSessionRecord } from "../types.js";

export function buildDerivedTagMigrationRecordPageLines(record: DerivedTagReviewSessionRecord): OntologyTextLine[] {
  return buildOntologyExplorerEntityDetailLines(record.entityRecord, { includeHeader: false });
}

export function buildDerivedTagMigrationRecordPageTextLines(record: DerivedTagReviewSessionRecord): string[] {
  return buildDerivedTagMigrationRecordPageLines(record).map((line) => `${" ".repeat(line.indent ?? 0)}${line.text}`);
}
