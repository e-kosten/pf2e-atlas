import { defineFacetConcepts, mergeCanonicalConceptSeeds, type FacetlessConceptSeed } from "../../builders.js";
import { CANONICAL_FACETS } from "../../facets.js";

const responseDemandSeeds: Record<string, FacetlessConceptSeed> = {
  community_outbreak: {
    text: {
      definition: "Heterogeneous browse family; rows split across response_demand, delivery, and theme.",
    },
  },
  time_critical_resolution: {
    text: {
      definition: "Response-side pressure family with approved operational exceptions.",
    },
  },
};

export const responseDemandSeedsByKind = mergeCanonicalConceptSeeds([
  defineFacetConcepts(
    CANONICAL_FACETS.RESPONSE_DEMAND.RESPONSE_DEMAND,
    responseDemandSeeds,
  ),
]);
