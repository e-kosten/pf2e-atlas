import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalInfluenceProjectionDeclarations = [
  defineConceptProjections("charm_influence", {
    spell: {
      axis: "influence",
      family: "influence",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Wins cooperation through friendliness, fascination, admiration, or magically altered social regard.",
      appliesWhen: [
        "The spell's main value is improving a target's attitude, trust, or willingness to cooperate.",
        "The spell changes social reception more than it scripts exact behavior.",
      ],
      doesNotApplyWhen: [
        "The spell compels exact actions, overrides agency, or takes total control.",
        "The spell only manipulates mood without establishing a social bond or regard shift.",
      ],
      adjacentTags: ["emotion_control", "compulsion_control"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
