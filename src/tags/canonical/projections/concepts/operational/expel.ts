import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalExpelProjectionDeclarations = [
  defineConceptProjections("hostile_presence_expulsion", {
    affliction: {
      tag: "exorcism_resolution",
      label: "exorcism",
      axis: "response",
      family: "resolution_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because banishing, cleansing, or spiritually expelling a hostile presence is central to solving the affliction.",
      adjacentTags: ["cursebreaking_resolution", "ritual_appeasement_resolution"],
    },
    hazard: {
      tag: "exorcism_countermeasure",
      label: "exorcism",
      axis: "resolution",
      family: "countermeasure_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard best resolved through banishment, exorcism, consecration, or another spirit-cleansing answer.",
      appliesWhen: [
        "The hazard is naturally retrieved because cleansing, banishing, or sanctifying the hostile presence is a core answer path.",
        "A spiritual purge matters more than appeasement, anti-magic suppression, or physical mechanism work.",
      ],
      doesNotApplyWhen: [
        "The hazard mainly wants offerings, restitution, or ritual respect rather than expulsion.",
        "The hazard is magical or mechanical but not really spirit-cleansed out of existence.",
      ],
      adjacentTags: ["appeasement_countermeasure", "dispel_countermeasure"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      tag: "exorcism",
      label: "exorcism",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Banishes, expels, or spiritually drives out a hostile spirit, possession, haunt, or invading supernatural presence.",
      adjacentTags: ["curse_removal", "sanctification"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
