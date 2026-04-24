import type { NormalizedRecord } from "../../domain/record-types.js";
import type { OntologyNode, OntologyTextLine } from "../../domain/ontology-types.js";
import { humanizeOntologySearchIdentifier } from "../../domain/presentation-vocabulary.js";
import { buildScopeFilter, type SearchRequest } from "../../domain/search-request-types.js";
import { normalizeText } from "../../shared/utils.js";
import { mapNormalizedRecordToOntologyExplorerEntityRecord } from "./entity-record.js";
import { buildOntologyExplorerEntityDetailLines, buildOntologyExplorerEntitySummary } from "./presenter.js";

const loadedOntologyChildren = new WeakMap<OntologyNode, readonly OntologyNode[]>();

export function titleCaseLabel(value: string): string {
  return humanizeOntologySearchIdentifier(value);
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
  entries: ReadonlyArray<readonly [string, string | number | null | undefined]>,
  description?: string,
): OntologyNode["detailLines"] {
  const lines: OntologyTextLine[] = [{ text: title, tone: "section" }];
  if (description) {
    lines.push({ text: description });
  }
  for (const [label, value] of entries) {
    const rendered = value === null || value === undefined || value === "" ? "(none)" : String(value);
    lines.push({ text: `${label}: ${rendered}` });
  }
  return lines;
}

function cloneSearchRequest(request: Readonly<SearchRequest>): SearchRequest {
  return structuredClone(request);
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
    query: node.query ? { ...node.query, request: cloneSearchRequest(node.query.request) } : undefined,
    selection: node.selection ? { ...node.selection, allowedStates: [...node.selection.allowedStates] } : undefined,
  };
}

export function getOntologyNodeChildren(node: OntologyNode | undefined): readonly OntologyNode[] {
  if (!node) {
    return [];
  }
  if (node.children) {
    return node.children;
  }
  if (!node.loadChildren) {
    return [];
  }

  const cached = loadedOntologyChildren.get(node);
  if (cached) {
    return cached;
  }

  const children = node.loadChildren();
  loadedOntologyChildren.set(node, children);
  return children;
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
      label: "Open exact record lookup",
      request: {
        mode: "lookup",
        search: {
          query: record.name,
        },
        filter: buildScopeFilter(record.category, record.subcategory),
        limit: 5,
      },
    },
  };
}
