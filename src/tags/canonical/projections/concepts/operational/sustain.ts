import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalSustainProjectionDeclarations = [
  defineConceptProjections("regeneration", {
    creature: {
      tag: "regeneration_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Regenerates or requires special suppression or finishing countermeasures.",
    },
  }),
] satisfies ConceptProjectionDeclaration[];
