import type { SearchCategory, SearchSubcategory } from "../types.js";
import { normalizeText, uniqueSorted } from "../utils.js";

export type DerivedTagContext = {
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  descriptionText: string | null;
  traits: string[];
  families?: string[];
  references?: DerivedTagReference[];
};

export type DerivedTagReference = {
  recordKey: string;
  packName: string;
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  traits: string[];
};

export type TextMatchScope = "either" | "name" | "description";
type TextMatchMode = "token" | "phrase" | "template";
type TemplatePlaceholder = "n" | "d" | "r";

export type TextAnchor = string | {
  value: string;
  mode?: TextMatchMode;
  scope?: TextMatchScope;
};

export type ReferencePredicate = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  packName?: string;
  nameAny?: string[];
  traitsAny?: string[];
  traitsAll?: string[];
};

export type TextProximityConstraint = {
  terms: TextAnchor[];
  window: number;
  ordered?: boolean;
  scope?: TextMatchScope;
  minTermsMatched?: number;
};

export type DerivedTagMatchClause = {
  score?: number;
  traitsAny?: string[];
  traitsAll?: string[];
  traitsNone?: string[];
  familiesAny?: string[];
  familiesAll?: string[];
  familiesNone?: string[];
  textAny?: TextAnchor[];
  minTextAnyMatches?: number;
  textAll?: TextAnchor[];
  textNear?: TextProximityConstraint[];
  textNotNear?: TextProximityConstraint[];
  referencesAny?: string[];
  referencesAll?: string[];
  referencesWhere?: ReferencePredicate[];
  minReferenceMatches?: number;
};

export type DerivedTagRule = {
  tag: string;
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
  threshold?: number;
  requiresTags?: string[];
  anyOf?: DerivedTagMatchClause[];
  allOf?: DerivedTagMatchClause[];
  noneOf?: DerivedTagMatchClause[];
};

type NormalizedTextView = {
  text: string;
  tokens: string[];
  tokenSet: Set<string>;
};

type NormalizedTextAnchor =
  | {
    value: string;
    mode: "token" | "phrase";
    scope: TextMatchScope;
  }
  | {
    mode: "template";
    scope: TextMatchScope;
    templateParts: TemplateTokenPart[];
  };

type TemplateTokenPart =
  | {
    type: "literal";
    value: string;
  }
  | {
    type: "placeholder";
    value: TemplatePlaceholder;
  };

type NormalizedDerivedTagReference = {
  key: string;
  packName: string;
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  traits: Set<string>;
};

type NormalizedDerivedTagContext = {
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  traits: Set<string>;
  families: Set<string>;
  name: NormalizedTextView;
  description: NormalizedTextView;
  referenceKeys: Set<string>;
  references: NormalizedDerivedTagReference[];
};

function buildTextView(value: string): NormalizedTextView {
  const text = normalizeText(value);
  const tokens = text.length > 0 ? text.split(" ") : [];
  return {
    text,
    tokens,
    tokenSet: new Set(tokens),
  };
}

function normalizeTemplateAnchorValue(value: string): TemplateTokenPart[] {
  const raw = value.toLowerCase().replace(/&nbsp;/g, " ");
  const parts: TemplateTokenPart[] = [];
  const placeholderPattern = /\{([ndr])\}/g;
  let offset = 0;

  for (const match of raw.matchAll(placeholderPattern)) {
    const index = match.index ?? 0;
    const literal = normalizeText(raw.slice(offset, index));
    if (literal.length > 0) {
      parts.push(...literal.split(" ").map((token) => ({
        type: "literal" as const,
        value: token,
      })));
    }

    const placeholder = match[1]?.toLowerCase();
    if (placeholder === "n" || placeholder === "d" || placeholder === "r") {
      parts.push({
        type: "placeholder",
        value: placeholder,
      });
    }

    offset = index + match[0].length;
  }

  const trailingLiteral = normalizeText(raw.slice(offset));
  if (trailingLiteral.length > 0) {
    parts.push(...trailingLiteral.split(" ").map((token) => ({
      type: "literal" as const,
      value: token,
    })));
  }

  return parts;
}

