import type { NormalizedRecord } from "../domain/record-types.js";
import { formatOntologySearchVocabularyLabel } from "../domain/presentation-vocabulary.js";
import type { DerivedTagTerminalLine } from "./framework/types.js";

export function formatTerminalBreadcrumb(segments: ReadonlyArray<string | null | undefined>): string {
  return segments.map((segment) => segment?.trim()).filter((segment): segment is string => Boolean(segment)).join(" > ");
}

function buildSearchResultMetadataParts(record: NormalizedRecord): string[] {
  const metadata: string[] = [];

  if (record.level !== null) {
    metadata.push(`L${record.level}`);
  }
  if (record.rarity && record.rarity !== "common") {
    metadata.push(formatOntologySearchVocabularyLabel(record.rarity));
  }
  metadata.push(record.packLabel);

  return metadata;
}

export function buildSearchResultRowLine(
  record: NormalizedRecord,
  options: {
    selected: boolean;
  },
): DerivedTagTerminalLine {
  const metadata = buildSearchResultMetadataParts(record);
  return {
    text: metadata.length > 0 ? `${record.name} | ${metadata.join(" | ")}` : record.name,
    tone: options.selected ? "selected" : "default",
    noWrap: true,
  };
}
