import type { OntologyTextLine } from "../../../domain/derived-tag-types.js";
import { buildOntologyExplorerEntityDetailLines } from "../../../app/ontology/entity-record.js";
import type { DerivedTagReviewSessionRecord } from "../types.js";

function renderPlainTextLine(line: OntologyTextLine): string {
  const content = line.href ? (line.plainTextFallback ?? `${line.text}: ${line.href}`) : line.text;
  return `${" ".repeat(line.indent ?? 0)}${content}`;
}

export function buildDerivedTagMigrationRecordPageLines(record: DerivedTagReviewSessionRecord): OntologyTextLine[] {
  return buildOntologyExplorerEntityDetailLines(record.entityRecord, { includeHeader: false });
}

export function buildDerivedTagMigrationRecordPageTextLines(record: DerivedTagReviewSessionRecord): string[] {
  return buildDerivedTagMigrationRecordPageLines(record).map(renderPlainTextLine);
}
