import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalAssaultProjectionDeclarations = [
  defineConceptProjections("illusion_assault", {
    hazard: {
      axis: "effect",
      family: "perception_control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Hazard that attacks through deceptive reflections, phantasms, or other hostile illusion-driven distortions.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
