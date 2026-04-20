import type { DerivedTagAuthoredCategoryOntology } from "../../types.js";
import type { DerivedTagManagedCategory } from "../manifest.js";
import { AFFLICTION_DERIVED_TAG_ONTOLOGY } from "./affliction.js";
import { CREATURE_DERIVED_TAG_ONTOLOGY } from "./creature.js";
import { EQUIPMENT_DERIVED_TAG_ONTOLOGY } from "./equipment.js";
import { HAZARD_DERIVED_TAG_ONTOLOGY } from "./hazard.js";
import { SPELL_DERIVED_TAG_ONTOLOGY } from "./spell.js";

export {
  AFFLICTION_DERIVED_TAG_ONTOLOGY,
  CREATURE_DERIVED_TAG_ONTOLOGY,
  EQUIPMENT_DERIVED_TAG_ONTOLOGY,
  HAZARD_DERIVED_TAG_ONTOLOGY,
  SPELL_DERIVED_TAG_ONTOLOGY,
};

export const DERIVED_TAG_ONTOLOGY_BY_CATEGORY: Record<
  DerivedTagManagedCategory,
  DerivedTagAuthoredCategoryOntology
> = {
  affliction: AFFLICTION_DERIVED_TAG_ONTOLOGY,
  creature: CREATURE_DERIVED_TAG_ONTOLOGY,
  equipment: EQUIPMENT_DERIVED_TAG_ONTOLOGY,
  hazard: HAZARD_DERIVED_TAG_ONTOLOGY,
  spell: SPELL_DERIVED_TAG_ONTOLOGY,
};

export { flattenDerivedTagAuthoredCategoryOntology, fromFamily, fromTag } from "./utils.js";
