import winkNLP, { type PartOfSpeech } from "wink-nlp";
import model from "wink-eng-lite-web-model";

import type { SearchCategory, SearchSubcategory } from "../types.js";
import { normalizeText, uniqueSorted } from "../utils.js";

const nlp = winkNLP(model);
const its = nlp.its;

export type DerivedTagContext = {
  recordKey?: string;
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  descriptionText: string | null;
  blurbText?: string | null;
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

export type TextMatchScope = "either" | "name" | "description" | "blurb";
type PatternPlaceholder = "number" | "dice" | "range";
export type TextAnchorPosConstraint = PartOfSpeech | PartOfSpeech[];

export type TextAnchor = string | {
  value: string;
  scope?: TextMatchScope;
  pos?: TextAnchorPosConstraint[];
};

export type ReferencePredicate = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  packName?: string;
  nameAny?: string[];
  traitsAny?: string[];
  traitsAll?: string[];
};

export type TextNearConstraint = {
  all: TextAnchor[];
  window: number;
  ordered?: boolean;
  scope?: TextMatchScope;
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
  textNear?: TextNearConstraint[];
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
  tokens: string[];
  posTags: PartOfSpeech[];
};

type NormalizedTextAnchor = {
  scope: TextMatchScope;
  patternParts: PatternTokenPart[];
  posByToken?: PartOfSpeech[][];
};

type PatternTokenPart =
  | {
    type: "literal";
    tokens: string[];
  }
  | {
    type: "placeholder";
    value: PatternPlaceholder;
  }
  | {
    type: "alternative";
    tokenOptions: string[][];
  }
  | {
    type: "optional";
    tokens: string[];
  }
  | {
    type: "gap";
    min: number;
    max: number;
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
  blurb: NormalizedTextView;
  referenceKeys: Set<string>;
  references: NormalizedDerivedTagReference[];
};

type CompiledReferencePredicate = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  packName?: string;
  nameAny?: string[];
  traitsAny?: string[];
  traitsAll?: string[];
};

type CompiledTextNearConstraint = {
  all: NormalizedTextAnchor[];
  window: number;
  ordered: boolean;
  scope: TextMatchScope;
};

type CompiledDerivedTagMatchClause = {
  score?: number;
  traitsAny?: string[];
  traitsAll?: string[];
  traitsNone?: string[];
  familiesAny?: string[];
  familiesAll?: string[];
  familiesNone?: string[];
  textAny?: NormalizedTextAnchor[];
  minTextAnyMatches?: number;
  textAll?: NormalizedTextAnchor[];
  textNear?: CompiledTextNearConstraint[];
  referencesAny?: string[];
  referencesAll?: string[];
  referencesWhere?: CompiledReferencePredicate[];
  minReferenceMatches?: number;
};

type CompiledDerivedTagRule = {
  tag: string;
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
  threshold?: number;
  requiresTags?: string[];
  anyOf?: CompiledDerivedTagMatchClause[];
  allOf?: CompiledDerivedTagMatchClause[];
  noneOf?: CompiledDerivedTagMatchClause[];
};

const normalizedStringAnchorCache = new Map<string, NormalizedTextAnchor | null>();
const normalizedObjectAnchorCache = new WeakMap<Exclude<TextAnchor, string>, NormalizedTextAnchor | null>();
const compiledRulesCache = new WeakMap<DerivedTagRule[], {
  rules: CompiledDerivedTagRule[];
  posScopes: Set<TextMatchScope>;
}>();

function analyzeTextPartOfSpeech(normalized: string, tokens: string[]): PartOfSpeech[] {
  if (normalized.length === 0 || tokens.length === 0) {
    return [];
  }

  const doc = nlp.readDoc(normalized);
  const analyzedTokens = doc.tokens().out(its.normal) as string[];
  const analyzedPos = doc.tokens().out(its.pos) as PartOfSpeech[];
  if (
    analyzedTokens.length !== tokens.length
    || analyzedPos.length !== tokens.length
    || analyzedTokens.some((token, index) => token !== tokens[index])
  ) {
    return tokens.map(() => "X");
  }

  return analyzedPos;
}

function buildTextView(value: string, analyzePartOfSpeech = false): NormalizedTextView {
  const normalized = normalizeText(value);
  const tokens = normalized.length > 0 ? normalized.split(" ") : [];
  return {
    tokens,
    posTags: analyzePartOfSpeech ? analyzeTextPartOfSpeech(normalized, tokens) : [],
  };
}

