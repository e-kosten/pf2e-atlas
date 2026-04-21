import type { AuthoredDerivedTagRule } from "../../domain/derived-tag-types.js";
import type { DerivedTagManagedCategory } from "../manifest.js";
import { AFFLICTION_AUTHORED_DERIVED_TAG_RULES } from "./affliction.js";
import { CREATURE_AUTHORED_DERIVED_TAG_RULES } from "./creature.js";
import { EQUIPMENT_AUTHORED_DERIVED_TAG_RULES } from "./equipment.js";
import { HAZARD_AUTHORED_DERIVED_TAG_RULES } from "./hazard.js";
import { SPELL_AUTHORED_DERIVED_TAG_RULES } from "./spell.js";

export {
  AFFLICTION_AUTHORED_DERIVED_TAG_RULES,
  CREATURE_AUTHORED_DERIVED_TAG_RULES,
  EQUIPMENT_AUTHORED_DERIVED_TAG_RULES,
  HAZARD_AUTHORED_DERIVED_TAG_RULES,
  SPELL_AUTHORED_DERIVED_TAG_RULES,
};

export const DERIVED_TAG_AUTHORED_RULES_BY_CATEGORY: Record<DerivedTagManagedCategory, AuthoredDerivedTagRule[]> = {
  affliction: AFFLICTION_AUTHORED_DERIVED_TAG_RULES,
  creature: CREATURE_AUTHORED_DERIVED_TAG_RULES,
  equipment: EQUIPMENT_AUTHORED_DERIVED_TAG_RULES,
  hazard: HAZARD_AUTHORED_DERIVED_TAG_RULES,
  spell: SPELL_AUTHORED_DERIVED_TAG_RULES,
};

export { compileAuthoredDerivedTagRules } from "./compiler.js";
