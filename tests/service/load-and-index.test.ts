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


describe("Pf2eDataService / Load and Index", () => {
  const createdRoots: string[] = [];

  afterEach(async () => {
    await cleanupCreatedRoots(createdRoots);
  });

  it("fails loudly when the required sqlite-vec extension cannot be loaded", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    await expect(loadTestService(fixture, {
      vectorExtensionLoader: () => {
        throw new Error("simulated extension load failure");
      },
    })).rejects.toThrow(/Failed to load required sqlite-vec extension/);
  });

  it("uses batched semantic-only embedding text during rebuild", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    const indexPath = path.join(fixture.root, ".cache", "pf2e-index.sqlite");
    const tracking = {
      embedCalls: [] as string[],
      embedManyCalls: [] as string[][],
    };

    const service = await loadTestService(fixture, {
      indexPath,
      embeddingProviderFactory: createEmbeddingBatchTrackingProviderFactory(tracking, {
        provider: "hash",
        model: "batch-capture-model",
        revision: null,
        dimensions: 8,
      }),
    });
    service.close();

    expect(tracking.embedCalls).toHaveLength(0);
    expect(tracking.embedManyCalls.length).toBeGreaterThan(0);

    const shipCaptainText = tracking.embedManyCalls
      .flat()
      .find((text) => text.includes("Ship Captain"));
    expect(shipCaptainText).toBeDefined();
    expect(shipCaptainText).toContain("Deck Order 1");
    expect(shipCaptainText).toContain("auditory");
    expect(shipCaptainText).not.toContain("coordinates the crew around starboard rigging routines");
    expect(shipCaptainText).not.toContain("Deck Order 41");

    const db = new DatabaseSync(indexPath);
    const row = db.prepare("SELECT search_text AS searchText FROM records WHERE name = ?").get("Ship Captain") as { searchText: string } | undefined;
    db.close();

    expect(row?.searchText).toContain("Deck Order 41");
    expect(row?.searchText).toContain("auditory");
    expect(row?.searchText).not.toContain("coordinates the crew around starboard rigging routines");
  });

  it("derives canonical afflictions from staged nested items while keeping host search text compact", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    const indexPath = path.join(fixture.root, ".cache", "pf2e-index.sqlite");

    await import("node:fs/promises").then(({ mkdir }) => mkdir(
      path.join(fixture.root, "packs", "pf2e", "bestiary-family-ability-glossary", "ghoul"),
      { recursive: true },
    ));

    await writeJson(path.join(fixture.root, "packs", "pf2e", "bestiary-family-ability-glossary", "ghoul", "ghoul-fever.json"), {
      _id: "ghoulfeversource",
      name: "Ghoul Fever",
      type: "action",
      system: {
        category: "offensive",
        description: {
          value: "<p><strong>Saving Throw</strong> @Check[fortitude|dc:18]</p><p><strong>Stage 1</strong> @UUID[Compendium.pf2e.conditionitems.Item.Sickened]{Sickened 1} (1 day)</p><p><strong>Stage 2</strong> @UUID[Compendium.pf2e.conditionitems.Item.Unconscious] (1 day)</p>",
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        traits: {
          rarity: "common",
          value: ["disease"],
        },
      },
    });
    await writeJson(path.join(fixture.root, "packs", "pf2e", "equipment-srd", "lethargy-poison.json"), {
      _id: "lethargypoison",
      name: "Lethargy Poison",
      type: "consumable",
      system: {
        category: "poison",
        description: {
          value: "<p><strong>Saving Throw</strong> @Check[fortitude|dc:20]</p><p><strong>Maximum Duration</strong> 4 rounds</p><p><strong>Stage 1</strong> @UUID[Compendium.pf2e.conditionitems.Item.Slowed]{Slowed 1} (1 round)</p><p><strong>Stage 2</strong> @UUID[Compendium.pf2e.conditionitems.Item.Unconscious] (1 round)</p>",
        },
        level: {
          value: 2,
        },
        price: {
          value: {
            gp: 7,
          },
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        traits: {
          rarity: "common",
          value: ["injury", "poison", "sleep"],
        },
      },
    });
    await writeJson(path.join(fixture.root, "packs", "pf2e", "pathfinder-monster-core", "ghoul-brute.json"), {
      _id: "ghoulbrute",
      name: "Ghoul Brute",
      type: "npc",
      items: [
        {
          _id: "ghoulfever1",
          _stats: {
            compendiumSource: "Compendium.pf2e.bestiary-family-ability-glossary.Item.ghoulfeversource",
          },
          name: "Ghoul Fever",
          type: "action",
          system: {
            category: "offensive",
            slug: "ghoul-ghoul-fever",
            description: {
              value: "<p><strong>Saving Throw</strong> @Check[fortitude|dc:18]</p><p><strong>Stage 1</strong> @UUID[Compendium.pf2e.conditionitems.Item.Sickened]{Sickened 1} (1 day)</p><p><strong>Stage 2</strong> @UUID[Compendium.pf2e.conditionitems.Item.Unconscious] (1 day)</p>",
            },
            traits: {
              value: ["disease"],
            },
          },
        },
      ],
      system: {
        details: {
          level: {
            value: 2,
          },
          publication: {
            title: "Pathfinder Monster Core",
          },
          publicNotes: "<p>A diseased ghoul brute.</p>",
        },
        traits: {
          rarity: "common",
          value: ["undead"],
          size: {
            value: "med",
          },
        },
      },
    });
    await writeJson(path.join(fixture.root, "packs", "pf2e", "pathfinder-monster-core", "drow-poisoner.json"), {
      _id: "drowpoisoner",
      name: "Drow Poisoner",
      type: "npc",
      items: [
        {
          _id: "lethargypoison1",
          _stats: {
            compendiumSource: "Compendium.pf2e.equipment-srd.Item.lethargypoison",
          },
          name: "Lethargy Poison",
          type: "consumable",
          system: {
            category: "poison",
            slug: "lethargy-poison",
            description: {
              value: "<p><strong>Saving Throw</strong> @Check[fortitude|dc:20]</p><p><strong>Maximum Duration</strong> 4 rounds</p><p><strong>Stage 1</strong> @UUID[Compendium.pf2e.conditionitems.Item.Slowed]{Slowed 1} (1 round)</p><p><strong>Stage 2</strong> @UUID[Compendium.pf2e.conditionitems.Item.Unconscious] (1 round)</p>",
            },
            traits: {
              value: ["injury", "poison", "sleep"],
            },
          },
        },
        {
          _id: "sneakattack1",
          name: "Sneak Attack",
          type: "action",
          system: {
            category: "offensive",
            slug: "sneak-attack",
            description: {
              value: "<p>The drow poisoner deals an extra 2d6 precision damage to off-guard creatures.</p>",
            },
            traits: {
              value: [],
            },
          },
        },
      ],
      system: {
        details: {
          level: {
            value: 4,
          },
          publication: {
            title: "Pathfinder Monster Core",
          },
          publicNotes: "<p>A poison-using drow assassin.</p>",
        },
        traits: {
          rarity: "common",
          value: ["elf", "humanoid"],
          size: {
            value: "med",
          },
        },
      },
    });
    await writeJson(path.join(fixture.root, "packs", "pf2e", "pathfinder-monster-core", "zeal-damned-ghoul.json"), {
      _id: "zealghoul",
      name: "Zeal-damned Ghoul",
      type: "npc",
      items: [
        {
          _id: "ghoulfever2",
          name: "Ghoul Fever",
          type: "action",
          system: {
            category: "offensive",
            slug: "ghoul-fever",
            description: {
              value: "<p><strong>Saving Throw</strong> @Check[fortitude|dc:18]</p><p><strong>Stage 1</strong> @UUID[Compendium.pf2e.conditionitems.Item.Sickened]{Sickened 1} (1 day)</p><p><strong>Stage 2</strong> @UUID[Compendium.pf2e.conditionitems.Item.Unconscious] (1 day)</p>",
            },
            traits: {
              value: ["disease"],
            },
          },
        },
      ],
      system: {
        details: {
          level: {
            value: 3,
          },
          publication: {
            title: "Pathfinder Monster Core",
          },
          publicNotes: "<p>A branded ghoul fanatic.</p>",
        },
        traits: {
          rarity: "common",
          value: ["undead"],
          size: {
            value: "med",
          },
        },
      },
    });
    await writeJson(path.join(fixture.root, "packs", "pf2e", "pathfinder-monster-core", "ghoul-crocodile.json"), {
      _id: "ghoulcrocodile",
      name: "Ghoul Crocodile",
      type: "npc",
      items: [
        {
          _id: "ghoulfever3",
          name: "Ghoul Fever",
          type: "action",
          system: {
            category: "offensive",
            description: {
              value: "<p><strong>Saving Throw</strong> @Check[fortitude|dc:18]</p><p><strong>Stage 1</strong> @UUID[Compendium.pf2e.conditionitems.Item.Sickened]{Sickened 1} (1 day)</p><p><strong>Stage 2</strong> @UUID[Compendium.pf2e.conditionitems.Item.Unconscious] (1 day)</p>",
            },
            traits: {
              value: ["disease"],
            },
          },
        },
      ],
      system: {
        details: {
          level: {
            value: 3,
          },
          publication: {
            title: "Pathfinder Monster Core",
          },
          publicNotes: "<p>An undead crocodile spreading disease.</p>",
        },
        traits: {
          rarity: "common",
          value: ["undead", "animal"],
          size: {
            value: "lg",
          },
        },
      },
    });

    const service = await loadTestService(fixture, { indexPath });

    const ghoulFever = service.lookup("Ghoul Fever", { category: "affliction" }).match;
    expect(ghoulFever?.packName).toBe("derived-afflictions");
    expect(ghoulFever?.subcategory).toBe("disease");
    expect(ghoulFever?.descriptionText).toContain("Saving Throw");

    const lethargyPoison = service.lookup("Lethargy Poison", { category: "affliction" }).match;
    expect(lethargyPoison?.packName).toBe("derived-afflictions");
    expect(lethargyPoison?.subcategory).toBe("poison");

    const afflictionResults = await service.search({ category: "affliction", query: "ghoul fever" });
    expect(afflictionResults.records.map((record) => record.name)).toContain("Ghoul Fever");

    const creatureResults = await service.search({ category: "creature", query: "ghoul fever" });
    expect(creatureResults.records.map((record) => record.name)).toContain("Ghoul Brute");

    const graph = service.getRuleGraph([ghoulFever!.recordKey], {
      includeOutgoing: true,
      maxOutgoingPerPrimary: 10,
    });
    expect(graph.outgoing.records.map((record) => record.name)).toContain("Ghoul Brute");

    const db = new DatabaseSync(indexPath);
    const canonicalRows = db.prepare(`
      SELECT name, raw_json AS rawJson
      FROM records
      WHERE pack_name = 'derived-afflictions'
      ORDER BY name ASC
    `).all() as Array<{ name: string; rawJson: string }>;
    const instanceRows = db.prepare(`
      SELECT COUNT(*) AS total
      FROM records
      WHERE pack_name = 'derived-affliction-instances'
    `).get() as { total: number };
    const ghoulCanonicalRows = db.prepare(`
      SELECT record_key AS recordKey, raw_json AS rawJson
      FROM records
      WHERE pack_name = 'derived-afflictions' AND name = 'Ghoul Fever'
    `).all() as Array<{ recordKey: string; rawJson: string }>;
    const ghoulInstanceRows = db.prepare(`
      SELECT raw_json AS rawJson
      FROM records
      WHERE pack_name = 'derived-affliction-instances' AND name = 'Ghoul Fever'
      ORDER BY record_key ASC
    `).all() as Array<{ rawJson: string }>;
    const ghoulBruteRow = db.prepare(`
      SELECT search_text AS searchText
      FROM records
      WHERE name = 'Ghoul Brute'
    `).get() as { searchText: string } | undefined;
    db.close();

    expect(canonicalRows.map((row) => row.name)).toEqual(["Ghoul Fever", "Lethargy Poison"]);
    expect(instanceRows.total).toBe(6);
    expect(ghoulCanonicalRows).toHaveLength(1);
    expect(JSON.parse(ghoulCanonicalRows[0]!.rawJson)._derived.aliasNormalizationKeys).toEqual(expect.arrayContaining([
      "record:bestiary-family-ability-glossary:ghoulfeversource",
      "slug:disease:ghoul fever",
      "name:disease:ghoul fever",
    ]));
    expect(ghoulInstanceRows).toHaveLength(4);
    expect(new Set(ghoulInstanceRows.map((row) => JSON.parse(row.rawJson)._derived.normalizationKey))).toEqual(
      new Set(["record:bestiary-family-ability-glossary:ghoulfeversource"]),
    );
    expect(ghoulBruteRow?.searchText).toContain("Ghoul Fever");
    expect(ghoulBruteRow?.searchText).toContain("disease");
    expect(ghoulBruteRow?.searchText).toContain("Sickened 1");
    expect(ghoulBruteRow?.searchText).not.toContain("Saving Throw");
    expect(ghoulBruteRow?.searchText).not.toContain("The victim");

    service.close();
  });

  it("logs a final rebuild stage timing summary", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    const progressLogs: string[] = [];

    const service = await loadTestService(fixture, {
      progressLogger: (message) => progressLogs.push(message),
      embeddingProviderFactory: createFakeEmbeddingProviderFactory({
        provider: "hash",
        model: "timing-model",
        revision: null,
        dimensions: 8,
      }),
    });
    service.close();

    expect(progressLogs).toContain("Index rebuild stage timings:");
    expect(progressLogs).toEqual(expect.arrayContaining([
      expect.stringMatching(/- Embedding provider load:/),
      expect.stringMatching(/- Source signature:/),
      expect.stringMatching(/- Scan and normalize records:/),
      expect.stringMatching(/- Resolve families, references, tags, and aliases:/),
      expect.stringMatching(/- Write records and lexical search metadata:/),
      expect.stringMatching(/- Generate canonical embeddings:/),
      expect.stringMatching(/- Insert vector rows:/),
      expect.stringMatching(/- Total rebuild time:/),
    ]));
  });

  it("loads an unchanged SQLite index and requires explicit rebuild when the source changes", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    const indexPath = path.join(fixture.root, ".cache", "pf2e-index.sqlite");

    const firstService = await loadTestService(fixture, { indexPath });
    expect(firstService.getStats()).toEqual({ packCount: 16, recordCount: 121 });
    firstService.close();

    const firstMtime = (await import("node:fs/promises")).stat(indexPath).then((details) => details.mtimeMs);
    const unchangedService = await openPreparedTestService(fixture, { indexPath });
    expect(unchangedService.getStats()).toEqual({ packCount: 16, recordCount: 121 });
    unchangedService.close();
    const secondMtime = (await import("node:fs/promises")).stat(indexPath).then((details) => details.mtimeMs);
    expect(await secondMtime).toBe(await firstMtime);

    await writeJson(path.join(fixture.root, "packs", "pf2e", "pathfinder-monster-core", "sea-ghoul.json"), {
      _id: "monster2",
      name: "Sea Ghoul",
      type: "npc",
      system: {
        details: {
          level: {
            value: 2,
          },
          publication: {
            title: "Pathfinder Monster Core",
          },
          publicNotes: "<p>Rotting sailor undead.</p>",
        },
        traits: {
          rarity: "common",
          value: ["undead", "water"],
          size: {
            value: "med",
          },
        },
      },
    });

    await expect(openPreparedTestService(fixture, { indexPath })).rejects.toThrow(/index .* stale/i);

    const rebuiltService = await loadTestService(fixture, { indexPath });
    expect(rebuiltService.getStats()).toEqual({ packCount: 16, recordCount: 122 });
    expect(rebuiltService.lookup("Sea Ghoul", { category: "creature" }).match?.name).toBe("Sea Ghoul");
    rebuiltService.close();
  });

  it("treats untracked JSON files in git checkouts as stale source changes", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    await initializeGitFixture(fixture.root);
    const indexPath = path.join(fixture.root, ".cache", "pf2e-index.sqlite");

    const firstService = await loadTestService(fixture, { indexPath });
    expect(firstService.getStats()).toEqual({ packCount: 16, recordCount: 121 });
    firstService.close();

    await writeJson(path.join(fixture.root, "packs", "pf2e", "pathfinder-monster-core", "sea-ghoul-untracked.json"), {
      _id: "monster-untracked",
      name: "Sea Ghoul Scout",
      type: "npc",
      system: {
        details: {
          level: {
            value: 2,
          },
          publication: {
            title: "Pathfinder Monster Core",
          },
          publicNotes: "<p>An untracked undead sailor.</p>",
        },
        traits: {
          rarity: "common",
          value: ["undead", "water"],
          size: {
            value: "med",
          },
        },
      },
    });

    await expect(openPreparedTestService(fixture, { indexPath })).rejects.toThrow(/index .* stale/i);

    const rebuiltService = await loadTestService(fixture, { indexPath });
    expect(rebuiltService.lookup("Sea Ghoul Scout", { category: "creature" }).match?.name).toBe("Sea Ghoul Scout");
    rebuiltService.close();
  });

  it("returns indexed raw data even when the source file is gone", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);
    const sourcePath = path.join(fixture.root, "packs", "pf2e", "actions", "raise-a-shield.json");
    await import("node:fs/promises").then(({ rm }) => rm(sourcePath, { force: true }));

    const record = service.getRecord("actions:shield1");
    expect(record?.name).toBe("Raise a Shield");
    expect(record?.raw).toMatchObject({
      _id: "shield1",
      name: "Raise a Shield",
      type: "action",
    });
  });

  it("rebuilds the index when embedding identity changes", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    const indexPath = path.join(fixture.root, ".cache", "pf2e-index.sqlite");

    const firstService = await loadTestService(fixture, {
      indexPath,
      embeddingProviderFactory: createFakeEmbeddingProviderFactory({
        provider: "hf-local",
        model: "model-a",
        revision: "rev-a",
        dimensions: 3,
      }),
    });
    firstService.close();

    let db = new DatabaseSync(indexPath);
    let metadata = new Map(
      (db.prepare("SELECT key, value FROM metadata").all() as Array<{ key: string; value: string }>).map((row) => [row.key, row.value]),
    );
    db.close();
    expect(metadata.get("embedding_provider")).toBe("hf-local");
    expect(metadata.get("embedding_model")).toBe("model-a");
    expect(metadata.get("embedding_revision")).toBe("rev-a");
    expect(metadata.get("embedding_dimensions")).toBe("3");

    const reusedService = await openPreparedTestService(fixture, {
      indexPath,
      embeddingProviderFactory: createFakeEmbeddingProviderFactory({
        provider: "hf-local",
        model: "model-a",
        revision: "rev-a",
        dimensions: 3,
      }),
    });
    reusedService.close();

    db = new DatabaseSync(indexPath);
    metadata = new Map(
      (db.prepare("SELECT key, value FROM metadata").all() as Array<{ key: string; value: string }>).map((row) => [row.key, row.value]),
    );
    db.close();
    expect(metadata.get("embedding_model")).toBe("model-a");

    await expect(
      Pf2eDataService.load(fixture.root, fixture.manifestPath, {
        indexPath,
        embedding: TEST_HASH_EMBEDDING,
        embeddingProviderFactory: createFakeEmbeddingProviderFactory({
          provider: "hf-local",
          model: "model-b",
          revision: "rev-b",
          dimensions: 3,
        }),
      }),
    ).rejects.toThrow(/embedding model changed/i);

    const rebuiltService = await loadTestService(fixture, {
      indexPath,
      embeddingProviderFactory: createFakeEmbeddingProviderFactory({
        provider: "hf-local",
        model: "model-b",
        revision: "rev-b",
        dimensions: 3,
      }),
    });
    rebuiltService.close();

    db = new DatabaseSync(indexPath);
    metadata = new Map(
      (db.prepare("SELECT key, value FROM metadata").all() as Array<{ key: string; value: string }>).map((row) => [row.key, row.value]),
    );
    db.close();
    expect(metadata.get("embedding_model")).toBe("model-b");
    expect(metadata.get("embedding_revision")).toBe("rev-b");
  });

  it("surfaces embedding-provider warnings during load", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture, {
      embeddingProviderFactory: createFakeEmbeddingProviderFactory(
        {
          provider: "hash",
          model: "feature-hash-192",
          revision: null,
          dimensions: 192,
        },
        ["Fell back to hash embeddings."],
      ),
    });

    expect(service.warnings).toContain("Fell back to hash embeddings.");
    service.close();
  });

  it("fails fast when hf-local embeddings are unavailable at runtime", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    await expect(
      Pf2eDataService.load(fixture.root, fixture.manifestPath, {
        embedding: {
          provider: "hf-local",
          modelId: "missing-local-model",
          modelRevision: "main",
          cachePath: path.join(fixture.root, ".cache", "hf-models"),
          localModelPath: null,
        },
        embeddingProviderFactory: async () => {
          throw new Error("cached model assets not found");
        },
      }),
    ).rejects.toThrow(/cached model assets not found/);
  });

  it("fails fast when the SQLite index is missing", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    await expect(
      openPreparedTestService(fixture, {
        indexPath: path.join(fixture.root, ".cache", "missing-index.sqlite"),
      }),
    ).rejects.toThrow(/index not found/i);
  });
});
