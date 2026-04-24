import type { NormalizedRecord } from "../domain/record-types.js";
import {
  formatOntologySearchVocabularyLabel,
  humanizeOntologySearchIdentifier,
} from "../domain/presentation-vocabulary.js";
import type { DerivedTagTerminalLine } from "./framework/types.js";

export type TerminalListDetailGroup = {
  key: string;
  label: string;
};

export type TerminalListDetailRowMetadata = {
  subtitle?: string;
  badges?: string[];
  metadataParts?: string[];
};

export type TerminalListDetailMetadataField = {
  label: string;
  value: string;
};

export function formatTerminalBreadcrumb(segments: ReadonlyArray<string | null | undefined>): string {
  return segments
    .map((segment) => segment?.trim())
    .filter((segment): segment is string => Boolean(segment))
    .map((segment) => humanizeOntologySearchIdentifier(segment))
    .join(" > ");
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

export function buildSearchResultRowMetadata(record: NormalizedRecord): TerminalListDetailRowMetadata {
  return {
    metadataParts: buildSearchResultMetadataParts(record),
  };
}

export function buildTerminalListDetailGroupLine(group: TerminalListDetailGroup): DerivedTagTerminalLine {
  return {
    text: group.label,
    tone: "section",
    noWrap: true,
  };
}

export function buildTerminalListDetailMetadataLines(
  metadata: readonly TerminalListDetailMetadataField[],
): DerivedTagTerminalLine[] {
  return metadata.map((entry) => ({
    text: `${entry.label}: ${entry.value}`,
    noWrap: true,
  }));
}

export function buildTerminalResultRowLine(
  title: string,
  options: {
    selected: boolean;
    metadata?: TerminalListDetailRowMetadata | null;
  },
): DerivedTagTerminalLine {
  const metadata = options.metadata ?? null;
  const parts = [
    title,
    ...(metadata?.subtitle ? [metadata.subtitle] : []),
    ...(metadata?.metadataParts ?? []),
    ...(metadata?.badges?.map((badge) => `[${badge}]`) ?? []),
  ];

  return {
    text: parts.join(" | "),
    tone: options.selected ? "selected" : "default",
    noWrap: true,
  };
}

export function buildSearchResultRowLine(
  record: NormalizedRecord,
  options: {
    selected: boolean;
  },
): DerivedTagTerminalLine {
  return buildTerminalResultRowLine(record.name, {
    selected: options.selected,
    metadata: buildSearchResultRowMetadata(record),
  });
}
