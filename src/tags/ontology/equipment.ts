import type { DerivedTagOntologyFamily, DerivedTagOntologyTag } from "../../types.js";

export const EQUIPMENT_DERIVED_TAG_ONTOLOGY_FAMILIES: DerivedTagOntologyFamily[] = [
  {
    category: "equipment",
    subcategories: [
      "consumable"
    ],
    family: "function",
    description: "Beneficial consumable outcome and recovery tags."
  },
  {
    category: "equipment",
    subcategories: [
      "consumable"
    ],
    family: "polarity",
    description: "Offense/support polarity and delivery-style consumable tags."
  },
  {
    category: "equipment",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    family: "purpose",
    description: "Utility and logistics gear-purpose tags, including armor use cases."
  },
  {
    category: "equipment",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "ammo",
      "armor",
      "weapon"
    ],
    family: "access_system",
    description: "Storage, draw, and ammunition-handling equipment that changes how gear is accessed in play."
  },
  {
    category: "equipment",
    subcategories: [
      "ammo"
    ],
    family: "ammunition_payload",
    description: "Ammunition payload tags for recurring on-hit effects and tailored projectile roles."
  },
  {
    category: "equipment",
    subcategories: [
      "ammo",
      "consumable"
    ],
    family: "impact",
    description: "Hostile equipment tags for recurring target impairments and disabling outcomes."
  },
  {
    category: "equipment",
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
    family: "expedition",
    description: "Travel, provisioning, mounted-combat, and aquatic-operations equipment."
  },
  {
    category: "equipment",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    family: "communication",
    description: "Coordination, signaling, and message-relay equipment."
  },
  {
    category: "equipment",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    family: "infiltration",
    description: "Appearance-changing and social-passing equipment across gear and consumables."
  },
  {
    category: "equipment",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    family: "magic_interference",
    description: "Equipment that disrupts hostile magic or protects against it."
  },
  {
    category: "equipment",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "shield"
    ],
    family: "defense_profile",
    description: "Shield and armor tags for interception, cover, and hazard-facing protection."
  },
  {
    category: "equipment",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    family: "security",
    description: "Intrusion-warning gear and consumables."
  }
];

