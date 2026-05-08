import { buildCanonicalConceptMap, type CanonicalConceptSeed } from "../builders.js";
import { CANONICAL_VOCABULARY } from "../vocabulary.js";

const DERIVED_TAG_AGGREGATE_CANONICAL_CONCEPTS_BY_ID_SEEDS: Record<string, CanonicalConceptSeed> = {
  breaching: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.BARRIER,
    operation: CANONICAL_VOCABULARY.OPERATION.BREACH,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  communication: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  consultation: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  cosmic_framework_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  displacement_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.DISPLACEMENT,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  elemental_plane_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  environmental_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.ENVIRONMENTAL,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  expedition: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  guarding_hazard: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.FUNCTION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAZARD_FUNCTION,
  },
  lower_plane_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  movement_traversal: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  perception_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.PERCEPTION,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  problem_discovery: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.PROBLEM,
    operation: CANONICAL_VOCABULARY.OPERATION.DISCOVER,
    text: {
      definition: "Discovery-side spell tags normalize as operational discover concepts.",
    },
  },
  problem_resolution: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.PROBLEM,
    operation: CANONICAL_VOCABULARY.OPERATION.RESOLVE,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  reconnaissance: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  security: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  transformation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.FORM,
    operation: CANONICAL_VOCABULARY.OPERATION.TRANSFORM,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  upper_plane_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  wayfinding: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
};

export const DERIVED_TAG_AGGREGATE_CANONICAL_CONCEPTS_BY_ID = buildCanonicalConceptMap(
  CANONICAL_VOCABULARY.SCHEMA.KIND.AGGREGATE,
  DERIVED_TAG_AGGREGATE_CANONICAL_CONCEPTS_BY_ID_SEEDS,
);
