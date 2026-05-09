import { defineFacetConcepts, mergeCanonicalConceptSeeds, type FacetlessConceptSeed } from "../../builders.js";
import { CANONICAL_FACETS } from "../../facets.js";

const progressionProfileSeeds: Record<string, FacetlessConceptSeed> = {
  cumulative_transformation: {},
  delayed_onset: {},
  recurrent_flare: {},
  terminal_collapse: {},
};

export const progressionSeedsByKind = mergeCanonicalConceptSeeds([
  defineFacetConcepts(
    CANONICAL_FACETS.PROGRESSION.PROGRESSION_PROFILE,
    progressionProfileSeeds,
  ),
]);
