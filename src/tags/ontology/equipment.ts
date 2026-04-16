import type { DerivedTagAuthoredCategoryOntology } from "../../types.js";

export const EQUIPMENT_DERIVED_TAG_ONTOLOGY = {
  category: "equipment",
  families: {
    function: {
      subcategories: [
        "consumable"
      ],
      description: "Beneficial consumable outcome and recovery tags.",
      tags: [
        {
          tag: "beneficial",
          description: "Broad support-oriented consumable with non-hostile intent.",
          assignmentMode: "deterministic"
        },
        {
          tag: "healing_support",
          description: "Restores hit points or provides direct healing.",
          assignmentMode: "deterministic"
        },
        {
          tag: "anti_poison",
          description: "Helps resist, prevent, or recover from poison.",
          assignmentMode: "deterministic"
        },
        {
          tag: "anti_disease",
          description: "Helps resist, prevent, or recover from disease.",
          assignmentMode: "deterministic"
        },
        {
          tag: "condition_support",
          description: "Helps clear or mitigate harmful conditions.",
          assignmentMode: "deterministic"
        },
        {
          tag: "mental_recovery",
          description: "Helps stabilize emotions or recover from mental conditions.",
          assignmentMode: "deterministic"
        },
        {
          tag: "escape_support",
          description: "Helps flee, slip away, or break free.",
          assignmentMode: "deterministic"
        },
        {
          tag: "senses_support",
          description: "Improves vision or other senses.",
          assignmentMode: "deterministic"
        },
        {
          tag: "energy_resistance",
          description: "Grants resistance against one or more energy types.",
          assignmentMode: "deterministic"
        },
        {
          tag: "buff_support",
          description: "Provides a general beneficial enhancement or bonus.",
          assignmentMode: "deterministic"
        },
        {
          tag: "fortune_support",
          description: "Improves a creature's odds with rerolls, better-result effects, or failure rescue.",
          assignmentMode: "deterministic"
        }
      ]
    },
    polarity: {
      subcategories: [
        "consumable"
      ],
      description: "Offense/support polarity and delivery-style consumable tags.",
      tags: [
        {
          tag: "offensive",
          description: "Hostile consumable primarily meant to harm or debilitate a target.",
          assignmentMode: "deterministic"
        },
        {
          tag: "self_buff",
          description: "Support consumable primarily applied to the user.",
          assignmentMode: "deterministic"
        },
        {
          tag: "ally_support",
          description: "Support consumable that can directly benefit another creature.",
          assignmentMode: "deterministic"
        },
        {
          tag: "weapon_applied",
          description: "Offensive consumable applied to a weapon before use.",
          assignmentMode: "deterministic"
        },
        {
          tag: "thrown_offense",
          description: "Offensive consumable delivered by throwing it.",
          assignmentMode: "deterministic"
        },
        {
          tag: "ingested_offense",
          description: "Offensive consumable delivered when swallowed or consumed.",
          assignmentMode: "deterministic"
        },
        {
          tag: "contact_offense",
          description: "Offensive consumable delivered through touch or skin contact.",
          assignmentMode: "deterministic"
        }
      ]
    },
    purpose: {
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon"
      ],
      description: "Utility and logistics gear-purpose tags, including armor use cases.",
      tags: [
        {
          tag: "climbing",
          description: "Helps climb, rappel, or navigate vertical obstacles.",
          assignmentMode: "deterministic"
        },
        {
          tag: "lock_bypass",
          description: "Helps open locks or bypass secured entry points.",
          assignmentMode: "deterministic"
        },
        {
          tag: "concealable",
          description: "Easy to hide on the person or carry discreetly.",
          assignmentMode: "deterministic"
        },
        {
          tag: "scouting",
          description: "Helps observe, survey, or reconnoiter an area.",
          assignmentMode: "deterministic"
        },
        {
          tag: "mobility",
          description: "Improves movement or traversal flexibility.",
          assignmentMode: "deterministic"
        },
        {
          tag: "stealth_support",
          description: "Helps move quietly or avoid notice.",
          assignmentMode: "deterministic"
        },
        {
          tag: "illumination",
          description: "Produces or improves light in dark environments.",
          assignmentMode: "deterministic"
        },
        {
          tag: "survival",
          description: "Supports wilderness travel, shelter, or long-term field use.",
          assignmentMode: "deterministic"
        },
        {
          tag: "navigation",
          description: "Helps track direction, route, or position.",
          assignmentMode: "deterministic"
        },
        {
          tag: "tracking",
          description: "Helps follow trails, mark a target, or relocate something later.",
          assignmentMode: "deterministic"
        },
        {
          tag: "anti_tracking",
          description: "Helps hide your trail, mask scent, or make pursuit harder.",
          assignmentMode: "deterministic"
        },
        {
          tag: "transport",
          description: "Helps move creatures or cargo from place to place.",
          assignmentMode: "deterministic"
        },
        {
          tag: "trap_bypass",
          description: "Helps disarm, disable, or get past traps.",
          assignmentMode: "deterministic"
        },
        {
          tag: "carry_support",
          description: "Helps stow, carry, or organize equipment.",
          assignmentMode: "deterministic"
        },
        {
          tag: "restraint_escape",
          description: "Helps break free from grabs, restraints, or similar immobilizing holds.",
          assignmentMode: "deterministic"
        },
        {
          tag: "restraint_capture",
          description: "Helps capture, bind, or keep a target restrained.",
          assignmentMode: "deterministic"
        }
      ]
    },
    access_system: {
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "ammo",
        "armor",
        "weapon"
      ],
      description: "Storage, draw, and ammunition-handling equipment that changes how gear is accessed in play.",
      tags: [
        {
          tag: "extradimensional_storage",
          description: "Provides bag-of-holding-style storage through extradimensional or magically expanded space.",
          assignmentMode: "deterministic"
        },
        {
          tag: "weapon_staging",
          description: "Holsters, sheaths, scabbards, or bandoliers that stage weapons for quick draw or organized carry.",
          assignmentMode: "deterministic"
        },
        {
          tag: "ammo_management",
          description: "Magazines or related gear that manage repeating-weapon ammunition or reload workflow.",
          assignmentMode: "deterministic"
        }
      ]
    },
    ammunition_payload: {
      subcategories: [
        "ammo"
      ],
      description: "Ammunition payload tags for recurring on-hit effects and tailored projectile roles.",
      tags: [
        {
          tag: "creature_bane",
          description: "Tailored ammunition for a selected creature type or trait.",
          assignmentMode: "deterministic"
        },
        {
          tag: "elemental_payload",
          description: "Ammunition that delivers an elemental or reagent-based payload on impact.",
          assignmentMode: "deterministic"
        },
        {
          tag: "explosive_payload",
          description: "Ammunition that detonates or scatters area damage on impact.",
          assignmentMode: "deterministic"
        },
        {
          tag: "spell_payload",
          description: "Ammunition that delivers, casts, or imposes a spell effect on hit.",
          assignmentMode: "deterministic"
        }
      ]
    },
    impact: {
      subcategories: [
        "ammo",
        "consumable"
      ],
      description: "Hostile equipment tags for recurring target impairments and disabling outcomes.",
      tags: [
        {
          tag: "mobility_impairment",
          description: "Impairs movement through slowing, restraining, sticking, or immobilizing effects.",
          assignmentMode: "deterministic"
        },
        {
          tag: "sensory_impairment",
          description: "Impairs sight, hearing, or other senses through blinding, dazzling, or deafening effects.",
          assignmentMode: "deterministic"
        },
        {
          tag: "mental_impairment",
          description: "Impairs thought, judgment, composure, or behavior through fear, confusion, or similar effects.",
          assignmentMode: "deterministic"
        },
        {
          tag: "physical_debilitation",
          description: "Weakens the body through drained vitality, sickness, fatigue, clumsiness, or similar bodily degradation.",
          assignmentMode: "deterministic"
        },
        {
          tag: "sedation",
          description: "Induces sleep, lethargy, unconsciousness, or similar incapacitating drowsiness.",
          assignmentMode: "deterministic"
        }
      ]
    },
    expedition: {
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
        "armor",
        "shield",
        "weapon"
      ],
      description: "Travel, provisioning, mounted-combat, and aquatic-operations equipment.",
      tags: [
        {
          tag: "mounted_support",
          description: "Supports mounted combat, rider control, saddle use, or mount-specific loadouts.",
          assignmentMode: "deterministic"
        },
        {
          tag: "sustenance",
          description: "Provides food, feed, water, or other practical nourishment for travel and survival.",
          assignmentMode: "deterministic"
        },
        {
          tag: "aquatic_support",
          description: "Helps with swimming, underwater breathing, flotation, water-surface travel, or watercraft use.",
          assignmentMode: "deterministic"
        }
      ]
    },
    communication: {
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable"
      ],
      description: "Coordination, signaling, and message-relay equipment.",
      tags: [
        {
          tag: "signaling",
          description: "Helps draw attention, mark a location, or coordinate allies.",
          assignmentMode: "deterministic"
        },
        {
          tag: "message_delivery",
          description: "Sends, stores, or relays actual content across time or distance.",
          assignmentMode: "deterministic"
        }
      ]
    },
    infiltration: {
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable"
      ],
      description: "Appearance-changing and social-passing equipment across gear and consumables.",
      tags: [
        {
          tag: "disguise",
          description: "Helps alter appearance or impersonate another identity.",
          assignmentMode: "deterministic"
        },
        {
          tag: "social_infiltration",
          description: "Helps blend into a group or pass under social scrutiny.",
          assignmentMode: "deterministic"
        },
        {
          tag: "concealment",
          description: "Helps obscure a creature, item, or area from sight or make it harder to perceive.",
          assignmentMode: "deterministic"
        }
      ]
    },
    magic_interference: {
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable"
      ],
      description: "Equipment that disrupts hostile magic or protects against it.",
      tags: [
        {
          tag: "countermagic",
          description: "Counteracts, dispels, suppresses, or shuts down magic.",
          assignmentMode: "deterministic"
        },
        {
          tag: "magic_protection",
          description: "Protects the user or target against hostile magical effects.",
          assignmentMode: "deterministic"
        }
      ]
    },
    defense_profile: {
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "shield"
      ],
      description: "Shield and armor tags for interception, cover, and hazard-facing protection.",
      tags: [
        {
          tag: "ally_cover",
          description: "Provides cover or upgraded cover to nearby allies.",
          assignmentMode: "deterministic"
        },
        {
          tag: "projectile_defense",
          description: "Intercepts, redirects, or absorbs ranged attacks and projectiles.",
          assignmentMode: "deterministic"
        },
        {
          tag: "hazard_shielding",
          description: "Protects against environmental hazards, area effects, or other damaging exposures.",
          assignmentMode: "deterministic"
        }
      ]
    },
    security: {
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable"
      ],
      description: "Intrusion-warning gear and consumables.",
      tags: [
        {
          tag: "alarm",
          description: "Alerts you or others when a watched area, threshold, or device is triggered.",
          assignmentMode: "deterministic"
        }
      ]
    }
  }
} satisfies DerivedTagAuthoredCategoryOntology;