function normalizePatternLiteral(value: string): string[] {
  const normalized = normalizeText(value);
  return normalized.length > 0 ? normalized.split(" ") : [];
}

function createPatternSyntaxError(value: string, reason: string): Error {
  return new Error(`Invalid pattern anchor "${value}": ${reason}`);
}

function parsePatternCall(expression: string, fullValue: string): { operator: string; body: string } {
  const match = expression.match(/^([a-z]+)\((.*)\)$/);
  if (!match) {
    throw createPatternSyntaxError(fullValue, `unknown expression "${expression}"`);
  }

  const [, operator = "", body = ""] = match;
  return { operator, body };
}

function parsePatternAlternatives(body: string, fullValue: string): string[][] {
  if (body.includes("(") || body.includes(")")) {
    throw createPatternSyntaxError(fullValue, "nested expressions are not supported");
  }

  const rawBranches = body.split(",");
  if (rawBranches.some((branch) => normalizePatternLiteral(branch).length === 0)) {
    throw createPatternSyntaxError(fullValue, "alt(...) cannot contain empty branches");
  }

  const branches = rawBranches.map((branch) => normalizePatternLiteral(branch));

  if (branches.length < 2) {
    throw createPatternSyntaxError(fullValue, "alt(...) requires at least two non-empty branches");
  }

  return branches;
}

function parseOptionalPattern(body: string, fullValue: string): string[] {
  if (body.includes("(") || body.includes(")")) {
    throw createPatternSyntaxError(fullValue, "nested expressions are not supported");
  }

  const tokens = normalizePatternLiteral(body);
  if (tokens.length === 0) {
    throw createPatternSyntaxError(fullValue, "opt(...) requires non-empty literal text");
  }

  return tokens;
}

function parseGapPattern(body: string, fullValue: string): { min: number; max: number } {
  if (body.includes("(") || body.includes(")")) {
    throw createPatternSyntaxError(fullValue, "nested expressions are not supported");
  }

  const parts = body.split(",").map((part) => part.trim());
  if (parts.length < 1 || parts.length > 2 || parts.some((part) => part.length === 0)) {
    throw createPatternSyntaxError(fullValue, "gap(...) must have one or two non-empty numeric arguments");
  }

  const values = parts.map((part) => Number(part));
  if (values.some((value) => !Number.isInteger(value) || value < 0)) {
    throw createPatternSyntaxError(fullValue, "gap(...) arguments must be non-negative integers");
  }

  const [singleOrMin, maybeMax] = values;
  const min = parts.length === 1 ? 0 : (singleOrMin ?? 0);
  const max = parts.length === 1 ? (singleOrMin ?? 0) : (maybeMax ?? 0);

  if (min > max) {
    throw createPatternSyntaxError(fullValue, "gap(...) minimum cannot exceed maximum");
  }

  return { min, max };
}

function parsePatternExpression(expression: string, fullValue: string): PatternTokenPart {
  if (expression === "number" || expression === "dice" || expression === "range") {
    return {
      type: "placeholder",
      value: expression,
    };
  }

  const { operator, body } = parsePatternCall(expression, fullValue);
  if (operator === "alt") {
    return {
      type: "alternative",
      tokenOptions: parsePatternAlternatives(body, fullValue),
    };
  }
  if (operator === "opt") {
    return {
      type: "optional",
      tokens: parseOptionalPattern(body, fullValue),
    };
  }
  if (operator === "gap") {
    const { min, max } = parseGapPattern(body, fullValue);
    return {
      type: "gap",
      min,
      max,
    };
  }

  throw createPatternSyntaxError(fullValue, `unknown operator "${operator}"`);
}

