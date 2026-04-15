import type {
  NormalizedRecord,
  PackInfo,
  RecordDetail,
  RuleReferenceEdge,
  SearchRecordExplanation,
} from "../types.js";
import { getMetadataFieldSpecsByPresentation } from "../domain/metadata-field-registry.js";

function assignPresentedMetadata(
  target: Record<string, unknown>,
  record: NormalizedRecord,
  presentation: "summary" | "detail",
): void {
  for (const spec of getMetadataFieldSpecsByPresentation(presentation)) {
    if (spec.presentWhen && !spec.presentWhen(record)) {
      continue;
    }
    target[spec.field] = record[spec.recordProperty];
  }
}

export function summarizeRecord(
  record: NormalizedRecord,
  detail: RecordDetail = "full",
  explanation?: SearchRecordExplanation,
): Record<string, unknown> {
  const summary: Record<string, unknown> = {
    recordKey: record.recordKey,
    id: record.id,
    name: record.name,
    aliases: record.aliases,
    category: record.category,
    subcategory: record.subcategory,
    rawRecordType: record.type,
    packName: record.packName,
    packLabel: record.packLabel,
    descriptionSnippet: record.descriptionSnippet,
  };

  assignPresentedMetadata(summary, record, "summary");

  if (explanation) {
    summary.searchExplain = explanation;
  }

  if (record.variantFamilyKey) {
    Object.assign(summary, {
      variantFamilyKey: record.variantFamilyKey,
      variantBaseName: record.variantBaseName,
      variantLabel: record.variantLabel,
      variantAxes: record.variantAxes,
      variantConfidence: record.variantConfidence,
      variantSource: record.variantSource,
    });
  }

  if (detail === "minimal") {
    return summary;
  }

  Object.assign(summary, {
    descriptionText: record.descriptionText,
    blurbText: record.blurbText,
    itemMetrics: record.itemMetrics,
    actorMetrics: record.actorMetrics,
  });
  assignPresentedMetadata(summary, record, "detail");

  if (detail === "full") {
    summary.sourcePath = record.sourcePath;
  }

  return summary;
}

export function summarizePack(pack: PackInfo): Record<string, unknown> {
  return {
    name: pack.name,
    label: pack.label,
    documentType: pack.documentType,
    declaredPath: pack.declaredPath,
    resolvedPath: pack.resolvedPath,
    recordCount: pack.recordCount,
  };
}

export function formatSearchResult(prefix: string, total: number, records: NormalizedRecord[]): string {
  const lines = records.map((record) => {
    const level = record.level !== null ? `level ${record.level}` : "level n/a";
    const subtype = record.subcategory ? `/${record.subcategory}` : "";
    const aliasText = record.aliases.length > 0 ? `; aliases: ${record.aliases.join(", ")}` : "";
    return `- ${record.name} (${record.packLabel}, ${record.category}${subtype}, ${level}${aliasText})`;
  });

  return [prefix, `Total matches: ${total}`, ...lines].join("\n");
}

export function summarizeEdge(edge: RuleReferenceEdge): Record<string, unknown> {
  return {
    fromRecordKey: edge.fromRecordKey,
    toRecordKey: edge.toRecordKey,
    displayText: edge.displayText,
    referenceText: edge.referenceText,
    direction: edge.direction,
    relationshipType: edge.relationshipType,
    sourcePackName: edge.sourcePackName,
    sourceRecordType: edge.sourceRecordType,
    sourceDocumentType: edge.sourceDocumentType,
    sourceCategory: edge.sourceCategory,
  };
}
