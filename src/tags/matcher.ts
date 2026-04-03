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
type TextMatchMode = "token" | "phrase";

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

function normalizeAnchor(anchor: TextAnchor): { value: string; mode: TextMatchMode; scope: TextMatchScope } | null {
  const raw = typeof anchor === "string" ? { value: anchor } : anchor;
  const value = normalizeText(raw.value);
  if (!value) {
    return null;
  }

  return {
    value,
    mode: raw.mode ?? (value.includes(" ") ? "phrase" : "token"),
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

function matchesTextAnchor(context: NormalizedDerivedTagContext, anchor: TextAnchor): boolean {
  const normalized = normalizeAnchor(anchor);
  if (!normalized) {
    return false;
  }

  return getTextViews(context, normalized.scope).some(({ view }) => {
    if (normalized.mode === "token") {
      return view.tokenSet.has(normalized.value);
    }

    return containsPhrase(view.text, normalized.value);
  });
}

function countMatchingTextAnchors(context: NormalizedDerivedTagContext, anchors: TextAnchor[]): number {
  return anchors.reduce((count, anchor) => count + (matchesTextAnchor(context, anchor) ? 1 : 0), 0);
}

type TextOccurrence = {
  start: number;
  end: number;
};

function findTextOccurrences(
  view: NormalizedTextView,
  viewScope: Exclude<TextMatchScope, "either">,
  anchor: TextAnchor,
): TextOccurrence[] {
  const normalized = normalizeAnchor(anchor);
  if (!normalized) {
    return [];
  }
  if (normalized.scope !== "either" && normalized.scope !== viewScope) {
    return [];
  }

  if (normalized.mode === "token") {
    return view.tokens.flatMap((token, index) => token === normalized.value ? [{ start: index, end: index }] : []);
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
