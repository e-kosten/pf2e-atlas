import type {
  AuthoredDerivedTagRule,
  DerivedTagAuthoredRuleBlockers,
  DerivedTagAuthoredTextNearConstraint,
  DerivedTagAuthoredTextValue,
} from "../../types.js";
import type { PublishedDerivedTagOntology } from "../runtime/catalog-utils.js";
import type { DerivedTagRule, ReferencePredicate, TextAnchor, TextNearConstraint } from "../runtime/matcher.js";
import { normalizeDerivedTag } from "../runtime/shared.js";

function normalizeStringList(values: string[] | undefined, context: string): string[] | undefined {
  if (!values) {
    return undefined;
  }

  const normalized = [...new Set(values.map((value) => value.trim()).filter(Boolean))];
  if (normalized.length === 0) {
    throw new Error(`${context} must include at least one non-empty value.`);
  }

  return normalized;
}

function normalizeTextValue(value: DerivedTagAuthoredTextValue, context: string): TextAnchor {
  const normalized =
    typeof value === "string"
      ? { value, scope: "either" as const }
      : { value: value.value, scope: value.scope ?? ("either" as const) };

  if (normalized.value.includes("{{") || normalized.value.includes("}}")) {
    throw new Error(`${context} cannot use matcher pattern syntax.`);
  }
  if (normalized.value.trim().length === 0) {
    throw new Error(`${context} must include non-empty text.`);
  }

  return typeof value === "string" ? normalized.value : normalized;
}

function normalizeTextValues(
  values: DerivedTagAuthoredTextValue[] | undefined,
  context: string,
): TextAnchor[] | undefined {
  if (!values) {
    return undefined;
  }

  if (values.length === 0) {
    throw new Error(`${context} must include at least one text value.`);
  }

  return values.map((value, index) => normalizeTextValue(value, `${context}[${index}]`));
}

function normalizeReferencePredicate(predicate: ReferencePredicate, context: string): ReferencePredicate {
  const nameAny = normalizeStringList(predicate.nameAny, `${context}.nameAny`);
  const traitsAny = normalizeStringList(predicate.traitsAny, `${context}.traitsAny`);
  const traitsAll = normalizeStringList(predicate.traitsAll, `${context}.traitsAll`);
  const normalized = {
    ...(predicate.category ? { category: predicate.category } : {}),
    ...(predicate.subcategory ? { subcategory: predicate.subcategory } : {}),
    ...(predicate.packName ? { packName: predicate.packName.trim() } : {}),
    ...(nameAny ? { nameAny } : {}),
    ...(traitsAny ? { traitsAny } : {}),
    ...(traitsAll ? { traitsAll } : {}),
  };

  if (Object.keys(normalized).length === 0) {
    throw new Error(`${context} must declare at least one predicate field.`);
  }

  return normalized;
}

function normalizeReferencePredicates(
  predicates: ReferencePredicate[] | undefined,
  context: string,
): ReferencePredicate[] | undefined {
  if (!predicates) {
    return undefined;
  }

  if (predicates.length === 0) {
    throw new Error(`${context} must include at least one predicate.`);
  }

  return predicates.map((predicate, index) => normalizeReferencePredicate(predicate, `${context}[${index}]`));
}

function normalizeTextNearConstraints(
  constraints: DerivedTagAuthoredTextNearConstraint[] | undefined,
  context: string,
): TextNearConstraint[] | undefined {
  if (!constraints) {
    return undefined;
  }

  if (constraints.length === 0) {
    throw new Error(`${context} must include at least one proximity constraint.`);
  }

  return constraints.map((constraint, index) => {
    if (!Number.isInteger(constraint.window) || constraint.window < 0) {
      throw new Error(`${context}[${index}].window must be a non-negative integer.`);
    }
    if (constraint.all.length < 2) {
      throw new Error(`${context}[${index}].all must include at least two text anchors.`);
    }

    return {
      all: constraint.all.map((value, anchorIndex) =>
        normalizeTextValue(value, `${context}[${index}].all[${anchorIndex}]`),
      ),
      window: constraint.window,
      ...(constraint.ordered ? { ordered: true } : {}),
      ...(constraint.scope ? { scope: constraint.scope } : {}),
    };
  });
}

