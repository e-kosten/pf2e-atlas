import type { DerivedTagExemplarCategory } from "../../types.js";
import type { DerivedTagManagedCategory } from "../manifest.js";
import { AFFLICTION_DERIVED_TAG_EXEMPLARS } from "./affliction.js";
import { CREATURE_DERIVED_TAG_EXEMPLARS } from "./creature.js";
import { EQUIPMENT_DERIVED_TAG_EXEMPLARS } from "./equipment.js";
import { HAZARD_DERIVED_TAG_EXEMPLARS } from "./hazard.js";
import { SPELL_DERIVED_TAG_EXEMPLARS } from "./spell.js";

export {
  AFFLICTION_DERIVED_TAG_EXEMPLARS,
  CREATURE_DERIVED_TAG_EXEMPLARS,
  EQUIPMENT_DERIVED_TAG_EXEMPLARS,
  HAZARD_DERIVED_TAG_EXEMPLARS,
  SPELL_DERIVED_TAG_EXEMPLARS,
};

export const DERIVED_TAG_EXEMPLARS_BY_CATEGORY: Record<DerivedTagManagedCategory, DerivedTagExemplarCategory> = {
  affliction: AFFLICTION_DERIVED_TAG_EXEMPLARS,
  creature: CREATURE_DERIVED_TAG_EXEMPLARS,
  equipment: EQUIPMENT_DERIVED_TAG_EXEMPLARS,
  hazard: HAZARD_DERIVED_TAG_EXEMPLARS,
  spell: SPELL_DERIVED_TAG_EXEMPLARS,
};
