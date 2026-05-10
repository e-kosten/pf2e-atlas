import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const equipmentEffectProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.EFFECT_CONSUMABLE_ROLE, {
    ally_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Support consumable that can directly benefit another creature.",
      subcategories: ["consumable"],
    },
    offensive: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hostile consumable primarily meant to harm or debilitate a target.",
      subcategories: ["consumable"],
    },
    self_buff: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Support consumable primarily applied to the user.",
      subcategories: ["consumable"],
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.EFFECT_DELIVERY_PROFILE, {
    contact_offense: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Offensive consumable delivered through touch or skin contact.",
      subcategories: ["consumable"],
    },
    ingested_offense: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Offensive consumable delivered when swallowed or consumed.",
      subcategories: ["consumable"],
    },
    thrown_offense: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Offensive consumable delivered by throwing it.",
      subcategories: ["consumable"],
    },
    weapon_applied: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Offensive consumable applied to a weapon before use.",
      subcategories: ["consumable"],
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.EFFECT_FUNCTION, {
    anti_bleed: {
      concept: "bleed_remediation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps staunch bleeding, end persistent bleed damage, or close ongoing wounds.",
      subcategories: ["consumable"],
    },
    anti_confusion: {
      concept: "confusion_remediation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps clear confusion, restore mental steadiness, or recover from disordered thinking.",
      subcategories: ["consumable"],
    },
    anti_disease: {
      concept: "disease_remediation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps resist, prevent, or recover from disease.",
      subcategories: ["consumable"],
    },
    anti_fear: {
      concept: "fear_remediation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps resist fear, recover from frightened effects, or steady courage.",
      subcategories: ["consumable"],
    },
    anti_paralysis: {
      concept: "paralysis_remediation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps break paralysis, restore movement, or free a creature from immobilizing body shutdown.",
      subcategories: ["consumable"],
    },
    anti_petrification: {
      concept: "petrification_remediation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps prevent or reverse petrification and other stone-turning effects.",
      subcategories: ["consumable"],
    },
    anti_poison: {
      concept: "poison_remediation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps resist, prevent, or recover from poison.",
      subcategories: ["consumable"],
    },
    buff_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Provides a general beneficial enhancement or bonus.",
      subcategories: ["consumable"],
    },
    condition_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps clear or mitigate harmful conditions.",
      subcategories: ["consumable"],
    },
    energy_resistance: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Grants resistance against one or more energy types.",
      subcategories: ["consumable"],
    },
    escape_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps flee, slip away, or break free.",
      subcategories: ["consumable"],
    },
    fortune_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Improves a creature's odds with rerolls, better-result effects, or failure rescue.",
      subcategories: ["consumable"],
    },
    healing_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Restores hit points or provides direct healing.",
      subcategories: ["consumable"],
    },
    mental_recovery: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps stabilize emotions or recover from mental conditions.",
      subcategories: ["consumable"],
    },
    senses_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Improves vision or other senses.",
      subcategories: ["consumable"],
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.EFFECT_IMPACT, {
    mental_impairment: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs thought, judgment, composure, or behavior through fear, confusion, or similar effects.",
      subcategories: ["ammo", "consumable"],
    },
    mobility_impairment: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs movement through slowing, restraining, sticking, or immobilizing effects.",
      subcategories: ["ammo", "consumable"],
    },
    physical_debilitation: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Weakens the body through drained vitality, sickness, fatigue, clumsiness, or similar bodily degradation.",
      subcategories: ["ammo", "consumable"],
    },
    sedation: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Induces sleep, lethargy, unconsciousness, or similar incapacitating drowsiness.",
      subcategories: ["ammo", "consumable"],
    },
    sensory_impairment: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs sight, hearing, or other senses through blinding, dazzling, or deafening effects.",
      subcategories: ["ammo", "consumable"],
    },
    silencing: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Suppresses speech, voice, or other sound-dependent action through gagging, muting, or numbing effects.",
      subcategories: ["ammo", "consumable"],
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.EFFECT_OFFENSIVE_PROFILE, {
    anti_caster_disruption: {
      concept: "caster_disruption_remediation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Punishes casting, disrupts concentration, or is naturally retrieved to make enemy spell use unreliable.",
      subcategories: ["ammo", "consumable"],
      adjacentTags: ["silencing", "countermagic"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    burst_damage: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Delivers a front-loaded blast, detonation, or splash pattern that users naturally retrieve as immediate damage.",
      subcategories: ["ammo", "consumable"],
      adjacentTags: ["crowd_clearing", "persistent_damage"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    crowd_clearing: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Excels at damaging clusters, swarms, or tightly packed weaker enemies rather than focusing on one target.",
      subcategories: ["ammo", "consumable"],
      adjacentTags: ["burst_damage", "line_of_sight_control"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    line_of_sight_control: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Obscures vision, fills an area with smoke, or otherwise denies clear sight lines as the item's main offensive or tactical job.",
      subcategories: ["ammo", "consumable"],
      adjacentTags: ["sensory_impairment", "restraint_capture"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    persistent_damage: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Imposes ongoing damage through burning, acid, bleed, poison, or another lingering payload that keeps hurting after the initial hit.",
      subcategories: ["ammo", "consumable"],
      adjacentTags: ["burst_damage", "physical_debilitation"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"equipment">[];
