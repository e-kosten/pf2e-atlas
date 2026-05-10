import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalBurstProjectionDeclarations = [
  defineConceptProjections("proximity_burst", {
    hazard: {
      axis: "effect",
      family: "attack_vector",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard that erupts in an immediate burst, blast, or detonation when a victim comes near or crosses a point.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
