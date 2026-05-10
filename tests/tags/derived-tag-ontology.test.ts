import { describe, expect, it } from "vitest";

import { groupDerivedTagOntology } from "../../src/tags/runtime/publication/catalog.js";
import {
  getCurrentDerivedTagOntologyConcepts,
  getCurrentDerivedTagOntologyFamilies,
  getCurrentDerivedTagOntologyTags,
  getCurrentDerivedTagOntologyTranslations,
  getCurrentPublicDerivedTagOntologyFamilies,
  getCurrentPublicDerivedTagOntologyTags,
} from "../../src/tags/runtime.js";

function matcherAnyString(): unknown {
  return expect.any(String);
}

function matcherArrayContaining(values: unknown[]): unknown {
  return expect.arrayContaining(values);
}

function matcherObjectContaining(value: Record<string, unknown>): unknown {
  return expect.objectContaining(value);
}

function matcherStringContaining(value: string): unknown {
  return expect.stringContaining(value);
}

describe("derived tag ontology", () => {
  it("publishes unique category-scoped families and tags with assignment modes", () => {
    const familiesByCategory = new Map<string, Set<string>>();
    const tagsByCategory = new Map<string, Map<string, string>>();

    for (const family of getCurrentDerivedTagOntologyFamilies()) {
      const categoryFamilies = familiesByCategory.get(family.category) ?? new Set<string>();
      expect(categoryFamilies.has(family.family)).toBe(false);
      categoryFamilies.add(family.family);
      familiesByCategory.set(family.category, categoryFamilies);
    }

    for (const tag of getCurrentDerivedTagOntologyTags()) {
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

    for (const tag of getCurrentDerivedTagOntologyTags()) {
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
    const spellTransformation = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "transformation",
    );
    expect(spellTransformation).toEqual(
      matcherObjectContaining({
        tag: "transformation",
        assignmentMode: "composite",
        compositeOfAnyTags: ["battle_form", "animal_form", "elemental_form"],
      }),
    );
    const spellReconnaissance = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "reconnaissance",
    );
    expect(spellReconnaissance).toEqual(
      matcherObjectContaining({
        family: "reconnaissance",
        assignmentMode: "composite",
        compositeOfAnyTags: ["scouting", "tracking", "scouting_summons"],
      }),
    );
    const spellRevelation = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "revelation",
    );
    expect(spellRevelation).toEqual(
      matcherObjectContaining({
        family: "revelation",
        assignmentMode: "composite",
        compositeOfAnyTags: [
          "magic_detection",
          "invisibility_reveal",
          "truth_reveal",
          "curse_revelation",
          "hazard_revelation",
        ],
      }),
    );
    const spellWayfinding = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "wayfinding",
    );
    expect(spellWayfinding).toEqual(
      matcherObjectContaining({
        family: "wayfinding",
        assignmentMode: "composite",
        compositeOfAnyTags: ["navigation", "tracking", "long_range_teleport", "planar_travel"],
      }),
    );
    const spellSecurity = getCurrentDerivedTagOntologyTags().find((tag) => tag.category === "spell" && tag.tag === "security");
    expect(spellSecurity).toEqual(
      matcherObjectContaining({
        family: "security",
        assignmentMode: "composite",
        compositeOfAnyTags: ["alarm", "scrying_protection", "protective_ward", "countermagic"],
      }),
    );
    const spellConsultation = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "consultation",
    );
    expect(spellConsultation).toEqual(
      matcherObjectContaining({
        family: "consultation",
        assignmentMode: "composite",
        compositeOfAnyTags: ["lore_consultation", "problem_diagnosis", "omen_guidance"],
      }),
    );
    const spellLockBypass = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "lock_bypass",
    );
    expect(spellLockBypass).toEqual(
      matcherObjectContaining({
        family: "access_bypass",
        assignmentMode: "hybrid",
        adjacentTags: ["trap_bypass", "barrier_bypass"],
        appliesWhen: matcherArrayContaining([
          "The spell is naturally retrieved to unlock, unseal, or open a secured entry point, door, chest, manacle, or similar closure.",
        ]),
      }),
    );
    const spellMechanismManipulation = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "mechanism_manipulation",
    );
    expect(spellMechanismManipulation).toEqual(
      matcherObjectContaining({
        family: "access_bypass",
        assignmentMode: "hybrid",
        adjacentTags: ["lock_bypass", "trap_bypass"],
        appliesWhen: matcherArrayContaining([
          "The spell is naturally retrieved to operate a lever, button, latch, control panel, pressure surface, or similar mechanism from a safe or unusual position.",
        ]),
      }),
    );
    const equipmentStealthSupport = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "equipment" && tag.tag === "stealth_support",
    );
    expect(equipmentStealthSupport).toEqual(
      matcherObjectContaining({
        family: "infiltration",
        assignmentMode: "deterministic",
      }),
    );
    const spellStealthSupport = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "stealth_support",
    );
    expect(spellStealthSupport).toEqual(
      matcherObjectContaining({
        family: "infiltration",
        assignmentMode: "hybrid",
        adjacentTags: ["concealment", "silencing"],
        appliesWhen: matcherArrayContaining([
          "The spell is naturally retrieved to help a creature move quietly, avoid notice, pass unseen, or keep a covert approach from drawing attention.",
        ]),
      }),
    );
    const spellInfiltration = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "infiltration",
    );
    expect(spellInfiltration).toEqual(
      matcherObjectContaining({
        family: "infiltration",
        assignmentMode: "composite",
        compositeOfAnyTags: ["stealth_support", "disguise", "social_infiltration"],
      }),
    );
    const equipmentBarrierBypass = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "equipment" && tag.tag === "barrier_bypass",
    );
    expect(equipmentBarrierBypass).toEqual(
      matcherObjectContaining({
        family: "access_bypass",
        assignmentMode: "deterministic",
      }),
    );
    const equipmentMechanismManipulation = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "equipment" && tag.tag === "mechanism_manipulation",
    );
    expect(equipmentMechanismManipulation).toEqual(
      matcherObjectContaining({
        family: "access_bypass",
        assignmentMode: "deterministic",
      }),
    );
    const equipmentReconnaissance = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "equipment" && tag.tag === "reconnaissance",
    );
    expect(equipmentReconnaissance).toEqual(
      matcherObjectContaining({
        family: "reconnaissance",
        assignmentMode: "composite",
        compositeOfAnyTags: ["scouting", "illumination", "surveillance_recording", "tracking", "anti_tracking"],
      }),
    );
    const equipmentSecurity = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "equipment" && tag.tag === "security",
    );
    expect(equipmentSecurity).toEqual(
      matcherObjectContaining({
        family: "security",
        assignmentMode: "composite",
        compositeOfAnyTags: ["alarm", "scrying_protection", "tamper_evidence"],
      }),
    );
    const equipmentBurstDamage = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "equipment" && tag.tag === "burst_damage",
    );
    expect(equipmentBurstDamage).toEqual(
      matcherObjectContaining({
        family: "offensive_profile",
        assignmentMode: "hybrid",
        adjacentTags: ["crowd_clearing", "persistent_damage"],
      }),
    );
    const scoutSupport = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "equipment" && tag.tag === "scout_support",
    );
    expect(scoutSupport).toEqual(
      matcherObjectContaining({
        family: "party_role",
        assignmentMode: "hybrid",
      }),
    );
    const shieldSupport = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "equipment" && tag.tag === "shield_support",
    );
    expect(shieldSupport).toEqual(
      matcherObjectContaining({
        family: "play_pattern",
        assignmentMode: "hybrid",
      }),
    );
    const cursebreakingResolution = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "affliction" && tag.tag === "cursebreaking_resolution",
    );
    expect(cursebreakingResolution).toEqual(
      matcherObjectContaining({
        family: "resolution_profile",
        assignmentMode: "hybrid",
        adjacentTags: ["countermagic_resolution", "ritual_appeasement_resolution"],
      }),
    );
    const spellResolution = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "resolution",
    );
    expect(spellResolution).toEqual(
      matcherObjectContaining({
        family: "resolution",
        assignmentMode: "composite",
        compositeOfAnyTags: [
          "curse_removal",
          "exorcism",
          "sanctification",
          "ritual_appeasement",
          "quarantine_containment",
          "contamination_cleanup",
          "source_revelation",
          "source_cleanup",
        ],
      }),
    );
    const spellCommunication = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "communication",
    );
    expect(spellCommunication).toEqual(
      matcherObjectContaining({
        family: "communication",
        assignmentMode: "composite",
        compositeOfAnyTags: ["signaling", "telepathic_communication", "message_delivery", "translation_support"],
      }),
    );
    const equipmentResolution = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "equipment" && tag.tag === "resolution",
    );
    expect(equipmentResolution).toEqual(
      matcherObjectContaining({
        family: "resolution",
        assignmentMode: "composite",
        compositeOfAnyTags: [
          "curse_removal",
          "sanctification",
          "ritual_appeasement",
          "source_revelation",
          "quarantine_containment",
          "contamination_cleanup",
          "source_cleanup",
        ],
      }),
    );
    const fungalInfested = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "creature" && tag.tag === "fungal_infested",
    );
    expect(fungalInfested).toEqual(
      matcherObjectContaining({
        family: "corruption_profile",
        assignmentMode: "hybrid",
        adjacentTags: ["disease_vector", "body_horror"],
      }),
    );

    const urbanSetting = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "creature" && tag.tag === "urban_setting",
    );
    expect(urbanSetting).toEqual(
      matcherObjectContaining({
        family: "site_setting",
        assignmentMode: "editorial",
        appliesWhen: matcherArrayContaining([
          "The creature is primarily framed as belonging in city or sewer encounter spaces.",
        ]),
        adjacentTags: ["small_settlement_setting", "fortress_setting"],
      }),
    );
    const organizedUndeadSocietySetting = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "creature" && tag.tag === "organized_undead_society_setting",
    );
    expect(organizedUndeadSocietySetting).toEqual(
      matcherObjectContaining({
        family: "regional_setting",
        assignmentMode: "hybrid",
        adjacentTags: ["undead_war_torn_region_setting", "urban_setting"],
        appliesWhen: matcherArrayContaining([
          "Use when the creature is naturally retrieved for an undead-ruled state, necromantic civil order, corpse-backed labor system, or other organized deathless society rather than an isolated tomb or graveyard.",
        ]),
      }),
    );
    const gothicHorrorLandSetting = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "creature" && tag.tag === "gothic_horror_land_setting",
    );
    expect(gothicHorrorLandSetting).toEqual(
      matcherObjectContaining({
        family: "regional_setting",
        assignmentMode: "hybrid",
        adjacentTags: ["graveyard_setting", "folk_horror"],
        description: matcherStringContaining("Ustalav"),
      }),
    );
    const truthReveal = getCurrentDerivedTagOntologyTags().find((tag) => tag.category === "spell" && tag.tag === "truth_reveal");
    expect(truthReveal).toEqual(
      matcherObjectContaining({
        family: "revelation",
        assignmentMode: "hybrid",
        adjacentTags: ["magic_detection", "memory_manipulation"],
        appliesWhen: matcherArrayContaining([
          "The spell's retrieval value comes from exposing deception, forcing truthful answers, or stripping away false presentation.",
        ]),
      }),
    );
    const spellTracking = getCurrentDerivedTagOntologyTags().find((tag) => tag.category === "spell" && tag.tag === "tracking");
    expect(spellTracking).toEqual(
      matcherObjectContaining({
        family: "reconnaissance",
        assignmentMode: "hybrid",
        adjacentTags: ["scouting", "navigation"],
        appliesWhen: matcherArrayContaining([
          "The spell is naturally retrieved to find a named target, trace a quarry, or point the caster toward a specific creature, object, or place.",
        ]),
      }),
    );
    const hazardRevelation = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "hazard_revelation",
    );
    expect(hazardRevelation).toEqual(
      matcherObjectContaining({
        family: "revelation",
        assignmentMode: "hybrid",
        adjacentTags: ["magic_detection", "scouting"],
        appliesWhen: matcherArrayContaining([
          "The spell is naturally retrieved to uncover traps, hidden dangers, secret magical wards, or dangerous concealed features in a location.",
        ]),
      }),
    );
    const proceduralBypass = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "hazard" && tag.tag === "procedural_bypass",
    );
    expect(proceduralBypass).toEqual(
      matcherObjectContaining({
        family: "countermeasure_profile",
        assignmentMode: "hybrid",
        adjacentTags: ["physical_disarm", "false_safe_route"],
        appliesWhen: matcherArrayContaining([
          "The clean answer is learning and executing the hazard's safe procedure, sequence, or pattern rather than destroying it.",
        ]),
      }),
    );
    const spellExpedition = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "expedition",
    );
    expect(spellExpedition).toEqual(
      matcherObjectContaining({
        family: "expedition",
        assignmentMode: "composite",
        compositeOfAnyTags: [
          "navigation",
          "flight",
          "aquatic_support",
          "sustenance",
          "field_shelter",
          "environmental_adaptation",
          "wayfinding",
        ],
      }),
    );
    const hazardSourceTracing = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "hazard" && tag.tag === "source_tracing",
    );
    expect(hazardSourceTracing).toEqual(
      matcherObjectContaining({
        family: "problem_shape",
        assignmentMode: "hybrid",
        adjacentTags: ["observation_first", "source_cleanup_countermeasure"],
        appliesWhen: matcherArrayContaining([
          "The hazard is naturally retrieved because identifying the source object, origin point, or contamination engine is a major part of solving it.",
        ]),
      }),
    );
    const contaminationCleanupCountermeasure = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "hazard" && tag.tag === "contamination_cleanup_countermeasure",
    );
    expect(contaminationCleanupCountermeasure).toEqual(
      matcherObjectContaining({
        family: "countermeasure_profile",
        assignmentMode: "hybrid",
        adjacentTags: ["quarantine_containment_countermeasure", "source_cleanup_countermeasure"],
        appliesWhen: matcherArrayContaining([
          "The hazard is naturally retrieved because cleansing tainted ground, polluted air, cursed runoff, spores, or lingering residue is a core answer path.",
        ]),
      }),
    );
    const environmentalHazard = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "hazard" && tag.tag === "environmental_hazard",
    );
    expect(environmentalHazard).toEqual(
      matcherObjectContaining({
        family: "environmental_danger",
        assignmentMode: "composite",
        compositeOfAnyTags: matcherArrayContaining(["fire_hazard", "contamination_hazard", "cursefield_hazard"]),
      }),
    );
    const perceptionHazard = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "hazard" && tag.tag === "perception_hazard",
    );
    expect(perceptionHazard).toEqual(
      matcherObjectContaining({
        family: "perception_control",
        assignmentMode: "composite",
        compositeOfAnyTags: ["navigation_disruption", "illusion_assault", "false_safe_route"],
      }),
    );
    const guardingHazard = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "hazard" && tag.tag === "guarding_hazard",
    );
    expect(guardingHazard).toEqual(
      matcherObjectContaining({
        family: "function",
        assignmentMode: "composite",
        compositeOfAnyTags: ["alarm", "barrier_lockdown", "sentinel_guardian"],
      }),
    );
    const crewMember = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "creature" && tag.tag === "crew_member",
    );
    expect(crewMember).toEqual(
      matcherObjectContaining({
        family: "cohort_role",
        assignmentMode: "editorial",
        adjacentTags: ["nautical_setting", "escort_npc"],
      }),
    );

    const habitatFamily = getCurrentDerivedTagOntologyFamilies().find(
      (family) => family.category === "creature" && family.family === "habitat_setting",
    );
    expect(habitatFamily?.description).toContain("habitat tags");
    const combatRoleFamily = getCurrentDerivedTagOntologyFamilies().find(
      (family) => family.category === "creature" && family.family === "combat_role",
    );
    expect(combatRoleFamily?.description).toContain("encounter assembly");
    const cohortRoleFamily = getCurrentDerivedTagOntologyFamilies().find(
      (family) => family.category === "creature" && family.family === "cohort_role",
    );
    expect(cohortRoleFamily?.description).toContain("roster-construction");

    const groupedCatalog = groupDerivedTagOntology({
      families: getCurrentDerivedTagOntologyFamilies(),
      tags: getCurrentDerivedTagOntologyTags(),
    });
    const groupedTransformation = groupedCatalog.find(
      (entry) => entry.category === "spell" && entry.family === "transformation",
    );
    expect(groupedTransformation?.tags).toEqual(
      matcherArrayContaining([
        matcherObjectContaining({
          value: "transformation",
          assignmentMode: "composite",
          compositeOfAnyTags: ["battle_form", "animal_form", "elemental_form"],
        }),
      ]),
    );
    const groupedRevelation = groupedCatalog.find(
      (entry) => entry.category === "spell" && entry.family === "revelation",
    );
    expect(groupedRevelation?.axis).toBe("utility");
    expect(groupedRevelation?.tags).toEqual(
      matcherArrayContaining([
        matcherObjectContaining({
          value: "revelation",
          assignmentMode: "composite",
        }),
      ]),
    );
    const groupedConsultation = groupedCatalog.find(
      (entry) => entry.category === "spell" && entry.family === "consultation",
    );
    expect(groupedConsultation?.axis).toBe("utility");
    expect(groupedConsultation?.tags).toEqual(
      matcherArrayContaining([
        matcherObjectContaining({
          value: "consultation",
          assignmentMode: "composite",
        }),
        matcherObjectContaining({
          value: "omen_guidance",
          assignmentMode: "hybrid",
        }),
      ]),
    );
    const groupedSpellAccessBypass = groupedCatalog.find(
      (entry) => entry.category === "spell" && entry.family === "access_bypass",
    );
    expect(groupedSpellAccessBypass?.axis).toBe("utility");
    expect(groupedSpellAccessBypass?.tags).toEqual(
      matcherArrayContaining([
        matcherObjectContaining({
          value: "lock_bypass",
          assignmentMode: "hybrid",
        }),
        matcherObjectContaining({
          value: "barrier_bypass",
          assignmentMode: "hybrid",
        }),
        matcherObjectContaining({
          value: "mechanism_manipulation",
          assignmentMode: "hybrid",
        }),
      ]),
    );
    const groupedEquipmentAccessBypass = groupedCatalog.find(
      (entry) => entry.category === "equipment" && entry.family === "access_bypass",
    );
    expect(groupedEquipmentAccessBypass?.axis).toBe("utility");
    expect(groupedEquipmentAccessBypass?.tags).toEqual(
      matcherArrayContaining([
        matcherObjectContaining({
          value: "lock_bypass",
          assignmentMode: "deterministic",
        }),
        matcherObjectContaining({
          value: "barrier_bypass",
          assignmentMode: "deterministic",
        }),
        matcherObjectContaining({
          value: "mechanism_manipulation",
          assignmentMode: "deterministic",
        }),
      ]),
    );
    const groupedEquipmentPlayPattern = groupedCatalog.find(
      (entry) => entry.category === "equipment" && entry.family === "play_pattern",
    );
    expect(groupedEquipmentPlayPattern?.axis).toBe("party_role");
    expect(groupedEquipmentPlayPattern?.tags).toEqual(
      matcherArrayContaining([
        matcherObjectContaining({
          value: "shield_support",
          assignmentMode: "hybrid",
        }),
        matcherObjectContaining({
          value: "action_economy_support",
          assignmentMode: "hybrid",
        }),
      ]),
    );
    const groupedEquipmentOffensiveProfile = groupedCatalog.find(
      (entry) => entry.category === "equipment" && entry.family === "offensive_profile",
    );
    expect(groupedEquipmentOffensiveProfile?.axis).toBe("effect");
    expect(groupedEquipmentOffensiveProfile?.tags).toEqual(
      matcherArrayContaining([
        matcherObjectContaining({
          value: "burst_damage",
          assignmentMode: "hybrid",
        }),
        matcherObjectContaining({
          value: "anti_caster_disruption",
          assignmentMode: "hybrid",
        }),
      ]),
    );
    const groupedHazardEnvironmentalDanger = groupedCatalog.find(
      (entry) => entry.category === "hazard" && entry.family === "environmental_danger",
    );
    expect(groupedHazardEnvironmentalDanger?.tags).toEqual(
      matcherArrayContaining([
        matcherObjectContaining({
          value: "environmental_hazard",
          assignmentMode: "composite",
        }),
        matcherObjectContaining({
          value: "contamination_hazard",
          assignmentMode: "hybrid",
        }),
      ]),
    );
    const groupedCreatureCohortRole = groupedCatalog.find(
      (entry) => entry.category === "creature" && entry.family === "cohort_role",
    );
    expect(groupedCreatureCohortRole?.axis).toBe("encounter");
    expect(groupedCreatureCohortRole?.tags).toEqual(
      matcherArrayContaining([
        matcherObjectContaining({
          value: "crew_member",
          assignmentMode: "editorial",
        }),
        matcherObjectContaining({
          value: "patrol_member",
          assignmentMode: "editorial",
        }),
      ]),
    );
    const groupedCombatRole = groupedCatalog.find(
      (entry) => entry.category === "creature" && entry.family === "combat_role",
    );
    expect(groupedCombatRole?.tags).toEqual(
      matcherArrayContaining([
        matcherObjectContaining({
          value: "brute_combatant",
          description: matcherAnyString(),
          assignmentMode: "hybrid",
          adjacentTags: ["defender_combatant", "artillery_combatant"],
        }),
        matcherObjectContaining({
          value: "artillery_combatant",
          description: matcherAnyString(),
          assignmentMode: "hybrid",
        }),
        matcherObjectContaining({
          value: "support_combatant",
          description: matcherAnyString(),
          assignmentMode: "hybrid",
        }),
      ]),
    );

    const equipmentPurpose = groupedCatalog.find(
      (entry) => entry.category === "equipment" && entry.family === "purpose",
    );
    expect(equipmentPurpose).toBeUndefined();
  });

  it("publishes canonical concept metadata and translation records alongside stable tag ids", () => {
    const poisonRemediation = getCurrentDerivedTagOntologyConcepts().find((concept) => concept.id === "poison_remediation");
    expect(poisonRemediation).toEqual(
      matcherObjectContaining({
        id: "poison_remediation",
        label: "poison_remediation",
        schemaKind: "operational",
        domainId: "poison",
        operation: "remediate",
      }),
    );

    const spellAntiPoison = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "anti_poison",
    );
    expect(spellAntiPoison).toEqual(
      matcherObjectContaining({
        family: "support",
        tag: "anti_poison",
        label: "poison_remediation",
        canonicalConceptId: "poison_remediation",
        translationStatus: "provisional",
        schemaKind: "operational",
        domainId: "poison",
        operation: "remediate",
      }),
    );

    const spellRevelation = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "revelation",
    );
    expect(spellRevelation).toEqual(
      matcherObjectContaining({
        label: "problem_discovery",
        canonicalConceptId: "problem_discovery",
        schemaKind: "aggregate",
      }),
    );

    expect(
      getCurrentDerivedTagOntologyTranslations().find(
        (translation) => translation.currentCategory === "equipment" && translation.currentTag === "beneficial",
      ),
    ).toEqual(
      matcherObjectContaining({
        translationStatus: "dropped",
      }),
    );
    expect(
      getCurrentDerivedTagOntologyTags().some((tag) => tag.category === "equipment" && tag.tag === "beneficial"),
    ).toBe(false);
  });

  it("publishes a settled-only public ontology surface", () => {
    expect(getCurrentPublicDerivedTagOntologyFamilies().length).toBeGreaterThan(0);
    expect(getCurrentPublicDerivedTagOntologyTags().length).toBeGreaterThan(0);
    expect(getCurrentPublicDerivedTagOntologyTags().every((tag) => tag.translationStatus === "mapped")).toBe(true);
    expect(getCurrentPublicDerivedTagOntologyTags().some((tag) => tag.category === "spell")).toBe(false);
    expect(
      getCurrentPublicDerivedTagOntologyTags().some((tag) => tag.category === "affliction" && tag.tag === "community_outbreak"),
    ).toBe(true);
    expect(
      getCurrentPublicDerivedTagOntologyTags().some((tag) => tag.category === "equipment" && tag.tag === "anti_poison"),
    ).toBe(true);
  });

  it("publishes creature family and tag hierarchy through the canonical runtime surface", () => {
    const creatureFamilies = getCurrentDerivedTagOntologyFamilies().filter((family) => family.category === "creature");
    const creatureTags = getCurrentDerivedTagOntologyTags().filter((tag) => tag.category === "creature");

    expect(
      creatureFamilies.find((family) => family.family === "habitat_setting")?.description,
    ).toContain("habitat tags");
    expect(
      creatureFamilies.find((family) => family.family === "combat_role")?.description,
    ).toContain("tactical");
    expect(
      creatureFamilies.find((family) => family.family === "cohort_role")?.description,
    ).toContain("roster-construction");
    expect(
      creatureFamilies.find((family) => family.family === "scene_role")?.description,
    ).toContain("immediate-scenario");
    expect(
      creatureFamilies.find((family) => family.family === "social_role")?.description,
    ).toContain("outside one immediate encounter slot");
    expect(
      creatureFamilies.find((family) => family.family === "corruption_profile")?.description,
    ).toContain("corruption and taint");
    expect(
      creatureFamilies.find((family) => family.family === "visual_motif")?.description,
    ).toContain("visual motifs");
    expect(
      creatureFamilies.find((family) => family.family === "genre_motif")?.description,
    ).toContain("genre-tone");
    expect(
      creatureFamilies.find((family) => family.family === "story_motif")?.description,
    ).toContain("plot-driving motifs");

    const urbanSetting = creatureTags.find((tag) => tag.family === "site_setting" && tag.tag === "urban_setting");
    expect(urbanSetting).toEqual(
      matcherObjectContaining({
        tag: "urban_setting",
        assignmentMode: "editorial",
        adjacentTags: ["small_settlement_setting", "fortress_setting"],
      }),
    );
    const authorityNpc = creatureTags.find((tag) => tag.family === "social_role" && tag.tag === "authority_npc");
    expect(authorityNpc).toEqual(
      matcherObjectContaining({
        tag: "authority_npc",
        assignmentMode: "editorial",
        adjacentTags: ["profession_npc", "civic_npc"],
        appliesWhen: matcherArrayContaining([
          "Formal office or rank is the main retrieval hook, even if the creature also serves as a civic_npc or enforcer_npc in the scene.",
        ]),
      }),
    );
    const guideNpc = creatureTags.find((tag) => tag.family === "social_role" && tag.tag === "guide_npc");
    expect(guideNpc).toEqual(
      matcherObjectContaining({
        tag: "guide_npc",
        assignmentMode: "editorial",
        adjacentTags: ["profession_npc", "rural_setting"],
        appliesWhen: matcherArrayContaining([
          "Leading others through terrain, routes, borders, or dangerous travel spaces is central to the creature's world-facing identity.",
        ]),
      }),
    );
    const infiltratorNpc = creatureTags.find((tag) => tag.family === "scene_role" && tag.tag === "infiltrator_npc");
    expect(infiltratorNpc).toEqual(
      matcherObjectContaining({
        tag: "infiltrator_npc",
        assignmentMode: "editorial",
        adjacentTags: ["enforcer_npc", "criminal_npc"],
        appliesWhen: matcherArrayContaining([
          "This tag answers the creature's immediate scenario function rather than its broader profession, faction post, or criminal affiliation.",
        ]),
      }),
    );
    const guardianNpc = creatureTags.find((tag) => tag.family === "scene_role" && tag.tag === "guardian_npc");
    expect(guardianNpc).toEqual(
      matcherObjectContaining({
        tag: "guardian_npc",
        assignmentMode: "editorial",
        adjacentTags: ["enforcer_npc", "watcher_npc"],
      }),
    );
    const reinforcementThreat = creatureTags.find(
      (tag) => tag.family === "threat_profile" && tag.tag === "reinforcement_threat",
    );
    expect(reinforcementThreat).toEqual(
      matcherObjectContaining({
        tag: "reinforcement_threat",
        assignmentMode: "hybrid",
        adjacentTags: ["spawn_creator", "commander_combatant"],
        appliesWhen: matcherArrayContaining([
          "Use when the creature's main prep significance is that it adds bodies, activates subordinates, or sharply force-multiplies nearby allies.",
        ]),
      }),
    );
    const bruteCombatant = creatureTags.find((tag) => tag.family === "combat_role" && tag.tag === "brute_combatant");
    expect(bruteCombatant).toEqual(
      matcherObjectContaining({
        tag: "brute_combatant",
        assignmentMode: "hybrid",
        adjacentTags: ["defender_combatant", "artillery_combatant"],
      }),
    );
    const creatureGenreMotifTags = creatureTags.filter((tag) => tag.family === "genre_motif");
    const creatureStoryMotifTags = creatureTags.filter((tag) => tag.family === "story_motif");
    const creatureVisualMotifTags = creatureTags.filter((tag) => tag.family === "visual_motif");
    const crewMemberAuthored = creatureTags.find((tag) => tag.family === "cohort_role" && tag.tag === "crew_member");
    expect(crewMemberAuthored).toEqual(
      matcherObjectContaining({
        tag: "crew_member",
        assignmentMode: "editorial",
        adjacentTags: ["nautical_setting", "escort_npc"],
      }),
    );
    const tricksterMischief = creatureGenreMotifTags.find((tag) => tag.tag === "trickster_mischief");
    expect(tricksterMischief).toEqual(
      matcherObjectContaining({
        tag: "trickster_mischief",
        assignmentMode: "hybrid",
        adjacentTags: ["carnival_show", "disguised_pretender"],
        appliesWhen: matcherArrayContaining([
          "Pranks, baiting humor, whimsical menace, or deliberate trickster conduct are a central retrieval hook.",
        ]),
      }),
    );
    const dreamNightmare = creatureGenreMotifTags.find((tag) => tag.tag === "dream_nightmare");
    expect(dreamNightmare).toEqual(
      matcherObjectContaining({
        tag: "dream_nightmare",
        assignmentMode: "editorial",
        adjacentTags: ["dreamlands_setting", "cosmic_dread"],
        doesNotApplyWhen: matcherArrayContaining([
          "Dreamlands placement alone is better captured by dreamlands_setting.",
        ]),
      }),
    );
    const folkHorror = creatureGenreMotifTags.find((tag) => tag.tag === "folk_horror");
    expect(folkHorror).toEqual(
      matcherObjectContaining({
        tag: "folk_horror",
        assignmentMode: "editorial",
        adjacentTags: ["rural_setting", "funerary_mourning"],
        appliesWhen: matcherArrayContaining([
          "The creature evokes old-country fear, harvest rites gone wrong, scarecrow dread, witchcraft omen, or taboo-laden local folklore.",
        ]),
      }),
    );
    expect(creatureVisualMotifTags.map((tag) => tag.tag)).toEqual(
      matcherArrayContaining(["mask_motif", "mirror_motif", "living_toy", "living_artwork"]),
    );
    expect(creatureStoryMotifTags.map((tag) => tag.tag)).toEqual(
      matcherArrayContaining([
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
      ]),
    );
    const prophecyOmen = creatureStoryMotifTags.find((tag) => tag.tag === "prophecy_omen");
    expect(prophecyOmen).toEqual(
      matcherObjectContaining({
        tag: "prophecy_omen",
        assignmentMode: "editorial",
        adjacentTags: ["apocalypse_ruin", "ancestral_legacy"],
      }),
    );
    const occultConspiracy = creatureStoryMotifTags.find((tag) => tag.tag === "occult_conspiracy");
    expect(occultConspiracy).toEqual(
      matcherObjectContaining({
        tag: "occult_conspiracy",
        assignmentMode: "editorial",
        adjacentTags: ["paranoia_surveillance", "forbidden_knowledge"],
        doesNotApplyWhen: matcherArrayContaining([
          "The stronger fit is ritual_ceremony, paranoia_surveillance, or forbidden_knowledge rather than hidden-network manipulation.",
        ]),
      }),
    );
    const telepathicCommunication = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "equipment" && tag.tag === "telepathic_communication",
    );
    expect(telepathicCommunication).toEqual(
      matcherObjectContaining({
        family: "communication",
        assignmentMode: "deterministic",
        adjacentTags: ["signaling", "message_delivery"],
        appliesWhen: matcherArrayContaining([
          "The item's retrieval value comes from silent psychic coordination, mind-to-mind speech, or communication that bypasses ordinary sound.",
        ]),
      }),
    );
    const equipmentTranslationSupport = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "equipment" && tag.tag === "translation_support",
    );
    expect(equipmentTranslationSupport).toEqual(
      matcherObjectContaining({
        family: "communication",
        assignmentMode: "deterministic",
        adjacentTags: ["telepathic_communication", "message_delivery"],
        appliesWhen: matcherArrayContaining([
          "The item's retrieval value comes from understanding foreign languages, translating speech, or decoding otherwise unreadable text or symbols.",
        ]),
      }),
    );
    const spellTelepathicCommunication = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "telepathic_communication",
    );
    expect(spellTelepathicCommunication).toEqual(
      matcherObjectContaining({
        family: "communication",
        assignmentMode: "hybrid",
        description: matcherStringContaining("mind-to-mind communication"),
      }),
    );
    const spellAntiPoison = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "anti_poison",
    );
    expect(spellAntiPoison).toEqual(
      matcherObjectContaining({
        family: "support",
        assignmentMode: "hybrid",
        adjacentTags: ["affliction_cleanup", "anti_disease"],
      }),
    );
    const spellRitualAppeasement = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "ritual_appeasement",
    );
    expect(spellRitualAppeasement).toEqual(
      matcherObjectContaining({
        family: "resolution",
        assignmentMode: "hybrid",
        adjacentTags: ["sanctification", "exorcism"],
      }),
    );
    const equipmentSourceRevelation = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "equipment" && tag.tag === "source_revelation",
    );
    expect(equipmentSourceRevelation).toEqual(
      matcherObjectContaining({
        family: "resolution",
        assignmentMode: "deterministic",
        adjacentTags: ["source_cleanup", "contamination_cleanup"],
        appliesWhen: matcherArrayContaining([
          "The item's retrieval value comes from finding or confirming the hidden source of a curse, contamination, outbreak, or spiritually tainted problem.",
        ]),
      }),
    );
    const equipmentRitualAppeasement = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "equipment" && tag.tag === "ritual_appeasement",
    );
    expect(equipmentRitualAppeasement).toEqual(
      matcherObjectContaining({
        family: "resolution",
        assignmentMode: "deterministic",
        adjacentTags: ["ritual_support", "sanctification"],
        appliesWhen: matcherArrayContaining([
          "The item's retrieval value comes from helping perform offerings, appeasement rites, restitution rituals, or ceremonial observance meant to settle a supernatural grievance.",
        ]),
      }),
    );
    const equipmentEnvironmentalAdaptation = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "equipment" && tag.tag === "environmental_adaptation",
    );
    expect(equipmentEnvironmentalAdaptation).toEqual(
      matcherObjectContaining({
        family: "expedition",
        assignmentMode: "deterministic",
        adjacentTags: ["aquatic_support", "camp_setup", "hazard_shielding"],
        appliesWhen: matcherArrayContaining([
          "The item's retrieval value comes from surviving punishing climate, altitude, breathing hazards, immersion pressure, or similar expedition environments.",
        ]),
      }),
    );
    const spellScryingProtection = getCurrentDerivedTagOntologyTags().find(
      (tag) => tag.category === "spell" && tag.tag === "scrying_protection",
    );
    expect(spellScryingProtection).toEqual(
      matcherObjectContaining({
        family: "security",
        assignmentMode: "hybrid",
        adjacentTags: ["alarm", "countermagic"],
        appliesWhen: matcherArrayContaining([
          "The spell is naturally retrieved to keep plans, sanctums, identities, or conversations hidden from magical spying.",
        ]),
      }),
    );

    const creatureRuntimeFamilies = getCurrentDerivedTagOntologyFamilies().filter((family) => family.category === "creature");
    const creatureRuntimeTags = getCurrentDerivedTagOntologyTags().filter((tag) => tag.category === "creature");
    expect(creatureRuntimeFamilies).toContainEqual(
      matcherObjectContaining({
        category: "creature",
        family: "site_setting",
      }),
    );
    expect(creatureRuntimeFamilies).toContainEqual(
      matcherObjectContaining({
        category: "creature",
        family: "social_role",
      }),
    );
    expect(creatureRuntimeFamilies).toContainEqual(
      matcherObjectContaining({
        category: "creature",
        family: "scene_role",
      }),
    );
    expect(creatureRuntimeFamilies).toContainEqual(
      matcherObjectContaining({
        category: "creature",
        family: "cohort_role",
      }),
    );
    expect(creatureRuntimeFamilies).toContainEqual(
      matcherObjectContaining({
        category: "creature",
        family: "story_motif",
      }),
    );
    expect(creatureRuntimeTags).toContainEqual(
      matcherObjectContaining({
        category: "creature",
        family: "site_setting",
        tag: "urban_setting",
      }),
    );
    expect(creatureRuntimeTags).toContainEqual(
      matcherObjectContaining({
        category: "creature",
        family: "combat_role",
        tag: "artillery_combatant",
      }),
    );
    expect(creatureRuntimeTags).toContainEqual(
      matcherObjectContaining({
        category: "creature",
        family: "cohort_role",
        tag: "crew_member",
      }),
    );
  });
});
