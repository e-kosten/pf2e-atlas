import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const afflictionEffectProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.AFFLICTION.EFFECT_IMPACT, {
    action_denial: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Prevents normal action-taking through paralysis, stupefying shutdown, or similarly severe operational lockout.",
      appliesWhen: [
        "The affliction is naturally retrieved because victims become unable to act, respond, or complete normal turns through paralysis, shutdown, or severe stupor.",
        "Operational lockout matters more than ordinary weakness, pain, or mood distortion.",
      ],
      doesNotApplyWhen: [
        "The affliction only slows, weakens, or frightens the victim without truly preventing action-taking.",
        "The stronger fit is mobility_impairment, sedation, or mental_impairment rather than hard action denial.",
      ],
      adjacentTags: ["mobility_impairment", "sedation"],
    },
    cognitive_impairment: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Dulls thought, memory, decision-making, or mental clarity without being just fear or confusion.",
    },
    healing_suppression: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Prevents normal healing or sharply reduces healing received.",
    },
    mental_impairment: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs judgment, emotions, or perception through confusion, fear, or delirium.",
    },
    mobility_impairment: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Reduces speed, stiffens movement, or leaves the victim immobilized.",
    },
    physical_debilitation: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Weakens the body through exhaustion, sickness, drained vitality, blood loss, or similar bodily degradation.",
    },
    sedation: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Induces sleep, deep drowsiness, trance-like unconsciousness, or difficulty waking.",
    },
    sensory_impairment: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Blinds, deafens, or otherwise suppresses perception and the senses.",
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.AFFLICTION.EFFECT_PHYSIOLOGY_OVERRIDE, {
    hemorrhagic_failure: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Causes uncontrolled bleeding, blood loss, vessel rupture, or similar collapse of the body's circulatory integrity.",
      appliesWhen: [
        "The affliction is naturally retrieved because bleeding, blood loss, ruptured vessels, or circulatory collapse are major consequences rather than incidental symptoms.",
        "The bodily failure pattern matters more than the source theme of the disease.",
      ],
      doesNotApplyWhen: [
        "The affliction only references corrupted blood as flavor without major bleeding or circulatory breakdown consequences.",
        "The stronger fit is blood_rot or physical_debilitation because hemorrhage is not the core downstream effect.",
      ],
      adjacentTags: ["blood_rot", "physical_debilitation"],
    },
    petrification: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Turns flesh to stone or otherwise locks the body into a rigid mineralized state.",
      appliesWhen: [
        "The affliction is naturally retrieved because victims end up petrified, stone-locked, or mineralized.",
        "The end-state of stony immobilization matters more than the thematic cause.",
      ],
      doesNotApplyWhen: [
        "The affliction only trends toward calcification without actually functioning as a petrifying condition.",
        "The stronger fit is petrifying_corruption because the gradual corruption process is the real hook.",
      ],
      adjacentTags: ["petrifying_corruption", "mobility_impairment"],
    },
    respiratory_impairment: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Prevents normal breathing or fills the victim's lungs with water, fluid, or similar suffocating effects.",
    },
    transformative_corruption: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Progressively transforms the body into crystal, plant matter, fungus, or another corrupted form.",
    },
    wasting_hunger: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Imposes unnatural starvation, ravenous craving, or a consuming metabolic drive that degrades the body over time.",
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"affliction">[];
