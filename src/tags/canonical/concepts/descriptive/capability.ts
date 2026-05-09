import { defineFacetConcepts, mergeCanonicalConceptSeeds, type FacetlessConceptSeed } from "../../builders.js";
import { CANONICAL_VOCABULARY } from "../../vocabulary.js";
import { CANONICAL_FACETS } from "../../facets.js";

const capabilitySeeds: Record<string, FacetlessConceptSeed> = {
  action_economy_support: {},
  alarm: {},
  alchemical_crafting: {},
  ally_cover: {},
  ally_support: {},
  ammo_management: {},
  anti_tracking: {},
  aquatic_support: {},
  camp_setup: {},
  carry_support: {},
  caster_support: {},
  climbing: {},
  combat_maneuver_support: {},
  companion_handling_support: {},
  companion_support: {},
  concealable: {},
  concealment: {},
  defender_support: {},
  disguise: {},
  emergency_recovery: {},
  environmental_adaptation: {},
  extraction_teleport: {},
  extradimensional_storage: {},
  face_support: {},
  fall_protection: {},
  field_shelter: {},
  flight: {},
  focus_magic_support: {},
  forgery_support: {},
  hazard_shielding: {},
  healer_support: {},
  illumination: {},
  long_range_teleport: {},
  lore_consultation: {},
  medical_support: {},
  message_delivery: {},
  mobility: {},
  mounted_support: {},
  navigation: {},
  offensive: {},
  omen_guidance: {},
  planar_travel: {},
  problem_diagnosis: {},
  projectile_defense: {},
  ranged_striker_support: {},
  reload_support: {},
  repair_support: {},
  ritual_support: {},
  scout_support: {},
  scouting: {},
  scrying_protection: {},
  self_buff: {},
  shield_support: {},
  short_range_teleport: {},
  signaling: {},
  skirmisher_support: {},
  social_infiltration: {},
  stealth_entry_support: {},
  stealth_support: {},
  surveillance_recording: {},
  survival: {},
  sustenance: {},
  tamper_evidence: {},
  telepathic_communication: {},
  thrown_weapon_support: {},
  tracking: {},
  translation_support: {},
  transport: {},
  weapon_staging: {},
  writing_recordkeeping: {},
};

const castingProfileSeeds: Record<string, FacetlessConceptSeed> = {
  arcane_spellcaster: {},
  divine_spellcaster: {},
  dragon_spellcaster: {},
  occult_spellcaster: {},
  primal_spellcaster: {},
  ritualist_creature: {},
};

const defensiveCapabilitySeeds: Record<string, FacetlessConceptSeed> = {
  energy_resistance: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.ENERGY,
    operation: CANONICAL_VOCABULARY.OPERATION.RESIST,
    text: {
      definition:
        "Heterogeneous browse family split across remediation, support operations, and capability concepts. Capability-style support concept rather than direct answer path.",
    },
  },
};

const sensoryCapabilitySeeds: Record<string, FacetlessConceptSeed> = {
  senses_support: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.SENSES,
    operation: CANONICAL_VOCABULARY.OPERATION.SUPPORT,
    text: {
      definition:
        "Heterogeneous browse family split across remediation, support operations, and capability concepts. Capability-style enhancement concept rather than direct answer path.",
    },
  },
};

const targetingCapabilitySeeds: Record<string, FacetlessConceptSeed> = {
  creature_bane: {
    text: {
      definition:
        "Payload rows reuse delivery semantics, with targeting specialization overrides. Targeting specialization rather than delivery.",
    },
  },
};

export const capabilitySeedsByKind = mergeCanonicalConceptSeeds([
  defineFacetConcepts(
    CANONICAL_FACETS.CAPABILITY.CAPABILITY,
    capabilitySeeds,
  ),
  defineFacetConcepts(
    CANONICAL_FACETS.CAPABILITY.CASTING_PROFILE,
    castingProfileSeeds,
  ),
  defineFacetConcepts(
    CANONICAL_FACETS.CAPABILITY.DEFENSIVE_CAPABILITY,
    defensiveCapabilitySeeds,
  ),
  defineFacetConcepts(
    CANONICAL_FACETS.CAPABILITY.SENSORY_CAPABILITY,
    sensoryCapabilitySeeds,
  ),
  defineFacetConcepts(
    CANONICAL_FACETS.CAPABILITY.TARGETING_CAPABILITY,
    targetingCapabilitySeeds,
  ),
]);
