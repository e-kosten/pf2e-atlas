import { describe, expect, it } from "vitest";

import { groupDerivedTagOntology } from "../../src/tags/runtime/catalog-utils.js";
import { CREATURE_DERIVED_TAG_ONTOLOGY } from "../../src/tags/ontology/creature.js";
import { flattenDerivedTagAuthoredCategoryOntology } from "../../src/tags/ontology/utils.js";
import {
  DERIVED_TAG_ONTOLOGY_FAMILIES,
  DERIVED_TAG_ONTOLOGY_TAGS,
} from "../../src/tags/index.js";

describe("derived tag ontology", () => {
  it("publishes unique category-scoped families and tags with assignment modes", () => {
    const familiesByCategory = new Map<string, Set<string>>();
    const tagsByCategory = new Map<string, Map<string, string>>();

    for (const family of DERIVED_TAG_ONTOLOGY_FAMILIES) {
      const categoryFamilies = familiesByCategory.get(family.category) ?? new Set<string>();
      expect(categoryFamilies.has(family.family)).toBe(false);
      categoryFamilies.add(family.family);
      familiesByCategory.set(family.category, categoryFamilies);
    }

    for (const tag of DERIVED_TAG_ONTOLOGY_TAGS) {
      expect(tag.assignmentMode).toBeDefined();
      const categoryTags = tagsByCategory.get(tag.category) ?? new Map<string, string>();
      const existingFamily = categoryTags.get(tag.tag);
      if (existingFamily) {
        expect(existingFamily).toBe(tag.family);
      } else {
        categoryTags.set(tag.tag, tag.family);
      }
      tagsByCategory.set(tag.category, categoryTags);
    }

    for (const tag of DERIVED_TAG_ONTOLOGY_TAGS) {
      const categoryTags = tagsByCategory.get(tag.category) ?? new Map<string, string>();
      for (const adjacentTag of tag.adjacentTags ?? []) {
        expect(categoryTags.has(adjacentTag)).toBe(true);
      }
      for (const childTag of tag.compositeOfAnyTags ?? []) {
        expect(categoryTags.has(childTag)).toBe(true);
      }
    }
  });

  it("keeps explicit composite tags and only derives grouped catalog views at the boundary", () => {
    const spellTransformation = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "transformation");
    expect(spellTransformation).toEqual(expect.objectContaining({
      tag: "transformation",
      assignmentMode: "composite",
      compositeOfAnyTags: ["battle_form", "animal_form", "elemental_form"],
    }));
    const spellReconnaissance = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "reconnaissance");
    expect(spellReconnaissance).toEqual(expect.objectContaining({
      family: "reconnaissance",
      assignmentMode: "composite",
      compositeOfAnyTags: ["scouting", "tracking", "scouting_summons"],
    }));
    const spellRevelation = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "revelation");
    expect(spellRevelation).toEqual(expect.objectContaining({
      family: "revelation",
      assignmentMode: "composite",
      compositeOfAnyTags: ["magic_detection", "invisibility_reveal", "truth_reveal", "curse_revelation", "hazard_revelation"],
    }));
    const spellWayfinding = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "wayfinding");
    expect(spellWayfinding).toEqual(expect.objectContaining({
      family: "wayfinding",
      assignmentMode: "composite",
      compositeOfAnyTags: ["navigation", "tracking", "long_range_teleport", "planar_travel"],
    }));
    const spellSecurity = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "security");
    expect(spellSecurity).toEqual(expect.objectContaining({
      family: "security",
      assignmentMode: "composite",
      compositeOfAnyTags: ["alarm", "scrying_protection", "protective_ward", "countermagic"],
    }));
    const spellConsultation = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "consultation");
    expect(spellConsultation).toEqual(expect.objectContaining({
      family: "consultation",
      assignmentMode: "composite",
      compositeOfAnyTags: ["lore_consultation", "problem_diagnosis", "omen_guidance"],
    }));
    const spellLockBypass = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "lock_bypass");
    expect(spellLockBypass).toEqual(expect.objectContaining({
      family: "access_bypass",
      assignmentMode: "hybrid",
      adjacentTags: ["trap_bypass", "barrier_bypass"],
      appliesWhen: expect.arrayContaining([
        "The spell is naturally retrieved to unlock, unseal, or open a secured entry point, door, chest, manacle, or similar closure.",
      ]),
    }));
    const spellMechanismManipulation = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "mechanism_manipulation");
    expect(spellMechanismManipulation).toEqual(expect.objectContaining({
      family: "access_bypass",
      assignmentMode: "hybrid",
      adjacentTags: ["lock_bypass", "trap_bypass"],
      appliesWhen: expect.arrayContaining([
        "The spell is naturally retrieved to operate a lever, button, latch, control panel, pressure surface, or similar mechanism from a safe or unusual position.",
      ]),
    }));
    const equipmentStealthSupport = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "equipment" && tag.tag === "stealth_support");
    expect(equipmentStealthSupport).toEqual(expect.objectContaining({
      family: "infiltration",
      assignmentMode: "deterministic",
    }));
    const spellStealthSupport = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "stealth_support");
    expect(spellStealthSupport).toEqual(expect.objectContaining({
      family: "infiltration",
      assignmentMode: "hybrid",
      adjacentTags: ["concealment", "silencing"],
      appliesWhen: expect.arrayContaining([
        "The spell is naturally retrieved to help a creature move quietly, avoid notice, pass unseen, or keep a covert approach from drawing attention.",
      ]),
    }));
    const spellInfiltration = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "infiltration");
    expect(spellInfiltration).toEqual(expect.objectContaining({
      family: "infiltration",
      assignmentMode: "composite",
      compositeOfAnyTags: ["stealth_support", "disguise", "social_infiltration"],
    }));
    const equipmentBarrierBypass = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "equipment" && tag.tag === "barrier_bypass");
    expect(equipmentBarrierBypass).toEqual(expect.objectContaining({
      family: "access_bypass",
      assignmentMode: "deterministic",
    }));
    const equipmentMechanismManipulation = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "equipment" && tag.tag === "mechanism_manipulation");
    expect(equipmentMechanismManipulation).toEqual(expect.objectContaining({
      family: "access_bypass",
      assignmentMode: "deterministic",
    }));
    const equipmentReconnaissance = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "equipment" && tag.tag === "reconnaissance");
    expect(equipmentReconnaissance).toEqual(expect.objectContaining({
      family: "reconnaissance",
      assignmentMode: "composite",
      compositeOfAnyTags: ["scouting", "illumination", "surveillance_recording", "tracking", "anti_tracking"],
    }));
    const equipmentSecurity = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "equipment" && tag.tag === "security");
    expect(equipmentSecurity).toEqual(expect.objectContaining({
      family: "security",
      assignmentMode: "composite",
      compositeOfAnyTags: ["alarm", "scrying_protection", "tamper_evidence"],
    }));
    const equipmentBurstDamage = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "equipment" && tag.tag === "burst_damage");
    expect(equipmentBurstDamage).toEqual(expect.objectContaining({
      family: "offensive_profile",
      assignmentMode: "hybrid",
      adjacentTags: ["crowd_clearing", "persistent_damage"],
    }));
    const scoutSupport = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "equipment" && tag.tag === "scout_support");
    expect(scoutSupport).toEqual(expect.objectContaining({
      family: "party_role",
      assignmentMode: "hybrid",
    }));
    const shieldSupport = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "equipment" && tag.tag === "shield_support");
    expect(shieldSupport).toEqual(expect.objectContaining({
      family: "play_pattern",
      assignmentMode: "hybrid",
    }));
    const cursebreakingResolution = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "affliction" && tag.tag === "cursebreaking_resolution");
    expect(cursebreakingResolution).toEqual(expect.objectContaining({
      family: "resolution_profile",
      assignmentMode: "hybrid",
      adjacentTags: ["countermagic_resolution", "ritual_appeasement_resolution"],
    }));
    const spellResolution = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "resolution");
    expect(spellResolution).toEqual(expect.objectContaining({
      family: "resolution",
      assignmentMode: "composite",
      compositeOfAnyTags: ["curse_removal", "exorcism", "sanctification", "quarantine_containment", "contamination_cleanup", "source_revelation", "source_cleanup"],
    }));
    const spellCommunication = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "communication");
    expect(spellCommunication).toEqual(expect.objectContaining({
      family: "communication",
      assignmentMode: "composite",
      compositeOfAnyTags: ["signaling", "telepathic_communication", "message_delivery", "translation_support"],
    }));
    const equipmentResolution = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "equipment" && tag.tag === "resolution");
    expect(equipmentResolution).toEqual(expect.objectContaining({
      family: "resolution",
      assignmentMode: "composite",
      compositeOfAnyTags: ["curse_removal", "sanctification", "source_revelation", "quarantine_containment", "contamination_cleanup", "source_cleanup"],
    }));
    const fungalInfested = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "creature" && tag.tag === "fungal_infested");
    expect(fungalInfested).toEqual(expect.objectContaining({
      family: "corruption_profile",
      assignmentMode: "hybrid",
      adjacentTags: ["disease_vector", "body_horror"],
    }));

    const urbanSetting = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "creature" && tag.tag === "urban_setting");
    expect(urbanSetting).toEqual(expect.objectContaining({
      family: "site_setting",
      assignmentMode: "editorial",
      appliesWhen: expect.arrayContaining([
        "The creature is primarily framed as belonging in city or sewer encounter spaces.",
      ]),
      adjacentTags: ["small_settlement_setting", "fortress_setting"],
    }));
    const organizedUndeadSocietySetting = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "creature" && tag.tag === "organized_undead_society_setting");
    expect(organizedUndeadSocietySetting).toEqual(expect.objectContaining({
      family: "regional_setting",
      assignmentMode: "hybrid",
      adjacentTags: ["undead_war_torn_region_setting", "urban_setting"],
      appliesWhen: expect.arrayContaining([
        "Use when the creature is naturally retrieved for an undead-ruled state, necromantic civil order, corpse-backed labor system, or other organized deathless society rather than an isolated tomb or graveyard.",
      ]),
    }));
    const gothicHorrorLandSetting = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "creature" && tag.tag === "gothic_horror_land_setting");
    expect(gothicHorrorLandSetting).toEqual(expect.objectContaining({
      family: "regional_setting",
      assignmentMode: "hybrid",
      adjacentTags: ["graveyard_setting", "folk_horror"],
      description: expect.stringContaining("Ustalav"),
    }));
    const truthReveal = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "truth_reveal");
    expect(truthReveal).toEqual(expect.objectContaining({
      family: "revelation",
      assignmentMode: "hybrid",
      adjacentTags: ["magic_detection", "memory_manipulation"],
      appliesWhen: expect.arrayContaining([
        "The spell's retrieval value comes from exposing deception, forcing truthful answers, or stripping away false presentation.",
      ]),
    }));
    const spellTracking = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "tracking");
    expect(spellTracking).toEqual(expect.objectContaining({
      family: "reconnaissance",
      assignmentMode: "hybrid",
      adjacentTags: ["scouting", "navigation"],
      appliesWhen: expect.arrayContaining([
        "The spell is naturally retrieved to find a named target, trace a quarry, or point the caster toward a specific creature, object, or place.",
      ]),
    }));
    const hazardRevelation = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "hazard_revelation");
    expect(hazardRevelation).toEqual(expect.objectContaining({
      family: "revelation",
      assignmentMode: "hybrid",
      adjacentTags: ["magic_detection", "scouting"],
      appliesWhen: expect.arrayContaining([
        "The spell is naturally retrieved to uncover traps, hidden dangers, secret magical wards, or dangerous concealed features in a location.",
      ]),
    }));
    const proceduralBypass = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "hazard" && tag.tag === "procedural_bypass");
    expect(proceduralBypass).toEqual(expect.objectContaining({
      family: "countermeasure_profile",
      assignmentMode: "hybrid",
      adjacentTags: ["physical_disarm", "false_safe_route"],
      appliesWhen: expect.arrayContaining([
        "The clean answer is learning and executing the hazard's safe procedure, sequence, or pattern rather than destroying it.",
      ]),
    }));
    const contaminationCleanupCountermeasure = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "hazard" && tag.tag === "contamination_cleanup_countermeasure");
    expect(contaminationCleanupCountermeasure).toEqual(expect.objectContaining({
      family: "countermeasure_profile",
      assignmentMode: "hybrid",
      adjacentTags: ["quarantine_containment_countermeasure", "source_cleanup_countermeasure"],
      appliesWhen: expect.arrayContaining([
        "The hazard is naturally retrieved because cleansing tainted ground, polluted air, cursed runoff, spores, or lingering residue is a core answer path.",
      ]),
    }));
    const environmentalHazard = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "hazard" && tag.tag === "environmental_hazard");
    expect(environmentalHazard).toEqual(expect.objectContaining({
      family: "environmental_danger",
      assignmentMode: "composite",
      compositeOfAnyTags: expect.arrayContaining(["fire_hazard", "contamination_hazard", "cursefield_hazard"]),
    }));
    const perceptionHazard = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "hazard" && tag.tag === "perception_hazard");
    expect(perceptionHazard).toEqual(expect.objectContaining({
      family: "perception_control",
      assignmentMode: "composite",
      compositeOfAnyTags: ["navigation_disruption", "illusion_assault", "false_safe_route"],
    }));
    const guardingHazard = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "hazard" && tag.tag === "guarding_hazard");
    expect(guardingHazard).toEqual(expect.objectContaining({
      family: "function",
      assignmentMode: "composite",
      compositeOfAnyTags: ["alarm", "barrier_lockdown", "sentinel_guardian"],
    }));
    const crewMember = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "creature" && tag.tag === "crew_member");
    expect(crewMember).toEqual(expect.objectContaining({
      family: "cohort_role",
      assignmentMode: "editorial",
      adjacentTags: ["nautical_setting", "escort_npc"],
    }));

    const habitatFamily = DERIVED_TAG_ONTOLOGY_FAMILIES.find((family) => family.category === "creature" && family.family === "habitat_setting");
    expect(habitatFamily?.description).toContain("habitat tags");
    const combatRoleFamily = DERIVED_TAG_ONTOLOGY_FAMILIES.find((family) => family.category === "creature" && family.family === "combat_role");
    expect(combatRoleFamily?.description).toContain("encounter assembly");
    const cohortRoleFamily = DERIVED_TAG_ONTOLOGY_FAMILIES.find((family) => family.category === "creature" && family.family === "cohort_role");
    expect(cohortRoleFamily?.description).toContain("roster-construction");

    const groupedCatalog = groupDerivedTagOntology({
      families: DERIVED_TAG_ONTOLOGY_FAMILIES,
      tags: DERIVED_TAG_ONTOLOGY_TAGS,
    });
    const groupedTransformation = groupedCatalog.find((entry) => entry.category === "spell" && entry.family === "transformation");
    expect(groupedTransformation?.tags).toEqual(expect.arrayContaining([
      expect.objectContaining({
        value: "transformation",
        assignmentMode: "composite",
        compositeOfAnyTags: ["battle_form", "animal_form", "elemental_form"],
      }),
    ]));
    const groupedRevelation = groupedCatalog.find((entry) => entry.category === "spell" && entry.family === "revelation");
    expect(groupedRevelation?.axis).toBe("utility");
    expect(groupedRevelation?.tags).toEqual(expect.arrayContaining([
      expect.objectContaining({
        value: "revelation",
        assignmentMode: "composite",
      }),
    ]));
    const groupedConsultation = groupedCatalog.find((entry) => entry.category === "spell" && entry.family === "consultation");
    expect(groupedConsultation?.axis).toBe("utility");
    expect(groupedConsultation?.tags).toEqual(expect.arrayContaining([
      expect.objectContaining({
        value: "consultation",
        assignmentMode: "composite",
      }),
      expect.objectContaining({
        value: "omen_guidance",
        assignmentMode: "hybrid",
      }),
    ]));
    const groupedSpellAccessBypass = groupedCatalog.find((entry) => entry.category === "spell" && entry.family === "access_bypass");
    expect(groupedSpellAccessBypass?.axis).toBe("utility");
    expect(groupedSpellAccessBypass?.tags).toEqual(expect.arrayContaining([
      expect.objectContaining({
        value: "lock_bypass",
        assignmentMode: "hybrid",
      }),
      expect.objectContaining({
        value: "barrier_bypass",
        assignmentMode: "hybrid",
      }),
      expect.objectContaining({
        value: "mechanism_manipulation",
        assignmentMode: "hybrid",
      }),
    ]));
    const groupedEquipmentAccessBypass = groupedCatalog.find((entry) => entry.category === "equipment" && entry.family === "access_bypass");
    expect(groupedEquipmentAccessBypass?.axis).toBe("utility");
    expect(groupedEquipmentAccessBypass?.tags).toEqual(expect.arrayContaining([
      expect.objectContaining({
        value: "lock_bypass",
        assignmentMode: "deterministic",
      }),
      expect.objectContaining({
        value: "barrier_bypass",
        assignmentMode: "deterministic",
      }),
      expect.objectContaining({
        value: "mechanism_manipulation",
        assignmentMode: "deterministic",
      }),
    ]));
    const groupedEquipmentPlayPattern = groupedCatalog.find((entry) => entry.category === "equipment" && entry.family === "play_pattern");
    expect(groupedEquipmentPlayPattern?.axis).toBe("party_role");
    expect(groupedEquipmentPlayPattern?.tags).toEqual(expect.arrayContaining([
      expect.objectContaining({
        value: "shield_support",
        assignmentMode: "hybrid",
      }),
      expect.objectContaining({
        value: "action_economy_support",
        assignmentMode: "hybrid",
      }),
    ]));
    const groupedEquipmentOffensiveProfile = groupedCatalog.find((entry) => entry.category === "equipment" && entry.family === "offensive_profile");
    expect(groupedEquipmentOffensiveProfile?.axis).toBe("effect");
    expect(groupedEquipmentOffensiveProfile?.tags).toEqual(expect.arrayContaining([
      expect.objectContaining({
        value: "burst_damage",
        assignmentMode: "hybrid",
      }),
      expect.objectContaining({
        value: "anti_caster_disruption",
        assignmentMode: "hybrid",
      }),
    ]));
    const groupedHazardEnvironmentalDanger = groupedCatalog.find((entry) => entry.category === "hazard" && entry.family === "environmental_danger");
    expect(groupedHazardEnvironmentalDanger?.tags).toEqual(expect.arrayContaining([
      expect.objectContaining({
        value: "environmental_hazard",
        assignmentMode: "composite",
      }),
      expect.objectContaining({
        value: "contamination_hazard",
        assignmentMode: "hybrid",
      }),
    ]));
    const groupedCreatureCohortRole = groupedCatalog.find((entry) => entry.category === "creature" && entry.family === "cohort_role");
    expect(groupedCreatureCohortRole?.axis).toBe("encounter");
    expect(groupedCreatureCohortRole?.tags).toEqual(expect.arrayContaining([
      expect.objectContaining({
        value: "crew_member",
        assignmentMode: "editorial",
      }),
      expect.objectContaining({
        value: "patrol_member",
        assignmentMode: "editorial",
      }),
    ]));
    const groupedCombatRole = groupedCatalog.find((entry) => entry.category === "creature" && entry.family === "combat_role");
    expect(groupedCombatRole?.tags).toEqual(expect.arrayContaining([
      expect.objectContaining({
        value: "brute_combatant",
        description: expect.any(String),
        assignmentMode: "hybrid",
        adjacentTags: ["defender_combatant", "artillery_combatant"],
      }),
      expect.objectContaining({
        value: "artillery_combatant",
        description: expect.any(String),
        assignmentMode: "hybrid",
      }),
      expect.objectContaining({
        value: "support_combatant",
        description: expect.any(String),
        assignmentMode: "hybrid",
      }),
    ]));

    const equipmentPurpose = groupedCatalog.find((entry) => entry.category === "equipment" && entry.family === "purpose");
    expect(equipmentPurpose).toBeUndefined();
  });

  it("authors category-scoped ontology with explicit family hierarchy before flattening", () => {
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.category).toBe("creature");
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.families.setting.description).toContain("Legacy umbrella family");
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.families.habitat_setting.description).toContain("habitat tags");
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.families.combat_role.description).toContain("tactical");
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.families.cohort_role.description).toContain("roster-construction");
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.families.scene_role.description).toContain("immediate-scenario");
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.families.social_role.description).toContain("outside one immediate encounter slot");
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.families.corruption_profile.description).toContain("corruption and taint");
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.families.visual_motif.description).toContain("visual motifs");
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.families.genre_motif.description).toContain("genre-tone");
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.families.story_motif.description).toContain("plot-driving motifs");

    const urbanSetting = CREATURE_DERIVED_TAG_ONTOLOGY.families.site_setting.tags.find((tag) => tag.tag === "urban_setting");
    expect(urbanSetting).toEqual(expect.objectContaining({
      tag: "urban_setting",
      assignmentMode: "editorial",
      adjacentTags: ["small_settlement_setting", "fortress_setting"],
    }));
    const authorityNpc = CREATURE_DERIVED_TAG_ONTOLOGY.families.social_role.tags.find((tag) => tag.tag === "authority_npc");
    expect(authorityNpc).toEqual(expect.objectContaining({
      tag: "authority_npc",
      assignmentMode: "editorial",
      adjacentTags: ["profession_npc", "civic_npc"],
      appliesWhen: expect.arrayContaining([
        "Formal office or rank is the main retrieval hook, even if the creature also serves as a civic_npc or enforcer_npc in the scene.",
      ]),
    }));
    const guideNpc = CREATURE_DERIVED_TAG_ONTOLOGY.families.social_role.tags.find((tag) => tag.tag === "guide_npc");
    expect(guideNpc).toEqual(expect.objectContaining({
      tag: "guide_npc",
      assignmentMode: "editorial",
      adjacentTags: ["profession_npc", "rural_setting"],
      appliesWhen: expect.arrayContaining([
        "Leading others through terrain, routes, borders, or dangerous travel spaces is central to the creature's world-facing identity.",
      ]),
    }));
    const infiltratorNpc = CREATURE_DERIVED_TAG_ONTOLOGY.families.scene_role.tags.find((tag) => tag.tag === "infiltrator_npc");
    expect(infiltratorNpc).toEqual(expect.objectContaining({
      tag: "infiltrator_npc",
      assignmentMode: "editorial",
      adjacentTags: ["enforcer_npc", "criminal_npc"],
      appliesWhen: expect.arrayContaining([
        "This tag answers the creature's immediate scenario function rather than its broader profession, faction post, or criminal affiliation.",
      ]),
    }));
    const guardianNpc = CREATURE_DERIVED_TAG_ONTOLOGY.families.scene_role.tags.find((tag) => tag.tag === "guardian_npc");
    expect(guardianNpc).toEqual(expect.objectContaining({
      tag: "guardian_npc",
      assignmentMode: "editorial",
      adjacentTags: ["enforcer_npc", "watcher_npc"],
    }));
    const reinforcementThreat = CREATURE_DERIVED_TAG_ONTOLOGY.families.threat_profile.tags.find((tag) => tag.tag === "reinforcement_threat");
    expect(reinforcementThreat).toEqual(expect.objectContaining({
      tag: "reinforcement_threat",
      assignmentMode: "hybrid",
      adjacentTags: ["spawn_creator", "commander_combatant"],
      appliesWhen: expect.arrayContaining([
        "Use when the creature's main prep significance is that it adds bodies, activates subordinates, or sharply force-multiplies nearby allies.",
      ]),
    }));
    const bruteCombatant = CREATURE_DERIVED_TAG_ONTOLOGY.families.combat_role.tags.find((tag) => tag.tag === "brute_combatant");
    expect(bruteCombatant).toEqual(expect.objectContaining({
      tag: "brute_combatant",
      assignmentMode: "hybrid",
      adjacentTags: ["defender_combatant", "artillery_combatant"],
    }));
    const crewMemberAuthored = CREATURE_DERIVED_TAG_ONTOLOGY.families.cohort_role.tags.find((tag) => tag.tag === "crew_member");
    expect(crewMemberAuthored).toEqual(expect.objectContaining({
      tag: "crew_member",
      assignmentMode: "editorial",
      adjacentTags: ["nautical_setting", "escort_npc"],
    }));
    const tricksterMischief = CREATURE_DERIVED_TAG_ONTOLOGY.families.genre_motif.tags.find((tag) => tag.tag === "trickster_mischief");
    expect(tricksterMischief).toEqual(expect.objectContaining({
      tag: "trickster_mischief",
      assignmentMode: "hybrid",
      adjacentTags: ["carnival_show", "disguised_pretender"],
      appliesWhen: expect.arrayContaining([
        "Pranks, baiting humor, whimsical menace, or deliberate trickster conduct are a central retrieval hook.",
      ]),
    }));
    const dreamNightmare = CREATURE_DERIVED_TAG_ONTOLOGY.families.genre_motif.tags.find((tag) => tag.tag === "dream_nightmare");
    expect(dreamNightmare).toEqual(expect.objectContaining({
      tag: "dream_nightmare",
      assignmentMode: "editorial",
      adjacentTags: ["dreamlands_setting", "cosmic_dread"],
      doesNotApplyWhen: expect.arrayContaining([
        "Dreamlands placement alone is better captured by dreamlands_setting.",
      ]),
    }));
    const folkHorror = CREATURE_DERIVED_TAG_ONTOLOGY.families.genre_motif.tags.find((tag) => tag.tag === "folk_horror");
    expect(folkHorror).toEqual(expect.objectContaining({
      tag: "folk_horror",
      assignmentMode: "editorial",
      adjacentTags: ["rural_setting", "funerary_mourning"],
      appliesWhen: expect.arrayContaining([
        "The creature evokes old-country fear, harvest rites gone wrong, scarecrow dread, witchcraft omen, or taboo-laden local folklore.",
      ]),
    }));
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.families.visual_motif.tags.map((tag) => tag.tag)).toEqual(expect.arrayContaining([
      "mask_motif",
      "mirror_motif",
      "living_toy",
      "living_artwork",
    ]));
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.families.story_motif.tags.map((tag) => tag.tag)).toEqual(expect.arrayContaining([
      "prophecy_omen",
      "corrupted_sacred",
      "vengeful_tragedy",
      "paranoia_surveillance",
      "disguised_pretender",
      "courtly_pageantry",
      "decadence_decline",
      "ancestral_legacy",
      "seasonal_festival",
      "apocalypse_ruin",
      "forbidden_knowledge",
      "cursed_transformation",
      "obsession_fixation",
      "occult_conspiracy",
    ]));
    const prophecyOmen = CREATURE_DERIVED_TAG_ONTOLOGY.families.story_motif.tags.find((tag) => tag.tag === "prophecy_omen");
    expect(prophecyOmen).toEqual(expect.objectContaining({
      tag: "prophecy_omen",
      assignmentMode: "editorial",
      adjacentTags: ["apocalypse_ruin", "ancestral_legacy"],
    }));
    const occultConspiracy = CREATURE_DERIVED_TAG_ONTOLOGY.families.story_motif.tags.find((tag) => tag.tag === "occult_conspiracy");
    expect(occultConspiracy).toEqual(expect.objectContaining({
      tag: "occult_conspiracy",
      assignmentMode: "editorial",
      adjacentTags: ["paranoia_surveillance", "forbidden_knowledge"],
      doesNotApplyWhen: expect.arrayContaining([
        "The stronger fit is ritual_ceremony, paranoia_surveillance, or forbidden_knowledge rather than hidden-network manipulation.",
      ]),
    }));
    const telepathicCommunication = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "equipment" && tag.tag === "telepathic_communication");
    expect(telepathicCommunication).toEqual(expect.objectContaining({
      family: "communication",
      assignmentMode: "deterministic",
      adjacentTags: ["signaling", "message_delivery"],
      appliesWhen: expect.arrayContaining([
        "The item's retrieval value comes from silent psychic coordination, mind-to-mind speech, or communication that bypasses ordinary sound.",
      ]),
    }));
    const equipmentTranslationSupport = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "equipment" && tag.tag === "translation_support");
    expect(equipmentTranslationSupport).toEqual(expect.objectContaining({
      family: "communication",
      assignmentMode: "deterministic",
      adjacentTags: ["telepathic_communication", "message_delivery"],
      appliesWhen: expect.arrayContaining([
        "The item's retrieval value comes from understanding foreign languages, translating speech, or decoding otherwise unreadable text or symbols.",
      ]),
    }));
    const spellTelepathicCommunication = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "telepathic_communication");
    expect(spellTelepathicCommunication).toEqual(expect.objectContaining({
      family: "communication",
      assignmentMode: "hybrid",
      description: expect.stringContaining("mind-to-mind communication"),
    }));
    const equipmentSourceRevelation = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "equipment" && tag.tag === "source_revelation");
    expect(equipmentSourceRevelation).toEqual(expect.objectContaining({
      family: "resolution",
      assignmentMode: "deterministic",
      adjacentTags: ["source_cleanup", "contamination_cleanup"],
      appliesWhen: expect.arrayContaining([
        "The item's retrieval value comes from finding or confirming the hidden source of a curse, contamination, outbreak, or spiritually tainted problem.",
      ]),
    }));
    const equipmentEnvironmentalAdaptation = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "equipment" && tag.tag === "environmental_adaptation");
    expect(equipmentEnvironmentalAdaptation).toEqual(expect.objectContaining({
      family: "expedition",
      assignmentMode: "deterministic",
      adjacentTags: ["aquatic_support", "camp_setup", "hazard_shielding"],
      appliesWhen: expect.arrayContaining([
        "The item's retrieval value comes from surviving punishing climate, altitude, breathing hazards, immersion pressure, or similar expedition environments.",
      ]),
    }));
    const spellScryingProtection = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "scrying_protection");
    expect(spellScryingProtection).toEqual(expect.objectContaining({
      family: "security",
      assignmentMode: "hybrid",
      adjacentTags: ["alarm", "countermagic"],
      appliesWhen: expect.arrayContaining([
        "The spell is naturally retrieved to keep plans, sanctums, identities, or conversations hidden from magical spying.",
      ]),
    }));

    const flattened = flattenDerivedTagAuthoredCategoryOntology(CREATURE_DERIVED_TAG_ONTOLOGY);
    expect(flattened.families).toContainEqual(expect.objectContaining({
      category: "creature",
      family: "setting",
    }));
    expect(flattened.families).toContainEqual(expect.objectContaining({
      category: "creature",
      family: "site_setting",
    }));
    expect(flattened.families).toContainEqual(expect.objectContaining({
      category: "creature",
      family: "social_role",
    }));
    expect(flattened.families).toContainEqual(expect.objectContaining({
      category: "creature",
      family: "scene_role",
    }));
    expect(flattened.families).toContainEqual(expect.objectContaining({
      category: "creature",
      family: "cohort_role",
    }));
    expect(flattened.families).toContainEqual(expect.objectContaining({
      category: "creature",
      family: "story_motif",
    }));
    expect(flattened.tags).toContainEqual(expect.objectContaining({
      category: "creature",
      family: "site_setting",
      tag: "urban_setting",
    }));
    expect(flattened.tags).toContainEqual(expect.objectContaining({
      category: "creature",
      family: "combat_role",
      tag: "artillery_combatant",
    }));
    expect(flattened.tags).toContainEqual(expect.objectContaining({
      category: "creature",
      family: "cohort_role",
      tag: "crew_member",
    }));
  });
});
