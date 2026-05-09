import { defineFacetConcepts, mergeCanonicalConceptSeeds, type FacetlessConceptSeed } from "../../builders.js";
import { CANONICAL_FACETS } from "../../facets.js";

const boundObjectFamilySeeds: Record<string, FacetlessConceptSeed> = {
  animated_object: {},
  animated_statue: {},
  possessed_object: {},
};

const creatureFamilySeeds: Record<string, FacetlessConceptSeed> = {
  sinspawn_family: {},
  undead_family: {},
};

export const creatureFamilySeedsByKind = mergeCanonicalConceptSeeds([
  defineFacetConcepts(
    CANONICAL_FACETS.CREATURE_FAMILY.BOUND_OBJECT_FAMILY,
    boundObjectFamilySeeds,
  ),
  defineFacetConcepts(
    CANONICAL_FACETS.CREATURE_FAMILY.CREATURE_FAMILY,
    creatureFamilySeeds,
  ),
]);
