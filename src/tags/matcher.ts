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

type DerivedTagMatchClause = {
  score?: number;
  traitsAny?: string[];
  traitsAll?: string[];
  familiesAny?: string[];
  familiesAll?: string[];
  textAny?: TextAnchor[];
  textAll?: TextAnchor[];
  referencesAny?: string[];
  referencesAll?: string[];
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
  tokenSet: Set<string>;
};

type NormalizedDerivedTagContext = {
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  traits: Set<string>;
  families: Set<string>;
  name: NormalizedTextView;
  description: NormalizedTextView;
  referenceKeys: Set<string>;
};

function buildTextView(value: string): NormalizedTextView {
  const text = normalizeText(value);
  return {
    text,
    tokenSet: new Set(text.length > 0 ? text.split(" ") : []),
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

function matchesTextAnchor(context: NormalizedDerivedTagContext, anchor: TextAnchor): boolean {
  const normalized = normalizeAnchor(anchor);
  if (!normalized) {
    return false;
  }

  const views = normalized.scope === "name"
    ? [context.name]
    : normalized.scope === "description"
      ? [context.description]
      : [context.name, context.description];

  return views.some((view) => {
    if (normalized.mode === "token") {
      return view.tokenSet.has(normalized.value);
    }

    return containsPhrase(view.text, normalized.value);
  });
}

function matchesClause(context: NormalizedDerivedTagContext, clause: DerivedTagMatchClause): boolean {
  if (clause.traitsAny && !clause.traitsAny.some((trait) => context.traits.has(normalizeText(trait)))) {
    return false;
  }
  if (clause.traitsAll && !clause.traitsAll.every((trait) => context.traits.has(normalizeText(trait)))) {
    return false;
  }
  if (clause.familiesAny && !clause.familiesAny.some((family) => context.families.has(normalizeText(family)))) {
    return false;
  }
  if (clause.familiesAll && !clause.familiesAll.every((family) => context.families.has(normalizeText(family)))) {
    return false;
  }
  if (clause.textAny && !clause.textAny.some((anchor) => matchesTextAnchor(context, anchor))) {
    return false;
  }
  if (clause.textAll && !clause.textAll.every((anchor) => matchesTextAnchor(context, anchor))) {
    return false;
  }
  if (clause.referencesAny && !clause.referencesAny.some((reference) => context.referenceKeys.has(normalizeDerivedTagReference(reference)))) {
    return false;
  }
  if (clause.referencesAll && !clause.referencesAll.every((reference) => context.referenceKeys.has(normalizeDerivedTagReference(reference)))) {
    return false;
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
  const context: NormalizedDerivedTagContext = {
    category: input.category,
    subcategory: input.subcategory,
    traits: new Set(input.traits.map((trait) => normalizeText(trait)).filter(Boolean)),
    families: new Set((input.families ?? []).map((family) => normalizeText(family)).filter(Boolean)),
    name: buildTextView(input.name),
    description: buildTextView(input.descriptionText ?? ""),
    referenceKeys: new Set((input.references ?? []).map((reference) => normalizeDerivedTagReference(`${reference.packName}:${reference.name}`))),
  };
  const tags = new Set<string>();

  for (const rule of rules) {
    if (matchesRule(context, tags, rule)) {
      tags.add(rule.tag);
    }
  }

  return uniqueSorted([...tags]);
}
