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
        category: "affliction",
        family: "impact",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "mental_impairment", description: expect.any(String) }),
          expect.objectContaining({ value: "mobility_impairment", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "setting",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "freshwater_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "swamp_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "coastal_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "island_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "plains_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "canyon_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "graveyard_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "ruins_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "temple_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "fortress_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "wasteland_setting", description: expect.any(String) }),
          expect.objectContaining({ value: "volcanic_setting", description: expect.any(String) }),
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
        { value: "trap", count: 3 },
        { value: "magical", count: 2 },
        { value: "mechanical", count: 2 },
      ],
    });

    expect(service.listFilterValues({
      field: "derivedTags",
      category: "equipment",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["alarm", "beneficial", "offensive", "climbing", "lock_bypass", "mental_recovery", "carry_support", "tracking", "anti_tracking", "restraint_escape", "restraint_capture"]));
    expect(service.listFilterValues({
      field: "derivedTags",
      category: "spell",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["alarm", "disguise", "social_infiltration"]));
    expect(service.listFilterValues({
      field: "derivedTags",
      category: "hazard",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["alarm", "mobility_impairment", "restraint_capture"]));
    expect(service.listFilterValues({
      field: "derivedTags",
      category: "affliction",
    }).values.map((entry) => entry.value)).toEqual(["mental_impairment", "mobility_impairment"]);
    expect(service.listFilterValues({
      field: "derivedTags",
      category: "creature",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["aquatic_setting", "freshwater_setting", "coastal_setting", "temple_setting"]));

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
        { value: "trap", count: 3 },
        { value: "haunt", count: 1 },
      ],
    });

    expect(service.listFilterValues({
      field: "publicationTitle",
      category: "spell",
    })).toEqual({
      field: "publicationTitle",
      values: [
        { value: "Pathfinder Player Core", count: 4 },
        { value: "Pathfinder Secrets of Magic", count: 2 },
        { value: "Pathfinder Dark Archive", count: 1 },
      ],
    });

    expect(service.listFilterValues({
      field: "traditions",
      category: "spell",
    })).toEqual({
      field: "traditions",
      values: [
        { value: "occult", count: 5 },
        { value: "arcane", count: 4 },
        { value: "primal", count: 1 },
      ],
    });

    expect(service.listFilterValues({
      field: "spellKinds",
    category: "spell",
    })).toEqual({
      field: "spellKinds",
      values: [
        { value: "focus", count: 1 },
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
      field: "damageTypes",
      category: "equipment",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["positive", "slashing"]));

    expect(service.listFilterValues({
      field: "itemCategory",
      category: "equipment",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["backpack", "consumable", "equipment", "weapon"]));

    expect(service.listFilterValues({
      field: "rarity",
      category: "creature",
    }).values.map((entry) => entry.value)).toEqual(["common", "rare", "uncommon", "unique"]);

    expect(service.listFilterValues({
      field: "size",
      category: "creature",
    }).values.map((entry) => entry.value)).toEqual(["med", "lg", "sm"]);

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
        { value: "Spells", count: 4 },
        { value: "Spells SRD", count: 3 },
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
        { value: "rule", count: 6 },
        { value: "hazard", count: 3 },
      ],
    });

    expect(() => service.listFilterValues({
      field: "traits",
      scopes: [{ category: "feat", subcategories: ["action"] }],
    })).toThrow(/does not belong to category "feat"/i);
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
