import { defineFacetConcepts, mergeCanonicalConceptSeeds, type FacetlessConceptSeed } from "../../builders.js";
import { CANONICAL_FACETS } from "../../facets.js";

const hazardFunctionSeeds: Record<string, FacetlessConceptSeed> = {
  ambush_burst: {},
  area_denial: {},
  attrition_pressure: {},
  barrier_lockdown: {},
  forced_separation: {},
  forced_separation_hazard: {},
  guard_post: {},
  pursuit_punisher: {},
  resource_drain: {},
  spawned_attackers: {},
  zone_denial: {},
};

export const functionSeedsByKind = mergeCanonicalConceptSeeds([
  defineFacetConcepts(
    CANONICAL_FACETS.FUNCTION.HAZARD_FUNCTION,
    hazardFunctionSeeds,
  ),
]);