export const EQUIPMENT_DERIVED_TAG_ONTOLOGY_TAGS: DerivedTagOntologyTag[] = [
  {
    category: "equipment",
    family: "function",
    tag: "beneficial",
    description: "Broad support-oriented consumable with non-hostile intent.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "function",
    tag: "healing_support",
    description: "Restores hit points or provides direct healing.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "function",
    tag: "anti_poison",
    description: "Helps resist, prevent, or recover from poison.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "function",
    tag: "anti_disease",
    description: "Helps resist, prevent, or recover from disease.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "function",
    tag: "condition_support",
    description: "Helps clear or mitigate harmful conditions.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "function",
    tag: "mental_recovery",
    description: "Helps stabilize emotions or recover from mental conditions.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "function",
    tag: "escape_support",
    description: "Helps flee, slip away, or break free.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "function",
    tag: "senses_support",
    description: "Improves vision or other senses.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "function",
    tag: "energy_resistance",
    description: "Grants resistance against one or more energy types.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "function",
    tag: "buff_support",
    description: "Provides a general beneficial enhancement or bonus.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "function",
    tag: "fortune_support",
    description: "Improves a creature's odds with rerolls, better-result effects, or failure rescue.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "polarity",
    tag: "offensive",
    description: "Hostile consumable primarily meant to harm or debilitate a target.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "polarity",
    tag: "self_buff",
    description: "Support consumable primarily applied to the user.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "polarity",
    tag: "ally_support",
    description: "Support consumable that can directly benefit another creature.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "polarity",
    tag: "weapon_applied",
    description: "Offensive consumable applied to a weapon before use.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "polarity",
    tag: "thrown_offense",
    description: "Offensive consumable delivered by throwing it.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "polarity",
    tag: "ingested_offense",
    description: "Offensive consumable delivered when swallowed or consumed.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "polarity",
    tag: "contact_offense",
    description: "Offensive consumable delivered through touch or skin contact.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "climbing",
    description: "Helps climb, rappel, or navigate vertical obstacles.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "lock_bypass",
    description: "Helps open locks or bypass secured entry points.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "concealable",
    description: "Easy to hide on the person or carry discreetly.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "scouting",
    description: "Helps observe, survey, or reconnoiter an area.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "mobility",
    description: "Improves movement or traversal flexibility.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "stealth_support",
    description: "Helps move quietly or avoid notice.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "illumination",
    description: "Produces or improves light in dark environments.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "survival",
    description: "Supports wilderness travel, shelter, or long-term field use.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "navigation",
    description: "Helps track direction, route, or position.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "tracking",
    description: "Helps follow trails, mark a target, or relocate something later.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "anti_tracking",
    description: "Helps hide your trail, mask scent, or make pursuit harder.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "transport",
    description: "Helps move creatures or cargo from place to place.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "trap_bypass",
    description: "Helps disarm, disable, or get past traps.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "carry_support",
    description: "Helps stow, carry, or organize equipment.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "restraint_escape",
    description: "Helps break free from grabs, restraints, or similar immobilizing holds.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "purpose",
    tag: "restraint_capture",
    description: "Helps capture, bind, or keep a target restrained.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "access_system",
    tag: "extradimensional_storage",
    description: "Provides bag-of-holding-style storage through extradimensional or magically expanded space.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "access_system",
    tag: "weapon_staging",
    description: "Holsters, sheaths, scabbards, or bandoliers that stage weapons for quick draw or organized carry.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "access_system",
    tag: "ammo_management",
    description: "Magazines or related gear that manage repeating-weapon ammunition or reload workflow.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "ammunition_payload",
    tag: "creature_bane",
    description: "Tailored ammunition for a selected creature type or trait.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "ammunition_payload",
    tag: "elemental_payload",
    description: "Ammunition that delivers an elemental or reagent-based payload on impact.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "ammunition_payload",
    tag: "explosive_payload",
    description: "Ammunition that detonates or scatters area damage on impact.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "ammunition_payload",
    tag: "spell_payload",
    description: "Ammunition that delivers, casts, or imposes a spell effect on hit.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "impact",
    tag: "mobility_impairment",
    description: "Impairs movement through slowing, restraining, sticking, or immobilizing effects.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "impact",
    tag: "sensory_impairment",
    description: "Impairs sight, hearing, or other senses through blinding, dazzling, or deafening effects.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "impact",
    tag: "mental_impairment",
    description: "Impairs thought, judgment, composure, or behavior through fear, confusion, or similar effects.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "impact",
    tag: "physical_debilitation",
    description: "Weakens the body through drained vitality, sickness, fatigue, clumsiness, or similar bodily degradation.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "impact",
    tag: "sedation",
    description: "Induces sleep, lethargy, unconsciousness, or similar incapacitating drowsiness.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "expedition",
    tag: "mounted_support",
    description: "Supports mounted combat, rider control, saddle use, or mount-specific loadouts.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "expedition",
    tag: "sustenance",
    description: "Provides food, feed, water, or other practical nourishment for travel and survival.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "expedition",
    tag: "aquatic_support",
    description: "Helps with swimming, underwater breathing, flotation, water-surface travel, or watercraft use.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "communication",
    tag: "signaling",
    description: "Helps draw attention, mark a location, or coordinate allies.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "communication",
    tag: "message_delivery",
    description: "Sends, stores, or relays actual content across time or distance.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "infiltration",
    tag: "disguise",
    description: "Helps alter appearance or impersonate another identity.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "infiltration",
    tag: "social_infiltration",
    description: "Helps blend into a group or pass under social scrutiny.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "infiltration",
    tag: "concealment",
    description: "Helps obscure a creature, item, or area from sight or make it harder to perceive.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "magic_interference",
    tag: "countermagic",
    description: "Counteracts, dispels, suppresses, or shuts down magic.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "magic_interference",
    tag: "magic_protection",
    description: "Protects the user or target against hostile magical effects.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "defense_profile",
    tag: "ally_cover",
    description: "Provides cover or upgraded cover to nearby allies.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "defense_profile",
    tag: "projectile_defense",
    description: "Intercepts, redirects, or absorbs ranged attacks and projectiles.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "defense_profile",
    tag: "hazard_shielding",
    description: "Protects against environmental hazards, area effects, or other damaging exposures.",
    assignmentMode: "deterministic"
  },
  {
    category: "equipment",
    family: "security",
    tag: "alarm",
    description: "Alerts you or others when a watched area, threshold, or device is triggered.",
    assignmentMode: "deterministic"
  }
];
