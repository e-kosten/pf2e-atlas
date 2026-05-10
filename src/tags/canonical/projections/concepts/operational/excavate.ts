import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalExcavateProjectionDeclarations = [
  defineConceptProjections("excavation", {
    equipment: {
      axis: "utility",
      family: "breaching",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Helps dig, cut through earth or stone, or otherwise open a route by excavation or practical earth-moving work.",
      subcategories: ["gear", "kit", "consumable", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
