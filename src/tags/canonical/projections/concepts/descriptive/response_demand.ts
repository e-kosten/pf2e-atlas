import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const DescriptiveResponseDemandProjectionDeclarations = [
  defineConceptProjections("community_outbreak", {
    affliction: {
      axis: "disease_model",
      family: "epidemiological_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Framed around camp-, village-, ship-, monastery-, or settlement-scale spread rather than one isolated victim.",
      appliesWhen: [
        "The affliction is naturally retrieved as a spreading local crisis affecting a community, camp, ship, or institution.",
        "Outbreak management, quarantine pressure, or multi-victim spread matters more than a single infected host.",
      ],
      doesNotApplyWhen: [
        "The affliction remains a one-target curse or isolated infection without wider spread framing.",
        "The stronger fit is epidemic_pestilence only because of plague flavor, but community-scale spread is not actually present.",
      ],
      adjacentTags: ["epidemic_pestilence", "carrier_vector"],
    },
  }),
  defineConceptProjections("time_critical_resolution", {
    affliction: {
      tag: "cure_clock_urgency",
      axis: "response",
      family: "response_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creates immediate pressure to diagnose and cure the affliction before a fast-moving catastrophic endpoint arrives.",
      adjacentTags: ["terminal_collapse", "delayed_onset"],
    },
  }),
] satisfies ConceptProjectionDeclaration[];
