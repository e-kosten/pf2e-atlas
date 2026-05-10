import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalEmitProjectionDeclarations = [
  defineConceptProjections("projectile_emitter", {
    hazard: {
      axis: "effect",
      family: "attack_vector",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hazard that fires bolts, beams, jets, sprays, or similar directed emissions from a fixed emitter.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
