import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalDemolishProjectionDeclarations = [
  defineConceptProjections("demolition", {
    equipment: {
      axis: "utility",
      family: "breaching",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Designed for blasting, collapsing, or otherwise violently dismantling structures and obstacles.",
      subcategories: ["gear", "kit", "consumable", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