function normalizeAnchor(anchor: TextAnchor): NormalizedTextAnchor | null {
  const raw = typeof anchor === "string" ? { value: anchor } : anchor;
  const mode = raw.mode ?? (raw.value.includes(" ") ? "phrase" : "token");

  if (mode === "template") {
    const templateParts = normalizeTemplateAnchorValue(raw.value);
    if (templateParts.length === 0) {
      return null;
    }

    return {
      mode,
      scope: raw.scope ?? "either",
      templateParts,
    };
  }

  const value = normalizeText(raw.value);
  if (!value) {
    return null;
  }
  return {
    value,
    mode,
    scope: raw.scope ?? "either",
  };
}

function containsPhrase(text: string, phrase: string): boolean {
  return ` ${text} `.includes(` ${phrase} `);
}

function getTextViews(
  context: NormalizedDerivedTagContext,
  scope: TextMatchScope,
): Array<{ scope: Exclude<TextMatchScope, "either">; view: NormalizedTextView }> {
  if (scope === "name") {
    return [{ scope: "name", view: context.name }];
  }
  if (scope === "description") {
    return [{ scope: "description", view: context.description }];
  }

  return [
    { scope: "name", view: context.name },
    { scope: "description", view: context.description },
  ];
}

function matchTemplatePlaceholder(tokens: string[], index: number, placeholder: TemplatePlaceholder): number {
  const token = tokens[index];
  if (!token) {
    return 0;
  }

  if (placeholder === "n") {
    return /^\d+$/.test(token) ? 1 : 0;
  }

  if (placeholder === "d") {
    return /^\d+d\d+$/.test(token) ? 1 : 0;
  }

  const next = tokens[index + 1];
  if (!/^\d+$/.test(token) || !next || !["foot", "feet"].includes(next)) {
    return 0;
  }

  const geometry = tokens[index + 2];
  if (!geometry) {
    return 2;
  }

  return ["aura", "burst", "cone", "cube", "emanation", "line", "radius"].includes(geometry) ? 3 : 2;
}

function findTemplateOccurrences(
  view: NormalizedTextView,
  templateParts: TemplateTokenPart[],
): TextOccurrence[] {
  if (templateParts.length === 0) {
    return [];
  }

  const occurrences: TextOccurrence[] = [];
  for (let start = 0; start < view.tokens.length; start += 1) {
    let cursor = start;
    let matched = true;

    for (const part of templateParts) {
      if (part.type === "literal") {
        if (view.tokens[cursor] !== part.value) {
          matched = false;
          break;
        }
        cursor += 1;
        continue;
      }

      const length = matchTemplatePlaceholder(view.tokens, cursor, part.value);
      if (length === 0) {
        matched = false;
        break;
      }
      cursor += length;
    }

    if (matched && cursor > start) {
      occurrences.push({
        start,
        end: cursor - 1,
      });
    }
  }

  return occurrences;
}

function matchesTextAnchor(context: NormalizedDerivedTagContext, anchor: TextAnchor): boolean {
  const normalized = normalizeAnchor(anchor);
  if (!normalized) {
    return false;
  }

  return getTextViews(context, normalized.scope).some(({ scope, view }) => findNormalizedTextOccurrences(view, scope, normalized).length > 0);
}

function countMatchingTextAnchors(context: NormalizedDerivedTagContext, anchors: TextAnchor[]): number {
  return anchors.reduce((count, anchor) => count + (matchesTextAnchor(context, anchor) ? 1 : 0), 0);
}

type TextOccurrence = {
  start: number;
  end: number;
};

function findNormalizedTextOccurrences(
  view: NormalizedTextView,
  viewScope: Exclude<TextMatchScope, "either">,
  normalized: NormalizedTextAnchor,
): TextOccurrence[] {
  if (normalized.scope !== "either" && normalized.scope !== viewScope) {
    return [];
  }

  if (normalized.mode === "token") {
    return view.tokens.flatMap((token, index) => token === normalized.value ? [{ start: index, end: index }] : []);
  }

  if (normalized.mode === "template") {
    return findTemplateOccurrences(view, normalized.templateParts);
  }

  const phraseTokens = normalized.value.split(" ");
  const occurrences: TextOccurrence[] = [];
  for (let index = 0; index <= view.tokens.length - phraseTokens.length; index += 1) {
    let matches = true;
    for (let phraseIndex = 0; phraseIndex < phraseTokens.length; phraseIndex += 1) {
      if (view.tokens[index + phraseIndex] !== phraseTokens[phraseIndex]) {
        matches = false;
        break;
      }
    }

    if (matches) {
      occurrences.push({
        start: index,
        end: index + phraseTokens.length - 1,
      });
    }
  }

  return occurrences;
}

