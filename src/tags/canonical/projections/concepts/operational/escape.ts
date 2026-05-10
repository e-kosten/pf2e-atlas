import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalEscapeProjectionDeclarations = [
  defineConceptProjections("restraint_escape", {
    equipment: {
      axis: "utility",
      family: "restraint",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps break free from grabs, restraints, or similar immobilizing holds.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
