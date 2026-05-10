import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const DescriptiveCapabilityProjectionDeclarations = [
  defineConceptProjections("action_economy_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable because it compresses setup, speeds access, or meaningfully improves in-combat action efficiency.",
    },
  }),
  defineConceptProjections("alarm", {
    equipment: {
      axis: "utility",
      family: "security",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Alerts you or others when a watched area, threshold, or device is triggered.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Alerts guardians, onlookers, or nearby creatures to an intrusion.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "security",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Alerts you or others when a watched area, threshold, or ward is crossed.",
      appliesWhen: [
        "The spell is naturally retrieved to warn about intrusion, threshold crossing, tampering, or unwanted entry.",
        "Detection and notice matter more than directly stopping the intruder.",
      ],
      doesNotApplyWhen: [
        "The spell mainly protects, blocks, or hides the target without providing a warning function.",
        "The spell only reveals truth or magic generally rather than guarding a watched perimeter.",
      ],
      adjacentTags: ["protective_ward", "scrying_protection"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("alchemical_crafting", {
    equipment: {
      axis: "utility",
      family: "crafting_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports alchemical preparation, formula work, reagent handling, or crafting-related field setup.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("ally_cover", {
    equipment: {
      axis: "item_mechanical",
      family: "defense_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Provides cover or upgraded cover to nearby allies.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "shield"],
    },
  }),
  defineConceptProjections("ally_support", {
    equipment: {
      axis: "effect",
      family: "consumable_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Support consumable that can directly benefit another creature.",
      subcategories: ["consumable"],
    },
  }),
  defineConceptProjections("ammo_management", {
    equipment: {
      axis: "item_mechanical",
      family: "access_system",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Magazines or related gear that manage repeating-weapon ammunition or reload workflow.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "ammo", "armor", "weapon"],
    },
  }),
  defineConceptProjections("anti_tracking", {
    equipment: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps hide your trail, mask scent, or make pursuit harder.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("aquatic_support", {
    equipment: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps with swimming, underwater breathing, flotation, water-surface travel, or watercraft use.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "shield", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps with swimming, underwater breathing, water-surface travel, or other aquatic movement.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("arcane_spellcaster", {
    creature: {
      axis: "specialization",
      family: "casting_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creature whose spellcasting is substantially framed through arcane traditions, wizardry, runes, or similarly arcane technique.",
    },
  }),
  defineConceptProjections("camp_setup", {
    equipment: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports campsite creation, resting infrastructure, shelter setup, or extended overland staging.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "shield", "weapon"],
      appliesWhen: [
        "The item is naturally retrieved for making camp, resting outdoors, or supporting expedition downtime.",
        "Its value is in field habitation rather than only carrying gear or feeding travelers.",
      ],
      doesNotApplyWhen: [
        "The item only provides sustenance, mobility, or transport without campsite infrastructure.",
        "The item is merely general survival gear with no setup or shelter-facing role.",
      ],
      adjacentTags: ["sustenance", "carry_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("carry_support", {
    equipment: {
      axis: "utility",
      family: "carry_logistics",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps stow, carry, or organize equipment.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("caster_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to spellcasters for magical prep, casting reliability, spell defense, or spell-adjacent utility.",
    },
  }),
  defineConceptProjections("climbing", {
    equipment: {
      axis: "utility",
      family: "movement_traversal",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps climb, rappel, or navigate vertical obstacles.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("combat_maneuver_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to builds that rely on grappling, tripping, shoving, disarming, or other combat maneuvers to control enemies.",
    },
  }),
  defineConceptProjections("companion_handling_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable when a character's tactical workflow depends on commanding, outfitting, transporting, or protecting companions and mounts.",
    },
  }),
  defineConceptProjections("companion_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to handlers of animal companions, mounts, familiars, or other creature-side support play patterns.",
    },
  }),
  defineConceptProjections("concealable", {
    equipment: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Easy to hide on the person or carry discreetly.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("concealment", {
    equipment: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps obscure a creature, item, or area from sight or make it harder to perceive.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Makes a creature hard to see, hidden, concealed, or undetected.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("creature_bane", {
    equipment: {
      axis: "item_mechanical",
      family: "ammunition_payload",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Tailored ammunition for a selected creature type or trait.",
      subcategories: ["ammo"],
    },
  }),
  defineConceptProjections("defender_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to shield users, bodyguards, line-holders, or other defender-style characters.",
    },
  }),
  defineConceptProjections("disguise", {
    equipment: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps alter appearance or impersonate another identity.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps alter appearance or impersonate another identity.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("divine_spellcaster", {
    creature: {
      axis: "specialization",
      family: "casting_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creature whose spellcasting is substantially framed through divine prayer, sacred miracles, or deity-facing magic.",
    },
  }),
  defineConceptProjections("dragon_spellcaster", {
    creature: {
      axis: "specialization",
      family: "casting_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Dragon or archdragon variant with an explicit spellcaster stat block or named spellcaster presentation.",
      appliesWhen: [
        "Use when a dragon or archdragon variant explicitly presents meaningful spellcasting as part of its encounter identity.",
        "The spellcasting matters for prep and counterplay beyond incidental magical flavor.",
      ],
      doesNotApplyWhen: [
        "The dragon only has innate magical flavor, one-off magical actions, or a few utility effects without real spellcaster framing.",
        "The stronger fit is only a tradition-specific spellcaster tag without a dragon-specific spellcaster presentation.",
      ],
      adjacentTags: ["arcane_spellcaster", "ritualist_creature"],
    },
  }),
  defineConceptProjections("emergency_recovery", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable as a panic-button item for sudden healing, escape, stabilization, or critical condition rescue.",
    },
  }),
  defineConceptProjections("energy_resistance", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Grants resistance against one or more energy types.",
      subcategories: ["consumable"],
    },
  }),
  defineConceptProjections("environmental_adaptation", {
    equipment: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Helps travelers endure extreme weather, thin air, smoke, pressure, or other dangerous environmental exposure.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "shield", "weapon"],
      appliesWhen: [
        "The item's retrieval value comes from surviving punishing climate, altitude, breathing hazards, immersion pressure, or similar expedition environments.",
        "It is naturally sought as environmental survival gear rather than a general defense item or campsite tool.",
      ],
      doesNotApplyWhen: [
        "The item only protects against one incoming attack or hazard burst without broader travel-survival use.",
        "The item mainly creates camp infrastructure, carries provisions, or improves aquatic movement instead of adapting the user to the environment.",
      ],
      adjacentTags: ["aquatic_support", "camp_setup", "hazard_shielding"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Helps creatures endure hostile climates, thin air, smoke, pressure, vacuum, or other expedition-grade environmental extremes.",
      appliesWhen: [
        "The spell is naturally retrieved to survive extreme heat, cold, altitude, smoke, pressure, or other punishing environmental conditions during travel or exploration.",
        "Environmental endurance matters more than only resisting one attack form or creating a place to rest.",
      ],
      doesNotApplyWhen: [
        "The spell mainly grants combat resistance, a protective ward, or aquatic mobility without broader expedition-survival value.",
        "The spell only creates shelter or sustenance rather than adapting creatures to the surrounding environment.",
      ],
      adjacentTags: ["aquatic_support", "field_shelter", "resistance_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("extraction_teleport", {
    spell: {
      axis: "utility",
      family: "teleportation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Teleports a creature out of danger, through restraints, or away from immediate threat pressure.",
      appliesWhen: [
        "The spell is naturally retrieved as an escape, rescue, or anti-capture tool rather than only a movement spell.",
        "The reposition breaks danger, confinement, or immediate battlefield pressure.",
      ],
      doesNotApplyWhen: [
        "The spell is mostly a neutral short-range blink or a long-distance travel effect.",
        "The spell primarily opens planar movement rather than emergency extraction.",
      ],
      adjacentTags: ["short_range_teleport", "escape_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("extradimensional_storage", {
    equipment: {
      axis: "item_mechanical",
      family: "access_system",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Provides bag-of-holding-style storage through extradimensional or magically expanded space.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "ammo", "armor", "weapon"],
    },
  }),
  defineConceptProjections("face_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to negotiators, deceivers, diplomats, or other socially forward characters who need influence, disguise, or presentation support.",
    },
  }),
  defineConceptProjections("fall_protection", {
    equipment: {
      axis: "item_mechanical",
      family: "defense_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Reduces falling harm, cushions impact, or protects against vertical movement accidents and collapse.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "shield"],
    },
  }),
  defineConceptProjections("field_shelter", {
    spell: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creates shelter, refuge, or a protected resting place in the field.",
      appliesWhen: [
        "The spell is naturally retrieved to create a campsite refuge, safe resting place, or expedition shelter in hostile territory.",
        "Its value is prolonged field habitation or protected rest rather than momentary combat defense.",
      ],
      doesNotApplyWhen: [
        "The spell only creates a brief combat ward, cover effect, or instant defensive barrier.",
        "The spell merely transports creatures away instead of establishing a place to rest.",
      ],
      adjacentTags: ["protective_ward", "planar_travel"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("flight", {
    spell: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Grants flying movement, sustained aerial travel, or practical airborne maneuvering.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("focus_magic_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to focus-spell-heavy or magic-routine-driven builds that want more reliable magical cadence and recovery.",
    },
  }),
  defineConceptProjections("forgery_support", {
    equipment: {
      axis: "utility",
      family: "crafting_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports document falsification, seal imitation, signature copying, or bureaucratic deception.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      appliesWhen: [
        "The item's retrieval value comes from imitating documents, seals, credentials, or official paperwork.",
        "It supports passing administrative scrutiny rather than just changing clothing or physical appearance.",
      ],
      doesNotApplyWhen: [
        "The item only changes appearance or supports social disguise without document work.",
        "The item is a normal writing or archival tool with no deception-facing use.",
      ],
      adjacentTags: ["disguise", "writing_recordkeeping"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("hazard_shielding", {
    equipment: {
      axis: "item_mechanical",
      family: "defense_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Protects against environmental hazards, area effects, or other damaging exposures.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "shield"],
      appliesWhen: [
        "The item is naturally retrieved for surviving traps, breath weapons, alchemical blasts, or other hazardous exposure.",
        "Protection against scenes and effects matters more than only deflecting direct weapon attacks.",
      ],
      doesNotApplyWhen: [
        "The item's main value is cover against arrows or weapon strikes rather than wider hazard exposure.",
        "The item mainly protects against spells specifically, making magic_protection the stronger hook.",
      ],
      adjacentTags: ["projectile_defense", "magic_protection"],
    },
  }),
  defineConceptProjections("healer_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to medics, battlefield healers, or builds expected to stabilize, treat, or recover allies under pressure.",
    },
  }),
  defineConceptProjections("illumination", {
    equipment: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Produces or improves light in dark environments.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "sensory_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Produces practical light that brightens darkness, reveals an area, or lets creatures see more clearly.",
      adjacentTags: ["senses_support", "line_of_sight_control"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("long_range_teleport", {
    spell: {
      axis: "utility",
      family: "teleportation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Teleports creatures across major overland distances, settlements, or remote destinations.",
      appliesWhen: [
        "The spell is naturally retrieved for strategic travel, relocation, or bypassing long routes.",
        "The destination scale is substantially larger than one encounter map or immediate tactical space.",
      ],
      doesNotApplyWhen: [
        "The spell is mainly a tactical blink, extraction, or planar crossing effect.",
        "The spell only repositions creatures within the same immediate scene.",
      ],
      adjacentTags: ["short_range_teleport", "planar_travel"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("lore_consultation", {
    spell: {
      axis: "utility",
      family: "consultation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Provides interpretive insight, shared knowledge, or focused understanding about a subject, clue, history, or magical situation.",
      adjacentTags: ["truth_reveal", "problem_diagnosis"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("medical_support", {
    equipment: {
      axis: "utility",
      family: "crafting_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports first aid, diagnosis, treatment, or ongoing medical care outside direct magical healing.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("message_delivery", {
    equipment: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Sends, stores, or relays actual content across time or distance.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Sends, stores, or relays actual content across time or distance.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("mobility", {
    equipment: {
      axis: "utility",
      family: "movement_traversal",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Improves movement or traversal flexibility.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Helps move faster, gain movement modes, or traverse terrain more effectively.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("mounted_support", {
    equipment: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports mounted combat, rider control, saddle use, or mount-specific loadouts.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "shield", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("navigation", {
    equipment: {
      axis: "utility",
      family: "movement_traversal",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps track direction, route, or position.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Helps orient, guide a route, or identify a destination's direction.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("occult_spellcaster", {
    creature: {
      axis: "specialization",
      family: "casting_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creature whose spellcasting is substantially framed through occult lore, spirits, emotion, dreams, or esoteric mental power.",
    },
  }),
  defineConceptProjections("offensive", {
    equipment: {
      axis: "effect",
      family: "consumable_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hostile consumable primarily meant to harm or debilitate a target.",
      subcategories: ["consumable"],
    },
  }),
  defineConceptProjections("omen_guidance", {
    spell: {
      axis: "utility",
      family: "consultation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Asks for omens, directional guidance, or advisory insight about the best course of action, likely outcome, or strategic choice.",
      adjacentTags: ["lore_consultation", "wayfinding"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("planar_travel", {
    spell: {
      axis: "utility",
      family: "teleportation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Moves creatures between planes, through planar routes, or into extraplanar destinations.",
      appliesWhen: [
        "Crossing into another plane, demiplane, or extraplanar route is central to the spell's retrieval value.",
        "The spell is naturally retrieved for cosmological travel rather than mundane relocation.",
      ],
      doesNotApplyWhen: [
        "The spell only teleports within the same plane or functions as a normal long-distance travel tool.",
        "Extradimensional storage or shelter is present without real plane-crossing travel.",
      ],
      adjacentTags: ["long_range_teleport", "field_shelter"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("primal_spellcaster", {
    creature: {
      axis: "specialization",
      family: "casting_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creature whose spellcasting is substantially framed through nature, elemental power, druidic force, or instinctive primal magic.",
    },
  }),
  defineConceptProjections("problem_diagnosis", {
    spell: {
      axis: "utility",
      family: "consultation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Helps determine what hidden magical, spiritual, cursed, or otherwise obscure problem is actually affecting a target, site, or situation.",
      adjacentTags: ["curse_revelation", "magic_detection"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("projectile_defense", {
    equipment: {
      axis: "item_mechanical",
      family: "defense_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Intercepts, redirects, or absorbs ranged attacks and projectiles.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "shield"],
    },
  }),
  defineConceptProjections("ranged_striker_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to archers, gunners, or other builds that pressure from range through repeated attacks or reload workflows.",
    },
  }),
  defineConceptProjections("reload_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to ranged builds whose action flow depends on reloading efficiently or keeping ammunition ready.",
    },
  }),
  defineConceptProjections("repair_support", {
    equipment: {
      axis: "utility",
      family: "crafting_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports item repair, patchwork, upkeep, or restoring damaged gear and structures.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("ritual_support", {
    equipment: {
      axis: "utility",
      family: "crafting_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Supports ritual casting, ceremonial setup, circles, offerings, or other extended magical preparation.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      appliesWhen: [
        "The item meaningfully supports ceremonial, circle-based, offering-based, or extended-casting magic work.",
        "A user would retrieve it for magic preparation rather than ordinary crafting or adventuring gear.",
      ],
      doesNotApplyWhen: [
        "The item is only generally magical without helping ritual process or setup.",
        "The item is mainly a focus of worship or symbolism rather than ritual procedure.",
      ],
      adjacentTags: ["alchemical_crafting", "magic_protection"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("ritualist_creature", {
    creature: {
      axis: "specialization",
      family: "casting_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creature strongly associated with ritual casting, ceremonial magic, or extended occult or divine preparations.",
      appliesWhen: [
        "Use when ceremonial, circle-based, sacrificial, or extended-casting magic is a major reason to retrieve the creature.",
        "The creature is naturally used as a ritual leader, ritual threat, or ritual-supporting encounter element.",
      ],
      doesNotApplyWhen: [
        "The creature merely casts normal encounter spells without a meaningful ritual identity.",
        "The stronger fit is a tradition spellcaster tag and ritual work is only incidental flavor.",
      ],
      adjacentTags: ["divine_spellcaster", "occult_spellcaster"],
    },
  }),
  defineConceptProjections("scout_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to scouts, infiltrators, or advance-party play through quiet entry, recon, and information-gathering support.",
    },
  }),
  defineConceptProjections("scouting", {
    equipment: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps observe, survey, or reconnoiter an area.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Helps observe at a distance, extend senses, or locate a target.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("scrying_protection", {
    equipment: {
      axis: "utility",
      family: "security",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Blocks magical observation, remote viewing, or information leakage through divination-like effects.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      appliesWhen: [
        "The item's main value is preventing magical spying, remote observation, or divination-led tracking.",
        "A user would retrieve it to keep plans, rooms, or identities hidden from magical surveillance.",
      ],
      doesNotApplyWhen: [
        "The item only improves ordinary stealth or concealment without anti-divination protection.",
        "The item counters magic generally but is not particularly about observation or information leakage.",
      ],
      adjacentTags: ["alarm", "concealment"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "security",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Blocks magical observation, remote viewing, divinatory surveillance, or other information leakage from a protected target or space.",
      appliesWhen: [
        "The spell is naturally retrieved to keep plans, sanctums, identities, or conversations hidden from magical spying.",
        "Its core value is denying observation or divination rather than only raising an intrusion alarm.",
      ],
      doesNotApplyWhen: [
        "The spell only improves mundane concealment or silence without real anti-divination protection.",
        "The spell counters magic broadly but is not specifically about surveillance or remote observation.",
      ],
      adjacentTags: ["alarm", "countermagic"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("self_buff", {
    equipment: {
      axis: "effect",
      family: "consumable_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Support consumable primarily applied to the user.",
      subcategories: ["consumable"],
    },
  }),
  defineConceptProjections("senses_support", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Improves vision or other senses.",
      subcategories: ["consumable"],
    },
    spell: {
      axis: "utility",
      family: "sensory_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Enhances vision or other senses through darkvision, see invisible, sharpened perception, scent, or similar perceptual upgrades.",
      adjacentTags: ["scouting", "invisibility_reveal"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("shield_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to shield-forward play through improved blocking, readiness, or defensive shield workflow.",
    },
  }),
  defineConceptProjections("short_range_teleport", {
    spell: {
      axis: "utility",
      family: "teleportation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Teleports a creature across a short tactical distance, usually within the same scene or encounter area.",
      appliesWhen: [
        "The spell repositions a creature within the current encounter or scene rather than serving as expedition travel.",
        "The tactical blink or reposition is itself a major reason to retrieve the spell.",
      ],
      doesNotApplyWhen: [
        "The spell is mainly about escaping custody, extracting allies, or long-distance transport.",
        "The spell primarily crosses planes or major overland distances.",
      ],
      adjacentTags: ["extraction_teleport", "long_range_teleport"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("signaling", {
    equipment: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps draw attention, mark a location, or coordinate allies.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps draw attention, mark a location, or coordinate allies.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("skirmisher_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to mobile flankers, hit-and-run melee characters, or other skirmisher-style builds.",
    },
  }),
  defineConceptProjections("social_infiltration", {
    equipment: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps blend into a group or pass under social scrutiny.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps blend into a group or pass under social scrutiny.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("stealth_entry_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to quiet-entry, infiltration, burglary, or reconnaissance play that depends on avoiding notice and solving access problems discreetly.",
    },
  }),
  defineConceptProjections("stealth_support", {
    equipment: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Helps move quietly, avoid notice, muffle noise, or otherwise support covert entry and low-profile movement.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Helps move quietly, avoid notice, suppress noisy presence, or otherwise support covert entry and low-profile movement.",
      appliesWhen: [
        "The spell is naturally retrieved to help a creature move quietly, avoid notice, pass unseen, or keep a covert approach from drawing attention.",
        "The retrieval hook is quiet entry or low-profile movement rather than only broad battlefield obscurity.",
      ],
      doesNotApplyWhen: [
        "The spell only creates a combat concealment effect or visual obstruction without really supporting a covert approach.",
        "The spell changes appearance or social presentation without materially helping the target move unnoticed.",
      ],
      adjacentTags: ["concealment", "silencing"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("surveillance_recording", {
    equipment: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Captures, stores, or replays images, sound, or other evidence for later review.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      appliesWhen: [
        "The item's retrieval value comes from preserving sights, sounds, or observations for later replay, proof, or analysis.",
        "It is naturally sought as evidence capture, remote monitoring, or watch-post support rather than live conversation gear.",
      ],
      doesNotApplyWhen: [
        "The item only sends a live message or helps coordinate allies without retaining evidence.",
        "The item protects against observation instead of performing it.",
      ],
      adjacentTags: ["scouting", "tamper_evidence"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("survival", {
    equipment: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports wilderness travel, shelter, or long-term field use.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "shield", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("sustenance", {
    equipment: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Provides food, feed, water, or other practical nourishment for travel and survival.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "shield", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Provides food, water, rations, or practical nourishment for travel and survival.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("tamper_evidence", {
    equipment: {
      axis: "utility",
      family: "security",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Makes intrusion, opening, theft, or interference easier to notice after the fact.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      appliesWhen: [
        "The item is naturally retrieved to reveal whether a lock, crate, seal, letter, cache, or room has been disturbed.",
        "Evidence of interference matters more than immediate warning, direct defense, or anti-scrying.",
      ],
      doesNotApplyWhen: [
        "The item mainly alerts in real time when someone crosses a boundary.",
        "The item hides or protects a target without preserving signs of intrusion.",
      ],
      adjacentTags: ["alarm", "surveillance_recording"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("telepathic_communication", {
    equipment: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Enables silent mind-to-mind coordination, psychic speech, or communication that bypasses normal hearing.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      appliesWhen: [
        "The item's retrieval value comes from silent psychic coordination, mind-to-mind speech, or communication that bypasses ordinary sound.",
        "It is naturally sought when stealth, silence, distance, or noise would make spoken coordination unreliable.",
      ],
      doesNotApplyWhen: [
        "The item only boosts ordinary signaling, writing, or message relay without true mind-to-mind communication.",
        "The item is mainly about surveillance or recording rather than live coordination.",
      ],
      adjacentTags: ["signaling", "message_delivery"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creates direct mind-to-mind communication, silent tactical coordination, or psychic speech between creatures.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("thrown_weapon_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to builds that solve problems through thrown weapons, quick draws, or repeatable thrown-item pressure.",
    },
  }),
  defineConceptProjections("tracking", {
    equipment: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps follow trails, mark a target, or relocate something later.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Locates a specific creature, object, or destination, or follows a supernatural trail toward it.",
      appliesWhen: [
        "The spell is naturally retrieved to find a named target, trace a quarry, or point the caster toward a specific creature, object, or place.",
        "Target location matters more than broad sensory surveillance or general route guidance.",
      ],
      doesNotApplyWhen: [
        "The spell mainly reveals an area, extends senses, or scouts without locking onto a specific target.",
        "The spell only helps orient a journey or choose a route once the destination is already known.",
      ],
      adjacentTags: ["scouting", "navigation"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("translation_support", {
    equipment: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Bridges language barriers through translation, deciphering, script interpretation, or speech-understanding aids.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      appliesWhen: [
        "The item's retrieval value comes from understanding foreign languages, translating speech, or decoding otherwise unreadable text or symbols.",
        "It is naturally sought when communication fails because of language barriers rather than distance or secrecy.",
      ],
      doesNotApplyWhen: [
        "The item only stores, relays, or broadcasts messages without solving comprehension.",
        "The item only provides psychic communication between already understood participants.",
      ],
      adjacentTags: ["telepathic_communication", "message_delivery"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Bridges spoken or written language barriers through translation, comprehension, deciphering, or magically shared understanding.",
      appliesWhen: [
        "The spell is naturally retrieved to understand, translate, or make oneself understood across otherwise incompatible languages or scripts.",
        "Language access matters more than merely sending a message or speaking silently.",
      ],
      doesNotApplyWhen: [
        "The spell only transmits content farther or more privately without solving a language barrier.",
        "The spell reveals truth, thoughts, or memories without actually translating speech or writing.",
      ],
      adjacentTags: ["telepathic_communication", "message_delivery"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("transport", {
    equipment: {
      axis: "utility",
      family: "movement_traversal",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps move creatures or cargo from place to place.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("weapon_staging", {
    equipment: {
      axis: "item_mechanical",
      family: "access_system",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Holsters, sheaths, scabbards, or bandoliers that stage weapons for quick draw or organized carry.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "ammo", "armor", "weapon"],
    },
  }),
  defineConceptProjections("writing_recordkeeping", {
    equipment: {
      axis: "utility",
      family: "crafting_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports note-taking, mapmaking, copying text, archival work, or durable information storage.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
