import { deriveRecordTagsFromRules } from "./matcher.js";
import { normalizeDerivedTag, DerivedTagContext } from "./shared.js";
import { uniqueSorted } from "../utils.js";
import {
  buildDerivedTagSeedIndex,
  deriveCatalogTagDerivation,
  publishDerivedTagCatalog,
  resolveCatalogSeedRecordKeys,
  type DerivedTagDerivation,
} from "./catalog-utils.js";
import { DERIVED_TAG_SEED_LOOKUP } from "./catalog-seed-records.js";
import { AFFLICTION_DERIVED_TAG_CATALOG } from "./catalog/affliction.js";
import { CREATURE_DERIVED_TAG_CATALOG } from "./catalog/creature.js";
import { EQUIPMENT_DERIVED_TAG_CATALOG } from "./catalog/equipment.js";
import { HAZARD_DERIVED_TAG_CATALOG } from "./catalog/hazard.js";
import { SPELL_DERIVED_TAG_CATALOG } from "./catalog/spell.js";
import { AFFLICTION_DERIVED_TAG_RULES } from "./rules/affliction.js";
import { CREATURE_DERIVED_TAG_RULES } from "./rules/creature.js";
import { EQUIPMENT_DERIVED_TAG_RULES } from "./rules/equipment.js";
import { HAZARD_DERIVED_TAG_RULES } from "./rules/hazard.js";
import { SPELL_DERIVED_TAG_RULES } from "./rules/spell.js";

export { normalizeDerivedTag } from "./shared.js";

const DERIVED_TAG_RULES = [
  ...EQUIPMENT_DERIVED_TAG_RULES,
  ...SPELL_DERIVED_TAG_RULES,
  ...HAZARD_DERIVED_TAG_RULES,
  ...AFFLICTION_DERIVED_TAG_RULES,
  ...CREATURE_DERIVED_TAG_RULES,
];

const RAW_DERIVED_TAG_CATALOG = [
  ...EQUIPMENT_DERIVED_TAG_CATALOG,
  ...SPELL_DERIVED_TAG_CATALOG,
  ...HAZARD_DERIVED_TAG_CATALOG,
  ...AFFLICTION_DERIVED_TAG_CATALOG,
  ...CREATURE_DERIVED_TAG_CATALOG,
];
const DERIVED_TAG_SEED_INDEX = buildDerivedTagSeedIndex(RAW_DERIVED_TAG_CATALOG, DERIVED_TAG_SEED_LOOKUP);

export const DERIVED_TAG_CATALOG = publishDerivedTagCatalog(RAW_DERIVED_TAG_CATALOG);

export function deriveRecordTags(input: DerivedTagContext): string[] {
  return deriveRecordTagDerivation(input).tags;
}

export function deriveRecordTagDerivation(
  input: DerivedTagContext,
): DerivedTagDerivation {
  const ruleTags = deriveRecordTagsFromRules(DERIVED_TAG_RULES, input);
  return deriveCatalogTagDerivation(RAW_DERIVED_TAG_CATALOG, DERIVED_TAG_SEED_INDEX, input, ruleTags);
}

export function getDerivedTagSeedRecordKeys(
  tag: string,
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): string[] {
  return resolveCatalogSeedRecordKeys(DERIVED_TAG_SEED_INDEX, normalizeDerivedTag(tag), scope);
}

export function getDerivedTagFamilyTags(
  family: string,
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): string[] {
  const normalizedFamily = normalizeDerivedTag(family);
  const tags = new Set<string>();

  for (const entry of RAW_DERIVED_TAG_CATALOG) {
    if (normalizeDerivedTag(entry.family) !== normalizedFamily) {
      continue;
    }
    if (scope.category && entry.category !== scope.category) {
      continue;
    }
    if (scope.subcategory !== undefined && entry.subcategories && (scope.subcategory === null || !entry.subcategories.includes(scope.subcategory))) {
      continue;
    }

    for (const tag of entry.tags) {
      tags.add(normalizeDerivedTag(tag.value));
    }
    if (entry.promoteFamilyToTag) {
      tags.add(normalizedFamily);
    }
  }

  const resolved = uniqueSorted([...tags]);
  if (resolved.length === 0) {
    const renderedScope = scope.category
      ? scope.subcategory
        ? `${scope.category}/${scope.subcategory}`
        : scope.category
      : "all categories";
    throw new Error(`No derived tag family "${normalizedFamily}" matched scope "${renderedScope}".`);
  }

  return resolved;
}
