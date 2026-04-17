import type { DerivedTagTerminalLine } from "./terminal-ui.js";
import type { DerivedTagMigrationDecision, DerivedTagMigrationSessionRecord } from "./types.js";

function renderList(values: string[]): string {
  return values.length > 0 ? values.join(", ") : "(none)";
}

function normalizeNarrativeText(text: string): string {
  return text.replace(/\s+/g, " ").trim();
}

function compactNarrativeText(text: string, maxLength = 420): string {
  const normalized = normalizeNarrativeText(text);
  if (normalized.length <= maxLength) {
    return normalized;
  }

  const boundary = normalized.lastIndexOf(" ", maxLength - 1);
  const cutoff = boundary >= Math.floor(maxLength * 0.6) ? boundary : maxLength - 1;
  return `${normalized.slice(0, cutoff).trimEnd()}...`;
}

function getDecisionTag(decision: DerivedTagMigrationDecision): string {
  return decision.tag;
}

export function buildDerivedTagMigrationRecordContextLines(
  record: DerivedTagMigrationSessionRecord,
  decision: DerivedTagMigrationDecision,
): DerivedTagTerminalLine[] {
  const lines: DerivedTagTerminalLine[] = [
    { text: "Record Context", tone: "section" },
    { text: `Traits: ${renderList(record.traits)}`, indent: 2 },
    { text: `Current tags: ${renderList(record.currentDerivedTags)}`, indent: 2 },
  ];

  const currentSource = record.currentSources[getDecisionTag(decision)];
  if (currentSource) {
    lines.push({ text: `Current source for ${getDecisionTag(decision)}: ${currentSource}`, indent: 2 });
  }

  if (record.blurbText) {
    lines.push({ text: `Blurb: ${compactNarrativeText(record.blurbText)}`, indent: 2 });
  }

  if (record.descriptionText) {
    const normalizedDescription = normalizeNarrativeText(record.descriptionText);
    const normalizedBlurb = record.blurbText ? normalizeNarrativeText(record.blurbText) : null;
    if (!normalizedBlurb || normalizedDescription !== normalizedBlurb) {
      lines.push({ text: `Description: ${compactNarrativeText(record.descriptionText)}`, indent: 2 });
    }
  }

  return lines;
}

export function buildDerivedTagMigrationRecordContextTextLines(
  record: DerivedTagMigrationSessionRecord,
  decision: DerivedTagMigrationDecision,
): string[] {
  const lines = [
    `Traits: ${renderList(record.traits)}`,
    `Current tags: ${renderList(record.currentDerivedTags)}`,
  ];

  const currentSource = record.currentSources[getDecisionTag(decision)];
  if (currentSource) {
    lines.push(`Current source for ${getDecisionTag(decision)}: ${currentSource}`);
  }

  if (record.blurbText) {
    lines.push(`Blurb: ${compactNarrativeText(record.blurbText)}`);
  }

  if (record.descriptionText) {
    const normalizedDescription = normalizeNarrativeText(record.descriptionText);
    const normalizedBlurb = record.blurbText ? normalizeNarrativeText(record.blurbText) : null;
    if (!normalizedBlurb || normalizedDescription !== normalizedBlurb) {
      lines.push(`Description: ${compactNarrativeText(record.descriptionText)}`);
    }
  }

  return lines;
}
