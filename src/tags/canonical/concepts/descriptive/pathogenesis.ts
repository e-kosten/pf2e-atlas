import { defineFacetConcepts, mergeCanonicalConceptSeeds, type FacetlessConceptSeed } from "../../builders.js";
import { CANONICAL_FACETS } from "../../facets.js";

const pathogenesisSeeds: Record<string, FacetlessConceptSeed> = {
  bestial_transformation: {
    text: {
      definition: "Disease-pattern descriptors.",
    },
  },
  blood_rot: {
    text: {
      definition: "Disease-pattern descriptors.",
    },
  },
  fungal_growth: {
    text: {
      definition: "Disease-pattern descriptors.",
    },
  },
  infestation_implant: {
    text: {
      definition: "Disease-pattern descriptors.",
    },
  },
  petrifying_corruption: {
    text: {
      definition: "Disease-pattern descriptors.",
    },
  },
  rot_decay: {
    text: {
      definition: "Disease-pattern descriptors.",
    },
  },
};

export const pathogenesisSeedsByKind = mergeCanonicalConceptSeeds([
  defineFacetConcepts(
    CANONICAL_FACETS.PATHOGENESIS.PATHOGENESIS,
    pathogenesisSeeds,
  ),
]);