function compileBlockers(
  blockers: DerivedTagAuthoredRuleBlockers | undefined,
  context: string,
): NonNullable<DerivedTagRule["noneOf"]> | undefined {
  if (!blockers) {
    return undefined;
  }

  const clauses: NonNullable<DerivedTagRule["noneOf"]> = [];

  const traitsAny = normalizeStringList(blockers.traitsAny, `${context}.traitsAny`);
  if (traitsAny) {
    clauses.push({ traitsAny });
  }

  const traitsAll = normalizeStringList(blockers.traitsAll, `${context}.traitsAll`);
  if (traitsAll) {
    clauses.push({ traitsAll });
  }

  const familiesAny = normalizeStringList(blockers.familiesAny, `${context}.familiesAny`);
  if (familiesAny) {
    clauses.push({ familiesAny });
  }

  const familiesAll = normalizeStringList(blockers.familiesAll, `${context}.familiesAll`);
  if (familiesAll) {
    clauses.push({ familiesAll });
  }

  const textAny = normalizeTextValues(blockers.textAny, `${context}.textAny`);
  if (textAny) {
    clauses.push({ textAny });
  }

  const textAll = normalizeTextValues(blockers.textAll, `${context}.textAll`);
  if (textAll) {
    clauses.push({ textAll });
  }

  const textNear = normalizeTextNearConstraints(blockers.textNear, `${context}.textNear`);
  if (textNear) {
    clauses.push({ textNear });
  }

  const referencesAny = normalizeStringList(blockers.referencesAny, `${context}.referencesAny`);
  if (referencesAny) {
    clauses.push({ referencesAny });
  }

  const referencesAll = normalizeStringList(blockers.referencesAll, `${context}.referencesAll`);
  if (referencesAll) {
    clauses.push({ referencesAll });
  }

  const referencesWhere = normalizeReferencePredicates(blockers.referencesWhere, `${context}.referencesWhere`);
  if (referencesWhere) {
    const minReferenceMatches = blockers.minReferenceMatches;
    if (minReferenceMatches !== undefined && (!Number.isInteger(minReferenceMatches) || minReferenceMatches <= 0)) {
      throw new Error(`${context}.minReferenceMatches must be a positive integer.`);
    }
    clauses.push({
      referencesWhere,
      ...(minReferenceMatches !== undefined ? { minReferenceMatches } : {}),
    });
  } else if (blockers.minReferenceMatches !== undefined) {
    throw new Error(`${context}.minReferenceMatches requires referencesWhere.`);
  }

  if (clauses.length === 0) {
    throw new Error(`${context} must include at least one blocker condition.`);
  }

  return clauses;
}

function validateRuleTag(ontology: PublishedDerivedTagOntology, rule: AuthoredDerivedTagRule): void {
  const normalizedTag = normalizeDerivedTag(rule.tag);
  const ontologyTag = ontology.tagByKey.get(`${rule.category}:${normalizedTag}`);
  if (!ontologyTag) {
    throw new Error(
      `Authored derived tag rule "${normalizedTag}" in category "${rule.category}" does not exist in the published ontology.`,
    );
  }

  if (rule.subcategories && rule.subcategories.length > 0) {
    const family = ontology.familyByKey.get(`${rule.category}:${normalizeDerivedTag(ontologyTag.family)}`);
    if (family?.subcategories && family.subcategories.length > 0) {
      const invalid = rule.subcategories.find((subcategory) => !family.subcategories?.includes(subcategory));
      if (invalid) {
        throw new Error(
          `Authored derived tag rule "${normalizedTag}" in category "${rule.category}" cannot use subcategory "${invalid}" outside family "${ontologyTag.family}".`,
        );
      }
    }
  }
}

function validateCompositeRule(
  ontology: PublishedDerivedTagOntology,
  rule: Extract<AuthoredDerivedTagRule, { kind: "composite_tag" }>,
): string[] {
  const normalizedAnyTags = normalizeStringList(
    rule.when.anyTags,
    `Authored derived tag rule "${rule.tag}".when.anyTags`,
  );
  if (!normalizedAnyTags) {
    throw new Error(`Authored derived tag rule "${rule.tag}" must include at least one composite child tag.`);
  }

  const ontologyTag = ontology.tagByKey.get(`${rule.category}:${normalizeDerivedTag(rule.tag)}`);
  if (!ontologyTag) {
    throw new Error(
      `Authored derived tag composite rule "${rule.tag}" in category "${rule.category}" does not exist in the published ontology.`,
    );
  }

  if (ontologyTag.compositeOfAnyTags && ontologyTag.compositeOfAnyTags.length > 0) {
    throw new Error(
      `Authored derived tag composite rule "${rule.tag}" in category "${rule.category}" duplicates ontology composite behavior.`,
    );
  }

  for (const childTag of normalizedAnyTags) {
    if (!ontology.tagByKey.has(`${rule.category}:${normalizeDerivedTag(childTag)}`)) {
      throw new Error(
        `Authored derived tag composite rule "${rule.tag}" in category "${rule.category}" references unknown child tag "${childTag}".`,
      );
    }
  }

  return normalizedAnyTags;
}

