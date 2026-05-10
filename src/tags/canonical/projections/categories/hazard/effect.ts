import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const hazardEffectProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.HAZARD.EFFECT_ATTACK_VECTOR, {
    floor_eruption: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that attacks upward from the ground, floor, or a concealed underfoot chamber.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    overhead_strike: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hazard that drops, crashes, or attacks from the ceiling or another overhead position.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    projectile_emitter: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hazard that fires bolts, beams, jets, sprays, or similar directed emissions from a fixed emitter.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    proximity_burst: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard that erupts in an immediate burst, blast, or detonation when a victim comes near or crosses a point.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.HAZARD.EFFECT_ENVIRONMENTAL_DANGER, {
    acid_hazard: {
      concept: "acid_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on acid, corrosive spray, caustic runoff, or similar corrosive exposure.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    blight_hazard: {
      concept: "blight_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard centered on ecological ruin, land-sickening corruption, withering growth, or terrain spoiled by supernatural blight.",
      adjacentTags: ["contamination_hazard", "overgrowth_hazard"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    cold_hazard: {
      concept: "cold_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on ice, frost, freezing, blizzards, or other cold exposure.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    contamination_hazard: {
      concept: "contamination_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard centered on tainted residue, corruptive seepage, drifting spores, cursed runoff, or other lingering contamination of a space.",
      adjacentTags: ["poison_hazard", "respiratory_hazard"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    cursefield_hazard: {
      concept: "cursefield_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard centered on cursed ground, spiritually poisoned space, or a zone whose danger comes from active supernatural contamination rather than one mechanism.",
      adjacentTags: ["contamination_hazard", "judgment_haunt"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    electric_hazard: {
      concept: "electric_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on lightning, shock, static discharge, or electrical exposure.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    environmental_hazard: {
      concept: "environmental_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad environmental umbrella for elemental, toxic, contaminating, and terrain-corrupting hazards that threaten a space through recurring exposure.",
      adjacentTags: ["fire_hazard", "poison_hazard", "contamination_hazard"],
      compositeOfAnyTags: [
        "acid_hazard",
        "cold_hazard",
        "fire_hazard",
        "electric_hazard",
        "poison_hazard",
        "respiratory_hazard",
        "sound_hazard",
        "water_hazard",
        "contamination_hazard",
        "blight_hazard",
        "overgrowth_hazard",
        "cursefield_hazard",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    fire_hazard: {
      concept: "fire_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on open fire, flames, burning spread, or explosive ignition.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    overgrowth_hazard: {
      concept: "overgrowth_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard centered on choking roots, hostile vines, grasping thorns, or other dangerous living overgrowth that turns terrain against intruders.",
      adjacentTags: ["blight_hazard", "forced_movement"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    poison_hazard: {
      concept: "poison_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on poison gas, toxic delivery, or other poisonous exposure.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    respiratory_hazard: {
      concept: "respiratory_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on smoke, choking vapor, breathlessness, or impaired breathing.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    sound_hazard: {
      concept: "sound_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on sonic force, deafening noise, vibration, or resonant disruption.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    water_hazard: {
      concept: "water_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on floods, geysers, waves, surges, or other dangerous water exposure.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.HAZARD.EFFECT_FORCED_POSITION, {
    collapse_hazard: {
      concept: "collapse_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hazard built around collapsing structures, cave-ins, rockfalls, or crumbling ground.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    displacement_hazard: {
      concept: "displacement_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad displacement umbrella for hazards that drop, collapse, shove, or split creatures apart through positional disruption.",
      adjacentTags: ["pitfall", "collapse_hazard", "forced_separation"],
      compositeOfAnyTags: ["pitfall", "collapse_hazard", "forced_movement", "forced_separation"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    forced_movement: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hazard that pushes, pulls, drags, submerges, or otherwise forcibly repositions creatures.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    pitfall: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hazard built around a concealed pit, drop, or similar vertical fall trap.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.HAZARD.EFFECT_IMPACT, {
    mental_impairment: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs judgment, emotions, or perception through fear, confusion, or similar effects.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    mobility_impairment: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Paralyzes, immobilizes, or otherwise heavily hampers movement.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    sensory_impairment: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Blinds, deafens, dazzles, or otherwise suppresses a victim's ability to perceive the environment clearly.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.HAZARD.EFFECT_PERCEPTION_CONTROL, {
    false_safe_route: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that tempts intruders toward a seemingly safer path or escape line that is itself the trap.",
      appliesWhen: [
        "The hazard actively misdirects intruders toward a route that looks protective, faster, or safer but is actually the danger.",
        "The retrieval hook is deceptive path choice rather than only illusion damage or generic navigation confusion.",
      ],
      doesNotApplyWhen: [
        "The hazard merely scrambles orientation without presenting a tempting fake safe path.",
        "The hazard is solved through procedure, but misleading route presentation is not central to how it works.",
      ],
      adjacentTags: ["navigation_disruption", "procedural_bypass"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    illusion_assault: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Hazard that attacks through deceptive reflections, phantasms, or other hostile illusion-driven distortions.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    navigation_disruption: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Hazard that confounds routes, loops intruders, or scrambles navigation through distorted perception.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    perception_hazard: {
      concept: "perception_application",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad perception umbrella for hazards that attack through distorted routes, hostile illusion, or deceptive pathing rather than direct force alone.",
      adjacentTags: ["navigation_disruption", "illusion_assault", "false_safe_route"],
      compositeOfAnyTags: ["navigation_disruption", "illusion_assault", "false_safe_route"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"hazard">[];
