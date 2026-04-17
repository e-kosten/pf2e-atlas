import { describe, expect, it } from "vitest";

import { groupDerivedTagOntology } from "../../src/tags/runtime/catalog-utils.js";
import {
  DERIVED_TAG_ONTOLOGY_FAMILIES,
  DERIVED_TAG_ONTOLOGY_TAGS,
} from "../../src/tags/index.js";

describe("derived tag catalog", () => {
  it("publishes a compact derived-tag catalog", () => {
    expect(groupDerivedTagOntology({
      families: DERIVED_TAG_ONTOLOGY_FAMILIES,
      tags: DERIVED_TAG_ONTOLOGY_TAGS,
    })).toEqual(expect.arrayContaining([
      expect.objectContaining({
        category: "equipment",
        subcategories: ["consumable"],
        family: "function",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "mental_recovery", description: expect.any(String) }),
          expect.objectContaining({ value: "senses_support", description: expect.any(String) }),
          expect.objectContaining({ value: "energy_resistance", description: expect.any(String) }),
          expect.objectContaining({ value: "fortune_support", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "purpose",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "navigation", description: expect.any(String) }),
          expect.objectContaining({ value: "tracking", description: expect.any(String) }),
          expect.objectContaining({ value: "anti_tracking", description: expect.any(String) }),
          expect.objectContaining({ value: "carry_support", description: expect.any(String) }),
          expect.objectContaining({ value: "restraint_escape", description: expect.any(String) }),
          expect.objectContaining({ value: "restraint_capture", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "access_system",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "extradimensional_storage", description: expect.any(String) }),
          expect.objectContaining({ value: "weapon_staging", description: expect.any(String) }),
          expect.objectContaining({ value: "ammo_management", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        subcategories: ["ammo"],
        family: "ammunition_payload",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "creature_bane", description: expect.any(String) }),
          expect.objectContaining({ value: "elemental_payload", description: expect.any(String) }),
          expect.objectContaining({ value: "explosive_payload", description: expect.any(String) }),
          expect.objectContaining({ value: "spell_payload", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "impact",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "mobility_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "sensory_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "mental_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "physical_debilitation", description: expect.any(String) }),
          expect.objectContaining({ value: "sedation", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "communication",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "signaling", description: expect.any(String) }),
          expect.objectContaining({ value: "translation_support", description: expect.any(String) }),
          expect.objectContaining({ value: "message_delivery", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "expedition",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "environmental_adaptation", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "infiltration",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "disguise", description: expect.any(String) }),
          expect.objectContaining({ value: "social_infiltration", description: expect.any(String) }),
          expect.objectContaining({ value: "concealment", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "defense_profile",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "ally_cover", description: expect.any(String) }),
          expect.objectContaining({ value: "projectile_defense", description: expect.any(String) }),
          expect.objectContaining({ value: "hazard_shielding", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "magic_interference",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "countermagic", description: expect.any(String) }),
          expect.objectContaining({ value: "magic_protection", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "security",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "alarm", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "infiltration",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "disguise", description: expect.any(String) }),
          expect.objectContaining({ value: "social_infiltration", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "communication",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "signaling", description: expect.any(String) }),
          expect.objectContaining({ value: "translation_support", description: expect.any(String) }),
          expect.objectContaining({ value: "message_delivery", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "reconnaissance",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "scouting", description: expect.any(String) }),
          expect.objectContaining({ value: "tracking", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "revelation",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "hazard_revelation", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "wayfinding",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "navigation", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "traversal",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "mobility", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "control",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "fear_pressure", description: expect.any(String) }),
          expect.objectContaining({ value: "concealment", description: expect.any(String) }),
          expect.objectContaining({ value: "line_of_sight_control", description: expect.any(String) }),
          expect.objectContaining({ value: "battlefield_disruption", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "magic_interference",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "countermagic", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "security",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "alarm", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "impact",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "mental_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "sensory_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "forced_movement", description: expect.any(String) }),
          expect.objectContaining({ value: "restraint_capture", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "support",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "healing_support", description: expect.any(String) }),
          expect.objectContaining({ value: "temporary_hp_support", description: expect.any(String) }),
          expect.objectContaining({ value: "condition_support", description: expect.any(String) }),
          expect.objectContaining({ value: "affliction_cleanup", description: expect.any(String) }),
          expect.objectContaining({ value: "escape_support", description: expect.any(String) }),
          expect.objectContaining({ value: "protective_ward", description: expect.any(String) }),
          expect.objectContaining({ value: "death_prevention", description: expect.any(String) }),
          expect.objectContaining({ value: "resistance_support", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "expedition",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "aquatic_support", description: expect.any(String) }),
          expect.objectContaining({ value: "sustenance", description: expect.any(String) }),
          expect.objectContaining({ value: "field_shelter", description: expect.any(String) }),
          expect.objectContaining({ value: "environmental_adaptation", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "tempo",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "quickened_support", description: expect.any(String) }),
          expect.objectContaining({ value: "initiative_support", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "attrition",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "persistent_damage", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "summoner_support",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "eidolon_support", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "hazard",
        family: "mechanism",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "ward_trigger", description: expect.any(String) }),
          expect.objectContaining({ value: "pressure_trigger", description: expect.any(String) }),
          expect.objectContaining({ value: "tripwire_trigger", description: expect.any(String) }),
          expect.objectContaining({ value: "threshold_lockdown", description: expect.any(String) }),
          expect.objectContaining({ value: "control_interface", description: expect.any(String) }),
          expect.objectContaining({ value: "planar_breach", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "hazard",
        family: "function",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "alarm", description: expect.any(String) }),
          expect.objectContaining({ value: "restraint_capture", description: expect.any(String) }),
          expect.objectContaining({ value: "barrier_lockdown", description: expect.any(String) }),
          expect.objectContaining({ value: "spawned_attackers", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "hazard",
        family: "haunt_manifestation",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "life_drain_hazard", description: expect.any(String) }),
          expect.objectContaining({ value: "phantom_assailants", description: expect.any(String) }),
          expect.objectContaining({ value: "lure_compulsion", description: expect.any(String) }),
          expect.objectContaining({ value: "battlefield_disruption", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "hazard",
        family: "impact",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "mental_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "mobility_impairment", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "hazard",
        family: "environmental_danger",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "acid_hazard", description: expect.any(String) }),
          expect.objectContaining({ value: "cold_hazard", description: expect.any(String) }),
          expect.objectContaining({ value: "electric_hazard", description: expect.any(String) }),
          expect.objectContaining({ value: "fire_hazard", description: expect.any(String) }),
          expect.objectContaining({ value: "poison_hazard", description: expect.any(String) }),
          expect.objectContaining({ value: "respiratory_hazard", description: expect.any(String) }),
          expect.objectContaining({ value: "sound_hazard", description: expect.any(String) }),
          expect.objectContaining({ value: "water_hazard", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "hazard",
        family: "forced_position",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "pitfall", description: expect.any(String) }),
          expect.objectContaining({ value: "collapse_hazard", description: expect.any(String) }),
          expect.objectContaining({ value: "forced_movement", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "hazard",
        family: "perception_control",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "navigation_disruption", description: expect.any(String) }),
          expect.objectContaining({ value: "illusion_assault", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "hazard",
        family: "attack_vector",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "overhead_strike", description: expect.any(String) }),
          expect.objectContaining({ value: "projectile_emitter", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "affliction",
        family: "impact",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "mental_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "mobility_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "physical_debilitation", description: expect.any(String) }),
          expect.objectContaining({ value: "healing_suppression", description: expect.any(String) }),
          expect.objectContaining({ value: "cognitive_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "sensory_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "sedation", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "affliction",
        family: "pathogenesis",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "rot_decay", description: expect.any(String) }),
          expect.objectContaining({ value: "infestation_implant", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "affliction",
        family: "epidemiological_profile",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "epidemic_pestilence", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "affliction",
        family: "behavioral_override",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "compulsion", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "affliction",
        family: "physiology_override",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "respiratory_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "transformative_corruption", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "affliction",
        family: "metaphysical_profile",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "void_soul_corruption", description: expect.any(String) }),
          expect.objectContaining({ value: "nightmare_torment", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "setting",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "freshwater_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "swamp_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "underground_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "graveyard_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "coastal_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "astral_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "ethereal_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "plane_of_fire_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "plane_of_air_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "plane_of_water_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "plane_of_earth_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "elemental_plane_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "first_world_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "dreamlands_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "boneyard_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "heaven_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "nirvana_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "elysium_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "upper_plane_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "hell_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "abyss_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "abaddon_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "lower_plane_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "axis_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "shadow_plane_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "maelstrom_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "cosmic_framework_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "island_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "tian_xia_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "geb_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "gravelands_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "battlefield_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "jungle_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "plains_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "canyon_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "sky_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "temple_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "fortress_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "wasteland_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "volcanic_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "small_settlement_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "rural_setting", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "encounter_role",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "profession_npc", description: expect.any(String) }),
          expect.objectContaining({ value: "civic_npc", description: expect.any(String) }),
          expect.objectContaining({ value: "combatant_npc", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "ontology_cluster",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "undead_adjacent", description: expect.any(String), nativeOntologyPolicy: "aggregates_native_signals" }),
          expect.objectContaining({ value: "sinspawn_family", description: expect.any(String), nativeOntologyPolicy: "aggregates_native_signals" }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "casting_profile",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "dragon_spellcaster", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "threat_profile",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "possession_threat", description: expect.any(String) }),
          expect.objectContaining({ value: "life_drain_threat", description: expect.any(String) }),
          expect.objectContaining({ value: "spawn_creator", description: expect.any(String) }),
          expect.objectContaining({ value: "petrification_threat", description: expect.any(String) }),
          expect.objectContaining({ value: "regeneration_threat", description: expect.any(String) }),
          expect.objectContaining({ value: "ambush_grabber", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "motif",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "carnival_show", description: expect.any(String) }),
          expect.objectContaining({ value: "living_toy", description: expect.any(String) }),
          expect.objectContaining({ value: "living_artwork", description: expect.any(String) }),
          expect.objectContaining({ value: "trickster_chaos", description: expect.any(String) }),
          expect.objectContaining({ value: "mask_motif", description: expect.any(String) }),
          expect.objectContaining({ value: "faceless_horror", description: expect.any(String) }),
          expect.objectContaining({ value: "disguised_pretender", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "bound_object",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "animated_object", description: expect.any(String) }),
          expect.objectContaining({ value: "animated_statue", description: expect.any(String) }),
        ]),
      }),
    ]));
  });
});
