import { buildCanonicalConceptMap, type CanonicalConceptSeed } from "../builders.js";
import { CANONICAL_VOCABULARY } from "../vocabulary.js";

const DERIVED_TAG_DESCRIPTIVE_CANONICAL_CONCEPTS_BY_ID_SEEDS: Record<string, CanonicalConceptSeed> = {
  abaddon_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  abyss_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  action_denial: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  action_economy_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  alarm: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  alchemical_crafting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  alien_technology_wasteland_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.REGIONAL,
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  ally_cover: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  ally_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  ambush_burst: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.FUNCTION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAZARD_FUNCTION,
  },
  ambusher_combatant: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.TACTICAL,
  },
  ammo_management: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  ancestral_legacy: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  animated_object: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CREATURE_FAMILY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.BOUND_OBJECT_FAMILY,
  },
  animated_statue: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CREATURE_FAMILY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.BOUND_OBJECT_FAMILY,
  },
  anti_tracking: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  apocalypse_ruin: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  aquatic_hazard: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  aquatic_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  aquatic_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  arcane_spellcaster: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CASTING_PROFILE,
  },
  arctic_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  area_denial: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.FUNCTION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAZARD_FUNCTION,
  },
  artillery_combatant: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.TACTICAL,
  },
  artisan_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  astral_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  attrition_pressure: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.FUNCTION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAZARD_FUNCTION,
  },
  authority_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  axis_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  barrier_lockdown: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.FUNCTION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAZARD_FUNCTION,
  },
  battlefield_disruption: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAUNT_MANIFESTATION,
  },
  battlefield_hazard: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  battlefield_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  bestial_transformation: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.PATHOGENESIS,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PATHOGENESIS,
    text: {
      definition: "Disease-pattern descriptors.",
    },
  },
  blight_tainted: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CORRUPTION,
  },
  blood_rot: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.PATHOGENESIS,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PATHOGENESIS,
    text: {
      definition: "Disease-pattern descriptors.",
    },
  },
  body_horror: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  boneyard_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  bridge_passage_hazard: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  brute_combatant: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.TACTICAL,
  },
  camp_setup: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  canyon_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  captive_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  carnival_show: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  carrier_vector: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.DELIVERY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.VECTOR,
    text: {
      definition: "Heterogeneous browse family; rows split across response_demand, delivery, and theme.",
    },
  },
  carry_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  caster_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  civic_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  climbing: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  coastal_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  cognitive_impairment: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  combat_maneuver_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  commander_combatant: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.TACTICAL,
  },
  community_outbreak: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.RESPONSE_DEMAND,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.RESPONSE_DEMAND,
    text: {
      definition: "Heterogeneous browse family; rows split across response_demand, delivery, and theme.",
    },
  },
  companion_handling_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  companion_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  compulsion: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.BEHAVIOR_OVERRIDE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.BEHAVIOR_OVERRIDE,
  },
  concealable: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  concealment: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  contact_exposure: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.DELIVERY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.DELIVERY_PROFILE,
  },
  contact_offense: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.DELIVERY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.DELIVERY_PROFILE,
  },
  control_interface: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.MECHANISM,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.MECHANISM,
  },
  controller_combatant: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.TACTICAL,
  },
  corrupted_sacred: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  cosmic_dread: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  court_entourage: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  courtly_pageantry: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  creature_bane: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.TARGETING_CAPABILITY,
    text: {
      definition: "Payload rows reuse delivery semantics, with targeting specialization overrides. Targeting specialization rather than delivery.",
    },
  },
  crew_member: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  criminal_cell: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  criminal_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  cult_member: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  cumulative_transformation: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.PROGRESSION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PROGRESSION_PROFILE,
  },
  curse_marking: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  cursed_transformation: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  cursewarped: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CORRUPTION,
  },
  darklands_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.REGIONAL,
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  decadence_decline: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  defender_combatant: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.TACTICAL,
  },
  defender_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  delayed_onset: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.PROGRESSION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PROGRESSION_PROFILE,
  },
  demonic_scar_frontier_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.REGIONAL,
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  desert_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  disguise: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  disguised_pretender: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  divine_spellcaster: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CASTING_PROFILE,
  },
  dragon_spellcaster: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CASTING_PROFILE,
  },
  dream_nightmare: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  dreamborne_exposure: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.DELIVERY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.DELIVERY_PROFILE,
  },
  dreamlands_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  dungeon_hazard: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  elemental_payload: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.DELIVERY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PAYLOAD,
    text: {
      definition: "Payload rows reuse delivery semantics, with targeting specialization overrides.",
    },
  },
  elysium_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  emergency_recovery: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  endurance_pressure: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CHALLENGE_STRUCTURE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CHALLENGE_STRUCTURE,
  },
  energy_resistance: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.ENERGY,
    operation: CANONICAL_VOCABULARY.OPERATION.RESIST,
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.DEFENSIVE_CAPABILITY,
    text: {
      definition: "Heterogeneous browse family split across remediation, support operations, and capability concepts. Capability-style support concept rather than direct answer path.",
    },
  },
  enforcer_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  environmental_adaptation: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  epidemic_pestilence: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PESTILENCE,
    text: {
      definition: "Heterogeneous browse family; rows split across response_demand, delivery, and theme. Thematic rather than pure response-demand descriptor.",
    },
  },
  escort_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  ethereal_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  explosive_payload: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.DELIVERY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PAYLOAD,
    text: {
      definition: "Payload rows reuse delivery semantics, with targeting specialization overrides.",
    },
  },
  extraction_teleport: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  extradimensional_storage: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  face_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  faceless_horror: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  fall_protection: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  field_shelter: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  first_world_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  flight: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  focus_magic_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  folk_horror: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  forbidden_knowledge: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  forced_separation: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.FUNCTION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAZARD_FUNCTION,
  },
  forced_separation_hazard: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.FUNCTION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAZARD_FUNCTION,
  },
  forest_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  forgery_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  fortress_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  freshwater_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  funerary_mourning: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  fungal_growth: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.PATHOGENESIS,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PATHOGENESIS,
    text: {
      definition: "Disease-pattern descriptors.",
    },
  },
  fungal_infested: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CORRUPTION,
  },
  gothic_horror_land_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.REGIONAL,
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  graveyard_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  guard_post: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.FUNCTION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAZARD_FUNCTION,
  },
  guardian_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  guardian_retinue: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  guide_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  harrier_combatant: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.TACTICAL,
  },
  hazard_shielding: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  healer_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  healer_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  healing_suppression: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  heaven_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  hell_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  hemorrhagic_failure: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  illumination: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  industrial_grotesque: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  infestation_implant: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.PATHOGENESIS,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PATHOGENESIS,
    text: {
      definition: "Disease-pattern descriptors.",
    },
  },
  infestation_member: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  infiltrator_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  ingested_exposure: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.DELIVERY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.DELIVERY_PROFILE,
  },
  ingested_offense: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.DELIVERY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.DELIVERY_PROFILE,
  },
  inhaled_exposure: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.DELIVERY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.DELIVERY_PROFILE,
  },
  injury_exposure: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.DELIVERY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.DELIVERY_PROFILE,
  },
  innocence_twisted: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  island_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  judgment_haunt: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAUNT_MANIFESTATION,
  },
  jungle_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  life_drain_hazard: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAUNT_MANIFESTATION,
  },
  living_artwork: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  living_toy: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  long_range_teleport: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  lore_consultation: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  lure_compulsion: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAUNT_MANIFESTATION,
  },
  maelstrom_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  magic_blight_wasteland_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.REGIONAL,
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  maritime_superstition: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  mask_motif: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  medical_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  mental_impairment: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  merchant_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  message_delivery: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  mirror_motif: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  mobility: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  mobility_impairment: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  mountain_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  mounted_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  multi_stage_resolution: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CHALLENGE_STRUCTURE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CHALLENGE_STRUCTURE,
  },
  mwangi_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.REGIONAL,
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  nautical_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  navigation: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  nightmare_tainted: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CORRUPTION,
  },
  nightmare_torment: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  nirvana_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  observation_driven: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CHALLENGE_STRUCTURE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CHALLENGE_STRUCTURE,
  },
  obsession_fixation: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  occult_conspiracy: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  occult_spellcaster: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CASTING_PROFILE,
  },
  offensive: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  omen_guidance: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  organized_undead_society_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.REGIONAL,
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  pack_hunter: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  paranoia_surveillance: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  parasite_ridden: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CORRUPTION,
  },
  patrol_member: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  performer_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  petrification: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  petrifying_corruption: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.PATHOGENESIS,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PATHOGENESIS,
    text: {
      definition: "Disease-pattern descriptors.",
    },
  },
  phantom_assailants: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAUNT_MANIFESTATION,
  },
  physical_debilitation: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  plaguebearing: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CORRUPTION,
  },
  plains_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  planar_breach: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.MECHANISM,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.MECHANISM,
  },
  planar_travel: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  plane_of_air_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  plane_of_earth_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  plane_of_fire_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  plane_of_water_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  possessed_object: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CREATURE_FAMILY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.BOUND_OBJECT_FAMILY,
  },
  possession_haunt: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAUNT_MANIFESTATION,
  },
  possession_seed: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  predatory_seduction: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  pressure_trigger: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.MECHANISM,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.MECHANISM,
  },
  primal_spellcaster: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CASTING_PROFILE,
  },
  problem_diagnosis: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  profession_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  projectile_defense: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  prophecy_omen: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  pursuit_punisher: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.FUNCTION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAZARD_FUNCTION,
  },
  ranged_striker_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  recurrent_flare: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.PROGRESSION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PROGRESSION_PROFILE,
  },
  religious_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  reload_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  repair_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  replayed_tragedy: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAUNT_MANIFESTATION,
  },
  resource_drain: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.FUNCTION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAZARD_FUNCTION,
  },
  respiratory_impairment: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  revelry_excess: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  ritual_ceremony: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  ritual_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  ritualist_creature: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CASTING_PROFILE,
  },
  rot_decay: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.PATHOGENESIS,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PATHOGENESIS,
    text: {
      definition: "Disease-pattern descriptors.",
    },
  },
  ruins_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  rural_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  scholar_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  scout_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  scouting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  scrying_protection: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  seasonal_festival: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  sedation: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  seductive_temptation: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  self_buff: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  self_destructive_impulse: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.BEHAVIOR_OVERRIDE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.BEHAVIOR_OVERRIDE,
  },
  senses_support: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.SENSES,
    operation: CANONICAL_VOCABULARY.OPERATION.SUPPORT,
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SENSORY_CAPABILITY,
    text: {
      definition: "Heterogeneous browse family split across remediation, support operations, and capability concepts. Capability-style enhancement concept rather than direct answer path.",
    },
  },
  sensory_impairment: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  shadow_plane_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PLANAR,
  },
  shield_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  short_range_teleport: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  signaling: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  silencing: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
  },
  sinspawn_family: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CREATURE_FAMILY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CREATURE_FAMILY,
  },
  skirmisher_combatant: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.TACTICAL,
  },
  skirmisher_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  sky_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  small_settlement_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  social_infiltration: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  soul_binding: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  spawned_attackers: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.FUNCTION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAZARD_FUNCTION,
  },
  spell_payload: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.DELIVERY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PAYLOAD,
    text: {
      definition: "Payload rows reuse delivery semantics, with targeting specialization overrides.",
    },
  },
  stealth_entry_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  stealth_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  stitched_horror: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  support_combatant: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.TACTICAL,
  },
  surveillance_recording: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  survival: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  sustenance: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  swamp_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  tamper_evidence: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  telepathic_communication: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  temple_hazard: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  temple_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  terminal_collapse: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.PROGRESSION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.PROGRESSION_PROFILE,
  },
  threshold_lockdown: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.MECHANISM,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.MECHANISM,
  },
  thrown_offense: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.DELIVERY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.DELIVERY_PROFILE,
  },
  thrown_weapon_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  tian_xia_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.REGIONAL,
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  time_critical_resolution: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.RESPONSE_DEMAND,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.RESPONSE_DEMAND,
    text: {
      definition: "Response-side pressure family with approved operational exceptions.",
    },
  },
  timing_window: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CHALLENGE_STRUCTURE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CHALLENGE_STRUCTURE,
  },
  tomb_hazard: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  tracking: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  transformative_corruption: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  translation_support: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  transport: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  trickster_mischief: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  tripwire_trigger: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.MECHANISM,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.MECHANISM,
  },
  truth_compulsion: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.BEHAVIOR_OVERRIDE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.BEHAVIOR_OVERRIDE,
  },
  undead_family: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CREATURE_FAMILY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CREATURE_FAMILY,
  },
  undead_war_torn_region_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.REGIONAL,
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  underground_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  urban_hazard: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  urban_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  vengeful_tragedy: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.THEME,
  },
  violence_compulsion: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.BEHAVIOR_OVERRIDE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.BEHAVIOR_OVERRIDE,
  },
  void_soul_corruption: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  void_tainted: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.THEME,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CORRUPTION,
  },
  volcanic_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  warband_member: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  ward_trigger: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.MECHANISM,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.MECHANISM,
  },
  wasteland_setting: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HABITAT,
  },
  wasting_hunger: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.EFFECT,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.EFFECT_PROFILE,
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  watcher_npc: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.ROLE,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.ROLE,
  },
  waterborne_exposure: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.DELIVERY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.DELIVERY_PROFILE,
  },
  weapon_applied: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.DELIVERY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.DELIVERY_PROFILE,
  },
  weapon_staging: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  wilderness_hazard: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.SETTING,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.SITE,
  },
  writing_recordkeeping: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.CAPABILITY,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.CAPABILITY,
  },
  zone_denial: {
    primaryFacetKind: CANONICAL_VOCABULARY.FACET.KIND.FUNCTION,
    primaryFacetValue: CANONICAL_VOCABULARY.FACET.VALUE.HAZARD_FUNCTION,
  },
};

export const DERIVED_TAG_DESCRIPTIVE_CANONICAL_CONCEPTS_BY_ID = buildCanonicalConceptMap(
  CANONICAL_VOCABULARY.SCHEMA.KIND.DESCRIPTIVE,
  DERIVED_TAG_DESCRIPTIVE_CANONICAL_CONCEPTS_BY_ID_SEEDS,
);
