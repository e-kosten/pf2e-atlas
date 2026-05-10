import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const DescriptiveProgressionProjectionDeclarations = [
  defineConceptProjections("cumulative_transformation", {
    affliction: {
      axis: "disease_model",
      family: "progression_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Affliction whose stages progressively rewrite the victim into a visibly altered or corrupted form.",
      appliesWhen: [
        "The stage-by-stage march toward a changed body is a major reason to retrieve the affliction.",
        "Transformation is progressive and cumulative rather than an instantaneous one-step effect.",
      ],
      doesNotApplyWhen: [
        "The affliction only imposes one stable transformed state without escalation.",
        "The stronger fit is delayed_onset, terminal_collapse, or a more specific pathogenesis tag.",
      ],
      adjacentTags: ["transformative_corruption", "petrifying_corruption"],
    },
  }),
  defineConceptProjections("delayed_onset", {
    affliction: {
      axis: "disease_model",
      family: "progression_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Affliction whose symptoms, danger, or full transformation emerge after a notable delay rather than immediately.",
      appliesWhen: [
        "The affliction is naturally retrieved because the real danger appears after an incubation period, hidden delay, or deceptively quiet initial stage.",
        "The timing gap between exposure and serious consequence matters to prep, diagnosis, or quarantine decisions.",
      ],
      doesNotApplyWhen: [
        "The affliction starts harming victims right away even if it worsens later.",
        "The stronger fit is recurrent_flare or cumulative_transformation because the main hook is cycling or progressive change rather than delayed emergence.",
      ],
      adjacentTags: ["recurrent_flare", "cumulative_transformation"],
    },
  }),
  defineConceptProjections("recurrent_flare", {
    affliction: {
      axis: "disease_model",
      family: "progression_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Affliction that subsides and returns in episodes, repeating attacks, or cyclical symptom spikes.",
      appliesWhen: [
        "The affliction is naturally retrieved because symptoms repeatedly subside and then surge back in cycles, attacks, or flare-ups.",
        "Its pacing matters as a recurring problem rather than a single steady decline.",
      ],
      doesNotApplyWhen: [
        "The affliction mainly incubates once and then worsens in a straight line.",
        "The stronger fit is delayed_onset or terminal_collapse because recurrence is not the main progression hook.",
      ],
      adjacentTags: ["delayed_onset", "terminal_collapse"],
    },
  }),
  defineConceptProjections("terminal_collapse", {
    affliction: {
      axis: "disease_model",
      family: "progression_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Affliction explicitly framed around catastrophic bodily failure, death, or a final ruinous end state if unchecked.",
      appliesWhen: [
        "The affliction is naturally retrieved because the later stages end in death, total ruin, or another catastrophic final break point.",
        "The inevitability of a terminal end state matters more than day-to-day impairment or transformation imagery.",
      ],
      doesNotApplyWhen: [
        "The affliction is dangerous but does not build toward a distinct catastrophic endpoint.",
        "The stronger fit is cumulative_transformation or physical_debilitation because ongoing deterioration matters more than terminal collapse.",
      ],
      adjacentTags: ["cumulative_transformation", "physical_debilitation"],
    },
  }),
] satisfies ConceptProjectionDeclaration[];
