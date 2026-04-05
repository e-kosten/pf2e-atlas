import path from "node:path";

import type {
  BuildSourceEntry,
  NormalizedIndexRecord,
} from "./index-types.js";
import type { VariantSource } from "../types.js";
import {
  firstString,
  getNested,
  normalizeText,
  uniqueSorted,
} from "../utils.js";
import { getRecordSlug } from "./nested-item-utils.js";

const GRADE_LABELS = new Set(["minor", "lesser", "moderate", "greater", "major", "true"]);
const DRAGON_AGE_LABELS = new Set(["wyrmling", "young", "adult", "ancient"]);
const TRADITION_LABELS = new Set(["arcane", "divine", "occult", "primal"]);
const DAMAGE_TYPE_LABELS = new Set(["acid", "cold", "electricity", "fire", "poison", "sonic", "void", "vitality"]);
const SPECIALIZATION_LABELS = new Set([
  "abjuration",
  "conjuration",
  "divination",
  "enchantment",
  "evocation",
  "illusion",
  "necromancy",
  "transmutation",
]);

export type VariantAxis =
  | "rank"
  | "grade"
  | "damageType"
  | "tradition"
  | "dragonAge"
  | "specialization"
  | "other";

type CandidateVariantFamily = {
  baseName: string;
  label: string;
  axes: VariantAxis[];
  source: VariantSource;
  confidence: number;
};

type IndexedRecordEntry = BuildSourceEntry & { record: NormalizedIndexRecord };

function humanizeSlug(value: string): string {
  return value
    .split("-")
    .filter(Boolean)
    .map((segment) => /^[0-9]+(?:st|nd|rd|th)$/i.test(segment) ? segment.toLowerCase() : `${segment[0]?.toUpperCase() ?? ""}${segment.slice(1)}`)
    .join(" ")
    .trim();
}

function toFamilySlug(value: string): string {
  return normalizeText(value).replace(/\s+/g, "-");
}

function toTitleWords(value: string): string {
  return value
    .split(" ")
    .filter(Boolean)
    .map((segment) => /^[0-9]+(?:st|nd|rd|th)$/i.test(segment) ? segment.toLowerCase() : `${segment[0]?.toUpperCase() ?? ""}${segment.slice(1).toLowerCase()}`)
    .join(" ");
}

function inferVariantAxes(label: string): VariantAxis[] {
  const normalized = normalizeText(label);
  if (!normalized) {
    return ["other"];
  }

  if (/\b\d+(?:st|nd|rd|th)\s+(?:rank|level)\b/.test(normalized)) {
    return ["rank"];
  }
  if (GRADE_LABELS.has(normalized)) {
    return ["grade"];
  }
  if (DRAGON_AGE_LABELS.has(normalized)) {
    return ["dragonAge"];
  }
  if (TRADITION_LABELS.has(normalized)) {
    return ["tradition"];
  }
  if (DAMAGE_TYPE_LABELS.has(normalized)) {
    return ["damageType"];
  }
  if (SPECIALIZATION_LABELS.has(normalized)) {
    return ["specialization"];
  }

  return ["other"];
}

function parseParentheticalVariant(name: string): CandidateVariantFamily | null {
  const match = name.match(/^(.*?)\s+\(([^()]+)\)$/);
  const baseName = match?.[1]?.trim() ?? "";
  const label = match?.[2]?.trim() ?? "";
  if (!baseName || !label) {
    return null;
  }

  return {
    baseName,
    label,
    axes: inferVariantAxes(label),
    source: "namePattern",
    confidence: 0.95,
  };
}

function parseSuffixVariant(name: string): CandidateVariantFamily | null {
  const match = name.match(/^(.*?)(?:\s+|-)(Minor|Lesser|Moderate|Greater|Major|True|Wyrmling|Young|Adult|Ancient)$/i);
  const baseName = match?.[1]?.trim() ?? "";
  const label = match?.[2]?.trim() ?? "";
  if (!baseName || !label) {
    return null;
  }

  return {
    baseName,
    label: toTitleWords(label),
    axes: inferVariantAxes(label),
    source: "namePattern",
    confidence: 0.84,
  };
}

function normalizeVariantSlugLabel(value: string): string {
  const rankLabel = value.match(/^(\d+(?:st|nd|rd|th))-(rank|level)(?:-(spell))?$/i);
  if (rankLabel) {
    const trailing = rankLabel[3] ? " Spell" : "";
    return `${rankLabel[1]!.toLowerCase()}-${rankLabel[2]![0]!.toUpperCase()}${rankLabel[2]!.slice(1).toLowerCase()}${trailing}`;
  }

  return toTitleWords(humanizeSlug(value));
}

