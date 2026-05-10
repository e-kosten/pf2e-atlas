import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalCreateProjectionDeclarations = [
  defineConceptProjections("barrier_creation", {
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creates a wall, dome, cage, force barrier, or other discrete blocking structure that reshapes access lines.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("protective_ward", {
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Places a ward, sanctuary, shield, or protective boundary.",
      appliesWhen: [
        "The spell is naturally retrieved as a defensive ward, sanctuary, or protective boundary rather than only a resistance buff.",
        "Its value comes from shielding a creature, object, or space against incoming harm or intrusion.",
      ],
      doesNotApplyWhen: [
        "The spell only grants resistance, temporary Hit Points, or healing without a real warding or boundary element.",
        "The spell is mainly an alarm, anti-scrying, or mobility tool rather than direct protection.",
      ],
      adjacentTags: ["alarm", "resistance_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("spawn_creation", {
    creature: {
      tag: "spawn_creator",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creates additional threats through infestation, spawn-making, conversion, or implanted offspring.",
    },
  }),
] satisfies ConceptProjectionDeclaration[];
