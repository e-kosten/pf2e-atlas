import { describe, expect, it } from "vitest";

import { groupDerivedTagOntology } from "../../src/tags/runtime/catalog-utils.js";
import { DERIVED_TAG_ONTOLOGY_FAMILIES, DERIVED_TAG_ONTOLOGY_TAGS } from "../../src/tags/index.js";

function anyString(): unknown {
  return expect.any(String);
}

function arrayContaining(values: unknown[]): unknown {
  return expect.arrayContaining(values);
}

function objectContaining(value: Record<string, unknown>): unknown {
  return expect.objectContaining(value);
}

describe("derived tag catalog", () => {
  it("publishes a compact derived-tag catalog", () => {
    expect(
      groupDerivedTagOntology({
        families: DERIVED_TAG_ONTOLOGY_FAMILIES,
        tags: DERIVED_TAG_ONTOLOGY_TAGS,
      }),
    ).toEqual(
      arrayContaining([
        objectContaining({
          category: "equipment",
          family: "movement_traversal",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "movement_traversal", assignmentMode: "composite" }),
            objectContaining({ value: "navigation", description: anyString() }),
            objectContaining({ value: "transport", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "equipment",
          family: "access_bypass",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "barrier_bypass", description: anyString() }),
            objectContaining({ value: "lock_bypass", description: anyString() }),
            objectContaining({ value: "mechanism_manipulation", description: anyString() }),
            objectContaining({ value: "trap_bypass", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "equipment",
          family: "reconnaissance",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "reconnaissance", assignmentMode: "composite" }),
            objectContaining({ value: "scouting", description: anyString() }),
            objectContaining({ value: "surveillance_recording", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "equipment",
          family: "carry_logistics",
          axis: "utility",
          tags: arrayContaining([objectContaining({ value: "carry_support", description: anyString() })]),
        }),
        objectContaining({
          category: "equipment",
          family: "restraint",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "restraint_escape", description: anyString() }),
            objectContaining({ value: "restraint_capture", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "equipment",
          family: "anti_magic",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "countermagic", description: anyString() }),
            objectContaining({ value: "magic_protection", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "equipment",
          family: "breaching",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "breaching", assignmentMode: "composite" }),
            objectContaining({ value: "door_breaching", description: anyString() }),
            objectContaining({ value: "barrier_breaking", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "equipment",
          family: "resolution",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "resolution", assignmentMode: "composite" }),
            objectContaining({ value: "curse_removal", description: anyString() }),
            objectContaining({ value: "sanctification", description: anyString() }),
            objectContaining({ value: "ritual_appeasement", description: anyString() }),
            objectContaining({ value: "source_revelation", description: anyString() }),
            objectContaining({ value: "quarantine_containment", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "equipment",
          family: "offensive_profile",
          axis: "effect",
          tags: arrayContaining([
            objectContaining({ value: "burst_damage", description: anyString() }),
            objectContaining({ value: "line_of_sight_control", description: anyString() }),
            objectContaining({ value: "anti_caster_disruption", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "equipment",
          family: "party_role",
          axis: "party_role",
          tags: arrayContaining([
            objectContaining({ value: "defender_support", description: anyString() }),
            objectContaining({ value: "scout_support", description: anyString() }),
            objectContaining({ value: "face_support", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "equipment",
          family: "play_pattern",
          axis: "party_role",
          tags: arrayContaining([
            objectContaining({ value: "shield_support", description: anyString() }),
            objectContaining({ value: "action_economy_support", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "access_bypass",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "lock_bypass", description: anyString() }),
            objectContaining({ value: "barrier_bypass", description: anyString() }),
            objectContaining({ value: "mechanism_manipulation", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "infiltration",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "stealth_support", description: anyString() }),
            objectContaining({ value: "infiltration", assignmentMode: "composite" }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "consultation",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "consultation", assignmentMode: "composite" }),
            objectContaining({ value: "lore_consultation", description: anyString() }),
            objectContaining({ value: "problem_diagnosis", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "communication",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "communication", assignmentMode: "composite" }),
            objectContaining({ value: "telepathic_communication", description: anyString() }),
            objectContaining({ value: "message_delivery", description: anyString() }),
            objectContaining({ value: "translation_support", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "expedition",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "expedition", assignmentMode: "composite" }),
            objectContaining({ value: "field_shelter", description: anyString() }),
            objectContaining({ value: "environmental_adaptation", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "resolution",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "resolution", assignmentMode: "composite" }),
            objectContaining({ value: "curse_removal", description: anyString() }),
            objectContaining({ value: "ritual_appeasement", description: anyString() }),
            objectContaining({ value: "source_revelation", description: anyString() }),
            objectContaining({ value: "source_cleanup", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "revelation",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "truth_reveal", description: anyString() }),
            objectContaining({ value: "revelation", assignmentMode: "composite" }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "security",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "alarm", description: anyString() }),
            objectContaining({ value: "security", assignmentMode: "composite" }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "control",
          axis: "battlefield",
          tags: arrayContaining([
            objectContaining({ value: "countermagic", description: anyString() }),
            objectContaining({ value: "mobility_denial", description: anyString() }),
            objectContaining({ value: "anti_caster_disruption", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "support",
          axis: "support",
          tags: arrayContaining([
            objectContaining({ value: "anti_poison", description: anyString() }),
            objectContaining({ value: "anti_disease", description: anyString() }),
            objectContaining({ value: "quickened_support", description: anyString() }),
            objectContaining({ value: "eidolon_support", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "impact",
          axis: "effect",
          tags: arrayContaining([
            objectContaining({ value: "persistent_damage", description: anyString() }),
            objectContaining({ value: "burst_damage", description: anyString() }),
            objectContaining({ value: "crowd_clearing", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "hazard",
          family: "function",
          axis: "encounter",
          tags: arrayContaining([
            objectContaining({ value: "guarding_hazard", assignmentMode: "composite" }),
            objectContaining({ value: "zone_denial", description: anyString() }),
            objectContaining({ value: "sentinel_guardian", description: anyString() }),
            objectContaining({ value: "forced_separation_hazard", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "hazard",
          family: "countermeasure_profile",
          axis: "resolution",
          tags: arrayContaining([
            objectContaining({
              value: "quarantine_containment_countermeasure",
              description: anyString(),
            }),
            objectContaining({ value: "contamination_cleanup_countermeasure", description: anyString() }),
            objectContaining({ value: "source_cleanup_countermeasure", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "hazard",
          family: "environmental_danger",
          axis: "effect",
          tags: arrayContaining([
            objectContaining({ value: "environmental_hazard", assignmentMode: "composite" }),
            objectContaining({ value: "contamination_hazard", description: anyString() }),
            objectContaining({ value: "blight_hazard", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "hazard",
          family: "perception_control",
          axis: "effect",
          tags: arrayContaining([
            objectContaining({ value: "perception_hazard", assignmentMode: "composite" }),
            objectContaining({ value: "false_safe_route", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "hazard",
          family: "problem_shape",
          axis: "problem",
          tags: arrayContaining([
            objectContaining({ value: "observation_first", description: anyString() }),
            objectContaining({ value: "source_tracing", description: anyString() }),
            objectContaining({ value: "layered_resolution", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "affliction",
          family: "response_profile",
          axis: "response",
          tags: arrayContaining([
            objectContaining({ value: "outbreak_management", description: anyString() }),
            objectContaining({ value: "cure_clock_urgency", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "affliction",
          family: "resolution_profile",
          axis: "response",
          tags: arrayContaining([
            objectContaining({ value: "cursebreaking_resolution", description: anyString() }),
            objectContaining({ value: "quarantine_containment_resolution", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "creature",
          family: "habitat_setting",
          axis: "setting",
          tags: arrayContaining([
            objectContaining({ value: "aquatic_setting", description: anyString() }),
            objectContaining({ value: "mountain_setting", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "creature",
          family: "site_setting",
          axis: "setting",
          tags: arrayContaining([
            objectContaining({ value: "urban_setting", description: anyString() }),
            objectContaining({ value: "temple_setting", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "creature",
          family: "regional_setting",
          axis: "setting",
          tags: arrayContaining([
            objectContaining({ value: "organized_undead_society_setting", description: anyString() }),
            objectContaining({ value: "gothic_horror_land_setting", description: anyString() }),
            objectContaining({ value: "alien_technology_wasteland_setting", description: anyString() }),
            objectContaining({ value: "tian_xia_setting", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "creature",
          family: "corruption_profile",
          axis: "specialization",
          tags: arrayContaining([
            objectContaining({ value: "blight_tainted", description: anyString() }),
            objectContaining({ value: "fungal_infested", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "creature",
          family: "cohort_role",
          axis: "encounter",
          tags: arrayContaining([
            objectContaining({ value: "crew_member", description: anyString() }),
            objectContaining({ value: "cult_member", description: anyString() }),
            objectContaining({ value: "pack_hunter", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "creature",
          family: "planar_setting",
          axis: "setting",
          tags: arrayContaining([
            objectContaining({ value: "astral_setting", description: anyString() }),
            objectContaining({ value: "upper_plane_setting", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "creature",
          family: "scene_role",
          axis: "npc_role",
          tags: arrayContaining([
            objectContaining({ value: "enforcer_npc", description: anyString() }),
            objectContaining({ value: "infiltrator_npc", description: anyString() }),
            objectContaining({ value: "guardian_npc", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "creature",
          family: "social_role",
          axis: "npc_role",
          tags: arrayContaining([
            objectContaining({ value: "profession_npc", description: anyString() }),
            objectContaining({ value: "authority_npc", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "creature",
          family: "threat_profile",
          axis: "encounter",
          tags: arrayContaining([
            objectContaining({ value: "prey_control_threat", description: anyString() }),
            objectContaining({ value: "reinforcement_threat", description: anyString() }),
            objectContaining({ value: "infiltration_threat", description: anyString() }),
          ]),
        }),
      ]),
    );
  });
});
