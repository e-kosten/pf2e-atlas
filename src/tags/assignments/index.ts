import type { AuthoredDerivedTagAssignment } from "../runtime/derivation/assignments.js";
import type { DerivedTagManagedCategory } from "../manifest.js";
import { AFFLICTION_DERIVED_TAG_ASSIGNMENTS } from "./affliction.js";
import { CREATURE_DERIVED_TAG_ASSIGNMENTS } from "./creature.js";
import { EQUIPMENT_DERIVED_TAG_ASSIGNMENTS } from "./equipment.js";
import { HAZARD_DERIVED_TAG_ASSIGNMENTS } from "./hazard.js";
import { SPELL_DERIVED_TAG_ASSIGNMENTS } from "./spell.js";

export {
  AFFLICTION_DERIVED_TAG_ASSIGNMENTS,
  CREATURE_DERIVED_TAG_ASSIGNMENTS,
  EQUIPMENT_DERIVED_TAG_ASSIGNMENTS,
  HAZARD_DERIVED_TAG_ASSIGNMENTS,
  SPELL_DERIVED_TAG_ASSIGNMENTS,
};

export const DERIVED_TAG_ASSIGNMENTS_BY_CATEGORY: Record<DerivedTagManagedCategory, AuthoredDerivedTagAssignment[]> = {
  affliction: AFFLICTION_DERIVED_TAG_ASSIGNMENTS,
  creature: CREATURE_DERIVED_TAG_ASSIGNMENTS,
  equipment: EQUIPMENT_DERIVED_TAG_ASSIGNMENTS,
  hazard: HAZARD_DERIVED_TAG_ASSIGNMENTS,
  spell: SPELL_DERIVED_TAG_ASSIGNMENTS,
};