function findTextOccurrences(
  view: NormalizedTextView,
  viewScope: Exclude<TextMatchScope, "either">,
  anchor: TextAnchor,
): TextOccurrence[] {
  const normalized = normalizeAnchor(anchor);
  if (!normalized) {
    return [];
  }

  return findNormalizedTextOccurrences(view, viewScope, normalized);
}

function spansWithinWindow(occurrences: TextOccurrence[], window: number): boolean {
  if (occurrences.length === 0) {
    return false;
  }

  const start = Math.min(...occurrences.map((occurrence) => occurrence.start));
  const end = Math.max(...occurrences.map((occurrence) => occurrence.end));
  return end - start <= window;
}

function hasTextProximityInView(
  occurrenceLists: TextOccurrence[][],
  minimumMatches: number,
  window: number,
  ordered: boolean,
  termIndex = 0,
  chosen: TextOccurrence[] = [],
): boolean {
  if (chosen.length >= minimumMatches && spansWithinWindow(chosen, window)) {
    return true;
  }
  if (termIndex >= occurrenceLists.length) {
    return false;
  }
  if (chosen.length + (occurrenceLists.length - termIndex) < minimumMatches) {
    return false;
  }

  if (hasTextProximityInView(occurrenceLists, minimumMatches, window, ordered, termIndex + 1, chosen)) {
    return true;
  }

  for (const occurrence of occurrenceLists[termIndex] ?? []) {
    if (ordered && chosen.length > 0 && occurrence.start < (chosen.at(-1)?.start ?? 0)) {
      continue;
    }

    const nextChosen = [...chosen, occurrence];
    if (!spansWithinWindow(nextChosen, window)) {
      continue;
    }

    if (hasTextProximityInView(occurrenceLists, minimumMatches, window, ordered, termIndex + 1, nextChosen)) {
      return true;
    }
  }

  return false;
}

function matchesTextProximity(context: NormalizedDerivedTagContext, constraint: TextProximityConstraint): boolean {
  const minimumMatches = Math.max(1, Math.min(constraint.minTermsMatched ?? constraint.terms.length, constraint.terms.length));
  if (constraint.terms.length === 0) {
    return false;
  }

  for (const { scope, view } of getTextViews(context, constraint.scope ?? "either")) {
    const occurrenceLists = constraint.terms.map((term) => findTextOccurrences(view, scope, term));
    const termsWithMatches = occurrenceLists.filter((occurrences) => occurrences.length > 0).length;
    if (termsWithMatches < minimumMatches) {
      continue;
    }

    if (hasTextProximityInView(
      occurrenceLists,
      minimumMatches,
      Math.max(0, constraint.window),
      constraint.ordered ?? false,
    )) {
      return true;
    }
  }

  return false;
}

function matchesReferencePredicate(reference: NormalizedDerivedTagReference, predicate: ReferencePredicate): boolean {
  if (predicate.category && reference.category !== predicate.category) {
    return false;
  }
  if (predicate.subcategory && reference.subcategory !== predicate.subcategory) {
    return false;
  }
  if (predicate.packName && reference.packName !== normalizeText(predicate.packName)) {
    return false;
  }
  if (predicate.nameAny && !predicate.nameAny.some((name) => reference.name === normalizeText(name))) {
    return false;
  }
  if (predicate.traitsAny && !predicate.traitsAny.some((trait) => reference.traits.has(normalizeText(trait)))) {
    return false;
  }
  if (predicate.traitsAll && !predicate.traitsAll.every((trait) => reference.traits.has(normalizeText(trait)))) {
    return false;
  }

  return true;
}

function countMatchingReferences(context: NormalizedDerivedTagContext, predicates: ReferencePredicate[]): number {
  if (predicates.length === 0) {
    return 0;
  }

  return context.references.filter((reference) => predicates.some((predicate) => matchesReferencePredicate(reference, predicate))).length;
}

