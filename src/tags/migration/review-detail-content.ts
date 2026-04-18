import { buildOntologyExplorerEntityDetailLines } from "../../tui/ontology-explorer/entity-page.js";
import type { DerivedTagTerminalLine } from "../../tui/terminal-ui.js";
import type { DerivedTagMigrationSessionRecord } from "./types.js";

export function buildDerivedTagMigrationRecordPageLines(
  record: DerivedTagMigrationSessionRecord,
): DerivedTagTerminalLine[] {
  return buildOntologyExplorerEntityDetailLines(record.entityRecord, { includeHeader: false });
}

export function buildDerivedTagMigrationRecordPageTextLines(record: DerivedTagMigrationSessionRecord): string[] {
  return buildDerivedTagMigrationRecordPageLines(record).map((line) => `${" ".repeat(line.indent ?? 0)}${line.text}`);
}
