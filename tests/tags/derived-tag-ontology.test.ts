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

    const urbanSetting = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "creature" && tag.tag === "urban_setting");
    expect(urbanSetting).toEqual(expect.objectContaining({
      family: "site_setting",
      assignmentMode: "editorial",
      appliesWhen: expect.arrayContaining([
        "The creature is primarily framed as belonging in city or sewer encounter spaces.",
      ]),
      adjacentTags: ["small_settlement_setting", "fortress_setting"],
    }));
    const truthReveal = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "truth_reveal");
    expect(truthReveal).toEqual(expect.objectContaining({
      family: "communication",
      assignmentMode: "hybrid",
      adjacentTags: ["magic_detection", "memory_manipulation"],
      appliesWhen: expect.arrayContaining([
        "The spell's retrieval value comes from exposing deception, forcing truthful answers, or stripping away false presentation.",
      ]),
    }));
    const spellTracking = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "tracking");
    expect(spellTracking).toEqual(expect.objectContaining({
      family: "communication",
      assignmentMode: "hybrid",
      adjacentTags: ["scouting", "navigation"],
      appliesWhen: expect.arrayContaining([
        "The spell is naturally retrieved to find a named target, trace a quarry, or point the caster toward a specific creature, object, or place.",
      ]),
    }));
    const hazardRevelation = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "hazard_revelation");
    expect(hazardRevelation).toEqual(expect.objectContaining({
      family: "communication",
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

    const habitatFamily = DERIVED_TAG_ONTOLOGY_FAMILIES.find((family) => family.category === "creature" && family.family === "habitat_setting");
    expect(habitatFamily?.description).toContain("habitat tags");
    const combatRoleFamily = DERIVED_TAG_ONTOLOGY_FAMILIES.find((family) => family.category === "creature" && family.family === "combat_role");
    expect(combatRoleFamily?.description).toContain("encounter assembly");

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
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.families.encounter_role.description).toContain("scene-slot");
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.families.social_role.description).toContain("outside one immediate encounter slot");

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
        "Formal office or rank is the main retrieval hook, even if the creature also serves as a civic_npc or combatant_npc in the scene.",
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
    const infiltratorNpc = CREATURE_DERIVED_TAG_ONTOLOGY.families.encounter_role.tags.find((tag) => tag.tag === "infiltrator_npc");
    expect(infiltratorNpc).toEqual(expect.objectContaining({
      tag: "infiltrator_npc",
      assignmentMode: "editorial",
      adjacentTags: ["combatant_npc", "criminal_npc"],
      appliesWhen: expect.arrayContaining([
        "This tag answers the creature's immediate scenario function rather than its broader profession, faction post, or criminal affiliation.",
      ]),
    }));
    const summonerCommander = CREATURE_DERIVED_TAG_ONTOLOGY.families.threat_profile.tags.find((tag) => tag.tag === "summoner_commander");
    expect(summonerCommander).toEqual(expect.objectContaining({
      tag: "summoner_commander",
      assignmentMode: "hybrid",
      adjacentTags: ["spawn_creator", "commander_combatant"],
      appliesWhen: expect.arrayContaining([
        "Use when the creature's main prep significance is that it adds bodies, coordinates allies, or sharply elevates nearby minions.",
      ]),
    }));
    const bruteCombatant = CREATURE_DERIVED_TAG_ONTOLOGY.families.combat_role.tags.find((tag) => tag.tag === "brute_combatant");
    expect(bruteCombatant).toEqual(expect.objectContaining({
      tag: "brute_combatant",
      assignmentMode: "hybrid",
      adjacentTags: ["defender_combatant", "artillery_combatant"],
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
      family: "communication",
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
  });
});
