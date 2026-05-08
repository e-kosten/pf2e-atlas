import type { DerivedTagManagedCategory } from "../../manifest.js";
import type { DerivedTagCategoryProjection } from "../../../domain/derived-tag-types.js";
import { AFFLICTION_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG } from "./affliction.js";
import { CREATURE_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG } from "./creature.js";
import { EQUIPMENT_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG } from "./equipment.js";
import { HAZARD_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG } from "./hazard.js";
import { SPELL_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG } from "./spell.js";

export const DERIVED_TAG_CANONICAL_PROJECTIONS_BY_CATEGORY: Record<DerivedTagManagedCategory, Record<string, DerivedTagCategoryProjection>> = {
  affliction: AFFLICTION_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG,
  creature: CREATURE_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG,
  equipment: EQUIPMENT_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG,
  hazard: HAZARD_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG,
  spell: SPELL_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG,
};
