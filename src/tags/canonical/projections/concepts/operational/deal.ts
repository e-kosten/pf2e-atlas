import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalDealProjectionDeclarations = [
  defineConceptProjections("burst_damage", {
    equipment: {
      axis: "effect",
      family: "offensive_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Delivers a front-loaded blast, detonation, or splash pattern that users naturally retrieve as immediate damage.",
      subcategories: ["ammo", "consumable"],
      adjacentTags: ["crowd_clearing", "persistent_damage"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Delivers concentrated damage in a spike or splash pattern that users naturally retrieve as a damage-first answer.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("persistent_damage", {
    equipment: {
      axis: "effect",
      family: "offensive_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Imposes ongoing damage through burning, acid, bleed, poison, or another lingering payload that keeps hurting after the initial hit.",
      subcategories: ["ammo", "consumable"],
      adjacentTags: ["burst_damage", "physical_debilitation"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Directly inflicts persistent damage or grants attacks that reliably impose persistent damage.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
