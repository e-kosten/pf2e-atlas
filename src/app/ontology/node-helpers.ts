import type { MetadataFilterNode, NormalizedRecord, OntologyNode } from "../../types.js";
import { normalizeText } from "../../utils.js";
import { mapNormalizedRecordToOntologyExplorerEntityRecord } from "./entity-record.js";
import {
  buildOntologyExplorerEntityDetailLines,
  buildOntologyExplorerEntitySummary,
} from "./presenter.js";

export function titleCaseLabel(value: string): string {
  return value
    .split(/[_\s-]+/)
    .filter((segment) => segment.length > 0)
    .map((segment) => `${segment[0]!.toUpperCase()}${segment.slice(1)}`)
    .join(" ");
}

export function buildFilterText(...values: Array<string | null | undefined>): string {
  return values
    .flatMap((value) => (value ? [value] : []))
    .map((value) => normalizeText(value))
    .filter(Boolean)
    .join(" ");
}

export function buildKeyValueDetailLines(
  title: string,
  entries: Array<[string, string | number | null | undefined]>,
  description?: string,
): OntologyNode["detailLines"] {
  const lines: OntologyNode["detailLines"] = [{ text: title, tone: "section" }];
  if (description) {
    lines.push({ text: description });
  }
  for (const [label, value] of entries) {
    const rendered = value === null || value === undefined || value === "" ? "(none)" : String(value);
    lines.push({ text: `${label}: ${rendered}` });
  }
  return lines;
}

export function cloneMetadataFilterNode(metadata: MetadataFilterNode): MetadataFilterNode {
  return structuredClone(metadata);
}

export function cloneOntologyNode(node: OntologyNode, idPrefix?: string): OntologyNode {
  return {
    ...node,
    id: idPrefix ? `${idPrefix}:${node.id}` : node.id,
    children: node.children?.map((child) => cloneOntologyNode(child, idPrefix)),
    loadChildren: node.loadChildren
      ? () => node.loadChildren!().map((child) => cloneOntologyNode(child, idPrefix))
      : undefined,
    childPresentation: node.childPresentation ? { ...node.childPresentation } : undefined,
    groupValues: node.groupValues ? { ...node.groupValues } : undefined,
    query: node.query ? { ...node.query, filters: { ...node.query.filters } } : undefined,
    selection: node.selection ? { ...node.selection } : undefined,
  };
}

export function buildNormalizedRecordNode(record: NormalizedRecord): OntologyNode {
  const entityRecord = mapNormalizedRecordToOntologyExplorerEntityRecord(record);
  return {
    id: record.recordKey,
    kind: "record",
    label: record.name,
    filterText: buildFilterText(
      record.recordKey,
      record.name,
      record.category,
      record.subcategory ?? "",
      record.descriptionText ?? "",
      record.blurbText ?? "",
    ),
    listLabel: buildOntologyExplorerEntitySummary(entityRecord),
    detailTitle: "Record Details",
    detailLines: buildOntologyExplorerEntityDetailLines(entityRecord),
    query: {
      kind: "lookup",
      label: "Open exact record lookup",
      filters: {
        nameQuery: record.name,
        category: record.category,
        subcategory: record.subcategory ?? undefined,
        limit: 5,
      },
    },
  };
}
