import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalDisruptProjectionDeclarations = [
  defineConceptProjections("navigation_disruption", {
    hazard: {
      axis: "effect",
      family: "perception_control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Hazard that confounds routes, loops intruders, or scrambles navigation through distorted perception.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
