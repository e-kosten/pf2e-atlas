import { defineFacetConcepts, mergeCanonicalConceptSeeds, type FacetlessConceptSeed } from "../../builders.js";
import { CANONICAL_FACETS } from "../../facets.js";

const deliveryProfileSeeds: Record<string, FacetlessConceptSeed> = {
  contact_exposure: {},
  contact_offense: {},
  dreamborne_exposure: {},
  ingested_exposure: {},
  ingested_offense: {},
  inhaled_exposure: {},
  injury_exposure: {},
  thrown_offense: {},
  waterborne_exposure: {},
  weapon_applied: {},
};

const payloadSeeds: Record<string, FacetlessConceptSeed> = {
  elemental_payload: {
    text: {
      definition: "Payload rows reuse delivery semantics, with targeting specialization overrides.",
    },
  },
  explosive_payload: {
    text: {
      definition: "Payload rows reuse delivery semantics, with targeting specialization overrides.",
    },
  },
  spell_payload: {
    text: {
      definition: "Payload rows reuse delivery semantics, with targeting specialization overrides.",
    },
  },
};

const vectorSeeds: Record<string, FacetlessConceptSeed> = {
  carrier_vector: {
    text: {
      definition: "Heterogeneous browse family; rows split across response_demand, delivery, and theme.",
    },
  },
};

export const deliverySeedsByKind = mergeCanonicalConceptSeeds([
  defineFacetConcepts(
    CANONICAL_FACETS.DELIVERY.DELIVERY_PROFILE,
    deliveryProfileSeeds,
  ),
  defineFacetConcepts(
    CANONICAL_FACETS.DELIVERY.PAYLOAD,
    payloadSeeds,
  ),
  defineFacetConcepts(
    CANONICAL_FACETS.DELIVERY.VECTOR,
    vectorSeeds,
  ),
]);
