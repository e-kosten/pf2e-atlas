import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalInfiltrateProjectionDeclarations = [
  defineConceptProjections("infiltration", {
    creature: {
      tag: "infiltration_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Threat defined by disguise, replacement, infiltration, or remaining embedded among victims before the danger fully reveals itself.",
      adjacentTags: ["disguised_pretender", "possession_threat"],
    },
    equipment: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad infiltration umbrella for quiet-entry, discreet-carry, disguise, and covert-passing equipment.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      adjacentTags: ["stealth_support", "disguise", "social_infiltration"],
      compositeOfAnyTags: ["stealth_support", "concealable", "disguise", "social_infiltration", "concealment"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description: "Broad infiltration umbrella for quiet-entry, disguise, and covert social-passing spells.",
      adjacentTags: ["stealth_support", "disguise", "social_infiltration"],
      compositeOfAnyTags: ["stealth_support", "disguise", "social_infiltration"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
