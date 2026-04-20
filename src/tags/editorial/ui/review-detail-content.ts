import type { OntologyTextLine } from "../../../domain/index.js";
import { buildOntologyExplorerEntityDetailLines } from "../../entity-record.js";
import type { DerivedTagMigrationSessionRecord } from "../types.js";

export function buildDerivedTagMigrationRecordPageLines(record: DerivedTagMigrationSessionRecord): OntologyTextLine[] {
  return buildOntologyExplorerEntityDetailLines(record.entityRecord, { includeHeader: false });
}

export function buildDerivedTagMigrationRecordPageTextLines(record: DerivedTagMigrationSessionRecord): string[] {
  return buildDerivedTagMigrationRecordPageLines(record).map((line) => `${" ".repeat(line.indent ?? 0)}${line.text}`);
}
