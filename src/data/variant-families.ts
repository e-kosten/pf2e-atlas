import path from "node:path";

import type { BuildSourceEntry, NormalizedIndexRecord } from "./index-types.js";
import type { VariantSource } from "../domain/record-types.js";
import { normalizeText, uniqueSorted } from "../shared/utils.js";
import { firstString, getNested } from "./raw-utils.js";

const GRADE_LABELS = new Set(["minor", "lesser", "moderate", "greater", "major", "true"]);
const DAMAGE_TYPE_LABELS = new Set(["acid", "cold", "electricity", "fire", "poison", "sonic", "void", "vitality"]);
const DRAGON_AGE_LABELS = new Set(["wyrmling", "hatchling", "young", "juvenile", "adult", "ancient", "greatwyrm"]);
const SPECIALIZATION_LABELS = new Set(["spellcaster", "elite", "weak", "variant"]);
const GENDER_LABELS = new Set(["male", "female"]);
const CREATURE_SUFFIX_FAMILY_ALLOWLIST = new Map<string, string>([
  ["ghost", "ghost"],
  ["ghoul", "ghoul"],
  ["wight", "wight"],
  ["wraith", "wraith"],
]);
const STRUCTURAL_LINE_PREFIXES = [
  "activate",
  "effect",
  "frequency",
  "requirements",
  "craft requirements",
  "trigger",
  "duration",
  "critical success",
  "success",
  "failure",
  "critical failure",
  "stage 1",
  "stage 2",
  "stage 3",
  "saving throw",
];
const UUID_PATTERN = /@UUID\[([^\]]+)\](?:\{([^}]+)\})?/g;
const CHECK_PATTERN = /@Check\[[^\]]+\](?:\{([^}]+)\})?/g;
const INLINE_PATTERN = /@[A-Z][A-Za-z]+\[[^\]]+\](?:\{([^}]+)\})?/g;
const TITLE_GLUE_TOKENS = new Set([
  "a",
  "an",
  "the",
  "of",
  "and",
  "or",
  "for",
  "to",
  "in",
  "on",
  "at",
  "with",
  "without",
  "type",
  "mark",
]);

export type VariantAxis = "rank" | "grade" | "damageType" | "tradition" | "dragonAge" | "specialization" | "other";

type TitleCandidate = {
  baseName: string;
  label: string | null;
  axes: VariantAxis[];
  source: Exclude<VariantSource, "baseItem" | "slug" | "none">;
  confidence: number;
  fallbackEligible: boolean;
};

type GroupMember = {
  entry: IndexedRecordEntry;
  candidate: TitleCandidate;
  leadBlock: string;
  descriptionScore: number;
};

type TitleToken = {
  raw: string;
  normalized: string;
  meaningful: boolean;
  start: number;
  end: number;
};

type IndexedRecordEntry = BuildSourceEntry & { record: NormalizedIndexRecord };

type CandidateGroup = {
  baseName: string;
  category: string;
  packName: string;
  members: Map<string, GroupMember>;
};

function titleCaseToken(segment: string): string {
  if (/^[0-9]+(?:st|nd|rd|th)$/i.test(segment)) {
    return segment.toLowerCase();
  }

  return `${segment[0]?.toUpperCase() ?? ""}${segment.slice(1).toLowerCase()}`;
}

function humanizeSlug(value: string): string {
  return value
    .split("-")
    .filter(Boolean)
    .map((segment) => titleCaseToken(segment))
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
    .map((segment) => titleCaseToken(segment))
    .join(" ");
}

function splitCompositeLabel(label: string): string[] {
  return label
    .split(",")
    .map((part) => part.trim())
    .filter(Boolean);
}

function inferVariantAxes(label: string): VariantAxis[] {
  const normalized = normalizeText(label);
  if (!normalized) {
    return ["other"];
  }

  if (/\b\d+(?:st|nd|rd|th)\s+(?:rank|level)(?:\s+spell)?\b/.test(normalized)) {
    return ["rank"];
  }
  if (GRADE_LABELS.has(normalized)) {
    return ["grade"];
  }
  if (DAMAGE_TYPE_LABELS.has(normalized)) {
    return ["damageType"];
  }
  if (DRAGON_AGE_LABELS.has(normalized)) {
    return ["dragonAge"];
  }
  if (SPECIALIZATION_LABELS.has(normalized)) {
    return ["specialization"];
  }
  if (GENDER_LABELS.has(normalized)) {
    return [];
  }

  return ["other"];
}

function inferAxes(labels: string[]): VariantAxis[] {
  const axes = [...new Set(labels.flatMap((label) => inferVariantAxes(label)))] as VariantAxis[];
  axes.sort((left, right) => left.localeCompare(right));
  return axes.length > 0 ? axes : ["other"];
}

function isFallbackEligible(axes: VariantAxis[]): boolean {
  return axes.length > 0 && axes.every((axis) => axis === "rank" || axis === "grade");
}

