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
        family: "movement_traversal",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "movement_traversal", assignmentMode: "composite" }),
          expect.objectContaining({ value: "navigation", description: expect.any(String) }),
          expect.objectContaining({ value: "transport", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "access_bypass",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "barrier_bypass", description: expect.any(String) }),
          expect.objectContaining({ value: "lock_bypass", description: expect.any(String) }),
          expect.objectContaining({ value: "mechanism_manipulation", description: expect.any(String) }),
          expect.objectContaining({ value: "trap_bypass", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "reconnaissance",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "reconnaissance", assignmentMode: "composite" }),
          expect.objectContaining({ value: "scouting", description: expect.any(String) }),
          expect.objectContaining({ value: "surveillance_recording", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "carry_logistics",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "carry_support", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "restraint",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "restraint_escape", description: expect.any(String) }),
          expect.objectContaining({ value: "restraint_capture", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "anti_magic",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "countermagic", description: expect.any(String) }),
          expect.objectContaining({ value: "magic_protection", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "breaching",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "breaching", assignmentMode: "composite" }),
          expect.objectContaining({ value: "door_breaching", description: expect.any(String) }),
          expect.objectContaining({ value: "barrier_breaking", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "resolution",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "resolution", assignmentMode: "composite" }),
          expect.objectContaining({ value: "curse_removal", description: expect.any(String) }),
          expect.objectContaining({ value: "sanctification", description: expect.any(String) }),
          expect.objectContaining({ value: "ritual_appeasement", description: expect.any(String) }),
          expect.objectContaining({ value: "source_revelation", description: expect.any(String) }),
          expect.objectContaining({ value: "quarantine_containment", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "offensive_profile",
        axis: "effect",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "burst_damage", description: expect.any(String) }),
          expect.objectContaining({ value: "line_of_sight_control", description: expect.any(String) }),
          expect.objectContaining({ value: "anti_caster_disruption", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "party_role",
        axis: "party_role",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "defender_support", description: expect.any(String) }),
          expect.objectContaining({ value: "scout_support", description: expect.any(String) }),
          expect.objectContaining({ value: "face_support", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "play_pattern",
        axis: "party_role",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "shield_support", description: expect.any(String) }),
          expect.objectContaining({ value: "action_economy_support", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "access_bypass",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "lock_bypass", description: expect.any(String) }),
          expect.objectContaining({ value: "barrier_bypass", description: expect.any(String) }),
          expect.objectContaining({ value: "mechanism_manipulation", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "infiltration",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "stealth_support", description: expect.any(String) }),
          expect.objectContaining({ value: "infiltration", assignmentMode: "composite" }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "consultation",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "consultation", assignmentMode: "composite" }),
          expect.objectContaining({ value: "lore_consultation", description: expect.any(String) }),
          expect.objectContaining({ value: "problem_diagnosis", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "communication",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "communication", assignmentMode: "composite" }),
          expect.objectContaining({ value: "telepathic_communication", description: expect.any(String) }),
          expect.objectContaining({ value: "message_delivery", description: expect.any(String) }),
          expect.objectContaining({ value: "translation_support", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "expedition",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "expedition", assignmentMode: "composite" }),
          expect.objectContaining({ value: "field_shelter", description: expect.any(String) }),
          expect.objectContaining({ value: "environmental_adaptation", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "resolution",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "resolution", assignmentMode: "composite" }),
          expect.objectContaining({ value: "curse_removal", description: expect.any(String) }),
          expect.objectContaining({ value: "ritual_appeasement", description: expect.any(String) }),
          expect.objectContaining({ value: "source_revelation", description: expect.any(String) }),
          expect.objectContaining({ value: "source_cleanup", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "revelation",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "truth_reveal", description: expect.any(String) }),
          expect.objectContaining({ value: "revelation", assignmentMode: "composite" }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "security",
        axis: "utility",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "alarm", description: expect.any(String) }),
          expect.objectContaining({ value: "security", assignmentMode: "composite" }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "control",
        axis: "battlefield",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "countermagic", description: expect.any(String) }),
          expect.objectContaining({ value: "mobility_denial", description: expect.any(String) }),
          expect.objectContaining({ value: "anti_caster_disruption", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "support",
        axis: "support",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "anti_poison", description: expect.any(String) }),
          expect.objectContaining({ value: "anti_disease", description: expect.any(String) }),
          expect.objectContaining({ value: "quickened_support", description: expect.any(String) }),
          expect.objectContaining({ value: "eidolon_support", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "spell",
        family: "impact",
        axis: "effect",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "persistent_damage", description: expect.any(String) }),
          expect.objectContaining({ value: "burst_damage", description: expect.any(String) }),
          expect.objectContaining({ value: "crowd_clearing", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "hazard",
        family: "function",
        axis: "encounter",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "guarding_hazard", assignmentMode: "composite" }),
          expect.objectContaining({ value: "zone_denial", description: expect.any(String) }),
          expect.objectContaining({ value: "sentinel_guardian", description: expect.any(String) }),
          expect.objectContaining({ value: "forced_separation_hazard", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "hazard",
        family: "countermeasure_profile",
        axis: "resolution",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "quarantine_containment_countermeasure", description: expect.any(String) }),
          expect.objectContaining({ value: "contamination_cleanup_countermeasure", description: expect.any(String) }),
          expect.objectContaining({ value: "source_cleanup_countermeasure", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "hazard",
        family: "environmental_danger",
        axis: "effect",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "environmental_hazard", assignmentMode: "composite" }),
          expect.objectContaining({ value: "contamination_hazard", description: expect.any(String) }),
          expect.objectContaining({ value: "blight_hazard", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "hazard",
        family: "perception_control",
        axis: "effect",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "perception_hazard", assignmentMode: "composite" }),
          expect.objectContaining({ value: "false_safe_route", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "hazard",
        family: "problem_shape",
        axis: "problem",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "observation_first", description: expect.any(String) }),
          expect.objectContaining({ value: "source_tracing", description: expect.any(String) }),
          expect.objectContaining({ value: "layered_resolution", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "affliction",
        family: "response_profile",
        axis: "response",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "outbreak_management", description: expect.any(String) }),
          expect.objectContaining({ value: "cure_clock_urgency", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "affliction",
        family: "resolution_profile",
        axis: "response",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "cursebreaking_resolution", description: expect.any(String) }),
          expect.objectContaining({ value: "quarantine_containment_resolution", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "habitat_setting",
        axis: "setting",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "aquatic_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "mountain_setting", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "site_setting",
        axis: "setting",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "urban_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "temple_setting", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "regional_setting",
        axis: "setting",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "organized_undead_society_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "gothic_horror_land_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "alien_technology_wasteland_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "tian_xia_setting", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "corruption_profile",
        axis: "specialization",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "blight_tainted", description: expect.any(String) }),
          expect.objectContaining({ value: "fungal_infested", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "cohort_role",
        axis: "encounter",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "crew_member", description: expect.any(String) }),
          expect.objectContaining({ value: "cult_member", description: expect.any(String) }),
          expect.objectContaining({ value: "pack_hunter", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "planar_setting",
        axis: "setting",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "astral_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "upper_plane_setting", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "scene_role",
        axis: "npc_role",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "enforcer_npc", description: expect.any(String) }),
          expect.objectContaining({ value: "infiltrator_npc", description: expect.any(String) }),
          expect.objectContaining({ value: "guardian_npc", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "social_role",
        axis: "npc_role",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "profession_npc", description: expect.any(String) }),
          expect.objectContaining({ value: "authority_npc", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "threat_profile",
        axis: "encounter",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "prey_control_threat", description: expect.any(String) }),
          expect.objectContaining({ value: "reinforcement_threat", description: expect.any(String) }),
          expect.objectContaining({ value: "infiltration_threat", description: expect.any(String) }),
        ]),
      }),
    ]));
  });
});
