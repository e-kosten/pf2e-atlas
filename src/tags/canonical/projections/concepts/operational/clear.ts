import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalClearProjectionDeclarations = [
  defineConceptProjections("crowd_clearing", {
    equipment: {
      axis: "effect",
      family: "offensive_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Excels at damaging clusters, swarms, or tightly packed weaker enemies rather than focusing on one target.",
      subcategories: ["ammo", "consumable"],
      adjacentTags: ["burst_damage", "line_of_sight_control"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Damages or wipes clusters of weaker foes and is naturally retrieved as an anti-group answer.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
