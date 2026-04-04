import { DerivedTagCatalogEntry, type SearchSubcategory } from "../../types.js";

import { DISGUISE_SUBCATEGORIES, GEARISH_SUBCATEGORIES } from "../shared.js";

const PURPOSE_SUBCATEGORIES: SearchSubcategory[] = [...GEARISH_SUBCATEGORIES, "armor"];
const EXPEDITION_SUBCATEGORIES: SearchSubcategory[] = [...GEARISH_SUBCATEGORIES, "consumable", "armor", "shield", "weapon"];

export const EQUIPMENT_DERIVED_TAG_CATALOG: DerivedTagCatalogEntry[] = [
  {
    category: "equipment",
    subcategories: ["consumable"],
    family: "function",
    description: "Beneficial consumable outcome and recovery tags.",
    tags: [
      { value: "beneficial", description: "Broad support-oriented consumable with non-hostile intent." },
      { value: "healing_support", description: "Restores hit points or provides direct healing." },
      { value: "anti_poison", description: "Helps resist, prevent, or recover from poison." },
      { value: "anti_disease", description: "Helps resist, prevent, or recover from disease." },
      { value: "condition_support", description: "Helps clear or mitigate harmful conditions." },
      { value: "mental_recovery", description: "Helps stabilize emotions or recover from mental conditions." },
      { value: "escape_support", description: "Helps flee, slip away, or break free." },
      { value: "senses_support", description: "Improves vision or other senses." },
      { value: "energy_resistance", description: "Grants resistance against one or more energy types." },
      { value: "buff_support", description: "Provides a general beneficial enhancement or bonus." },
      { value: "fortune_support", description: "Improves a creature's odds with rerolls, better-result effects, or failure rescue." },
    ],
  },
  {
    category: "equipment",
    subcategories: ["consumable"],
    family: "polarity",
    description: "Offense/support polarity and delivery-style consumable tags.",
    tags: [
      { value: "offensive", description: "Hostile consumable primarily meant to harm or debilitate a target." },
      { value: "self_buff", description: "Support consumable primarily applied to the user." },
      { value: "ally_support", description: "Support consumable that can directly benefit another creature." },
      { value: "weapon_applied", description: "Offensive consumable applied to a weapon before use." },
      { value: "thrown_offense", description: "Offensive consumable delivered by throwing it." },
      { value: "ingested_offense", description: "Offensive consumable delivered when swallowed or consumed." },
      { value: "contact_offense", description: "Offensive consumable delivered through touch or skin contact." },
    ],
  },
  {
    category: "equipment",
    subcategories: PURPOSE_SUBCATEGORIES,
    family: "purpose",
    description: "Utility and logistics gear-purpose tags, including armor use cases.",
    tags: [
      { value: "climbing", description: "Helps climb, rappel, or navigate vertical obstacles." },
      { value: "lock_bypass", description: "Helps open locks or bypass secured entry points." },
      { value: "concealable", description: "Easy to hide on the person or carry discreetly." },
      { value: "scouting", description: "Helps observe, survey, or reconnoiter an area." },
      { value: "mobility", description: "Improves movement or traversal flexibility." },
      { value: "stealth_support", description: "Helps move quietly or avoid notice." },
      { value: "illumination", description: "Produces or improves light in dark environments." },
      { value: "survival", description: "Supports wilderness travel, shelter, or long-term field use." },
      { value: "navigation", description: "Helps track direction, route, or position." },
      { value: "tracking", description: "Helps follow trails, mark a target, or relocate something later." },
      { value: "anti_tracking", description: "Helps hide your trail, mask scent, or make pursuit harder." },
      { value: "transport", description: "Helps move creatures or cargo from place to place." },
      { value: "trap_bypass", description: "Helps disarm, disable, or get past traps." },
      { value: "carry_support", description: "Helps stow, carry, or organize equipment." },
      { value: "restraint_escape", description: "Helps break free from grabs, restraints, or similar immobilizing holds." },
      { value: "restraint_capture", description: "Helps capture, bind, or keep a target restrained." },
    ],
  },
  {
    category: "equipment",
    subcategories: ["ammo"],
    family: "ammunition_payload",
    description: "Ammunition payload tags for recurring on-hit effects and tailored projectile roles.",
    tags: [
      { value: "creature_bane", description: "Tailored ammunition for a selected creature type or trait." },
      { value: "elemental_payload", description: "Ammunition that delivers an elemental or reagent-based payload on impact." },
      { value: "explosive_payload", description: "Ammunition that detonates or scatters area damage on impact." },
    ],
  },
  {
    category: "equipment",
    subcategories: ["ammo", "consumable"],
    family: "impact",
    description: "Hostile equipment tags for recurring target impairments and disabling outcomes.",
    tags: [
      { value: "mobility_impairment", description: "Impairs movement through slowing, restraining, sticking, or immobilizing effects." },
      { value: "sensory_impairment", description: "Impairs sight, hearing, or other senses through blinding, dazzling, or deafening effects." },
      { value: "mental_impairment", description: "Impairs thought, judgment, composure, or behavior through fear, confusion, or similar effects." },
      { value: "physical_debilitation", description: "Weakens the body through drained vitality, sickness, fatigue, clumsiness, or similar bodily degradation." },
      { value: "sedation", description: "Induces sleep, lethargy, unconsciousness, or similar incapacitating drowsiness." },
    ],
  },
  {
    category: "equipment",
    subcategories: EXPEDITION_SUBCATEGORIES,
    family: "expedition",
    description: "Travel, provisioning, mounted-combat, and aquatic-operations equipment.",
    tags: [
      { value: "mounted_support", description: "Supports mounted combat, rider control, saddle use, or mount-specific loadouts." },
      { value: "sustenance", description: "Provides food, feed, water, or other practical nourishment for travel and survival." },
      { value: "aquatic_support", description: "Helps with swimming, underwater breathing, flotation, water-surface travel, or watercraft use." },
    ],
  },
  {
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    family: "communication",
    description: "Coordination, signaling, and message-relay equipment.",
    tags: [
      { value: "signaling", description: "Helps draw attention, mark a location, or coordinate allies." },
      { value: "message_delivery", description: "Sends, stores, or relays actual content across time or distance." },
    ],
  },
  {
    category: "equipment",
    subcategories: DISGUISE_SUBCATEGORIES,
    family: "infiltration",
    description: "Appearance-changing and social-passing equipment across gear and consumables.",
    tags: [
      { value: "disguise", description: "Helps alter appearance or impersonate another identity." },
      { value: "social_infiltration", description: "Helps blend into a group or pass under social scrutiny." },
      { value: "concealment", description: "Helps obscure a creature, item, or area from sight or make it harder to perceive." },
    ],
  },
  {
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    family: "magic_interference",
    description: "Equipment that disrupts hostile magic or protects against it.",
    tags: [
      { value: "countermagic", description: "Counteracts, dispels, suppresses, or shuts down magic." },
      { value: "magic_protection", description: "Protects the user or target against hostile magical effects." },
    ],
  },
  {
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "armor", "shield"],
    family: "defense_profile",
    description: "Shield and armor tags for interception, cover, and hazard-facing protection.",
    tags: [
      { value: "ally_cover", description: "Provides cover or upgraded cover to nearby allies." },
      { value: "projectile_defense", description: "Intercepts, redirects, or absorbs ranged attacks and projectiles." },
      { value: "hazard_shielding", description: "Protects against environmental hazards, area effects, or other damaging exposures." },
    ],
  },
  {
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    family: "security",
    description: "Intrusion-warning gear and consumables.",
    tags: [
      { value: "alarm", description: "Alerts you or others when a watched area, threshold, or device is triggered." },
    ],
  },
];
