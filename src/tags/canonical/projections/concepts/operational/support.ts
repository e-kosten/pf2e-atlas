import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalSupportProjectionDeclarations = [
  defineConceptProjections("buff_support", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Provides a general beneficial enhancement or bonus.",
      subcategories: ["consumable"],
    },
  }),
  defineConceptProjections("condition_support", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps clear or mitigate harmful conditions.",
      subcategories: ["consumable"],
    },
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Delays, suppresses, or removes afflictions and conditions.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("eidolon_support", {
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Directly benefits an eidolon or the summoner-eidolon bond.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("escape_support", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps flee, slip away, or break free.",
      subcategories: ["consumable"],
    },
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Helps a creature slip away, break free, flee, or evade pursuit.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("fortune_support", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Improves a creature's odds with rerolls, better-result effects, or failure rescue.",
      subcategories: ["consumable"],
    },
  }),
  defineConceptProjections("healing_support", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Restores hit points or provides direct healing.",
      subcategories: ["consumable"],
    },
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Directly restores hit points or accelerates recovery.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("initiative_support", {
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Improves initiative, pre-combat readiness, or the party's opening tempo before the first turn.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("quickened_support", {
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Grants extra actions, quickened condition benefits, or similar action-economy acceleration.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("resistance_support", {
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Grants resistance or immunity against energy, damage, or hazards.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("siege_support", {
    equipment: {
      axis: "utility",
      family: "breaching",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports attacking gates, fortifications, vehicles, or other larger hardened targets.",
      subcategories: ["gear", "kit", "consumable", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("temporary_hp_support", {
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Grants temporary Hit Points or similar buffer protection instead of restoring lost Hit Points.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