function normalizePatternAnchorValue(value: string): PatternTokenPart[] {
  const raw = value.toLowerCase().replace(/&nbsp;/g, " ");
  const parts: PatternTokenPart[] = [];
  let offset = 0;

  while (offset < raw.length) {
    const expressionStart = raw.indexOf("{{", offset);
    if (expressionStart === -1) {
      if (raw.slice(offset).includes("}}")) {
        throw createPatternSyntaxError(value, "unexpected closing expression block");
      }
      const literalTokens = normalizePatternLiteral(raw.slice(offset));
      if (literalTokens.length > 0) {
        parts.push({
          type: "literal",
          tokens: literalTokens,
        });
      }
      break;
    }

    if (raw.slice(offset, expressionStart).includes("}}")) {
      throw createPatternSyntaxError(value, "unexpected closing expression block");
    }

    const literalTokens = normalizePatternLiteral(raw.slice(offset, expressionStart));
    if (literalTokens.length > 0) {
      parts.push({
        type: "literal",
        tokens: literalTokens,
      });
    }

    const expressionEnd = raw.indexOf("}}", expressionStart + 2);
    if (expressionEnd === -1) {
      throw createPatternSyntaxError(value, "unclosed expression block");
    }

    const expression = raw.slice(expressionStart + 2, expressionEnd).trim();
    if (expression.length === 0) {
      throw createPatternSyntaxError(value, "empty expression block");
    }
    if (expression.includes("{{") || expression.includes("}}")) {
      throw createPatternSyntaxError(value, "nested expressions are not supported");
    }

    parts.push(parsePatternExpression(expression, value));
    offset = expressionEnd + 2;
  }

  if (parts.length === 0) {
    return [];
  }

  if (
    parts[0]?.type === "optional"
    || parts.at(-1)?.type === "optional"
    || parts[0]?.type === "gap"
    || parts.at(-1)?.type === "gap"
  ) {
    throw createPatternSyntaxError(value, "leading or trailing opt(...) or gap(...) is not supported");
  }

  return parts;
}

function normalizeAnchorPos(pos: TextAnchorPosConstraint[] | undefined): PartOfSpeech[][] | undefined {
  if (!pos || pos.length === 0) {
    return undefined;
  }

  const normalized = pos.map((constraint) => {
    const allowed = Array.isArray(constraint) ? constraint : [constraint];
    const deduped = [...new Set(allowed)];
    if (deduped.length === 0) {
      throw new Error("Text anchor POS constraints cannot contain empty token constraints.");
    }
    return deduped;
  });

  return normalized.length > 0 ? normalized : undefined;
}

function normalizeAnchor(anchor: TextAnchor): NormalizedTextAnchor | null {
  if (typeof anchor === "string") {
    const cacheKey = `either\0${anchor}`;
    const cached = normalizedStringAnchorCache.get(cacheKey);
    if (cached !== undefined) {
      return cached;
    }

    const compiled = normalizeAnchor({ value: anchor });
    normalizedStringAnchorCache.set(cacheKey, compiled);
    return compiled;
  }

  const cached = normalizedObjectAnchorCache.get(anchor);
  if (cached !== undefined) {
    return cached;
  }

  const raw = anchor;
  const patternParts = normalizePatternAnchorValue(raw.value);
  const compiled = patternParts.length === 0 ? null : {
    scope: raw.scope ?? "either",
    patternParts,
    posByToken: normalizeAnchorPos(raw.pos),
  };
  normalizedObjectAnchorCache.set(anchor, compiled);
  return compiled;
}

function normalizeStringList(values: string[] | undefined): string[] | undefined {
  if (!values || values.length === 0) {
    return undefined;
  }

  const normalized = values.map((value) => normalizeText(value)).filter(Boolean);
  return normalized.length > 0 ? normalized : undefined;
}

function compileAnchorList(anchors: TextAnchor[] | undefined): NormalizedTextAnchor[] | undefined {
  if (!anchors || anchors.length === 0) {
    return undefined;
  }

  const compiled = anchors
    .map((anchor) => normalizeAnchor(anchor))
    .filter((anchor): anchor is NormalizedTextAnchor => anchor !== null);

  return compiled.length > 0 ? compiled : undefined;
}

function compileTextNearConstraints(
  constraints: TextNearConstraint[] | undefined,
): CompiledTextNearConstraint[] | undefined {
  if (!constraints || constraints.length === 0) {
    return undefined;
  }

  const compiled = constraints.map((constraint) => {
      const all = compileAnchorList(constraint.all);
      if (!all || all.length === 0) {
        return null;
      }

      const compiledConstraint: CompiledTextNearConstraint = {
        all,
        window: Math.max(0, constraint.window),
        ordered: constraint.ordered ?? false,
        scope: constraint.scope ?? "either",
      };
      return compiledConstraint;
    });

  const filtered = compiled.filter((constraint): constraint is CompiledTextNearConstraint => constraint !== null);

  return filtered.length > 0 ? filtered : undefined;
}

