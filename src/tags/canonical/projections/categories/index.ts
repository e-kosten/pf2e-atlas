import { buildProjectionRecordsByCategory } from "../../builders.js";
import { DERIVED_TAG_CANONICAL_CONCEPTS_BY_ID } from "../../concepts/index.js";
import { afflictionProjectionDeclarations } from "./affliction/index.js";
import { creatureProjectionDeclarations } from "./creature/index.js";
import { equipmentProjectionDeclarations } from "./equipment/index.js";
import { hazardProjectionDeclarations } from "./hazard/index.js";
import { spellProjectionDeclarations } from "./spell/index.js";

export const DERIVED_TAG_CANONICAL_PROJECTIONS_BY_CATEGORY = buildProjectionRecordsByCategory(
  DERIVED_TAG_CANONICAL_CONCEPTS_BY_ID,
  [
    afflictionProjectionDeclarations,
    creatureProjectionDeclarations,
    equipmentProjectionDeclarations,
    hazardProjectionDeclarations,
    spellProjectionDeclarations,
  ],
);
