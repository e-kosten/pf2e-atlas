import { describe, expect, it } from "vitest";

import { DERIVED_TAG_CATALOG } from "../../src/tags/index.js";

describe("derived tag catalog", () => {
  it("publishes a compact derived-tag catalog", () => {
    expect(DERIVED_TAG_CATALOG).toEqual(expect.arrayContaining([
      expect.objectContaining({
        category: "equipment",
        subcategories: ["consumable"],
        family: "function",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "mental_recovery", description: expect.any(String) }),
          expect.objectContaining({ value: "senses_support", description: expect.any(String) }),
          expect.objectContaining({ value: "energy_resistance", description: expect.any(String) }),
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
        subcategories: ["ammo"],
        family: "ammunition_payload",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "creature_bane", description: expect.any(String) }),
          expect.objectContaining({ value: "elemental_payload", description: expect.any(String) }),
          expect.objectContaining({ value: "explosive_payload", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "impact",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "mobility_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "sensory_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "mental_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "sedation", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "communication",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "signaling", description: expect.any(String) }),
          expect.objectContaining({ value: "message_delivery", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "infiltration",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "disguise", description: expect.any(String) }),
          expect.objectContaining({ value: "social_infiltration", description: expect.any(String) }),
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
          expect.objectContaining({ value: "message_delivery", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "reconnaissance",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "scouting", description: expect.any(String) }),
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
          expect.objectContaining({ value: "condition_support", description: expect.any(String) }),
          expect.objectContaining({ value: "protective_ward", description: expect.any(String) }),
          expect.objectContaining({ value: "death_prevention", description: expect.any(String) }),
          expect.objectContaining({ value: "resistance_support", description: expect.any(String) }),
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
        ]),
      }),
      expect.objectContaining({
        category: "hazard",
        family: "attack_vector",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "overhead_strike", description: expect.any(String) }),
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
        category: "creature",
        family: "setting",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "freshwater_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "swamp_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "underground_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "graveyard_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "coastal_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "astral_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "first_world_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "boneyard_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "island_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "plains_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "canyon_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "temple_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "fortress_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "wasteland_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "volcanic_setting", description: expect.any(String) }),
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
    ]));
  });
});