function parseParentheticalCandidate(name: string): TitleCandidate | null {
  let remainder = name.trim();
  const labels: string[] = [];

  for (;;) {
    const match = remainder.match(/^(.*?)\s+\(([^()]+)\)$/);
    if (!match) {
      break;
    }

    const base = match[1]?.trim() ?? "";
    const rawLabel = match[2]?.trim() ?? "";
    if (!base || !rawLabel) {
      break;
    }

    remainder = base;
    labels.unshift(...splitCompositeLabel(rawLabel));
  }

  if (!remainder || labels.length === 0) {
    return null;
  }

  const axes = inferAxes(labels);
  return {
    baseName: remainder,
    label: labels.join(", "),
    axes,
    source: "namePattern",
    confidence: isFallbackEligible(axes) ? 0.74 : 0.6,
    fallbackEligible: isFallbackEligible(axes),
  };
}

function parseTrailingSuffixCandidate(name: string): TitleCandidate | null {
  const match = name.match(
    /^(.*?)(?:\s+|-)(\d+(?:st|nd|rd|th)[-\s]+(?:rank|level)(?:\s+spell)?|Minor|Lesser|Moderate|Greater|Major|True)$/i,
  );
  const baseName = match?.[1]?.trim() ?? "";
  const rawLabel = match?.[2]?.trim() ?? "";
  if (!baseName || !rawLabel) {
    return null;
  }

  const label = rawLabel
    .replace(/[-_]+/g, " ")
    .replace(/\b(rank|level|spell)\b/gi, (value) => titleCaseToken(value))
    .replace(/\s+/g, " ")
    .trim()
    .replace(/^(\d+(?:st|nd|rd|th)) /i, (_, ordinal: string) => `${ordinal.toLowerCase()} `);
  const axes = inferAxes([label]);

  return {
    baseName,
    label,
    axes,
    source: "namePattern",
    confidence: isFallbackEligible(axes) ? 0.76 : 0.62,
    fallbackEligible: isFallbackEligible(axes),
  };
}

function normalizeVariantSlugLabel(value: string): string {
  const rankLabel = value.match(/^(\d+(?:st|nd|rd|th))-(rank|level)(?:-(spell))?$/i);
  if (rankLabel) {
    const trailing = rankLabel[3] ? " Spell" : "";
    return `${rankLabel[1]!.toLowerCase()}-${titleCaseToken(rankLabel[2]!)}${trailing}`;
  }

  return toTitleWords(humanizeSlug(value));
}

function parsePathFallbackCandidate(record: NormalizedIndexRecord): TitleCandidate | null {
  const stem = path.basename(record.sourcePath, path.extname(record.sourcePath)).toLowerCase();
  const matchers: Array<{ pattern: RegExp; labelIndex: number }> = [
    { pattern: /^(.*)-(\d+(?:st|nd|rd|th)-rank(?:-spell)?)$/i, labelIndex: 2 },
    { pattern: /^(.*)-(\d+(?:st|nd|rd|th)-level(?:-spell)?)$/i, labelIndex: 2 },
    { pattern: /^(.*)-(minor|lesser|moderate|greater|major|true)$/i, labelIndex: 2 },
  ];

  for (const matcher of matchers) {
    const match = stem.match(matcher.pattern);
    if (!match) {
      continue;
    }

    const label = normalizeVariantSlugLabel(match[matcher.labelIndex] ?? "");
    const baseName = humanizeSlug(match[1] ?? "");
    if (!baseName || !label) {
      continue;
    }

    const axes = inferAxes([label]);
    if (!isFallbackEligible(axes)) {
      continue;
    }

    return {
      baseName,
      label,
      axes,
      source: "sourcePath",
      confidence: 0.7,
      fallbackEligible: true,
    };
  }

  return null;
}

function parseStructuredCandidate(entry: IndexedRecordEntry): TitleCandidate | null {
  return (
    parseParentheticalCandidate(entry.record.name) ??
    parseTrailingSuffixCandidate(entry.record.name) ??
    parsePathFallbackCandidate(entry.record)
  );
}

function parseEntryStructuredCandidate(entry: IndexedRecordEntry): TitleCandidate | null {
  if (entry.record.category === "creature") {
    return parseParentheticalCandidate(entry.record.name);
  }

  return parseStructuredCandidate(entry);
}

function buildSuffixScaffoldCandidate(entry: IndexedRecordEntry): TitleCandidate | null {
  const tokens = normalizeText(entry.record.name).split(" ").filter(Boolean);
  if (tokens.length < 2) {
    return null;
  }

  const baseTokens = tokens.slice(-2);
  const labelTokens = tokens.slice(0, -2);
  if (baseTokens.some((token) => TITLE_GLUE_TOKENS.has(token))) {
    return null;
  }
  const baseName = toTitleWords(baseTokens.join(" "));
  const label = labelTokens.length > 0 ? toTitleWords(labelTokens.join(" ")) : null;
  if (!baseName) {
    return null;
  }

  return {
    baseName,
    label,
    axes: label ? ["other"] : [],
    source: "namePattern",
    confidence: 0.52,
    fallbackEligible: false,
  };
}

