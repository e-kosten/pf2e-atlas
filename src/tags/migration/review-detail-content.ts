import { buildOntologyExplorerEntityDetailLines } from "../../app/ontology/presenter.js";
import type { OntologyTextLine } from "../../types.js";
import type { DerivedTagMigrationSessionRecord } from "./types.js";

export function buildDerivedTagMigrationRecordPageLines(record: DerivedTagMigrationSessionRecord): OntologyTextLine[] {
  return buildOntologyExplorerEntityDetailLines(record.entityRecord, { includeHeader: false });
}

export function buildDerivedTagMigrationRecordPageTextLines(record: DerivedTagMigrationSessionRecord): string[] {
  return buildDerivedTagMigrationRecordPageLines(record).map((line) => `${" ".repeat(line.indent ?? 0)}${line.text}`);
}
