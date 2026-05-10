import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalDominateProjectionDeclarations = [
  defineConceptProjections("domination", {
    spell: {
      axis: "influence",
      family: "influence",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Seizes sustained control over a target's actions, body, or tactical decision-making.",
      appliesWhen: [
        "The spell grants ongoing, high-authority control over what the target does rather than just one compelled action.",
        "A user would retrieve it as a takeover spell, not merely a charm or suggestion spell.",
      ],
      doesNotApplyWhen: [
        "The spell only improves attitude, stirs emotion, or issues narrower one-off compulsions.",
        "The spell mainly suppresses actions without redirecting them under the caster's control.",
      ],
      adjacentTags: ["compulsion_control", "action_denial"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