function buildLooseStemCandidate(entry: IndexedRecordEntry): TitleCandidate | null {
  const tokens = normalizeText(entry.record.name).split(" ").filter(Boolean);
  if (tokens.length < 3) {
    return null;
  }

  const candidateShapes = [
    {
      baseTokens: tokens.slice(0, -1),
      labelTokens: tokens.slice(-1),
    },
    {
      baseTokens: tokens.slice(-2),
      labelTokens: tokens.slice(0, -2),
    },
  ];

  for (const shape of candidateShapes) {
    if (shape.baseTokens.length < 2 || shape.labelTokens.length < 1 || shape.labelTokens.length > 2) {
      continue;
    }

    const baseName = toTitleWords(shape.baseTokens.join(" "));
    const label = toTitleWords(shape.labelTokens.join(" "));
    if (!baseName || !label) {
      continue;
    }

    return {
      baseName,
      label,
      axes: ["other"],
      source: "namePattern",
      confidence: 0.48,
      fallbackEligible: false,
    };
  }

  return null;
}

function exactLookupKey(entry: IndexedRecordEntry, name: string): string {
  const normalizedName = normalizeText(name);
  if (entry.record.category === "creature") {
    return `${entry.record.category}:${normalizedName}`;
  }

  return `${entry.record.category}:${entry.pack.name}:${normalizedName}`;
}

function groupLookupKey(entry: IndexedRecordEntry, baseName: string): string {
  const normalizedBaseName = normalizeText(baseName);
  if (entry.record.category === "creature") {
    return `${entry.record.category}:family:${normalizedBaseName}`;
  }

  return `${entry.record.category}:${entry.pack.name}:${normalizedBaseName}`;
}

function creatureFamilyKey(baseName: string): string {
  return `creature:family:${toFamilySlug(baseName)}`;
}

function deriveCreatureReferenceAxes(labels: string[]): VariantAxis[] {
  const axes = [
    ...new Set(labels.flatMap((label) => inferVariantAxes(label)).filter((axis) => axis !== "other")),
  ] as VariantAxis[];
  axes.sort((left, right) => left.localeCompare(right));
  return axes;
}

function chooseCreatureReferenceLabel(entry: IndexedRecordEntry, baseName: string, labels: string[]): string | null {
  const explicitLabel = labels.join(", ").trim();
  const normalizedName = normalizeText(entry.record.name);
  const normalizedBaseName = normalizeText(baseName);
  const hasOnlyGenericReferenceLabels = labels.every((label) => {
    const normalized = normalizeText(label);
    return SPECIALIZATION_LABELS.has(normalized) || GENDER_LABELS.has(normalized);
  });
  if (!normalizedName || normalizedName === normalizedBaseName) {
    return explicitLabel || null;
  }
  if (hasOnlyGenericReferenceLabels) {
    return entry.record.name;
  }
  if (normalizedName.includes(normalizedBaseName) && explicitLabel) {
    return explicitLabel;
  }

  return entry.record.name;
}

function extractRawCreatureBlurb(entry: IndexedRecordEntry): string | null {
  return firstString(getNested(entry.raw, ["system", "details", "blurb"]));
}

function isGenderOnlyCreatureReference(labels: string[]): boolean {
  return labels.length > 0 && labels.every((label) => GENDER_LABELS.has(normalizeText(label)));
}

function isRejectedGenderOnlyCreatureReference(
  baseTokens: string[],
  resolvedBaseEntries: IndexedRecordEntry[],
): boolean {
  if (baseTokens.length <= 1) {
    return true;
  }

  return resolvedBaseEntries.some((resolvedBaseEntry) => resolvedBaseEntry.record.traits.includes("humanoid"));
}

function singularizeCreatureReferenceToken(token: string): string | null {
  if (token.length <= 3) {
    return null;
  }

  if (token.endsWith("ies")) {
    return `${token.slice(0, -3)}y`;
  }

  if (/(xes|ches|shes|sses|zes)$/.test(token)) {
    return token.slice(0, -2);
  }

  if (token.endsWith("s") && !token.endsWith("ss")) {
    return token.slice(0, -1);
  }

  return null;
}

function buildCreatureReferenceBaseNames(baseTokens: string[]): string[] {
  const baseNames: string[] = [];
  const addBaseName = (tokens: string[]) => {
    const baseName = toTitleWords(tokens.join(" "));
    if (baseName && !baseNames.includes(baseName)) {
      baseNames.push(baseName);
    }
  };

  addBaseName(baseTokens);

  const lastToken = baseTokens.at(-1);
  if (!lastToken) {
    return baseNames;
  }

  const singularLastToken = singularizeCreatureReferenceToken(lastToken);
  if (!singularLastToken || singularLastToken === lastToken) {
    return baseNames;
  }

  addBaseName([...baseTokens.slice(0, -1), singularLastToken]);
  return baseNames;
}