function compileReferencePredicates(predicates: ReferencePredicate[] | undefined): CompiledReferencePredicate[] | undefined {
  if (!predicates || predicates.length === 0) {
    return undefined;
  }

  const compiled = predicates.map((predicate) => ({
    category: predicate.category,
    subcategory: predicate.subcategory,
    packName: predicate.packName ? normalizeText(predicate.packName) : undefined,
    nameAny: normalizeStringList(predicate.nameAny),
    traitsAny: normalizeStringList(predicate.traitsAny),
    traitsAll: normalizeStringList(predicate.traitsAll),
  }));

  return compiled.length > 0 ? compiled : undefined;
}

function compileClause(clause: DerivedTagMatchClause): CompiledDerivedTagMatchClause {
  return {
    score: clause.score,
    traitsAny: normalizeStringList(clause.traitsAny),
    traitsAll: normalizeStringList(clause.traitsAll),
    traitsNone: normalizeStringList(clause.traitsNone),
    familiesAny: normalizeStringList(clause.familiesAny),
    familiesAll: normalizeStringList(clause.familiesAll),
    familiesNone: normalizeStringList(clause.familiesNone),
    textAny: compileAnchorList(clause.textAny),
    minTextAnyMatches: clause.minTextAnyMatches,
    textAll: compileAnchorList(clause.textAll),
    textNear: compileTextNearConstraints(clause.textNear),
    referencesAny: clause.referencesAny?.map((reference) => normalizeDerivedTagReference(reference)),
    referencesAll: clause.referencesAll?.map((reference) => normalizeDerivedTagReference(reference)),
    referencesWhere: compileReferencePredicates(clause.referencesWhere),
    minReferenceMatches: clause.minReferenceMatches,
  };
}

function collectAnchorPosScopes(anchors: NormalizedTextAnchor[] | undefined, scopes: Set<TextMatchScope>): void {
  if (!anchors) {
    return;
  }

  for (const anchor of anchors) {
    if (anchor.posByToken) {
      scopes.add(anchor.scope);
    }
  }
}

function collectClausePosScopes(clause: CompiledDerivedTagMatchClause, scopes: Set<TextMatchScope>): void {
  collectAnchorPosScopes(clause.textAny, scopes);
  collectAnchorPosScopes(clause.textAll, scopes);

  for (const constraint of clause.textNear ?? []) {
    collectAnchorPosScopes(constraint.all, scopes);
  }
}

function getCompiledRules(rules: DerivedTagRule[]): {
  rules: CompiledDerivedTagRule[];
  posScopes: Set<TextMatchScope>;
} {
  const cached = compiledRulesCache.get(rules);
  if (cached) {
    return cached;
  }

  const compiledRules = rules.map((rule) => ({
    tag: rule.tag,
    category: rule.category,
    subcategories: rule.subcategories,
    threshold: rule.threshold,
    requiresTags: rule.requiresTags,
    anyOf: rule.anyOf?.map((clause) => compileClause(clause)),
    allOf: rule.allOf?.map((clause) => compileClause(clause)),
    noneOf: rule.noneOf?.map((clause) => compileClause(clause)),
  }));
  const posScopes = new Set<TextMatchScope>();
  for (const rule of compiledRules) {
    for (const clause of rule.anyOf ?? []) {
      collectClausePosScopes(clause, posScopes);
    }
    for (const clause of rule.allOf ?? []) {
      collectClausePosScopes(clause, posScopes);
    }
    for (const clause of rule.noneOf ?? []) {
      collectClausePosScopes(clause, posScopes);
    }
  }

  const compiled = {
    rules: compiledRules,
    posScopes,
  };
  compiledRulesCache.set(rules, compiled);
  return compiled;
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
  if (scope === "blurb") {
    return [{ scope: "blurb", view: context.blurb }];
  }

  return [
    { scope: "name", view: context.name },
    { scope: "description", view: context.description },
  ];
}

