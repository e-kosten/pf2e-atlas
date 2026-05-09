import { defineFacetConcepts, mergeCanonicalConceptSeeds, type FacetlessConceptSeed } from "../../builders.js";
import { CANONICAL_FACETS } from "../../facets.js";

const behaviorOverrideSeeds: Record<string, FacetlessConceptSeed> = {
  compulsion: {},
  self_destructive_impulse: {},
  truth_compulsion: {},
  violence_compulsion: {},
};

export const behaviorOverrideSeedsByKind = mergeCanonicalConceptSeeds([
  defineFacetConcepts(
    CANONICAL_FACETS.BEHAVIOR_OVERRIDE.BEHAVIOR_OVERRIDE,
    behaviorOverrideSeeds,
  ),
]);
