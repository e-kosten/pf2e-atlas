import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalDenyProjectionDeclarations = [
  defineConceptProjections("mobility_denial", {
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Pins, slows, grounds, or otherwise prevents normal repositioning without necessarily functioning as a full restraint effect.",
      adjacentTags: ["forced_movement", "restraint_capture"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
