import { defineFacetConcepts, mergeCanonicalConceptSeeds, type FacetlessConceptSeed } from "../../builders.js";
import { CANONICAL_FACETS } from "../../facets.js";

const mechanismSeeds: Record<string, FacetlessConceptSeed> =
  {
    control_interface: {},
    planar_breach: {},
    pressure_trigger: {},
    threshold_lockdown: {},
    tripwire_trigger: {},
    ward_trigger: {},
  };

export const mechanismSeedsByKind = mergeCanonicalConceptSeeds([
  defineFacetConcepts(
    CANONICAL_FACETS.MECHANISM.MECHANISM,
    mechanismSeeds,
  ),
]);
