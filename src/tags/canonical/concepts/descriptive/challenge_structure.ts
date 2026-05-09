import { defineFacetConcepts, mergeCanonicalConceptSeeds, type FacetlessConceptSeed } from "../../builders.js";
import { CANONICAL_FACETS } from "../../facets.js";

const challengeStructureSeeds: Record<string, FacetlessConceptSeed> = {
  endurance_pressure: {},
  multi_stage_resolution: {},
  observation_driven: {},
  timing_window: {},
};

export const challengeStructureSeedsByKind = mergeCanonicalConceptSeeds([
  defineFacetConcepts(
    CANONICAL_FACETS.CHALLENGE_STRUCTURE.CHALLENGE_STRUCTURE,
    challengeStructureSeeds,
  ),
]);
