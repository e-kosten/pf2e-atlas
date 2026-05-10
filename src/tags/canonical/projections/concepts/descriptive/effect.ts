import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const DescriptiveEffectProjectionDeclarations = [
  defineConceptProjections("action_denial", {
    affliction: {
      axis: "effect",
      family: "impact",
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
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Denies actions through paralysis, stupefying shutdown, slowed tempo, or similarly severe turn disruption.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("cognitive_impairment", {
    affliction: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Dulls thought, memory, decision-making, or mental clarity without being just fear or confusion.",
    },
  }),
  defineConceptProjections("curse_marking", {
    affliction: {
      axis: "metaphysical",
      family: "metaphysical_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Brands the victim with a curse mark, doom sign, inherited hex, or similarly explicit supernatural stigma.",
      appliesWhen: [
        "The affliction leaves an explicit mark, sign, sigil, stain, or named curse-brand that matters to its identity.",
        "A user would retrieve it for visible or narratively explicit cursed marking, not just soul damage.",
      ],
      doesNotApplyWhen: [
        "The affliction is metaphysical but has no distinct branded or marked stigma.",
        "The stronger fit is void_soul_corruption or soul_binding without a visible curse sign.",
      ],
      adjacentTags: ["void_soul_corruption", "soul_binding"],
    },
  }),
  defineConceptProjections("healing_suppression", {
    affliction: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Prevents normal healing or sharply reduces healing received.",
    },
  }),
  defineConceptProjections("hemorrhagic_failure", {
    affliction: {
      axis: "effect",
      family: "physiology_override",
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
  }),
  defineConceptProjections("mental_impairment", {
    affliction: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs judgment, emotions, or perception through confusion, fear, or delirium.",
    },
    equipment: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs thought, judgment, composure, or behavior through fear, confusion, or similar effects.",
      subcategories: ["ammo", "consumable"],
    },
    hazard: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs judgment, emotions, or perception through fear, confusion, or similar effects.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Impairs thought, composure, or agency through fear, confusion, or similarly hostile mental effects.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("mobility_impairment", {
    affliction: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Reduces speed, stiffens movement, or leaves the victim immobilized.",
    },
    equipment: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs movement through slowing, restraining, sticking, or immobilizing effects.",
      subcategories: ["ammo", "consumable"],
    },
    hazard: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Paralyzes, immobilizes, or otherwise heavily hampers movement.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("nightmare_torment", {
    affliction: {
      axis: "metaphysical",
      family: "metaphysical_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Centers on nightmares, dream-torment, or similarly sleep-haunting affliction framing.",
    },
  }),
  defineConceptProjections("petrification", {
    affliction: {
      axis: "effect",
      family: "physiology_override",
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
  }),
  defineConceptProjections("physical_debilitation", {
    affliction: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Weakens the body through exhaustion, sickness, drained vitality, blood loss, or similar bodily degradation.",
    },
    equipment: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Weakens the body through drained vitality, sickness, fatigue, clumsiness, or similar bodily degradation.",
      subcategories: ["ammo", "consumable"],
    },
  }),
  defineConceptProjections("possession_seed", {
    affliction: {
      axis: "metaphysical",
      family: "metaphysical_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Plants an invading spirit, hostile presence, or takeover-ready metaphysical foothold inside the victim.",
      appliesWhen: [
        "The affliction prepares a victim to be ridden, entered, replaced, or overtaken by another presence.",
        "A latent invading spirit or takeover-ready foothold is central to the condition's danger.",
      ],
      doesNotApplyWhen: [
        "The affliction only compels behavior without an actual possessing entity or metaphysical foothold.",
        "The stronger fit is soul_binding or compulsion because takeover is not really part of the disease model.",
      ],
      adjacentTags: ["compulsion", "soul_binding"],
    },
  }),
  defineConceptProjections("respiratory_impairment", {
    affliction: {
      axis: "effect",
      family: "physiology_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Prevents normal breathing or fills the victim's lungs with water, fluid, or similar suffocating effects.",
    },
  }),
  defineConceptProjections("sedation", {
    affliction: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Induces sleep, deep drowsiness, trance-like unconsciousness, or difficulty waking.",
    },
    equipment: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Induces sleep, lethargy, unconsciousness, or similar incapacitating drowsiness.",
      subcategories: ["ammo", "consumable"],
    },
  }),
  defineConceptProjections("sensory_impairment", {
    affliction: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Blinds, deafens, or otherwise suppresses perception and the senses.",
    },
    equipment: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs sight, hearing, or other senses through blinding, dazzling, or deafening effects.",
      subcategories: ["ammo", "consumable"],
    },
    hazard: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Blinds, deafens, dazzles, or otherwise suppresses a victim's ability to perceive the environment clearly.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Blinds, deafens, or otherwise directly suppresses a creature's senses.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("silencing", {
    equipment: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Suppresses speech, voice, or other sound-dependent action through gagging, muting, or numbing effects.",
      subcategories: ["ammo", "consumable"],
    },
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Suppresses speech, sound production, verbal casting, or other voice-dependent action.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("soul_binding", {
    affliction: {
      axis: "metaphysical",
      family: "metaphysical_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Pins, traps, anchors, or otherwise entangles the victim's soul with an object, oath, place, or hostile power.",
      appliesWhen: [
        "The affliction's metaphysical identity depends on the victim's soul being anchored, trapped, pledged, or externally entangled.",
        "A user would retrieve it for ghost anchors, cursed bindings, oath-linked doom, or similar soul-tether effects.",
      ],
      doesNotApplyWhen: [
        "The affliction only marks, weakens, or spiritually corrupts the victim without actually binding the soul to something.",
        "The stronger fit is curse_marking or possession_seed rather than tethering or anchoring.",
      ],
      adjacentTags: ["curse_marking", "possession_seed"],
    },
  }),
  defineConceptProjections("transformative_corruption", {
    affliction: {
      axis: "effect",
      family: "physiology_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Progressively transforms the body into crystal, plant matter, fungus, or another corrupted form.",
    },
  }),
  defineConceptProjections("void_soul_corruption", {
    affliction: {
      axis: "metaphysical",
      family: "metaphysical_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Attacks life force or the bond between body and soul through void or deathly corruption.",
    },
  }),
  defineConceptProjections("wasting_hunger", {
    affliction: {
      axis: "effect",
      family: "physiology_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Imposes unnatural starvation, ravenous craving, or a consuming metabolic drive that degrades the body over time.",
    },
  }),
] satisfies ConceptProjectionDeclaration[];