function stripTrailingLabel(name: string, label: string): string | null {
  const patterns = [
    new RegExp(`^(.+?)\\s+\\(${label.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\)$`, "i"),
    new RegExp(`^(.+?)(?:\\s+|-)${label.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}$`, "i"),
  ];

  for (const pattern of patterns) {
    const match = name.match(pattern);
    const base = match?.[1]?.trim() ?? "";
    if (base) {
      return base;
    }
  }

  return null;
}

function parsePathVariant(record: NormalizedIndexRecord): CandidateVariantFamily | null {
  const stem = path.basename(record.sourcePath, path.extname(record.sourcePath)).toLowerCase();
  const matchers: Array<{ pattern: RegExp; labelIndex: number }> = [
    { pattern: /^(.*)-(\d+(?:st|nd|rd|th)-rank(?:-spell)?)$/i, labelIndex: 2 },
    { pattern: /^(.*)-(\d+(?:st|nd|rd|th)-level(?:-spell)?)$/i, labelIndex: 2 },
    { pattern: /^(.*)-(minor|lesser|moderate|greater|major|true)$/i, labelIndex: 2 },
    { pattern: /^(.*)-(wyrmling|young|adult|ancient)$/i, labelIndex: 2 },
  ];

  for (const matcher of matchers) {
    const match = stem.match(matcher.pattern);
    if (!match) {
      continue;
    }

    const label = normalizeVariantSlugLabel(match[matcher.labelIndex] ?? "");
    if (!label) {
      continue;
    }

    const baseName = stripTrailingLabel(record.name, label) ?? humanizeSlug(match[1] ?? "");
    if (!baseName) {
      continue;
    }

    return {
      baseName,
      label,
      axes: inferVariantAxes(label),
      source: "sourcePath",
      confidence: 0.76,
    };
  }

  return null;
}

function buildVariantCandidate(entry: IndexedRecordEntry): CandidateVariantFamily | null {
  if (entry.record.category !== "equipment" && entry.record.category !== "spell") {
    return null;
  }

  const nameCandidate = parseParentheticalVariant(entry.record.name) ?? parseSuffixVariant(entry.record.name);
  const pathCandidate = parsePathVariant(entry.record);
  const primary = nameCandidate ?? pathCandidate;
  if (!primary) {
    return null;
  }

  const slug = getRecordSlug(entry.raw);
  const baseItem = firstString(getNested(entry.raw, ["system", "baseItem"]));
  let confidence = primary.confidence;
  let source: VariantSource = primary.source;
  const normalizedBase = normalizeText(primary.baseName);
  if (slug && normalizeText(slug.replace(/[-_]+/g, " ")).includes(normalizedBase)) {
    confidence += 0.02;
    source = source === "slug" ? "slug" : "composite";
  }
  if (baseItem) {
    confidence += 0.02;
    source = source === "baseItem" ? "baseItem" : "composite";
  }

  return {
    ...primary,
    confidence: Math.min(0.99, confidence),
    source,
  };
}

function variantGroupKey(entry: IndexedRecordEntry, candidate: CandidateVariantFamily): string {
  return [
    entry.record.category,
    toFamilySlug(entry.pack.name),
    toFamilySlug(candidate.baseName),
  ].join(":");
}

function distinctLabels(candidates: CandidateVariantFamily[]): number {
  return new Set(candidates.map((candidate) => normalizeText(candidate.label)).filter(Boolean)).size;
}

export function assignVariantFamilies(entries: IndexedRecordEntry[]): void {
  const candidates = entries
    .map((entry) => ({ entry, candidate: buildVariantCandidate(entry) }))
    .filter((entry): entry is { entry: IndexedRecordEntry; candidate: CandidateVariantFamily } => Boolean(entry.candidate));

  const groups = new Map<string, Array<{ entry: IndexedRecordEntry; candidate: CandidateVariantFamily }>>();
  for (const entry of candidates) {
    const key = variantGroupKey(entry.entry, entry.candidate);
    const bucket = groups.get(key) ?? [];
    bucket.push(entry);
    groups.set(key, bucket);
  }

  for (const [groupKey, group] of groups.entries()) {
    if (group.length < 2 || distinctLabels(group.map((entry) => entry.candidate)) < 2) {
      continue;
    }

    const sharedBaseName = group[0]!.candidate.baseName;
    const groupAxes = uniqueSorted(group.flatMap((entry) => entry.candidate.axes));
    const sources = new Set(group.map((entry) => entry.candidate.source));
    const source = sources.size === 1 ? [...sources][0]! : "composite";
    const groupBoost = Math.min(0.08, Math.max(0, group.length - 2) * 0.02);

    for (const { entry, candidate } of group) {
      entry.record.variantFamilyKey = groupKey;
      entry.record.variantBaseName = sharedBaseName;
      entry.record.variantLabel = candidate.label;
      entry.record.variantAxes = uniqueSorted([...groupAxes, ...candidate.axes]);
      entry.record.variantConfidence = Math.min(0.99, candidate.confidence + groupBoost);
      entry.record.variantSource = source;
    }
  }
}