function matchesClause(context: NormalizedDerivedTagContext, clause: DerivedTagMatchClause): boolean {
  if (clause.traitsAny && !clause.traitsAny.some((trait) => context.traits.has(normalizeText(trait)))) {
    return false;
  }
  if (clause.traitsAll && !clause.traitsAll.every((trait) => context.traits.has(normalizeText(trait)))) {
    return false;
  }
  if (clause.traitsNone && clause.traitsNone.some((trait) => context.traits.has(normalizeText(trait)))) {
    return false;
  }
  if (clause.familiesAny && !clause.familiesAny.some((family) => context.families.has(normalizeText(family)))) {
    return false;
  }
  if (clause.familiesAll && !clause.familiesAll.every((family) => context.families.has(normalizeText(family)))) {
    return false;
  }
  if (clause.familiesNone && clause.familiesNone.some((family) => context.families.has(normalizeText(family)))) {
    return false;
  }
  if (clause.textAny) {
    const minimumMatches = Math.max(1, Math.min(clause.minTextAnyMatches ?? 1, clause.textAny.length));
    if (countMatchingTextAnchors(context, clause.textAny) < minimumMatches) {
      return false;
    }
  }
  if (clause.textAll && !clause.textAll.every((anchor) => matchesTextAnchor(context, anchor))) {
    return false;
  }
  if (clause.textNear && !clause.textNear.some((constraint) => matchesTextProximity(context, constraint))) {
    return false;
  }
  if (clause.textNotNear && clause.textNotNear.some((constraint) => matchesTextProximity(context, constraint))) {
    return false;
  }
  if (clause.referencesAny && !clause.referencesAny.some((reference) => context.referenceKeys.has(normalizeDerivedTagReference(reference)))) {
    return false;
  }
  if (clause.referencesAll && !clause.referencesAll.every((reference) => context.referenceKeys.has(normalizeDerivedTagReference(reference)))) {
    return false;
  }
  if (clause.referencesWhere) {
    const minimumMatches = Math.max(1, clause.minReferenceMatches ?? 1);
    if (countMatchingReferences(context, clause.referencesWhere) < minimumMatches) {
      return false;
    }
  }
  return true;
}

function scoreClause(context: NormalizedDerivedTagContext, clause: DerivedTagMatchClause): number {
  if (!matchesClause(context, clause)) {
    return 0;
  }

  return clause.score ?? 1;
}

function matchesRule(
  context: NormalizedDerivedTagContext,
  tags: Set<string>,
  rule: DerivedTagRule,
): boolean {
  if (context.category !== rule.category) {
    return false;
  }
  if (rule.subcategories && (!context.subcategory || !rule.subcategories.includes(context.subcategory))) {
    return false;
  }
  if (rule.requiresTags && !rule.requiresTags.every((tag) => tags.has(tag))) {
    return false;
  }
  if (rule.allOf && !rule.allOf.every((clause) => matchesClause(context, clause))) {
    return false;
  }
  if (rule.noneOf && rule.noneOf.some((clause) => matchesClause(context, clause))) {
    return false;
  }

  if (rule.anyOf) {
    const totalScore = rule.anyOf.reduce((sum, clause) => sum + scoreClause(context, clause), 0);
    const threshold = rule.threshold ?? 1;
    if (totalScore < threshold) {
      return false;
    }
  }

  return true;
}

export function normalizeDerivedTagReference(value: string): string {
  const separatorIndex = value.indexOf(":");
  if (separatorIndex === -1) {
    return normalizeText(value);
  }

  const packName = value.slice(0, separatorIndex);
  const recordName = value.slice(separatorIndex + 1);
  return `${normalizeText(packName)}:${normalizeText(recordName)}`;
}

export function normalizeDerivedTag(value: string): string {
  return normalizeText(value).replace(/\s+/g, "_");
}

export function deriveRecordTagsFromRules(rules: DerivedTagRule[], input: DerivedTagContext): string[] {
  const references = (input.references ?? []).map((reference) => ({
    key: normalizeDerivedTagReference(`${reference.packName}:${reference.name}`),
    packName: normalizeText(reference.packName),
    name: normalizeText(reference.name),
    category: reference.category,
    subcategory: reference.subcategory,
    traits: new Set(reference.traits.map((trait) => normalizeText(trait)).filter(Boolean)),
  }));
  const context: NormalizedDerivedTagContext = {
    category: input.category,
    subcategory: input.subcategory,
    traits: new Set(input.traits.map((trait) => normalizeText(trait)).filter(Boolean)),
    families: new Set((input.families ?? []).map((family) => normalizeText(family)).filter(Boolean)),
    name: buildTextView(input.name),
    description: buildTextView(input.descriptionText ?? ""),
    referenceKeys: new Set(references.map((reference) => reference.key)),
    references,
  };
  const tags = new Set<string>();

  for (const rule of rules) {
    if (matchesRule(context, tags, rule)) {
      tags.add(rule.tag);
    }
  }

  return uniqueSorted([...tags]);
}
