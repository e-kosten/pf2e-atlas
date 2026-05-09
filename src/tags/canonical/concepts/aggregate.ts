import { buildCanonicalConceptMap, defineFacetConcepts, mergeCanonicalConceptSeeds, type CanonicalConceptSeed, type FacetlessConceptSeed } from "../builders.js";
import { CANONICAL_VOCABULARY } from "../vocabulary.js";
import { CANONICAL_FACETS } from "../facets.js";

const capabilityAggregateSeeds: Record<string, FacetlessConceptSeed> = {
  communication: {},
  consultation: {},
  expedition: {},
  movement_traversal: {},
  reconnaissance: {},
  security: {},
  wayfinding: {},
};

const settingAggregateSeeds: Record<string, FacetlessConceptSeed> = {
  cosmic_framework_setting: {},
  elemental_plane_setting: {},
  lower_plane_setting: {},
  upper_plane_setting: {},
};

const functionAggregateSeeds: Record<string, FacetlessConceptSeed> = {
  guarding_hazard: {},
};

const operationalizedAggregateSeeds: Record<string, CanonicalConceptSeed> = {
  breaching: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.BARRIER,
    operation: CANONICAL_VOCABULARY.OPERATION.BREACH,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  displacement_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.DISPLACEMENT,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  environmental_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.ENVIRONMENTAL,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
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
  transformation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.FORM,
    operation: CANONICAL_VOCABULARY.OPERATION.TRANSFORM,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
};

const aggregateFacetSeeds: Record<string, CanonicalConceptSeed> = mergeCanonicalConceptSeeds([
  defineFacetConcepts(CANONICAL_FACETS.CAPABILITY.CAPABILITY, capabilityAggregateSeeds),
  defineFacetConcepts(CANONICAL_FACETS.SETTING.PLANAR, settingAggregateSeeds),
  defineFacetConcepts(CANONICAL_FACETS.FUNCTION.HAZARD_FUNCTION, functionAggregateSeeds),
]);

const aggregateConceptSeedsById = mergeCanonicalConceptSeeds([
  operationalizedAggregateSeeds,
  aggregateFacetSeeds,
]);

export const DERIVED_TAG_AGGREGATE_CANONICAL_CONCEPTS_BY_ID = buildCanonicalConceptMap(
  CANONICAL_VOCABULARY.SCHEMA.KIND.AGGREGATE,
  aggregateConceptSeedsById,
);