function parseCreatureReferenceCandidate(
  entry: IndexedRecordEntry,
  exactNameLookup: Map<string, IndexedRecordEntry[]>,
  knownCreatureBaseNames: Set<string>,
): TitleCandidate | null {
  if (entry.record.category !== "creature") {
    return null;
  }

  const rawBlurb = extractRawCreatureBlurb(entry);
  if (!rawBlurb) {
    return null;
  }

  const tokens = normalizeText(rawBlurb).split(" ").filter(Boolean);
  if (tokens.length < 2 || tokens.length > 6) {
    return null;
  }

  const labelTokens: string[] = [];
  let index = 0;
  while (index < tokens.length) {
    const token = tokens[index]!;
    if (!DRAGON_AGE_LABELS.has(token) && !SPECIALIZATION_LABELS.has(token) && !GENDER_LABELS.has(token)) {
      break;
    }
    labelTokens.push(token);
    index += 1;
  }

  if (labelTokens.length === 0) {
    return null;
  }

  const baseTokens = tokens.slice(index);
  if (baseTokens.length === 0 || baseTokens.length > 3) {
    return null;
  }

  const cleanedLabels = labelTokens.map((token) => titleCaseToken(token));
  for (const baseName of buildCreatureReferenceBaseNames(baseTokens)) {
    const normalizedBaseName = normalizeText(baseName);
    if (!normalizedBaseName) {
      continue;
    }

    const resolvedBaseEntries = exactNameLookup.get(exactLookupKey(entry, baseName)) ?? [];
    if (!knownCreatureBaseNames.has(normalizedBaseName) && resolvedBaseEntries.length === 0) {
      continue;
    }

    if (
      isGenderOnlyCreatureReference(labelTokens) &&
      isRejectedGenderOnlyCreatureReference(baseTokens, resolvedBaseEntries)
    ) {
      continue;
    }

    return {
      baseName,
      label: chooseCreatureReferenceLabel(entry, baseName, cleanedLabels),
      axes: deriveCreatureReferenceAxes(cleanedLabels),
      source: "composite",
      confidence: 0.86,
      fallbackEligible: false,
    };
  }

  return null;
}

function parseCreatureSuffixFamilyCandidate(
  entry: IndexedRecordEntry,
  exactNameLookup: Map<string, IndexedRecordEntry[]>,
): TitleCandidate | null {
  if (entry.record.category !== "creature") {
    return null;
  }

  const normalizedName = normalizeText(entry.record.name);
  if (!normalizedName) {
    return null;
  }

  for (const [baseName, requiredTrait] of CREATURE_SUFFIX_FAMILY_ALLOWLIST) {
    if (normalizedName === baseName || !normalizedName.endsWith(` ${baseName}`)) {
      continue;
    }
    if (!entry.record.traits.includes(requiredTrait)) {
      continue;
    }

    const resolvedBaseEntries = exactNameLookup.get(exactLookupKey(entry, toTitleWords(baseName))) ?? [];
    if (!resolvedBaseEntries.some((baseEntry) => baseEntry.record.traits.includes(requiredTrait))) {
      continue;
    }

    return {
      baseName: toTitleWords(baseName),
      label: entry.record.name,
      axes: ["other"],
      source: "namePattern",
      confidence: 0.68,
      fallbackEligible: false,
    };
  }

  return null;
}

function normalizeTitleToken(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "");
}

