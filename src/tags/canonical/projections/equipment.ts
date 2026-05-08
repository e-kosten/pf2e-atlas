import type { DerivedTagCategoryProjection } from "../../../domain/derived-tag-types.js";

export const EQUIPMENT_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG: Record<string, DerivedTagCategoryProjection> = 
{
  action_economy_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "action_economy_support",
    currentTag: "action_economy_support",
    description: "Especially valuable because it compresses setup, speeds access, or meaningfully improves in-combat action efficiency.",
    family: "play_pattern",
    id: "equipment:action_economy_support",
    label: "action_economy_support",
    translationStatus: "mapped"
  },
  alarm: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "alarm",
    currentTag: "alarm",
    description: "Alerts you or others when a watched area, threshold, or device is triggered.",
    family: "security",
    id: "equipment:alarm",
    label: "alarm",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  alchemical_crafting: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "alchemical_crafting",
    currentTag: "alchemical_crafting",
    description: "Supports alchemical preparation, formula work, reagent handling, or crafting-related field setup.",
    family: "crafting_support",
    id: "equipment:alchemical_crafting",
    label: "alchemical_crafting",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  ally_cover: {
    assignmentMode: "deterministic",
    axis: "item_mechanical",
    category: "equipment",
    conceptId: "ally_cover",
    currentTag: "ally_cover",
    description: "Provides cover or upgraded cover to nearby allies.",
    family: "defense_profile",
    id: "equipment:ally_cover",
    label: "ally_cover",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "shield"
    ],
    translationStatus: "mapped"
  },
  ally_support: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "ally_support",
    currentTag: "ally_support",
    description: "Support consumable that can directly benefit another creature.",
    family: "consumable_role",
    id: "equipment:ally_support",
    label: "ally_support",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  ammo_management: {
    assignmentMode: "deterministic",
    axis: "item_mechanical",
    category: "equipment",
    conceptId: "ammo_management",
    currentTag: "ammo_management",
    description: "Magazines or related gear that manage repeating-weapon ammunition or reload workflow.",
    family: "access_system",
    id: "equipment:ammo_management",
    label: "ammo_management",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "ammo",
      "armor",
      "weapon"
    ],
    translationStatus: "mapped"
  },
  anti_bleed: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "bleed_remediation",
    currentTag: "anti_bleed",
    description: "Helps staunch bleeding, end persistent bleed damage, or close ongoing wounds.",
    family: "function",
    id: "equipment:anti_bleed",
    label: "bleed_remediation",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  anti_caster_disruption: {
    adjacentTags: [
      "silencing",
      "countermagic"
    ],
    assignmentMode: "hybrid",
    axis: "effect",
    category: "equipment",
    conceptId: "caster_disruption_remediation",
    currentTag: "anti_caster_disruption",
    description: "Punishes casting, disrupts concentration, or is naturally retrieved to make enemy spell use unreliable.",
    family: "offensive_profile",
    id: "equipment:anti_caster_disruption",
    label: "caster_disruption_remediation",
    subcategories: [
      "ammo",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  anti_confusion: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "confusion_remediation",
    currentTag: "anti_confusion",
    description: "Helps clear confusion, restore mental steadiness, or recover from disordered thinking.",
    family: "function",
    id: "equipment:anti_confusion",
    label: "confusion_remediation",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  anti_disease: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "disease_remediation",
    currentTag: "anti_disease",
    description: "Helps resist, prevent, or recover from disease.",
    family: "function",
    id: "equipment:anti_disease",
    label: "disease_remediation",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  anti_fear: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "fear_remediation",
    currentTag: "anti_fear",
    description: "Helps resist fear, recover from frightened effects, or steady courage.",
    family: "function",
    id: "equipment:anti_fear",
    label: "fear_remediation",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  anti_paralysis: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "paralysis_remediation",
    currentTag: "anti_paralysis",
    description: "Helps break paralysis, restore movement, or free a creature from immobilizing body shutdown.",
    family: "function",
    id: "equipment:anti_paralysis",
    label: "paralysis_remediation",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  anti_petrification: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "petrification_remediation",
    currentTag: "anti_petrification",
    description: "Helps prevent or reverse petrification and other stone-turning effects.",
    family: "function",
    id: "equipment:anti_petrification",
    label: "petrification_remediation",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  anti_poison: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "poison_remediation",
    currentTag: "anti_poison",
    description: "Helps resist, prevent, or recover from poison.",
    family: "function",
    id: "equipment:anti_poison",
    label: "poison_remediation",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  anti_tracking: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "anti_tracking",
    currentTag: "anti_tracking",
    description: "Helps hide your trail, mask scent, or make pursuit harder.",
    family: "reconnaissance",
    id: "equipment:anti_tracking",
    label: "anti_tracking",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  aquatic_support: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "aquatic_support",
    currentTag: "aquatic_support",
    description: "Helps with swimming, underwater breathing, flotation, water-surface travel, or watercraft use.",
    family: "expedition",
    id: "equipment:aquatic_support",
    label: "aquatic_support",
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
    translationStatus: "provisional"
  },
  barrier_breaking: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "barrier_breaking",
    currentTag: "barrier_breaking",
    description: "Designed to tear through walls, barricades, ice, webs, or other physical obstructions.",
    family: "breaching",
    id: "equipment:barrier_breaking",
    label: "barrier_breaking",
    subcategories: [
      "gear",
      "kit",
      "consumable",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  barrier_bypass: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "barrier_bypass",
    currentTag: "barrier_bypass",
    description: "Helps get through barred windows, grates, force screens, or other blocked passage without relying on brute-force breaching.",
    family: "access_bypass",
    id: "equipment:barrier_bypass",
    label: "barrier_bypass",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  breaching: {
    adjacentTags: [
      "door_breaching",
      "barrier_breaking",
      "demolition"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "equipment",
    compositeOfAnyTags: [
      "door_breaching",
      "barrier_breaking",
      "excavation",
      "siege_support",
      "demolition"
    ],
    conceptId: "breaching",
    currentTag: "breaching",
    description: "Broad force-entry umbrella for equipment used to break doors, barriers, fortifications, or route-blocking structures.",
    family: "breaching",
    id: "equipment:breaching",
    label: "breaching",
    subcategories: [
      "gear",
      "kit",
      "consumable",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  buff_support: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "buff_support",
    currentTag: "buff_support",
    description: "Provides a general beneficial enhancement or bonus.",
    family: "function",
    id: "equipment:buff_support",
    label: "buff_support",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  burst_damage: {
    adjacentTags: [
      "crowd_clearing",
      "persistent_damage"
    ],
    assignmentMode: "hybrid",
    axis: "effect",
    category: "equipment",
    conceptId: "burst_damage",
    currentTag: "burst_damage",
    description: "Delivers a front-loaded blast, detonation, or splash pattern that users naturally retrieve as immediate damage.",
    family: "offensive_profile",
    id: "equipment:burst_damage",
    label: "burst_damage",
    subcategories: [
      "ammo",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  camp_setup: {
    adjacentTags: [
      "sustenance",
      "carry_support"
    ],
    appliesWhen: [
      "The item is naturally retrieved for making camp, resting outdoors, or supporting expedition downtime.",
      "Its value is in field habitation rather than only carrying gear or feeding travelers."
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "camp_setup",
    currentTag: "camp_setup",
    description: "Supports campsite creation, resting infrastructure, shelter setup, or extended overland staging.",
    doesNotApplyWhen: [
      "The item only provides sustenance, mobility, or transport without campsite infrastructure.",
      "The item is merely general survival gear with no setup or shelter-facing role."
    ],
    family: "expedition",
    id: "equipment:camp_setup",
    label: "camp_setup",
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
    translationStatus: "provisional"
  },
  carry_support: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "carry_support",
    currentTag: "carry_support",
    description: "Helps stow, carry, or organize equipment.",
    family: "carry_logistics",
    id: "equipment:carry_support",
    label: "carry_support",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  caster_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "caster_support",
    currentTag: "caster_support",
    description: "Especially valuable to spellcasters for magical prep, casting reliability, spell defense, or spell-adjacent utility.",
    family: "party_role",
    id: "equipment:caster_support",
    label: "caster_support",
    translationStatus: "mapped"
  },
  climbing: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "climbing",
    currentTag: "climbing",
    description: "Helps climb, rappel, or navigate vertical obstacles.",
    family: "movement_traversal",
    id: "equipment:climbing",
    label: "climbing",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  combat_maneuver_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "combat_maneuver_support",
    currentTag: "combat_maneuver_support",
    description: "Especially valuable to builds that rely on grappling, tripping, shoving, disarming, or other combat maneuvers to control enemies.",
    family: "play_pattern",
    id: "equipment:combat_maneuver_support",
    label: "combat_maneuver_support",
    translationStatus: "mapped"
  },
  communication: {
    adjacentTags: [
      "signaling",
      "telepathic_communication",
      "message_delivery"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "equipment",
    compositeOfAnyTags: [
      "signaling",
      "telepathic_communication",
      "message_delivery",
      "translation_support"
    ],
    conceptId: "communication",
    currentTag: "communication",
    description: "Broad communication umbrella for equipment used to signal allies, relay messages, bridge language barriers, or coordinate silently.",
    family: "communication",
    id: "equipment:communication",
    label: "communication",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  companion_handling_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "companion_handling_support",
    currentTag: "companion_handling_support",
    description: "Especially valuable when a character's tactical workflow depends on commanding, outfitting, transporting, or protecting companions and mounts.",
    family: "play_pattern",
    id: "equipment:companion_handling_support",
    label: "companion_handling_support",
    translationStatus: "mapped"
  },
  companion_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "companion_support",
    currentTag: "companion_support",
    description: "Especially valuable to handlers of animal companions, mounts, familiars, or other creature-side support play patterns.",
    family: "party_role",
    id: "equipment:companion_support",
    label: "companion_support",
    translationStatus: "mapped"
  },
  concealable: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "concealable",
    currentTag: "concealable",
    description: "Easy to hide on the person or carry discreetly.",
    family: "infiltration",
    id: "equipment:concealable",
    label: "concealable",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  concealment: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "concealment",
    currentTag: "concealment",
    description: "Helps obscure a creature, item, or area from sight or make it harder to perceive.",
    family: "infiltration",
    id: "equipment:concealment",
    label: "concealment",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  condition_support: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "condition_support",
    currentTag: "condition_support",
    description: "Helps clear or mitigate harmful conditions.",
    family: "function",
    id: "equipment:condition_support",
    label: "condition_support",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  contact_offense: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "contact_offense",
    currentTag: "contact_offense",
    description: "Offensive consumable delivered through touch or skin contact.",
    family: "delivery_profile",
    id: "equipment:contact_offense",
    label: "contact_offense",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  contamination_cleanup: {
    adjacentTags: [
      "quarantine_containment",
      "source_cleanup"
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "contamination_cleanup",
    currentTag: "contamination_cleanup",
    description: "Helps neutralize tainted residue, clean corrupted surfaces, purify contaminated supplies, or scrub a dangerous site back to safety.",
    family: "resolution",
    id: "equipment:contamination_cleanup",
    label: "contamination_cleanup",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  countermagic: {
    adjacentTags: [
      "magic_protection",
      "scrying_protection"
    ],
    appliesWhen: [
      "The item's main value is actively cancelling, suppressing, or interfering with hostile or ongoing magic.",
      "A user would retrieve it as an anti-magic tool rather than a general protective charm."
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "active_magic_counteraction",
    currentTag: "countermagic",
    description: "Counteracts, dispels, suppresses, or shuts down magic.",
    doesNotApplyWhen: [
      "The item only protects the wearer from magical harm without disrupting the spell itself.",
      "The item focuses on blocking surveillance or hiding information rather than broader anti-magic interference."
    ],
    family: "anti_magic",
    id: "equipment:countermagic",
    label: "countermagic",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  creature_bane: {
    assignmentMode: "deterministic",
    axis: "item_mechanical",
    category: "equipment",
    conceptId: "creature_bane",
    currentTag: "creature_bane",
    description: "Tailored ammunition for a selected creature type or trait.",
    family: "ammunition_payload",
    id: "equipment:creature_bane",
    label: "creature_bane",
    subcategories: [
      "ammo"
    ],
    translationStatus: "mapped"
  },
  crowd_clearing: {
    adjacentTags: [
      "burst_damage",
      "line_of_sight_control"
    ],
    assignmentMode: "hybrid",
    axis: "effect",
    category: "equipment",
    conceptId: "crowd_clearing",
    currentTag: "crowd_clearing",
    description: "Excels at damaging clusters, swarms, or tightly packed weaker enemies rather than focusing on one target.",
    family: "offensive_profile",
    id: "equipment:crowd_clearing",
    label: "crowd_clearing",
    subcategories: [
      "ammo",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  curse_removal: {
    adjacentTags: [
      "sanctification",
      "source_cleanup"
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "curse_remediation",
    currentTag: "curse_removal",
    description: "Helps remove, break, or counteract curses as a direct answer path rather than only easing symptoms.",
    family: "resolution",
    id: "equipment:curse_removal",
    label: "curse_remediation",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  defender_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "defender_support",
    currentTag: "defender_support",
    description: "Especially valuable to shield users, bodyguards, line-holders, or other defender-style characters.",
    family: "party_role",
    id: "equipment:defender_support",
    label: "defender_support",
    translationStatus: "mapped"
  },
  demolition: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "demolition",
    currentTag: "demolition",
    description: "Designed for blasting, collapsing, or otherwise violently dismantling structures and obstacles.",
    family: "breaching",
    id: "equipment:demolition",
    label: "demolition",
    subcategories: [
      "gear",
      "kit",
      "consumable",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  disguise: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "disguise",
    currentTag: "disguise",
    description: "Helps alter appearance or impersonate another identity.",
    family: "infiltration",
    id: "equipment:disguise",
    label: "disguise",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  door_breaching: {
    adjacentTags: [
      "lock_bypass",
      "barrier_breaking"
    ],
    appliesWhen: [
      "The item's retrieval value is getting through doors, shutters, gates, or secured entry points.",
      "It solves access by force rather than by keys, stealth, or lock tools."
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "door_breaching",
    currentTag: "door_breaching",
    description: "Helps force doors, shutters, gates, or similar entry points open by strength, impact, or destructive entry.",
    doesNotApplyWhen: [
      "The item is for larger demolition or siegework rather than point-of-entry breach.",
      "The item bypasses access quietly through locks or trickery instead of force."
    ],
    family: "breaching",
    id: "equipment:door_breaching",
    label: "door_breaching",
    subcategories: [
      "gear",
      "kit",
      "consumable",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  elemental_payload: {
    assignmentMode: "deterministic",
    axis: "item_mechanical",
    category: "equipment",
    conceptId: "elemental_payload",
    currentTag: "elemental_payload",
    description: "Ammunition that delivers an elemental or reagent-based payload on impact.",
    family: "ammunition_payload",
    id: "equipment:elemental_payload",
    label: "elemental_payload",
    subcategories: [
      "ammo"
    ],
    translationStatus: "mapped"
  },
  emergency_recovery: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "emergency_recovery",
    currentTag: "emergency_recovery",
    description: "Especially valuable as a panic-button item for sudden healing, escape, stabilization, or critical condition rescue.",
    family: "party_role",
    id: "equipment:emergency_recovery",
    label: "emergency_recovery",
    translationStatus: "mapped"
  },
  energy_resistance: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "energy_resistance",
    currentTag: "energy_resistance",
    description: "Grants resistance against one or more energy types.",
    family: "function",
    id: "equipment:energy_resistance",
    label: "energy_resistance",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  environmental_adaptation: {
    adjacentTags: [
      "aquatic_support",
      "camp_setup",
      "hazard_shielding"
    ],
    appliesWhen: [
      "The item's retrieval value comes from surviving punishing climate, altitude, breathing hazards, immersion pressure, or similar expedition environments.",
      "It is naturally sought as environmental survival gear rather than a general defense item or campsite tool."
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "environmental_adaptation",
    currentTag: "environmental_adaptation",
    description: "Helps travelers endure extreme weather, thin air, smoke, pressure, or other dangerous environmental exposure.",
    doesNotApplyWhen: [
      "The item only protects against one incoming attack or hazard burst without broader travel-survival use.",
      "The item mainly creates camp infrastructure, carries provisions, or improves aquatic movement instead of adapting the user to the environment."
    ],
    family: "expedition",
    id: "equipment:environmental_adaptation",
    label: "environmental_adaptation",
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
    translationStatus: "provisional"
  },
  escape_support: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "escape_support",
    currentTag: "escape_support",
    description: "Helps flee, slip away, or break free.",
    family: "function",
    id: "equipment:escape_support",
    label: "escape_support",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  excavation: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "excavation",
    currentTag: "excavation",
    description: "Helps dig, cut through earth or stone, or otherwise open a route by excavation or practical earth-moving work.",
    family: "breaching",
    id: "equipment:excavation",
    label: "excavation",
    subcategories: [
      "gear",
      "kit",
      "consumable",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  expedition: {
    adjacentTags: [
      "survival",
      "sustenance",
      "environmental_adaptation"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "equipment",
    compositeOfAnyTags: [
      "survival",
      "mounted_support",
      "sustenance",
      "aquatic_support",
      "environmental_adaptation",
      "camp_setup"
    ],
    conceptId: "expedition",
    currentTag: "expedition",
    description: "Broad expedition umbrella for travel gear, camp support, sustainment, mounts, aquatic operations, and hostile-environment endurance.",
    family: "expedition",
    id: "equipment:expedition",
    label: "expedition",
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
    translationStatus: "provisional"
  },
  explosive_payload: {
    assignmentMode: "deterministic",
    axis: "item_mechanical",
    category: "equipment",
    conceptId: "explosive_payload",
    currentTag: "explosive_payload",
    description: "Ammunition that detonates or scatters area damage on impact.",
    family: "ammunition_payload",
    id: "equipment:explosive_payload",
    label: "explosive_payload",
    subcategories: [
      "ammo"
    ],
    translationStatus: "mapped"
  },
  extradimensional_storage: {
    assignmentMode: "deterministic",
    axis: "item_mechanical",
    category: "equipment",
    conceptId: "extradimensional_storage",
    currentTag: "extradimensional_storage",
    description: "Provides bag-of-holding-style storage through extradimensional or magically expanded space.",
    family: "access_system",
    id: "equipment:extradimensional_storage",
    label: "extradimensional_storage",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "ammo",
      "armor",
      "weapon"
    ],
    translationStatus: "mapped"
  },
  face_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "face_support",
    currentTag: "face_support",
    description: "Especially valuable to negotiators, deceivers, diplomats, or other socially forward characters who need influence, disguise, or presentation support.",
    family: "party_role",
    id: "equipment:face_support",
    label: "face_support",
    translationStatus: "mapped"
  },
  fall_protection: {
    assignmentMode: "deterministic",
    axis: "item_mechanical",
    category: "equipment",
    conceptId: "fall_protection",
    currentTag: "fall_protection",
    description: "Reduces falling harm, cushions impact, or protects against vertical movement accidents and collapse.",
    family: "defense_profile",
    id: "equipment:fall_protection",
    label: "fall_protection",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "shield"
    ],
    translationStatus: "mapped"
  },
  focus_magic_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "focus_magic_support",
    currentTag: "focus_magic_support",
    description: "Especially valuable to focus-spell-heavy or magic-routine-driven builds that want more reliable magical cadence and recovery.",
    family: "play_pattern",
    id: "equipment:focus_magic_support",
    label: "focus_magic_support",
    translationStatus: "mapped"
  },
  forgery_support: {
    adjacentTags: [
      "disguise",
      "writing_recordkeeping"
    ],
    appliesWhen: [
      "The item's retrieval value comes from imitating documents, seals, credentials, or official paperwork.",
      "It supports passing administrative scrutiny rather than just changing clothing or physical appearance."
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "forgery_support",
    currentTag: "forgery_support",
    description: "Supports document falsification, seal imitation, signature copying, or bureaucratic deception.",
    doesNotApplyWhen: [
      "The item only changes appearance or supports social disguise without document work.",
      "The item is a normal writing or archival tool with no deception-facing use."
    ],
    family: "crafting_support",
    id: "equipment:forgery_support",
    label: "forgery_support",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  fortune_support: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "fortune_support",
    currentTag: "fortune_support",
    description: "Improves a creature's odds with rerolls, better-result effects, or failure rescue.",
    family: "function",
    id: "equipment:fortune_support",
    label: "fortune_support",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  hazard_shielding: {
    adjacentTags: [
      "projectile_defense",
      "magic_protection"
    ],
    appliesWhen: [
      "The item is naturally retrieved for surviving traps, breath weapons, alchemical blasts, or other hazardous exposure.",
      "Protection against scenes and effects matters more than only deflecting direct weapon attacks."
    ],
    assignmentMode: "deterministic",
    axis: "item_mechanical",
    category: "equipment",
    conceptId: "hazard_shielding",
    currentTag: "hazard_shielding",
    description: "Protects against environmental hazards, area effects, or other damaging exposures.",
    doesNotApplyWhen: [
      "The item's main value is cover against arrows or weapon strikes rather than wider hazard exposure.",
      "The item mainly protects against spells specifically, making magic_protection the stronger hook."
    ],
    family: "defense_profile",
    id: "equipment:hazard_shielding",
    label: "hazard_shielding",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "shield"
    ],
    translationStatus: "mapped"
  },
  healer_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "healer_support",
    currentTag: "healer_support",
    description: "Especially valuable to medics, battlefield healers, or builds expected to stabilize, treat, or recover allies under pressure.",
    family: "party_role",
    id: "equipment:healer_support",
    label: "healer_support",
    translationStatus: "mapped"
  },
  healing_support: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "healing_support",
    currentTag: "healing_support",
    description: "Restores hit points or provides direct healing.",
    family: "function",
    id: "equipment:healing_support",
    label: "healing_support",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  illumination: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "illumination",
    currentTag: "illumination",
    description: "Produces or improves light in dark environments.",
    family: "reconnaissance",
    id: "equipment:illumination",
    label: "illumination",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  infiltration: {
    adjacentTags: [
      "stealth_support",
      "disguise",
      "social_infiltration"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "equipment",
    compositeOfAnyTags: [
      "stealth_support",
      "concealable",
      "disguise",
      "social_infiltration",
      "concealment"
    ],
    conceptId: "infiltration",
    currentTag: "infiltration",
    description: "Broad infiltration umbrella for quiet-entry, discreet-carry, disguise, and covert-passing equipment.",
    family: "infiltration",
    id: "equipment:infiltration",
    label: "infiltration",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  ingested_offense: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "ingested_offense",
    currentTag: "ingested_offense",
    description: "Offensive consumable delivered when swallowed or consumed.",
    family: "delivery_profile",
    id: "equipment:ingested_offense",
    label: "ingested_offense",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  line_of_sight_control: {
    adjacentTags: [
      "sensory_impairment",
      "restraint_capture"
    ],
    assignmentMode: "hybrid",
    axis: "effect",
    category: "equipment",
    conceptId: "line_of_sight_control",
    currentTag: "line_of_sight_control",
    description: "Obscures vision, fills an area with smoke, or otherwise denies clear sight lines as the item's main offensive or tactical job.",
    family: "offensive_profile",
    id: "equipment:line_of_sight_control",
    label: "line_of_sight_control",
    subcategories: [
      "ammo",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  lock_bypass: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "lock_bypass",
    currentTag: "lock_bypass",
    description: "Helps open locks or bypass secured entry points.",
    family: "access_bypass",
    id: "equipment:lock_bypass",
    label: "lock_bypass",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  magic_protection: {
    adjacentTags: [
      "countermagic",
      "hazard_shielding"
    ],
    appliesWhen: [
      "The item's value comes from warding the bearer against curses, spells, hostile magical conditions, or magical damage.",
      "Protection matters more than actually counteracting or suppressing the incoming magic."
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "magic_protection",
    currentTag: "magic_protection",
    description: "Protects the user or target against hostile magical effects.",
    doesNotApplyWhen: [
      "The item mainly shuts down active magic rather than defending a wearer or target.",
      "The stronger fit is scrying_protection because surveillance denial is the specific retrieval hook."
    ],
    family: "anti_magic",
    id: "equipment:magic_protection",
    label: "magic_protection",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  mechanism_manipulation: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "mechanism_manipulation",
    currentTag: "mechanism_manipulation",
    description: "Helps operate levers, latches, panels, pressure surfaces, or similar scene mechanisms from a safer or more advantageous position.",
    family: "access_bypass",
    id: "equipment:mechanism_manipulation",
    label: "mechanism_manipulation",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  medical_support: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "medical_support",
    currentTag: "medical_support",
    description: "Supports first aid, diagnosis, treatment, or ongoing medical care outside direct magical healing.",
    family: "crafting_support",
    id: "equipment:medical_support",
    label: "medical_support",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  mental_impairment: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "mental_impairment",
    currentTag: "mental_impairment",
    description: "Impairs thought, judgment, composure, or behavior through fear, confusion, or similar effects.",
    family: "impact",
    id: "equipment:mental_impairment",
    label: "mental_impairment",
    subcategories: [
      "ammo",
      "consumable"
    ],
    translationStatus: "mapped"
  },
  mental_recovery: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "mental_recovery",
    currentTag: "mental_recovery",
    description: "Helps stabilize emotions or recover from mental conditions.",
    family: "function",
    id: "equipment:mental_recovery",
    label: "mental_recovery",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  message_delivery: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "message_delivery",
    currentTag: "message_delivery",
    description: "Sends, stores, or relays actual content across time or distance.",
    family: "communication",
    id: "equipment:message_delivery",
    label: "message_delivery",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  mobility: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "mobility",
    currentTag: "mobility",
    description: "Improves movement or traversal flexibility.",
    family: "movement_traversal",
    id: "equipment:mobility",
    label: "mobility",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  mobility_impairment: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "mobility_impairment",
    currentTag: "mobility_impairment",
    description: "Impairs movement through slowing, restraining, sticking, or immobilizing effects.",
    family: "impact",
    id: "equipment:mobility_impairment",
    label: "mobility_impairment",
    subcategories: [
      "ammo",
      "consumable"
    ],
    translationStatus: "mapped"
  },
  mounted_support: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "mounted_support",
    currentTag: "mounted_support",
    description: "Supports mounted combat, rider control, saddle use, or mount-specific loadouts.",
    family: "expedition",
    id: "equipment:mounted_support",
    label: "mounted_support",
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
    translationStatus: "provisional"
  },
  movement_traversal: {
    adjacentTags: [
      "climbing",
      "navigation",
      "transport"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "equipment",
    compositeOfAnyTags: [
      "climbing",
      "mobility",
      "navigation",
      "transport"
    ],
    conceptId: "movement_traversal",
    currentTag: "movement_traversal",
    description: "Broad movement-and-travel umbrella for equipment that solves climbing, routefinding, repositioning, or transport problems.",
    family: "movement_traversal",
    id: "equipment:movement_traversal",
    label: "movement_traversal",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  navigation: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "navigation",
    currentTag: "navigation",
    description: "Helps track direction, route, or position.",
    family: "movement_traversal",
    id: "equipment:navigation",
    label: "navigation",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  offensive: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "offensive",
    currentTag: "offensive",
    description: "Hostile consumable primarily meant to harm or debilitate a target.",
    family: "consumable_role",
    id: "equipment:offensive",
    label: "offensive",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  persistent_damage: {
    adjacentTags: [
      "burst_damage",
      "physical_debilitation"
    ],
    assignmentMode: "hybrid",
    axis: "effect",
    category: "equipment",
    conceptId: "persistent_damage",
    currentTag: "persistent_damage",
    description: "Imposes ongoing damage through burning, acid, bleed, poison, or another lingering payload that keeps hurting after the initial hit.",
    family: "offensive_profile",
    id: "equipment:persistent_damage",
    label: "persistent_damage",
    subcategories: [
      "ammo",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  physical_debilitation: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "physical_debilitation",
    currentTag: "physical_debilitation",
    description: "Weakens the body through drained vitality, sickness, fatigue, clumsiness, or similar bodily degradation.",
    family: "impact",
    id: "equipment:physical_debilitation",
    label: "physical_debilitation",
    subcategories: [
      "ammo",
      "consumable"
    ],
    translationStatus: "mapped"
  },
  projectile_defense: {
    assignmentMode: "deterministic",
    axis: "item_mechanical",
    category: "equipment",
    conceptId: "projectile_defense",
    currentTag: "projectile_defense",
    description: "Intercepts, redirects, or absorbs ranged attacks and projectiles.",
    family: "defense_profile",
    id: "equipment:projectile_defense",
    label: "projectile_defense",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "shield"
    ],
    translationStatus: "mapped"
  },
  quarantine_containment: {
    adjacentTags: [
      "contamination_cleanup",
      "source_cleanup"
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "outbreak_containment",
    currentTag: "quarantine_containment",
    description: "Helps isolate victims, secure contaminated areas, or impose practical containment procedures that stop spread while treatment proceeds.",
    family: "resolution",
    id: "equipment:quarantine_containment",
    label: "outbreak_containment",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  ranged_striker_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "ranged_striker_support",
    currentTag: "ranged_striker_support",
    description: "Especially valuable to archers, gunners, or other builds that pressure from range through repeated attacks or reload workflows.",
    family: "party_role",
    id: "equipment:ranged_striker_support",
    label: "ranged_striker_support",
    translationStatus: "mapped"
  },
  reconnaissance: {
    adjacentTags: [
      "scouting",
      "tracking",
      "anti_tracking"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "equipment",
    compositeOfAnyTags: [
      "scouting",
      "illumination",
      "surveillance_recording",
      "tracking",
      "anti_tracking"
    ],
    conceptId: "reconnaissance",
    currentTag: "reconnaissance",
    description: "Broad recon umbrella for equipment used to scout, illuminate, record evidence, track targets, or frustrate pursuit.",
    family: "reconnaissance",
    id: "equipment:reconnaissance",
    label: "reconnaissance",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  reload_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "reload_support",
    currentTag: "reload_support",
    description: "Especially valuable to ranged builds whose action flow depends on reloading efficiently or keeping ammunition ready.",
    family: "play_pattern",
    id: "equipment:reload_support",
    label: "reload_support",
    translationStatus: "mapped"
  },
  repair_support: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "repair_support",
    currentTag: "repair_support",
    description: "Supports item repair, patchwork, upkeep, or restoring damaged gear and structures.",
    family: "crafting_support",
    id: "equipment:repair_support",
    label: "repair_support",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  resolution: {
    adjacentTags: [
      "curse_removal",
      "ritual_appeasement",
      "source_revelation",
      "contamination_cleanup",
      "source_cleanup"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "equipment",
    compositeOfAnyTags: [
      "curse_removal",
      "sanctification",
      "ritual_appeasement",
      "source_revelation",
      "quarantine_containment",
      "contamination_cleanup",
      "source_cleanup"
    ],
    conceptId: "problem_resolution",
    currentTag: "resolution",
    description: "Broad resolution umbrella for equipment used to break curses, sanctify places, contain spread, clean contamination, or solve a problem at its source.",
    family: "resolution",
    id: "equipment:resolution",
    label: "problem_resolution",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  restraint_capture: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "restraint_capture",
    currentTag: "restraint_capture",
    description: "Helps capture, bind, or keep a target restrained.",
    family: "restraint",
    id: "equipment:restraint_capture",
    label: "restraint_capture",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  restraint_escape: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "restraint_escape",
    currentTag: "restraint_escape",
    description: "Helps break free from grabs, restraints, or similar immobilizing holds.",
    family: "restraint",
    id: "equipment:restraint_escape",
    label: "restraint_escape",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  ritual_appeasement: {
    adjacentTags: [
      "ritual_support",
      "sanctification"
    ],
    appliesWhen: [
      "The item's retrieval value comes from helping perform offerings, appeasement rites, restitution rituals, or ceremonial observance meant to settle a supernatural grievance.",
      "It is naturally sought for placation or ritual satisfaction rather than direct cleansing, banishment, or ordinary worship."
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "ritual_appeasement",
    currentTag: "ritual_appeasement",
    description: "Supports offerings, restitution, funerary observance, or appeasement ceremonies used to satisfy a spirit, haunt, curse, or sacred demand without directly expelling it.",
    doesNotApplyWhen: [
      "The item only supports broad ritual process with no real appeasement, offering, or restitution-facing role.",
      "The stronger fit is sanctification or ritual_support because the item purifies generally or supports any rite rather than a placation answer path."
    ],
    family: "resolution",
    id: "equipment:ritual_appeasement",
    label: "ritual_appeasement",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  ritual_support: {
    adjacentTags: [
      "alchemical_crafting",
      "magic_protection"
    ],
    appliesWhen: [
      "The item meaningfully supports ceremonial, circle-based, offering-based, or extended-casting magic work.",
      "A user would retrieve it for magic preparation rather than ordinary crafting or adventuring gear."
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "ritual_support",
    currentTag: "ritual_support",
    description: "Supports ritual casting, ceremonial setup, circles, offerings, or other extended magical preparation.",
    doesNotApplyWhen: [
      "The item is only generally magical without helping ritual process or setup.",
      "The item is mainly a focus of worship or symbolism rather than ritual procedure."
    ],
    family: "crafting_support",
    id: "equipment:ritual_support",
    label: "ritual_support",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  sanctification: {
    adjacentTags: [
      "ritual_support",
      "ritual_appeasement",
      "curse_removal"
    ],
    appliesWhen: [
      "The item's retrieval value comes from consecrating, hallowing, purifying, or spiritually cleansing a target or place.",
      "It is naturally sought as part of sacred-site cleanup, anti-haunt work, or ritual purification rather than general divine symbolism."
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "sacred_taint_sanctification",
    currentTag: "sanctification",
    description: "Supports hallowing, consecration, spiritual purification, or cleansing rites applied to a creature, object, or site.",
    doesNotApplyWhen: [
      "The item is only religious, ceremonial, or devotional without materially helping purification or consecration.",
      "The stronger fit is ritual_support because the item supports a broad rite rather than sanctification in particular."
    ],
    family: "resolution",
    id: "equipment:sanctification",
    label: "sanctification",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  scout_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "scout_support",
    currentTag: "scout_support",
    description: "Especially valuable to scouts, infiltrators, or advance-party play through quiet entry, recon, and information-gathering support.",
    family: "party_role",
    id: "equipment:scout_support",
    label: "scout_support",
    translationStatus: "mapped"
  },
  scouting: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "scouting",
    currentTag: "scouting",
    description: "Helps observe, survey, or reconnoiter an area.",
    family: "reconnaissance",
    id: "equipment:scouting",
    label: "scouting",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  scrying_protection: {
    adjacentTags: [
      "alarm",
      "concealment"
    ],
    appliesWhen: [
      "The item's main value is preventing magical spying, remote observation, or divination-led tracking.",
      "A user would retrieve it to keep plans, rooms, or identities hidden from magical surveillance."
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "scrying_protection",
    currentTag: "scrying_protection",
    description: "Blocks magical observation, remote viewing, or information leakage through divination-like effects.",
    doesNotApplyWhen: [
      "The item only improves ordinary stealth or concealment without anti-divination protection.",
      "The item counters magic generally but is not particularly about observation or information leakage."
    ],
    family: "security",
    id: "equipment:scrying_protection",
    label: "scrying_protection",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  security: {
    adjacentTags: [
      "alarm",
      "scrying_protection",
      "tamper_evidence"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "equipment",
    compositeOfAnyTags: [
      "alarm",
      "scrying_protection",
      "tamper_evidence"
    ],
    conceptId: "security",
    currentTag: "security",
    description: "Broad security umbrella for gear that warns about intrusion, blocks magical spying, or reveals after-the-fact interference.",
    family: "security",
    id: "equipment:security",
    label: "security",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  sedation: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "sedation",
    currentTag: "sedation",
    description: "Induces sleep, lethargy, unconsciousness, or similar incapacitating drowsiness.",
    family: "impact",
    id: "equipment:sedation",
    label: "sedation",
    subcategories: [
      "ammo",
      "consumable"
    ],
    translationStatus: "mapped"
  },
  self_buff: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "self_buff",
    currentTag: "self_buff",
    description: "Support consumable primarily applied to the user.",
    family: "consumable_role",
    id: "equipment:self_buff",
    label: "self_buff",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  senses_support: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "senses_support",
    currentTag: "senses_support",
    description: "Improves vision or other senses.",
    family: "function",
    id: "equipment:senses_support",
    label: "senses_support",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  sensory_impairment: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "sensory_impairment",
    currentTag: "sensory_impairment",
    description: "Impairs sight, hearing, or other senses through blinding, dazzling, or deafening effects.",
    family: "impact",
    id: "equipment:sensory_impairment",
    label: "sensory_impairment",
    subcategories: [
      "ammo",
      "consumable"
    ],
    translationStatus: "mapped"
  },
  shield_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "shield_support",
    currentTag: "shield_support",
    description: "Especially valuable to shield-forward play through improved blocking, readiness, or defensive shield workflow.",
    family: "play_pattern",
    id: "equipment:shield_support",
    label: "shield_support",
    translationStatus: "mapped"
  },
  siege_support: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "siege_support",
    currentTag: "siege_support",
    description: "Supports attacking gates, fortifications, vehicles, or other larger hardened targets.",
    family: "breaching",
    id: "equipment:siege_support",
    label: "siege_support",
    subcategories: [
      "gear",
      "kit",
      "consumable",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  signaling: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "signaling",
    currentTag: "signaling",
    description: "Helps draw attention, mark a location, or coordinate allies.",
    family: "communication",
    id: "equipment:signaling",
    label: "signaling",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  silencing: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "silencing",
    currentTag: "silencing",
    description: "Suppresses speech, voice, or other sound-dependent action through gagging, muting, or numbing effects.",
    family: "impact",
    id: "equipment:silencing",
    label: "silencing",
    subcategories: [
      "ammo",
      "consumable"
    ],
    translationStatus: "mapped"
  },
  skirmisher_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "skirmisher_support",
    currentTag: "skirmisher_support",
    description: "Especially valuable to mobile flankers, hit-and-run melee characters, or other skirmisher-style builds.",
    family: "party_role",
    id: "equipment:skirmisher_support",
    label: "skirmisher_support",
    translationStatus: "mapped"
  },
  social_infiltration: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "social_infiltration",
    currentTag: "social_infiltration",
    description: "Helps blend into a group or pass under social scrutiny.",
    family: "infiltration",
    id: "equipment:social_infiltration",
    label: "social_infiltration",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  source_cleanup: {
    adjacentTags: [
      "source_revelation",
      "contamination_cleanup",
      "ritual_support"
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "source_cleanup",
    currentTag: "source_cleanup",
    description: "Helps find, remove, neutralize, or safely dispose of the cursed object, infected material, corrupted remains, or other source driving the problem.",
    family: "resolution",
    id: "equipment:source_cleanup",
    label: "source_cleanup",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  source_revelation: {
    adjacentTags: [
      "source_cleanup",
      "contamination_cleanup"
    ],
    appliesWhen: [
      "The item's retrieval value comes from finding or confirming the hidden source of a curse, contamination, outbreak, or spiritually tainted problem.",
      "It is naturally sought for tracing the origin or anchor rather than directly cleansing or disposing of it."
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "source_discovery",
    currentTag: "source_revelation",
    description: "Helps identify the cursed anchor, contaminated material, infected origin, hidden carrier, or other source driving the problem before cleanup begins.",
    doesNotApplyWhen: [
      "The item only helps perform a cleanup, disposal, or purification step after the source is already known.",
      "The stronger fit is medical_support, tracking, or ritual_support because the item supports a broader process without specifically revealing the source."
    ],
    family: "resolution",
    id: "equipment:source_revelation",
    label: "source_discovery",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  spell_payload: {
    assignmentMode: "deterministic",
    axis: "item_mechanical",
    category: "equipment",
    conceptId: "spell_payload",
    currentTag: "spell_payload",
    description: "Ammunition that delivers, casts, or imposes a spell effect on hit.",
    family: "ammunition_payload",
    id: "equipment:spell_payload",
    label: "spell_payload",
    subcategories: [
      "ammo"
    ],
    translationStatus: "mapped"
  },
  stealth_entry_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "stealth_entry_support",
    currentTag: "stealth_entry_support",
    description: "Especially valuable to quiet-entry, infiltration, burglary, or reconnaissance play that depends on avoiding notice and solving access problems discreetly.",
    family: "play_pattern",
    id: "equipment:stealth_entry_support",
    label: "stealth_entry_support",
    translationStatus: "mapped"
  },
  stealth_support: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "stealth_support",
    currentTag: "stealth_support",
    description: "Helps move quietly, avoid notice, muffle noise, or otherwise support covert entry and low-profile movement.",
    family: "infiltration",
    id: "equipment:stealth_support",
    label: "stealth_support",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  surveillance_recording: {
    adjacentTags: [
      "scouting",
      "tamper_evidence"
    ],
    appliesWhen: [
      "The item's retrieval value comes from preserving sights, sounds, or observations for later replay, proof, or analysis.",
      "It is naturally sought as evidence capture, remote monitoring, or watch-post support rather than live conversation gear."
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "surveillance_recording",
    currentTag: "surveillance_recording",
    description: "Captures, stores, or replays images, sound, or other evidence for later review.",
    doesNotApplyWhen: [
      "The item only sends a live message or helps coordinate allies without retaining evidence.",
      "The item protects against observation instead of performing it."
    ],
    family: "reconnaissance",
    id: "equipment:surveillance_recording",
    label: "surveillance_recording",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  survival: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "survival",
    currentTag: "survival",
    description: "Supports wilderness travel, shelter, or long-term field use.",
    family: "expedition",
    id: "equipment:survival",
    label: "survival",
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
    translationStatus: "provisional"
  },
  sustenance: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "sustenance",
    currentTag: "sustenance",
    description: "Provides food, feed, water, or other practical nourishment for travel and survival.",
    family: "expedition",
    id: "equipment:sustenance",
    label: "sustenance",
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
    translationStatus: "provisional"
  },
  tamper_evidence: {
    adjacentTags: [
      "alarm",
      "surveillance_recording"
    ],
    appliesWhen: [
      "The item is naturally retrieved to reveal whether a lock, crate, seal, letter, cache, or room has been disturbed.",
      "Evidence of interference matters more than immediate warning, direct defense, or anti-scrying."
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "tamper_evidence",
    currentTag: "tamper_evidence",
    description: "Makes intrusion, opening, theft, or interference easier to notice after the fact.",
    doesNotApplyWhen: [
      "The item mainly alerts in real time when someone crosses a boundary.",
      "The item hides or protects a target without preserving signs of intrusion."
    ],
    family: "security",
    id: "equipment:tamper_evidence",
    label: "tamper_evidence",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  telepathic_communication: {
    adjacentTags: [
      "signaling",
      "message_delivery"
    ],
    appliesWhen: [
      "The item's retrieval value comes from silent psychic coordination, mind-to-mind speech, or communication that bypasses ordinary sound.",
      "It is naturally sought when stealth, silence, distance, or noise would make spoken coordination unreliable."
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "telepathic_communication",
    currentTag: "telepathic_communication",
    description: "Enables silent mind-to-mind coordination, psychic speech, or communication that bypasses normal hearing.",
    doesNotApplyWhen: [
      "The item only boosts ordinary signaling, writing, or message relay without true mind-to-mind communication.",
      "The item is mainly about surveillance or recording rather than live coordination."
    ],
    family: "communication",
    id: "equipment:telepathic_communication",
    label: "telepathic_communication",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  thrown_offense: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "thrown_offense",
    currentTag: "thrown_offense",
    description: "Offensive consumable delivered by throwing it.",
    family: "delivery_profile",
    id: "equipment:thrown_offense",
    label: "thrown_offense",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  thrown_weapon_support: {
    assignmentMode: "hybrid",
    axis: "party_role",
    category: "equipment",
    conceptId: "thrown_weapon_support",
    currentTag: "thrown_weapon_support",
    description: "Especially valuable to builds that solve problems through thrown weapons, quick draws, or repeatable thrown-item pressure.",
    family: "play_pattern",
    id: "equipment:thrown_weapon_support",
    label: "thrown_weapon_support",
    translationStatus: "mapped"
  },
  tracking: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "tracking",
    currentTag: "tracking",
    description: "Helps follow trails, mark a target, or relocate something later.",
    family: "reconnaissance",
    id: "equipment:tracking",
    label: "tracking",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  translation_support: {
    adjacentTags: [
      "telepathic_communication",
      "message_delivery"
    ],
    appliesWhen: [
      "The item's retrieval value comes from understanding foreign languages, translating speech, or decoding otherwise unreadable text or symbols.",
      "It is naturally sought when communication fails because of language barriers rather than distance or secrecy."
    ],
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "translation_support",
    currentTag: "translation_support",
    description: "Bridges language barriers through translation, deciphering, script interpretation, or speech-understanding aids.",
    doesNotApplyWhen: [
      "The item only stores, relays, or broadcasts messages without solving comprehension.",
      "The item only provides psychic communication between already understood participants."
    ],
    family: "communication",
    id: "equipment:translation_support",
    label: "translation_support",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ],
    translationStatus: "provisional"
  },
  transport: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "transport",
    currentTag: "transport",
    description: "Helps move creatures or cargo from place to place.",
    family: "movement_traversal",
    id: "equipment:transport",
    label: "transport",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  trap_bypass: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "trap_bypass",
    currentTag: "trap_bypass",
    description: "Helps disarm, disable, or get past traps.",
    family: "access_bypass",
    id: "equipment:trap_bypass",
    label: "trap_bypass",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ],
    translationStatus: "provisional"
  },
  weapon_applied: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "equipment",
    conceptId: "weapon_applied",
    currentTag: "weapon_applied",
    description: "Offensive consumable applied to a weapon before use.",
    family: "delivery_profile",
    id: "equipment:weapon_applied",
    label: "weapon_applied",
    subcategories: [
      "consumable"
    ],
    translationStatus: "mapped"
  },
  weapon_staging: {
    assignmentMode: "deterministic",
    axis: "item_mechanical",
    category: "equipment",
    conceptId: "weapon_staging",
    currentTag: "weapon_staging",
    description: "Holsters, sheaths, scabbards, or bandoliers that stage weapons for quick draw or organized carry.",
    family: "access_system",
    id: "equipment:weapon_staging",
    label: "weapon_staging",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "ammo",
      "armor",
      "weapon"
    ],
    translationStatus: "mapped"
  },
  writing_recordkeeping: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "equipment",
    conceptId: "writing_recordkeeping",
    currentTag: "writing_recordkeeping",
    description: "Supports note-taking, mapmaking, copying text, archival work, or durable information storage.",
    family: "crafting_support",
    id: "equipment:writing_recordkeeping",
    label: "writing_recordkeeping",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ],
    translationStatus: "provisional"
  }
};
