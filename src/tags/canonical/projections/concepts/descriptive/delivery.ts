import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const DescriptiveDeliveryProjectionDeclarations = [
  defineConceptProjections("carrier_vector", {
    affliction: {
      axis: "disease_model",
      family: "epidemiological_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Spread through vermin, insects, bites, stings, or other living disease vectors.",
      appliesWhen: [
        "The affliction's spread is materially tied to rats, mosquitoes, parasites, infected animals, or other living carriers.",
        "Tracing or controlling the living vector is central to understanding the disease.",
      ],
      doesNotApplyWhen: [
        "The disease spreads through air, water, dreams, or contaminated objects without a real living carrier.",
        "Injury transmission happens, but the record does not frame a recurring vector population or host species.",
      ],
      adjacentTags: ["injury_exposure", "waterborne_exposure"],
    },
  }),
  defineConceptProjections("contact_exposure", {
    affliction: {
      axis: "disease_model",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Spread by touch, skin contact, slime, ooze, or another surface-level transfer mechanism.",
    },
  }),
  defineConceptProjections("contact_offense", {
    equipment: {
      axis: "effect",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Offensive consumable delivered through touch or skin contact.",
      subcategories: ["consumable"],
    },
  }),
  defineConceptProjections("dreamborne_exposure", {
    affliction: {
      axis: "disease_model",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Spread through sleep, nightmares, dreaming contact, or other oneiric pathways.",
      appliesWhen: [
        "The affliction is contracted or transmitted through dreams, sleep, shared nightmares, or oneiric contact.",
        "Dream-state exposure is central to how the condition reaches the victim.",
      ],
      doesNotApplyWhen: [
        "The affliction only causes nightmares after infection but is actually spread by wounds, air, or cursed objects.",
        "The stronger fit is nightmare_torment because the symptom profile matters more than the exposure path.",
      ],
      adjacentTags: ["nightmare_torment", "inhaled_exposure"],
    },
  }),
  defineConceptProjections("elemental_payload", {
    equipment: {
      axis: "item_mechanical",
      family: "ammunition_payload",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Ammunition that delivers an elemental or reagent-based payload on impact.",
      subcategories: ["ammo"],
    },
  }),
  defineConceptProjections("explosive_payload", {
    equipment: {
      axis: "item_mechanical",
      family: "ammunition_payload",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Ammunition that detonates or scatters area damage on impact.",
      subcategories: ["ammo"],
    },
  }),
  defineConceptProjections("ingested_exposure", {
    affliction: {
      axis: "disease_model",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Spread by swallowing contaminated food, drink, medicine, or another consumed substance.",
    },
  }),
  defineConceptProjections("ingested_offense", {
    equipment: {
      axis: "effect",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Offensive consumable delivered when swallowed or consumed.",
      subcategories: ["consumable"],
    },
  }),
  defineConceptProjections("inhaled_exposure", {
    affliction: {
      axis: "disease_model",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Spread through smoke, breath, spores, vapor, dust, or another inhaled medium.",
    },
  }),
  defineConceptProjections("injury_exposure", {
    affliction: {
      axis: "disease_model",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Spread through bites, stings, punctures, or other blood-entering injury vectors.",
    },
  }),
  defineConceptProjections("spell_payload", {
    equipment: {
      axis: "item_mechanical",
      family: "ammunition_payload",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Ammunition that delivers, casts, or imposes a spell effect on hit.",
      subcategories: ["ammo"],
    },
  }),
  defineConceptProjections("thrown_offense", {
    equipment: {
      axis: "effect",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Offensive consumable delivered by throwing it.",
      subcategories: ["consumable"],
    },
  }),
  defineConceptProjections("waterborne_exposure", {
    affliction: {
      axis: "disease_model",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Spread through tainted water, drowning contact, flood exposure, or cursed immersion.",
      appliesWhen: [
        "Contaminated water, immersion, flood contact, or cursed liquid exposure is central to how victims contract the affliction.",
        "A user would retrieve it for tainted wells, river contagion, drowning curses, or similar water-linked spread.",
      ],
      doesNotApplyWhen: [
        "The affliction merely affects aquatic creatures without actually spreading through water contact.",
        "The stronger fit is inhaled_exposure, ingested_exposure, or carrier_vector instead of water-linked transmission.",
      ],
      adjacentTags: ["ingested_exposure", "carrier_vector"],
    },
  }),
  defineConceptProjections("weapon_applied", {
    equipment: {
      axis: "effect",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Offensive consumable applied to a weapon before use.",
      subcategories: ["consumable"],
    },
  }),
] satisfies ConceptProjectionDeclaration[];