function tokenizeTitle(name: string): TitleToken[] {
  return [...name.matchAll(/[A-Za-z0-9'+-]+/g)]
    .map((match) => {
      const raw = match[0];
      const start = match.index;
      const normalized = normalizeTitleToken(raw);
      return {
        raw,
        normalized,
        meaningful: normalized.length > 0 && !TITLE_GLUE_TOKENS.has(normalized),
        start,
        end: start + raw.length,
      };
    })
    .filter((token) => token.normalized.length > 0);
}

function simplifyFoundryInlineMarkup(value: string): string {
  return value
    .replace(UUID_PATTERN, (_match, raw: string, label: string | undefined) => {
      if (label) {
        return label;
      }

      const token = raw.split(".").pop() ?? raw;
      return token.split(":").pop()?.replace(/[-_]+/g, " ") ?? "";
    })
    .replace(CHECK_PATTERN, (_match, label: string | undefined) => label ?? "")
    .replace(INLINE_PATTERN, (_match, label: string | undefined) => label ?? "");
}

function normalizeLeadSentence(value: string): string {
  return normalizeText(
    simplifyFoundryInlineMarkup(value)
      .replace(/\btype\s+(?:i|ii|iii|iv|v|vi|vii|viii|ix|x)\b/gi, "type {{ordinal}}")
      .replace(/\+\d+\b/g, "{{number}}")
      .replace(/\b\d+(?:st|nd|rd|th)\b/gi, "{{ordinal}}")
      .replace(/\b\d+\b/g, "{{number}}")
      .replace(/\b(one|two|three|four|five|six|seven|eight|nine|ten)\b/gi, "{{number}}"),
  );
}

function isStructuralLeadLine(value: string): boolean {
  return STRUCTURAL_LINE_PREFIXES.some((prefix) => value === prefix || value.startsWith(`${prefix} `));
}

function extractLeadBlock(descriptionText: string | null): string {
  if (!descriptionText) {
    return "";
  }

  const lines = descriptionText
    .split(/\n+/)
    .map((line) => normalizeText(simplifyFoundryInlineMarkup(line)))
    .filter(Boolean);

  while (lines.length > 0 && isStructuralLeadLine(lines[0]!)) {
    lines.shift();
  }

  return normalizeLeadSentence(lines[0] ?? "");
}

function commonPrefixTokens(left: string, right: string): number {
  const leftTokens = left.split(" ").filter(Boolean);
  const rightTokens = right.split(" ").filter(Boolean);
  const limit = Math.min(leftTokens.length, rightTokens.length);

  let count = 0;
  while (count < limit && leftTokens[count] === rightTokens[count]) {
    count += 1;
  }

  return count;
}

function leadBlockSimilarity(left: string, right: string): number {
  if (!left || !right) {
    return 0;
  }

  const leftTokens = left.split(" ").filter(Boolean);
  const rightTokens = right.split(" ").filter(Boolean);
  const minLength = Math.min(leftTokens.length, rightTokens.length);
  if (minLength === 0) {
    return 0;
  }

  return commonPrefixTokens(left, right) / minLength;
}

function longestCommonSubsequenceLength(left: string[], right: string[]): number {
  if (left.length === 0 || right.length === 0) {
    return 0;
  }

  const dp = Array.from({ length: left.length + 1 }, () => Array<number>(right.length + 1).fill(0));
  for (let leftIndex = 1; leftIndex <= left.length; leftIndex += 1) {
    for (let rightIndex = 1; rightIndex <= right.length; rightIndex += 1) {
      if (left[leftIndex - 1] === right[rightIndex - 1]) {
        dp[leftIndex]![rightIndex] = (dp[leftIndex - 1]![rightIndex - 1] ?? 0) + 1;
      } else {
        dp[leftIndex]![rightIndex] = Math.max(dp[leftIndex - 1]![rightIndex] ?? 0, dp[leftIndex]![rightIndex - 1] ?? 0);
      }
    }
  }

  return dp[left.length]![right.length] ?? 0;
}

function tokenSequenceSimilarity(left: string, right: string): number {
  const leftTokens = left.split(" ").filter(Boolean);
  const rightTokens = right.split(" ").filter(Boolean);
  if (leftTokens.length === 0 || rightTokens.length === 0) {
    return 0;
  }

  return longestCommonSubsequenceLength(leftTokens, rightTokens) / Math.max(leftTokens.length, rightTokens.length);
}

function descriptionPasses(left: string, right: string): boolean {
  if (!left || !right) {
    return false;
  }

  if (left === right) {
    return true;
  }

  const common = commonPrefixTokens(left, right);
  const similarity = leadBlockSimilarity(left, right);
  const sequenceSimilarity = tokenSequenceSimilarity(left, right);
  return (common >= 16 && similarity >= 0.35) || sequenceSimilarity >= 0.75;
}

function variantGroupKey(entry: IndexedRecordEntry, baseName: string): string {
  if (entry.record.category === "creature") {
    return creatureFamilyKey(baseName);
  }

  return [entry.record.category, toFamilySlug(entry.pack.name), toFamilySlug(baseName)].join(":");
}

function memberKey(entry: IndexedRecordEntry): string {
  return entry.record.recordKey;
}

function groupHasMeaningfulLabels(members: GroupMember[]): boolean {
  return members.some((member) => Boolean(member.candidate.label));
}

function isExactBaseMember(member: GroupMember, baseName: string): boolean {
  return normalizeText(member.entry.record.name) === normalizeText(baseName);
}

function bestDescriptionTemplate(members: GroupMember[]): GroupMember | null {
  const candidates = members.filter((member) => member.leadBlock.length > 0);
  if (candidates.length < 2) {
    return null;
  }

  let best: GroupMember | null = null;
  let bestScore = -1;
  for (const member of candidates) {
    const total = candidates.reduce((sum, candidate) => {
      if (candidate.entry.record.recordKey === member.entry.record.recordKey) {
        return sum;
      }

      return (
        sum +
        Math.max(
          leadBlockSimilarity(member.leadBlock, candidate.leadBlock),
          tokenSequenceSimilarity(member.leadBlock, candidate.leadBlock),
        )
      );
    }, 0);
    if (total > bestScore) {
      best = member;
      bestScore = total;
    }
  }

  return best;
}

function deriveDescriptionMembers(members: GroupMember[]): GroupMember[] {
  const template = bestDescriptionTemplate(members);
  if (!template) {
    return [];
  }

  return members
    .map((member) => ({
      ...member,
      descriptionScore: Math.max(
        member.leadBlock.length > 0 ? leadBlockSimilarity(template.leadBlock, member.leadBlock) : 0,
        tokenSequenceSimilarity(template.leadBlock, member.leadBlock),
      ),
    }))
    .filter((member) => {
      if (member.entry.record.recordKey === template.entry.record.recordKey) {
        return true;
      }

      return (
        descriptionPasses(template.leadBlock, member.leadBlock) ||
        tokenSequenceSimilarity(template.leadBlock, member.leadBlock) >= 0.75
      );
    });
}

function includeExactBaseMembers(
  descriptionMembers: GroupMember[],
  allMembers: GroupMember[],
  baseName: string,
): GroupMember[] {
  const included = new Map(descriptionMembers.map((member) => [member.entry.record.recordKey, member]));
  for (const member of allMembers) {
    if (isExactBaseMember(member, baseName)) {
      included.set(member.entry.record.recordKey, member);
    }
  }

  return [...included.values()];
}

function cleanDerivedLabel(value: string): string | null {
  const cleaned = value
    .replace(/^[\s,;:/-]+/, "")
    .replace(/[\s,;:/-]+$/, "")
    .replace(/^\((.*)\)$/s, "$1")
    .replace(/\)\s+\(/g, ", ")
    .replace(/[()]/g, "")
    .replace(/\s+/g, " ")
    .trim();
  return cleaned.length > 0 ? cleaned : null;
}

function meaningfulTitleSequence(tokens: TitleToken[]): string[] {
  return tokens.filter((token) => token.meaningful).map((token) => token.normalized);
}

function longestCommonSubsequence(left: string[], right: string[]): string[] {
  if (left.length === 0 || right.length === 0) {
    return [];
  }

  const dp = Array.from({ length: left.length + 1 }, () => Array<number>(right.length + 1).fill(0));
  for (let leftIndex = 1; leftIndex <= left.length; leftIndex += 1) {
    for (let rightIndex = 1; rightIndex <= right.length; rightIndex += 1) {
      if (left[leftIndex - 1] === right[rightIndex - 1]) {
        dp[leftIndex]![rightIndex] = (dp[leftIndex - 1]![rightIndex - 1] ?? 0) + 1;
      } else {
        dp[leftIndex]![rightIndex] = Math.max(dp[leftIndex - 1]![rightIndex] ?? 0, dp[leftIndex]![rightIndex - 1] ?? 0);
      }
    }
  }

  const result: string[] = [];
  let leftIndex = left.length;
  let rightIndex = right.length;
  while (leftIndex > 0 && rightIndex > 0) {
    if (left[leftIndex - 1] === right[rightIndex - 1]) {
      result.unshift(left[leftIndex - 1]!);
      leftIndex -= 1;
      rightIndex -= 1;
      continue;
    }

    if ((dp[leftIndex - 1]![rightIndex] ?? 0) >= (dp[leftIndex]![rightIndex - 1] ?? 0)) {
      leftIndex -= 1;
    } else {
      rightIndex -= 1;
    }
  }

  return result;
}

function findSubsequencePositions(tokens: TitleToken[], subsequence: string[]): number[] | null {
  const positions: number[] = [];
  let searchIndex = 0;
  for (const target of subsequence) {
    let found = false;
    while (searchIndex < tokens.length) {
      const token = tokens[searchIndex]!;
      if (token.meaningful && token.normalized === target) {
        positions.push(searchIndex);
        searchIndex += 1;
        found = true;
        break;
      }
      searchIndex += 1;
    }

    if (!found) {
      return null;
    }
  }

  return positions;
}

function deriveSharedTitleMetadata(members: GroupMember[]): {
  baseName: string;
  labels: Map<string, string | null>;
} | null {
  if (members.length < 2) {
    return null;
  }

  const tokenized = members.map((member) => ({
    member,
    tokens: tokenizeTitle(member.entry.record.name),
  }));
  let shared = meaningfulTitleSequence(tokenized[0]!.tokens);
  for (const item of tokenized.slice(1)) {
    shared = longestCommonSubsequence(shared, meaningfulTitleSequence(item.tokens));
    if (shared.length === 0) {
      return null;
    }
  }

  const firstPositions = findSubsequencePositions(tokenized[0]!.tokens, shared);
  if (!firstPositions || firstPositions.length === 0) {
    return null;
  }

  const firstStart = tokenized[0]!.tokens[firstPositions[0]!]!.start;
  const firstEnd = tokenized[0]!.tokens[firstPositions[firstPositions.length - 1]!]!.end;
  const baseName = tokenized[0]!.member.entry.record.name.slice(firstStart, firstEnd).trim();
  if (!baseName) {
    return null;
  }

  const labels = new Map<string, string | null>();
  for (const item of tokenized) {
    const positions = findSubsequencePositions(item.tokens, shared);
    if (!positions || positions.length === 0) {
      return null;
    }

    const start = item.tokens[positions[0]!]!.start;
    const end = item.tokens[positions[positions.length - 1]!]!.end;
    const prefix = cleanDerivedLabel(item.member.entry.record.name.slice(0, start));
    const suffix = cleanDerivedLabel(item.member.entry.record.name.slice(end));
    const label = [prefix, suffix].filter((part): part is string => Boolean(part)).join(", ");
    labels.set(item.member.entry.record.recordKey, label || null);
  }

  return { baseName, labels };
}

function needsBroaderEvidence(members: GroupMember[], baseName: string, sharedTitleBase: string | null): boolean {
  const effectiveBase = sharedTitleBase ?? baseName;
  const hasBase = members.some((member) => isExactBaseMember(member, effectiveBase));
  const hasStructuredAxis = members.some((member) => member.candidate.axes.some((axis) => axis !== "other"));
  const hasStackedLabel = members.some((member) => (member.candidate.label ?? "").includes(", "));
  const reliesOnLooseStemOnly = members
    .filter((member) => !isExactBaseMember(member, effectiveBase))
    .every((member) => member.candidate.confidence <= 0.52);
  const hasSharedTitleBase = Boolean(sharedTitleBase && normalizeText(sharedTitleBase).length > 0);
  return (
    !hasBase &&
    !hasStructuredAxis &&
    !hasStackedLabel &&
    reliesOnLooseStemOnly &&
    !hasSharedTitleBase &&
    members.length < 4
  );
}

function exactBaseCandidate(baseName: string): TitleCandidate {
  return {
    baseName,
    label: null,
    axes: [],
    source: "composite",
    confidence: 0.62,
    fallbackEligible: false,
  };
}

function assignGroup(
  members: GroupMember[],
  baseName: string,
  source: VariantSource,
  confidence: number,
  derivedLabels?: Map<string, string | null>,
): void {
  if (members.length < 2 || !groupHasMeaningfulLabels(members)) {
    return;
  }

  const axes = uniqueSorted(members.flatMap((member) => member.candidate.axes));
  const boost = Math.min(0.08, Math.max(0, members.length - 2) * 0.02);
  for (const member of members) {
    member.entry.record.variantFamilyKey = variantGroupKey(member.entry, baseName);
    member.entry.record.variantBaseName = baseName;
    member.entry.record.variantLabel = derivedLabels?.get(member.entry.record.recordKey) ?? member.candidate.label;
    member.entry.record.variantAxes = axes;
    member.entry.record.variantConfidence = Math.min(
      0.99,
      confidence + boost + Math.min(0.08, member.descriptionScore * 0.12),
    );
    member.entry.record.variantSource = source;
  }
}

export function assignVariantFamilies(entries: IndexedRecordEntry[]): void {
  const eligibleEntries = entries.filter(
    (entry) =>
      entry.record.category === "equipment" ||
      entry.record.category === "spell" ||
      entry.record.category === "creature",
  );
  const byExactName = new Map<string, IndexedRecordEntry[]>();
  const candidateGroups = new Map<string, CandidateGroup>();
  const knownCreatureBaseNames = new Set<string>();
  const structuredCandidates = new Map<string, TitleCandidate>();
  const structuredGroupKeys = new Map<string, string>();

  for (const entry of eligibleEntries) {
    const exactKey = exactLookupKey(entry, entry.record.name);
    const bucket = byExactName.get(exactKey) ?? [];
    bucket.push(entry);
    byExactName.set(exactKey, bucket);

    const candidate =
      parseEntryStructuredCandidate(entry) ??
      (entry.record.category === "creature"
        ? null
        : (buildSuffixScaffoldCandidate(entry) ?? buildLooseStemCandidate(entry)));
    if (!candidate) {
      continue;
    }

    if (entry.record.category === "creature") {
      knownCreatureBaseNames.add(normalizeText(candidate.baseName));
    }
    structuredCandidates.set(entry.record.recordKey, candidate);

    const groupKey = groupLookupKey(entry, candidate.baseName);
    structuredGroupKeys.set(entry.record.recordKey, groupKey);
    const group = candidateGroups.get(groupKey) ?? {
      baseName: candidate.baseName,
      category: entry.record.category,
      packName: entry.pack.name,
      members: new Map<string, GroupMember>(),
    };
    group.members.set(memberKey(entry), {
      entry,
      candidate,
      leadBlock: extractLeadBlock(entry.record.descriptionText),
      descriptionScore: 0,
    });
    candidateGroups.set(groupKey, group);
  }

  for (const entry of eligibleEntries) {
    if (entry.record.category !== "creature") {
      continue;
    }

    const candidate = parseCreatureReferenceCandidate(entry, byExactName, knownCreatureBaseNames);
    if (!candidate) {
      continue;
    }

    const existingCandidate = structuredCandidates.get(entry.record.recordKey);
    if (existingCandidate && existingCandidate.baseName === candidate.baseName) {
      continue;
    }

    const priorGroupKey = structuredGroupKeys.get(entry.record.recordKey);
    if (priorGroupKey) {
      candidateGroups.get(priorGroupKey)?.members.delete(memberKey(entry));
    }

    const groupKey = groupLookupKey(entry, candidate.baseName);
    const group = candidateGroups.get(groupKey) ?? {
      baseName: candidate.baseName,
      category: entry.record.category,
      packName: entry.pack.name,
      members: new Map<string, GroupMember>(),
    };
    group.members.set(memberKey(entry), {
      entry,
      candidate,
      leadBlock: extractLeadBlock(entry.record.descriptionText),
      descriptionScore: 0,
    });
    candidateGroups.set(groupKey, group);
    structuredCandidates.set(entry.record.recordKey, candidate);
    structuredGroupKeys.set(entry.record.recordKey, groupKey);
  }

  for (const entry of eligibleEntries) {
    if (entry.record.category !== "creature" || structuredCandidates.has(entry.record.recordKey)) {
      continue;
    }

    const candidate = parseCreatureSuffixFamilyCandidate(entry, byExactName);
    if (!candidate) {
      continue;
    }

    const groupKey = groupLookupKey(entry, candidate.baseName);
    const group = candidateGroups.get(groupKey) ?? {
      baseName: candidate.baseName,
      category: entry.record.category,
      packName: entry.pack.name,
      members: new Map<string, GroupMember>(),
    };
    group.members.set(memberKey(entry), {
      entry,
      candidate,
      leadBlock: extractLeadBlock(entry.record.descriptionText),
      descriptionScore: 0,
    });
    candidateGroups.set(groupKey, group);
    structuredCandidates.set(entry.record.recordKey, candidate);
    structuredGroupKeys.set(entry.record.recordKey, groupKey);
  }

  for (const group of candidateGroups.values()) {
    const normalizedBase = normalizeText(group.baseName);
    const baseLookupKey =
      group.category === "creature"
        ? `${group.category}:${normalizedBase}`
        : `${group.category}:${group.packName}:${normalizedBase}`;
    const baseMembers = byExactName.get(baseLookupKey) ?? [];
    for (const entry of baseMembers) {
      if (group.members.has(memberKey(entry))) {
        continue;
      }
      group.members.set(memberKey(entry), {
        entry,
        candidate: exactBaseCandidate(group.baseName),
        leadBlock: extractLeadBlock(entry.record.descriptionText),
        descriptionScore: 0,
      });
    }

    const members = [...group.members.values()];
    if (members.length < 2) {
      continue;
    }

    if (group.category === "creature") {
      const sources = new Set(members.map((member) => member.candidate.source));
      const source = sources.size === 1 ? [...sources][0]! : "composite";
      const confidence = Math.max(...members.map((member) => member.candidate.confidence));
      assignGroup(members, group.baseName, source, confidence);
      continue;
    }

    const fallbackMembers = members.filter(
      (member) => member.candidate.fallbackEligible || isExactBaseMember(member, group.baseName),
    );
    const labeledMembers = members.filter((member) => Boolean(member.candidate.label));
    const canUseFallback =
      labeledMembers.length > 0 && labeledMembers.every((member) => member.candidate.fallbackEligible);
    if (canUseFallback && fallbackMembers.length >= 2 && groupHasMeaningfulLabels(fallbackMembers)) {
      const sources = new Set(fallbackMembers.map((member) => member.candidate.source));
      const source = sources.size === 1 ? [...sources][0]! : "composite";
      const confidence = Math.max(...fallbackMembers.map((member) => member.candidate.confidence));
      assignGroup(fallbackMembers, group.baseName, source, confidence);
      continue;
    }

    const descriptionMembers = includeExactBaseMembers(deriveDescriptionMembers(members), members, group.baseName);
    if (descriptionMembers.length >= 2 && groupHasMeaningfulLabels(descriptionMembers)) {
      const sharedTitleMetadata = deriveSharedTitleMetadata(descriptionMembers);
      const resolvedBaseName = sharedTitleMetadata?.baseName ?? group.baseName;
      if (needsBroaderEvidence(descriptionMembers, group.baseName, sharedTitleMetadata?.baseName ?? null)) {
        continue;
      }

      const averageScore =
        descriptionMembers.reduce((sum, member) => sum + member.descriptionScore, 0) / descriptionMembers.length;
      assignGroup(
        descriptionMembers,
        resolvedBaseName,
        "composite",
        0.78 + Math.min(0.08, averageScore * 0.15),
        sharedTitleMetadata?.labels,
      );
      continue;
    }
  }
}
