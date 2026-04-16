import path from "node:path";
import { DatabaseSync } from "node:sqlite";

import { afterEach, describe, expect, it } from "vitest";

import { Pf2eDataService } from "../../src/data/service.js";
import { RankingConfigStore } from "../../src/search/ranking-config.js";
import {
  createCapturingEmbeddingProviderFactory,
  createEmbeddingBatchTrackingProviderFactory,
  createFakeEmbeddingProviderFactory,
  initializeGitFixture,
  loadTestService,
  openPreparedTestService,
  TEST_HASH_EMBEDDING,
  writeJson,
} from "../helpers/pf2e-fixture.js";
import {
  cleanupCreatedRoots,
  createFixture,
  createHardFilterFixture,
} from "../helpers/pf2e-service-fixture.js";


describe("Pf2eDataService / Rules and Vocabulary", () => {
  const createdRoots: string[] = [];

  afterEach(async () => {
    await cleanupCreatedRoots(createdRoots);
  });

  it("supports batch lookup, record fetch, and unified rule graph retrieval", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const lookups = service.lookupMany([{ name: "Refocus" }, { name: "Deep Focus" }], { coreOnly: true });
    expect(lookups.map((result) => result.match?.name)).toEqual(["Refocus", "Deep Focus"]);
    expect(lookups.map((result) => result.matchType)).toEqual(["exact", "exact"]);

    const records = service.getRecordsByKeys(["actions:action-refocus-1", "feats-srd:feat1"]);
    expect(records.map((record) => record.name)).toEqual(["Refocus", "Deep Focus"]);

    const defaultGraph = service.getRuleGraph(["conditionitems:Blinded"], { maxOutgoingPerPrimary: 5 });
    expect(defaultGraph.outgoing.records.map((record) => record.name)).toEqual(["Dazzled", "Seek"]);
    expect(defaultGraph.backlinks.records).toHaveLength(0);
    expect(defaultGraph.edges.every((edge) => edge.direction === "outgoing")).toBe(true);

    const combinedGraph = service.getRuleGraph(["feats-srd:feat1", "actions:action-refocus-1"], {
      includeOutgoing: true,
      includeBacklinks: true,
      maxOutgoingPerPrimary: 5,
      maxBacklinksPerPrimary: 10,
    });
    expect(combinedGraph.outgoing.records.map((record) => record.name)).toEqual(["Refocus"]);
    expect(combinedGraph.backlinks.records.map((record) => record.name)).toEqual(["Deep Focus", "Meditative Well"]);
    expect(combinedGraph.edges).toHaveLength(3);

    const backlinksOnly = service.getRuleGraph(["actions:action-refocus-1"], {
      includeOutgoing: false,
      includeBacklinks: true,
      maxBacklinksPerPrimary: 10,
    });
    expect(backlinksOnly.outgoing.records).toHaveLength(0);
    expect(backlinksOnly.backlinks.records.map((record) => record.name)).toEqual(["Deep Focus", "Meditative Well"]);
    expect(backlinksOnly.backlinks.edges.every((edge) => edge.direction === "backlink")).toBe(true);
    expect(backlinksOnly.backlinks.records.some((record) => record.type === "spell")).toBe(false);

    const emptyGraph = service.getRuleGraph(["actions:action-refocus-1"], {
      includeOutgoing: false,
      includeBacklinks: false,
    });
    expect(emptyGraph.outgoing.records).toHaveLength(0);
    expect(emptyGraph.backlinks.records).toHaveLength(0);
    expect(emptyGraph.edges).toHaveLength(0);
  });

  it("exposes indexed search vocabulary for agent planning", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const vocabulary = service.getSearchVocabulary({ traitLimitPerCategory: 4 });
    expect(vocabulary.categories.map((entry) => entry.value)).toEqual(expect.arrayContaining(["creature", "spell", "rule", "feat"]));
    expect(vocabulary.subcategories.map((entry) => entry.value)).toEqual(expect.arrayContaining(["condition", "action", "trap"]));
    expect(vocabulary.subcategories.map((entry) => entry.value)).not.toContain("primal");
    expect(vocabulary.rarities.map((entry) => entry.value)).toEqual(expect.arrayContaining(["common", "uncommon", "rare", "unique"]));
    expect(vocabulary.sizes.map((entry) => entry.value)).toEqual(expect.arrayContaining(["med", "sm", "lg"]));
    expect(vocabulary.traditions.map((entry) => entry.value)).toContain("primal");
    expect(vocabulary.spellKinds.map((entry) => entry.value)).toContain("focus");
    expect(vocabulary.commonTraitsByCategory.find((entry) => entry.category === "creature")?.traits.length).toBeGreaterThan(0);
    expect(vocabulary.commonDerivedTagsByCategory.find((entry) => entry.category === "equipment")?.tags.length).toBeGreaterThan(0);
    expect(vocabulary.commonDerivedTagsByCategory.find((entry) => entry.category === "spell")?.tags.length).toBeGreaterThan(0);
    expect(vocabulary.commonDerivedTagsByCategory.find((entry) => entry.category === "hazard")?.tags.length).toBeGreaterThan(0);
    expect(vocabulary.commonDerivedTagsByCategory.find((entry) => entry.category === "affliction")?.tags.length).toBeGreaterThan(0);
    expect(vocabulary.derivedTagCatalog).toEqual(expect.arrayContaining([
      expect.objectContaining({
        category: "equipment",
        family: "function",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "mental_recovery", description: expect.any(String) }),
          expect.objectContaining({ value: "energy_resistance", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "purpose",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "tracking", description: expect.any(String) }),
          expect.objectContaining({ value: "anti_tracking", description: expect.any(String) }),
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
        family: "infiltration",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "disguise", description: expect.any(String) }),
          expect.objectContaining({ value: "social_infiltration", description: expect.any(String) }),
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
        family: "reconnaissance",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "scouting", description: expect.any(String) }),
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
        family: "function",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "alarm", description: expect.any(String) }),
          expect.objectContaining({ value: "restraint_capture", description: expect.any(String) }),
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
          expect.objectContaining({ value: "fire_hazard", description: expect.any(String) }),
          expect.objectContaining({ value: "poison_hazard", description: expect.any(String) }),
          expect.objectContaining({ value: "respiratory_hazard", description: expect.any(String) }),
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
        category: "affliction",
        family: "impact",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "mental_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "mobility_impairment", description: expect.any(String) }),
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
          expect.objectContaining({ value: "coastal_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "astral_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "ethereal_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "plane_of_fire_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "plane_of_air_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "plane_of_water_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "plane_of_earth_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "elemental_plane_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "first_world_setting", description: expect.any(String) }),
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
          expect.objectContaining({ value: "graveyard_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "ruins_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "temple_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "fortress_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "wasteland_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "volcanic_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "small_settlement_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "sky_setting", description: expect.any(String) }),
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
        ]),
      }),
    ]));
  });

  it("lists live filter values across supported fields and scopes", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    expect(service.listFilterValues({
      field: "traits",
      category: "hazard",
      subcategory: "trap",
    })).toEqual({
      field: "traits",
      values: [
        { value: "trap", count: 16 },
        { value: "magical", count: 9 },
        { value: "mechanical", count: 8 },
      ],
    });

    expect(service.listFilterValues({
      field: "derivedTags",
      category: "equipment",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["alarm", "beneficial", "offensive", "climbing", "lock_bypass", "mental_recovery", "carry_support", "tracking", "anti_tracking", "restraint_escape", "restraint_capture", "mobility_impairment", "sensory_impairment", "mental_impairment", "sedation", "spell_payload"]));
    expect(service.listFilterValues({
      field: "derivedTags",
      category: "spell",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining([
      "alarm",
      "countermagic",
      "disguise",
      "eidolon_support",
      "initiative_support",
      "mobility",
      "navigation",
      "persistent_damage",
      "social_infiltration",
      "mental_impairment",
      "sensory_impairment",
      "forced_movement",
      "restraint_capture",
    ]));
    expect(service.listFilterValues({
      field: "derivedTags",
      category: "hazard",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["alarm", "mobility_impairment", "restraint_capture", "respiratory_hazard", "water_hazard", "spawned_attackers", "navigation_disruption", "illusion_assault", "overhead_strike"]));
    expect(service.listFilterValues({
      field: "derivedTags",
      category: "affliction",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining([
      "healing_suppression",
      "mental_impairment",
      "mobility_impairment",
      "sedation",
      "respiratory_impairment",
      "transformative_corruption",
      "rot_decay",
      "infestation_implant",
      "compulsion",
    ]));
    expect(service.listFilterValues({
      field: "derivedTags",
      category: "creature",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["aquatic_setting", "freshwater_setting", "coastal_setting", "astral_setting", "ethereal_setting", "plane_of_fire_setting", "plane_of_air_setting", "plane_of_water_setting", "plane_of_earth_setting", "elemental_plane_setting", "first_world_setting", "boneyard_setting", "heaven_setting", "nirvana_setting", "elysium_setting", "upper_plane_setting", "hell_setting", "abyss_setting", "abaddon_setting", "lower_plane_setting", "axis_setting", "shadow_plane_setting", "maelstrom_setting", "cosmic_framework_setting", "island_setting", "jungle_setting", "sky_setting", "temple_setting", "small_settlement_setting", "rural_setting", "civic_npc", "combatant_npc", "carnival_show", "living_toy", "living_artwork", "trickster_chaos"]));

    expect(service.listFilterValues({
      field: "families",
      category: "creature",
    })).toEqual({
      field: "families",
      values: [
        { value: "ghost", count: 1 },
        { value: "lich", count: 1 },
        { value: "mythic", count: 1 },
        { value: "seafarer", count: 1 },
        { value: "vampire", count: 1 },
      ],
    });

    expect(service.listFilterValues({
      field: "subcategories",
      category: "hazard",
    })).toEqual({
      field: "subcategories",
      values: [
        { value: "trap", count: 16 },
        { value: "haunt", count: 3 },
      ],
    });

    expect(service.listFilterValues({
      field: "publicationTitle",
      category: "spell",
    })).toEqual({
      field: "publicationTitle",
      values: [
        { value: "Pathfinder Player Core", count: 16 },
        { value: "Pathfinder Dark Archive", count: 4 },
        { value: "Pathfinder Secrets of Magic", count: 3 },
        { value: "Pathfinder Rage of Elements", count: 1 },
      ],
    });

    expect(service.listFilterValues({
      field: "sourceCategory",
      category: "creature",
    }).values.map((entry) => entry.value)).toEqual(["core", "rules", "adventure"]);

    expect(service.listFilterValues({
      field: "traditions",
      category: "spell",
    })).toEqual({
      field: "traditions",
      values: [
        { value: "arcane", count: 15 },
        { value: "occult", count: 15 },
        { value: "divine", count: 9 },
        { value: "primal", count: 8 },
      ],
    });

    expect(service.listFilterValues({
      field: "spellKinds",
    category: "spell",
    })).toEqual({
      field: "spellKinds",
      values: [
        { value: "cantrip", count: 1 },
        { value: "focus", count: 1 },
      ],
    });

    expect(service.listFilterValues({
      field: "saveType",
      category: "spell",
    })).toEqual({
      field: "saveType",
      values: [
        { value: "fortitude", count: 1 },
        { value: "reflex", count: 1 },
        { value: "will", count: 1 },
      ],
    });

    expect(service.listFilterValues({
      field: "areaType",
      category: "spell",
    })).toEqual({
      field: "areaType",
      values: [
        { value: "burst", count: 1 },
        { value: "cone", count: 1 },
      ],
    });

    expect(service.listFilterValues({
      field: "durationUnit",
      category: "spell",
    })).toEqual({
      field: "durationUnit",
      values: [
        { value: "minute", count: 2 },
        { value: "unlimited", count: 1 },
      ],
    });

    expect(service.listFilterValues({
      field: "weaponGroup",
      category: "equipment",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["bomb", "sword"]));

    expect(service.listFilterValues({
      field: "usage",
      category: "equipment",
    }).values.map((entry) => entry.value)).toEqual(["held-in-one-hand"]);

    expect(service.listFilterValues({
      field: "hands",
      category: "equipment",
    }).values.map((entry) => entry.value)).toEqual(["1"]);

    expect(service.listFilterValues({
      field: "actionCost",
      category: "spell",
    }).values.map((entry) => entry.value)).toEqual(["2"]);

    expect(service.listFilterValues({
      field: "rangeValue",
      category: "spell",
    }).values.map((entry) => entry.value)).toEqual(["30", "60"]);

    expect(service.listFilterValues({
      field: "areaValue",
      category: "spell",
    }).values.map((entry) => entry.value)).toEqual(["30", "5"]);

    expect(service.listFilterValues({
      field: "sustained",
      category: "spell",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["false", "true"]));

    expect(service.listFilterValues({
      field: "basicSave",
      category: "spell",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["false", "true"]));

    expect(service.listFilterValues({
      field: "damageTypes",
      category: "equipment",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["positive", "slashing"]));

    expect(service.listFilterValues({
      field: "itemCategory",
      category: "equipment",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["backpack", "consumable", "equipment", "weapon"]));

    expect(service.listFilterValues({
      field: "baseItem",
      category: "equipment",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["alchemical-bomb", "longsword"]));

    expect(service.listFilterValues({
      field: "rarity",
      category: "creature",
    }).values.map((entry) => entry.value)).toEqual(["common", "uncommon", "rare", "unique"]);

    expect(service.listFilterValues({
      field: "size",
      category: "creature",
    }).values.map((entry) => entry.value)).toEqual(["med", "lg", "sm", "huge", "tiny"]);

    expect(service.listFilterValues({
      field: "sources",
      category: "creature",
    }).values.map((entry) => entry.value)).toEqual(["core", "rules", "adventure"]);

    expect(service.listFilterValues({
      field: "packs",
      category: "spell",
    })).toEqual({
      field: "packs",
      values: [
        { value: "Spells", count: 15 },
        { value: "Spells SRD", count: 9 },
      ],
    });

    expect(service.listFilterValues({
      field: "categories",
      scopes: [
        { category: "hazards", subcategories: ["traps"] },
        { category: "rules", subcategories: ["conditions"] },
      ],
    })).toEqual({
      field: "categories",
      values: [
        { value: "hazard", count: 16 },
        { value: "rule", count: 6 },
      ],
    });

    expect(() => service.listFilterValues({
      field: "traits",
      scopes: [{ category: "feat", subcategories: ["action"] }],
    })).toThrow(/does not belong to category "feat"/i);
  });

  it("discovers live actor and item metric keys and scalar values through filter-value listing", async () => {
    const fixture = await createHardFilterFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    expect(service.listFilterValues({
      field: "actorMetrics",
      category: "creature",
      metricPrefix: "ability",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining([
      "ability.cha.mod",
      "ability.dex.mod",
      "ability.int.mod",
      "ability.str.mod",
    ]));

    expect(service.listFilterValues({
      field: "actorMetrics",
      category: "creature",
      metricPrefix: "speed",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining([
      "speed.fly.value",
      "speed.land.value",
    ]));

    expect(service.listFilterValues({
      field: "actorMetrics",
      category: "creature",
      metricPrefix: "sense",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining([
      "sense.scent.range",
    ]));

    expect(service.listFilterValues({
      field: "actorMetrics",
      category: "creature",
      metric: "save.best",
    })).toEqual({
      field: "actorMetrics",
      values: [
        { value: "fort", count: 1 },
        { value: "ref", count: 1 },
        { value: "will", count: 1 },
      ],
    });

    expect(service.listFilterValues({
      field: "actorMetrics",
      category: "creature",
      metric: "skill.arcana.proficient",
    })).toEqual({
      field: "actorMetrics",
      values: [
        { value: "false", count: 2 },
        { value: "true", count: 1 },
      ],
    });

    expect(() => service.listFilterValues({
      field: "actorMetrics",
      category: "creature",
      metric: "ability.int.mod",
    })).toThrow(/text and boolean metrics/i);

    expect(service.listFilterValues({
      field: "actorMetrics",
      category: "hazard",
      metricPrefix: "stealth",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining([
      "stealth.dc",
      "stealth.mod",
    ]));

    expect(service.listFilterValues({
      field: "actorMetrics",
      category: "hazard",
      metricPrefix: "disable",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining([
      "disable.dc.max",
      "disable.dc.min",
      "disable.crafting.dc.min",
      "disable.crafting.rank.min",
      "disable.thievery.dc.min",
      "disable.thievery.rank.min",
      "disable.religion.dc.min",
      "disable.religion.rank.min",
    ]));

    expect(service.listFilterValues({
      field: "actorMetrics",
      category: "hazard",
      metric: "save.best",
    })).toEqual({
      field: "actorMetrics",
      values: [
        { value: "fort", count: 1 },
        { value: "will", count: 1 },
      ],
    });

    expect(service.listFilterValues({
      field: "itemMetrics",
      category: "equipment",
      metricPrefix: "weapon",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining([
      "weapon.damage_dice",
      "weapon.damage_die_faces",
      "weapon.range_increment",
      "weapon.reload",
    ]));

    expect(service.listFilterValues({
      field: "itemMetrics",
      category: "equipment",
      metricPrefix: "shield",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining([
      "shield.ac_bonus",
      "shield.bt",
      "shield.hardness",
      "shield.hp",
    ]));

    expect(service.listFilterValues({
      field: "itemMetrics",
      category: "equipment",
      metricPrefix: "armor",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining([
      "armor.ac_bonus",
      "armor.check_penalty",
      "armor.dex_cap",
      "armor.speed_penalty",
      "armor.strength",
    ]));

    expect(service.listFilterValues({
      field: "senses",
      category: "creature",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["darkvision", "scent"]));

    expect(service.listFilterValues({
      field: "disableSkills",
      category: "hazard",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["crafting", "religion", "thievery"]));

    expect(service.listFilterValues({
      field: "isComplex",
      category: "hazard",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["false", "true"]));

    expect(() => service.listFilterValues({
      field: "itemMetrics",
      category: "equipment",
      metric: "weapon.reload",
    })).toThrow(/text and boolean metrics/i);
  });

  it("collects rule question context without synthesis", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const result = service.collectRuleQuestionContext({
      rules: ["Refocus"],
      includeBacklinks: true,
      maxOutgoingPerPrimary: 5,
      maxBacklinksPerPrimary: 5,
    });

    expect(result.primary[0]?.match?.name).toBe("Refocus");
    expect(result.outgoing.records).toHaveLength(0);
    expect(result.backlinks.records.map((record) => record.name)).toEqual(["Deep Focus", "Meditative Well"]);
    expect(result.edges).toHaveLength(2);
  });

  it("collects natural-language rule question context using canonical graph edges", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const result = service.collectRuleQuestionContext({
      question: "How does Deep Focus interplay with Refocus?",
      includeBacklinks: true,
      maxOutgoingPerPrimary: 5,
      maxBacklinksPerPrimary: 5,
    });

    expect(result.primary.map((entry) => entry.match?.name)).toEqual(["Deep Focus", "Refocus"]);
    expect(result.outgoing.records.map((record) => record.name)).toEqual(["Refocus"]);
    expect(result.backlinks.records.map((record) => record.name)).toEqual(["Deep Focus", "Meditative Well"]);
    expect(result.edges).toHaveLength(3);
  });
});
