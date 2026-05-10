import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const DescriptiveBehaviorOverrideProjectionDeclarations = [
  defineConceptProjections("compulsion", {
    affliction: {
      axis: "behavior",
      family: "behavioral_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Overrides agency through commanded behavior, forced truth-telling, or similarly scripted actions.",
    },
  }),
  defineConceptProjections("self_destructive_impulse", {
    affliction: {
      axis: "behavior",
      family: "behavioral_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Drives reckless self-harm, suicidal behavior, or dangerous compulsions against the victim's own interests.",
    },
  }),
  defineConceptProjections("truth_compulsion", {
    affliction: {
      axis: "behavior",
      family: "behavioral_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Forces confession, honesty, or involuntary revelation against the victim's will.",
      appliesWhen: [
        "The affliction is naturally retrieved because victims are forced to confess, answer honestly, or reveal hidden information.",
        "Involuntary disclosure matters more than broad obedience, mood change, or self-harm.",
      ],
      doesNotApplyWhen: [
        "The affliction only compels action or speech generally without a truth-telling or confession-facing hook.",
        "The stronger fit is compulsion or cognitive_impairment rather than forced honesty.",
      ],
      adjacentTags: ["compulsion", "self_destructive_impulse"],
    },
  }),
  defineConceptProjections("violence_compulsion", {
    affliction: {
      axis: "behavior",
      family: "behavioral_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Forces hostile aggression, murderous rage, or other attack-driven loss of self-control.",
      appliesWhen: [
        "The affliction is naturally retrieved because it drives victims to attack, maul, murder, or lash out at others.",
        "Violent outward aggression matters more than truthful speech, self-harm, or generic loss of agency.",
      ],
      doesNotApplyWhen: [
        "The affliction only makes the victim reckless, confused, or generally compelled without a real violence-forward pattern.",
        "The stronger fit is self_destructive_impulse or compulsion because aggression toward others is not central.",
      ],
      adjacentTags: ["compulsion", "self_destructive_impulse"],
    },
  }),
] satisfies ConceptProjectionDeclaration[];