function matchPatternPlaceholder(tokens: string[], index: number, placeholder: PatternPlaceholder): number {
  const token = tokens[index];
  if (!token) {
    return 0;
  }

  if (placeholder === "number") {
    return /^\d+$/.test(token) ? 1 : 0;
  }

  if (placeholder === "dice") {
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

function findPatternLiteralEnd(tokens: string[], index: number, literalTokens: string[]): number | null {
  if (literalTokens.length === 0) {
    return index;
  }

  for (let offset = 0; offset < literalTokens.length; offset += 1) {
    if (tokens[index + offset] !== literalTokens[offset]) {
      return null;
    }
  }

  return index + literalTokens.length;
}

function collectPatternMatchEnds(
  tokens: string[],
  index: number,
  patternParts: PatternTokenPart[],
  partIndex: number,
): number[] {
  if (partIndex >= patternParts.length) {
    return [index];
  }

  const part = patternParts[partIndex];
  if (!part) {
    return [index];
  }

  if (part.type === "literal") {
    const end = findPatternLiteralEnd(tokens, index, part.tokens);
    return end === null ? [] : collectPatternMatchEnds(tokens, end, patternParts, partIndex + 1);
  }

  if (part.type === "alternative") {
    const ends = new Set<number>();
    for (const option of part.tokenOptions) {
      const end = findPatternLiteralEnd(tokens, index, option);
      if (end !== null) {
        for (const matchEnd of collectPatternMatchEnds(tokens, end, patternParts, partIndex + 1)) {
          ends.add(matchEnd);
        }
      }
    }

    return [...ends];
  }

  if (part.type === "optional") {
    const ends = new Set<number>(collectPatternMatchEnds(tokens, index, patternParts, partIndex + 1));
    const end = findPatternLiteralEnd(tokens, index, part.tokens);
    if (end !== null) {
      for (const matchEnd of collectPatternMatchEnds(tokens, end, patternParts, partIndex + 1)) {
        ends.add(matchEnd);
      }
    }

    return [...ends];
  }

  if (part.type === "gap") {
    const ends = new Set<number>();
    const maxLength = Math.min(part.max, Math.max(0, tokens.length - index));
    for (let length = part.min; length <= maxLength; length += 1) {
      for (const matchEnd of collectPatternMatchEnds(tokens, index + length, patternParts, partIndex + 1)) {
        ends.add(matchEnd);
      }
    }

    return [...ends];
  }

  const length = matchPatternPlaceholder(tokens, index, part.value);
  if (length === 0) {
    return [];
  }

  return collectPatternMatchEnds(tokens, index + length, patternParts, partIndex + 1);
}

function findPatternOccurrences(view: NormalizedTextView, patternParts: PatternTokenPart[]): TextOccurrence[] {
  if (patternParts.length === 0) {
    return [];
  }

  const occurrences: TextOccurrence[] = [];
  for (let start = 0; start < view.tokens.length; start += 1) {
    const matchEnds = collectPatternMatchEnds(view.tokens, start, patternParts, 0);
    for (const end of matchEnds) {
      if (end > start) {
        occurrences.push({
          start,
          end: end - 1,
        });
      }
    }
  }

  return occurrences;
}

function matchesOccurrencePos(
  view: NormalizedTextView,
  occurrence: TextOccurrence,
  posByToken: PartOfSpeech[][] | undefined,
): boolean {
  if (!posByToken) {
    return true;
  }

  const occurrenceLength = occurrence.end - occurrence.start + 1;
  if (occurrenceLength !== posByToken.length) {
    return false;
  }

  for (let index = 0; index < posByToken.length; index += 1) {
    const pos = view.posTags[occurrence.start + index];
    const allowed = posByToken[index];
    if (!pos || !allowed?.includes(pos)) {
      return false;
    }
  }

  return true;
}

function matchesTextAnchor(context: NormalizedDerivedTagContext, anchor: NormalizedTextAnchor): boolean {
  return getTextViews(context, anchor.scope).some(({ scope, view }) => findNormalizedTextOccurrences(view, scope, anchor).length > 0);
}

function countMatchingTextAnchors(context: NormalizedDerivedTagContext, anchors: NormalizedTextAnchor[]): number {
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

  return findPatternOccurrences(view, normalized.patternParts)
    .filter((occurrence) => matchesOccurrencePos(view, occurrence, normalized.posByToken));
}

function findTextOccurrences(
  view: NormalizedTextView,
  viewScope: Exclude<TextMatchScope, "either">,
  anchor: NormalizedTextAnchor,
): TextOccurrence[] {
  return findNormalizedTextOccurrences(view, viewScope, anchor);
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

function matchesTextNear(context: NormalizedDerivedTagContext, constraint: CompiledTextNearConstraint): boolean {
  if (constraint.all.length === 0) {
    return false;
  }

  for (const { scope, view } of getTextViews(context, constraint.scope)) {
    const occurrenceLists = constraint.all.map((anchor) => findTextOccurrences(view, scope, anchor));
    if (occurrenceLists.some((occurrences) => occurrences.length === 0)) {
      continue;
    }

    if (hasTextProximityInView(
      occurrenceLists,
      constraint.all.length,
      constraint.window,
      constraint.ordered,
    )) {
      return true;
    }
  }

  return false;
}

function matchesReferencePredicate(reference: NormalizedDerivedTagReference, predicate: CompiledReferencePredicate): boolean {
  if (predicate.category && reference.category !== predicate.category) {
    return false;
  }
  if (predicate.subcategory && reference.subcategory !== predicate.subcategory) {
    return false;
  }
  if (predicate.packName && reference.packName !== predicate.packName) {
    return false;
  }
  if (predicate.nameAny && !predicate.nameAny.some((name) => reference.name === name)) {
    return false;
  }
  if (predicate.traitsAny && !predicate.traitsAny.some((trait) => reference.traits.has(trait))) {
    return false;
  }
  if (predicate.traitsAll && !predicate.traitsAll.every((trait) => reference.traits.has(trait))) {
    return false;
  }

  return true;
}

function countMatchingReferences(context: NormalizedDerivedTagContext, predicates: CompiledReferencePredicate[]): number {
  if (predicates.length === 0) {
    return 0;
  }

  return context.references.filter((reference) => predicates.some((predicate) => matchesReferencePredicate(reference, predicate))).length;
}

function matchesClause(context: NormalizedDerivedTagContext, clause: CompiledDerivedTagMatchClause): boolean {
  if (clause.traitsAny && !clause.traitsAny.some((trait) => context.traits.has(trait))) {
    return false;
  }
  if (clause.traitsAll && !clause.traitsAll.every((trait) => context.traits.has(trait))) {
    return false;
  }
  if (clause.traitsNone && clause.traitsNone.some((trait) => context.traits.has(trait))) {
    return false;
  }
  if (clause.familiesAny && !clause.familiesAny.some((family) => context.families.has(family))) {
    return false;
  }
  if (clause.familiesAll && !clause.familiesAll.every((family) => context.families.has(family))) {
    return false;
  }
  if (clause.familiesNone && clause.familiesNone.some((family) => context.families.has(family))) {
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
  if (clause.textNear && !clause.textNear.some((constraint) => matchesTextNear(context, constraint))) {
    return false;
  }
  if (clause.referencesAny && !clause.referencesAny.some((reference) => context.referenceKeys.has(reference))) {
    return false;
  }
  if (clause.referencesAll && !clause.referencesAll.every((reference) => context.referenceKeys.has(reference))) {
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

function scoreClause(context: NormalizedDerivedTagContext, clause: CompiledDerivedTagMatchClause): number {
  if (!matchesClause(context, clause)) {
    return 0;
  }

  return clause.score ?? 1;
}

function matchesRule(
  context: NormalizedDerivedTagContext,
  tags: Set<string>,
  rule: CompiledDerivedTagRule,
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
  const compiledRules = getCompiledRules(rules);
  const analyzeNamePos = compiledRules.posScopes.has("name") || compiledRules.posScopes.has("either");
  const analyzeDescriptionPos = compiledRules.posScopes.has("description") || compiledRules.posScopes.has("either");
  const analyzeBlurbPos = compiledRules.posScopes.has("blurb");
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
    name: buildTextView(input.name, analyzeNamePos),
    description: buildTextView(input.descriptionText ?? "", analyzeDescriptionPos),
    blurb: buildTextView(input.blurbText ?? "", analyzeBlurbPos),
    referenceKeys: new Set(references.map((reference) => reference.key)),
    references,
  };
  const tags = new Set<string>();

  for (const rule of compiledRules.rules) {
    if (matchesRule(context, tags, rule)) {
      tags.add(rule.tag);
    }
  }

  return uniqueSorted([...tags]);
}
