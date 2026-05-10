import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalBreachProjectionDeclarations = [
  defineConceptProjections("door_breaching", {
    equipment: {
      axis: "utility",
      family: "breaching",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Helps force doors, shutters, gates, or similar entry points open by strength, impact, or destructive entry.",
      subcategories: ["gear", "kit", "consumable", "weapon"],
      appliesWhen: [
        "The item's retrieval value is getting through doors, shutters, gates, or secured entry points.",
        "It solves access by force rather than by keys, stealth, or lock tools.",
      ],
      doesNotApplyWhen: [
        "The item is for larger demolition or siegework rather than point-of-entry breach.",
        "The item bypasses access quietly through locks or trickery instead of force.",
      ],
      adjacentTags: ["lock_bypass", "barrier_breaking"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
