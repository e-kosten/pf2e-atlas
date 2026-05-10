import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalProtectProjectionDeclarations = [
  defineConceptProjections("magic_protection", {
    equipment: {
      axis: "utility",
      family: "anti_magic",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Protects the user or target against hostile magical effects.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      appliesWhen: [
        "The item's value comes from warding the bearer against curses, spells, hostile magical conditions, or magical damage.",
        "Protection matters more than actually counteracting or suppressing the incoming magic.",
      ],
      doesNotApplyWhen: [
        "The item mainly shuts down active magic rather than defending a wearer or target.",
        "The stronger fit is scrying_protection because surveillance denial is the specific retrieval hook.",
      ],
      adjacentTags: ["countermagic", "hazard_shielding"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
