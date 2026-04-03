import { deriveRecordTagsFromRules } from "./matcher.js";
import { normalizeDerivedTag, DerivedTagContext } from "./shared.js";
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

export const DERIVED_TAG_CATALOG = [
  ...EQUIPMENT_DERIVED_TAG_CATALOG,
  ...SPELL_DERIVED_TAG_CATALOG,
  ...HAZARD_DERIVED_TAG_CATALOG,
  ...AFFLICTION_DERIVED_TAG_CATALOG,
  ...CREATURE_DERIVED_TAG_CATALOG,
];

export function deriveRecordTags(input: DerivedTagContext): string[] {
  return deriveRecordTagsFromRules(DERIVED_TAG_RULES, input);
}