function compileAuthoredRule(ontology: PublishedDerivedTagOntology, rule: AuthoredDerivedTagRule): DerivedTagRule[] {
  validateRuleTag(ontology, rule);
  const base = {
    tag: normalizeDerivedTag(rule.tag),
    category: rule.category,
    ...(rule.subcategories && rule.subcategories.length > 0 ? { subcategories: [...rule.subcategories] } : {}),
  };

  if (rule.intent !== "deterministic") {
    throw new Error(`Authored derived tag rule "${rule.tag}" must declare intent "deterministic".`);
  }

  switch (rule.kind) {
    case "trait_match": {
      const traitsAny = normalizeStringList(
        rule.when.traitsAny,
        `Authored derived tag rule "${rule.tag}".when.traitsAny`,
      );
      const traitsAll = normalizeStringList(
        rule.when.traitsAll,
        `Authored derived tag rule "${rule.tag}".when.traitsAll`,
      );
      const noneOf = compileBlockers(rule.blockers, `Authored derived tag rule "${rule.tag}".blockers`);
      if (!traitsAny && !traitsAll) {
        throw new Error(`Authored derived tag trait rule "${rule.tag}" must declare traitsAny or traitsAll.`);
      }
      return [
        {
          ...base,
          anyOf: [{ ...(traitsAny ? { traitsAny } : {}), ...(traitsAll ? { traitsAll } : {}) }],
          ...(noneOf ? { noneOf } : {}),
        },
      ];
    }
    case "family_match": {
      const familiesAny = normalizeStringList(
        rule.when.familiesAny,
        `Authored derived tag rule "${rule.tag}".when.familiesAny`,
      );
      const familiesAll = normalizeStringList(
        rule.when.familiesAll,
        `Authored derived tag rule "${rule.tag}".when.familiesAll`,
      );
      const noneOf = compileBlockers(rule.blockers, `Authored derived tag rule "${rule.tag}".blockers`);
      if (!familiesAny && !familiesAll) {
        throw new Error(`Authored derived tag family rule "${rule.tag}" must declare familiesAny or familiesAll.`);
      }
      return [
        {
          ...base,
          anyOf: [{ ...(familiesAny ? { familiesAny } : {}), ...(familiesAll ? { familiesAll } : {}) }],
          ...(noneOf ? { noneOf } : {}),
        },
      ];
    }
    case "reference_match": {
      const referencesAny = normalizeStringList(
        rule.when.referencesAny,
        `Authored derived tag rule "${rule.tag}".when.referencesAny`,
      );
      const referencesAll = normalizeStringList(
        rule.when.referencesAll,
        `Authored derived tag rule "${rule.tag}".when.referencesAll`,
      );
      const referencesWhere = normalizeReferencePredicates(
        rule.when.referencesWhere as ReferencePredicate[] | undefined,
        `Authored derived tag rule "${rule.tag}".when.referencesWhere`,
      );
      const noneOf = compileBlockers(rule.blockers, `Authored derived tag rule "${rule.tag}".blockers`);
      if (!referencesAny && !referencesAll && !referencesWhere) {
        throw new Error(
          `Authored derived tag reference rule "${rule.tag}" must declare referencesAny, referencesAll, or referencesWhere.`,
        );
      }
      if (
        rule.when.minReferenceMatches !== undefined &&
        (!Number.isInteger(rule.when.minReferenceMatches) || rule.when.minReferenceMatches <= 0)
      ) {
        throw new Error(`Authored derived tag rule "${rule.tag}".when.minReferenceMatches must be a positive integer.`);
      }
      if (rule.when.minReferenceMatches !== undefined && !referencesWhere) {
        throw new Error(`Authored derived tag rule "${rule.tag}".when.minReferenceMatches requires referencesWhere.`);
      }
      return [
        {
          ...base,
          anyOf: [
            {
              ...(referencesAny ? { referencesAny } : {}),
              ...(referencesAll ? { referencesAll } : {}),
              ...(referencesWhere ? { referencesWhere } : {}),
              ...(rule.when.minReferenceMatches !== undefined
                ? { minReferenceMatches: rule.when.minReferenceMatches }
                : {}),
            },
          ],
          ...(noneOf ? { noneOf } : {}),
        },
      ];
    }
    case "composite_tag": {
      const anyTags = validateCompositeRule(ontology, rule);
      return anyTags.map((childTag) => ({
        ...base,
        requiresTags: [normalizeDerivedTag(childTag)],
      }));
    }
    case "exact_text_match": {
      const textAny = normalizeTextValues(rule.when.textAny, `Authored derived tag rule "${rule.tag}".when.textAny`);
      const textAll = normalizeTextValues(rule.when.textAll, `Authored derived tag rule "${rule.tag}".when.textAll`);
      const noneOf = compileBlockers(rule.blockers, `Authored derived tag rule "${rule.tag}".blockers`);
      if (!textAny && !textAll) {
        throw new Error(`Authored derived tag exact text rule "${rule.tag}" must declare textAny or textAll.`);
      }
      return [
        {
          ...base,
          anyOf: [{ ...(textAny ? { textAny } : {}), ...(textAll ? { textAll } : {}) }],
          ...(noneOf ? { noneOf } : {}),
        },
      ];
    }
    case "text_context_match": {
      const textNear = normalizeTextNearConstraints(
        rule.when.textNear,
        `Authored derived tag rule "${rule.tag}".when.textNear`,
      );
      const noneOf = compileBlockers(rule.blockers, `Authored derived tag rule "${rule.tag}".blockers`);
      return [
        {
          ...base,
          anyOf: [{ textNear }],
          ...(noneOf ? { noneOf } : {}),
        },
      ];
    }
  }
}

export function compileAuthoredDerivedTagRules(
  ontology: PublishedDerivedTagOntology,
  rules: AuthoredDerivedTagRule[],
): DerivedTagRule[] {
  return rules.flatMap((rule) => compileAuthoredRule(ontology, rule));
}
