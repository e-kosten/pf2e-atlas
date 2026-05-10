import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalRecoverProjectionDeclarations = [
  defineConceptProjections("mental_recovery", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps stabilize emotions or recover from mental conditions.",
      subcategories: ["consumable"],
    },
  }),
] satisfies ConceptProjectionDeclaration[];
