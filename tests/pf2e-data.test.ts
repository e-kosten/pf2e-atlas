import { mkdtemp, mkdir, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import { Pf2eDataService } from "../src/pf2e-data.js";

async function writeJson(filePath: string, value: unknown): Promise<void> {
  await writeFile(filePath, JSON.stringify(value, null, 2));
}

async function createFixture(): Promise<{ root: string; manifestPath: string }> {
  const root = await mkdtemp(path.join(os.tmpdir(), "pf2e-mcp-test-"));
  const packRoot = path.join(root, "packs", "pf2e");
  const packNames = ["actions", "conditionitems", "pathfinder-monster-core", "pfs-season-1-bestiary", "spells"];

  await Promise.all(packNames.map(async (packName) => mkdir(path.join(packRoot, packName), { recursive: true })));

  await writeJson(path.join(root, "system.pf2e.json"), {
    packs: [
      {
        name: "actions",
        label: "Actions",
        path: "packs/actions",
        type: "Item",
      },
      {
        name: "conditionitems",
        label: "Conditions",
        path: "packs/conditionitems",
        type: "Item",
      },
      {
        name: "pathfinder-monster-core",
        label: "Pathfinder Monster Core",
        path: "packs/pathfinder-monster-core",
        type: "Actor",
      },
      {
        name: "pfs-season-1-bestiary",
        label: "Season 1",
        path: "packs/pfs-season-1-bestiary",
        type: "Actor",
      },
      {
        name: "spells",
        label: "Spells",
        path: "packs/spells",
        type: "Item",
      },
    ],
  });

  await writeJson(path.join(packRoot, "actions", "raise-a-shield.json"), {
    _id: "shield1",
    name: "Raise a Shield",
    type: "action",
    system: {
      description: {
        value: "<p>You protect yourself.</p>",
      },
      level: {
        value: 1,
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["defensive"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actions", "seek.json"), {
    _id: "Seek",
    name: "Seek",
    type: "action",
    system: {
      description: {
        value: "<p>You scan for things. Success can reveal @UUID[Compendium.pf2e.conditionitems.Item.Hidden]{Hidden} creatures.</p>",
      },
      level: {
        value: 1,
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["concentrate"],
      },
    },
  });

  await writeJson(path.join(packRoot, "conditionitems", "blinded.json"), {
    _id: "Blinded",
    name: "Blinded",
    type: "condition",
    system: {
      description: {
        value: "<p>You cannot see. Blinded overrides @UUID[Compendium.pf2e.conditionitems.Item.Dazzled]. You can @UUID[Compendium.pf2e.actions.Item.Seek]{Seek} to find creatures.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "conditionitems", "dazzled.json"), {
    _id: "Dazzled",
    name: "Dazzled",
    type: "condition",
    system: {
      description: {
        value: "<p>Bright lights and sparkles impede your vision.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "conditionitems", "hidden.json"), {
    _id: "Hidden",
    name: "Hidden",
    type: "condition",
    system: {
      description: {
        value: "<p>The observer knows your space but not your exact location.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "cythnigot.json"), {
    _id: "monster1",
    name: "Cythnigot",
    type: "npc",
    system: {
      details: {
        level: {
          value: 1,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Small aberration.</p>",
      },
      traits: {
        rarity: "uncommon",
        value: ["fiend", "qlippoth"],
        size: {
          value: "sm",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "ghost-sailor-core.json"), {
    _id: "ghost-sailor-core",
    name: "Ghost Sailor",
    type: "npc",
    system: {
      details: {
        level: {
          value: 3,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "bilge-skeleton-core.json"), {
    _id: "bilge-skeleton-core",
    name: "Bilge Skeleton",
    type: "npc",
    system: {
      details: {
        level: {
          value: 2,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A skeletal sailor with barnacles in its ribs.</p>",
      },
      traits: {
        rarity: "common",
        value: ["skeleton", "undead", "water"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pfs-season-1-bestiary", "ghost-sailor-adventure.json"), {
    _id: "ghost-sailor-adventure",
    name: "Ghost Sailor",
    type: "npc",
    system: {
      details: {
        level: {
          value: 3,
        },
        publication: {
          title: "Pathfinder Society Scenario #1-11: Haunted Harbor",
        },
        publicNotes: "<p>An undead mariner bound to a wrecked harbor beacon.</p>",
      },
      traits: {
        rarity: "common",
        value: ["ghost", "undead", "water"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pfs-season-1-bestiary", "bilge-skeleton-adventure.json"), {
    _id: "bilge-skeleton-adventure",
    name: "Bilge Skeleton",
    type: "npc",
    system: {
      details: {
        level: {
          value: 2,
        },
        publication: {
          title: "Pathfinder Society Scenario #1-11: Haunted Harbor",
        },
        publicNotes: "<p>A dockside skeleton that rose from a flooded hold.</p>",
      },
      traits: {
        rarity: "common",
        value: ["skeleton", "undead", "water"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "spells", "sea-blessing.json"), {
    _id: "spell1",
    name: "Sea Blessing",
    type: "spell",
    system: {
      actions: {
        value: 2,
      },
      description: {
        value: "<p>You call on ocean magic to bless a sailor.</p>",
      },
      level: {
        value: 2,
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      range: {
        value: "30 feet",
      },
      traits: {
        rarity: "common",
        traditions: ["primal"],
        value: ["water"],
      },
    },
  });

  return {
    root,
    manifestPath: path.join(root, "system.pf2e.json"),
  };
}

describe("Pf2eDataService", () => {
  const createdRoots: string[] = [];

  afterEach(async () => {
    await Promise.all(
      createdRoots.splice(0).map(async (root) => {
        await import("node:fs/promises").then(({ rm }) => rm(root, { recursive: true, force: true }));
      }),
    );
  });

  it("loads packs and records from the PF2E filesystem layout", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await Pf2eDataService.load(fixture.root, fixture.manifestPath);

    expect(service.listPacks()).toHaveLength(5);
    expect(service.getStats()).toEqual({ packCount: 5, recordCount: 11 });
    expect(service.getPack("Actions")?.name).toBe("actions");
  });

  it("supports lookup, listing, filtering, and derived metadata", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await Pf2eDataService.load(fixture.root, fixture.manifestPath);

    expect(service.lookup("Raise Shield").match?.name).toBe("Raise a Shield");
    expect(service.listRecords({ pack: "actions" }).records).toHaveLength(2);
    expect(service.search({ documentType: "Actor", traitsAll: ["fiend"] }).records[0]?.name).toBe("Cythnigot");
    expect(service.search({ documentType: "Actor", size: "sm" }).records[0]?.name).toBe("Cythnigot");
    expect(service.search({ mode: "lexical", themeQuery: "aberration", documentType: "Actor" }).records[0]?.name).toBe("Cythnigot");
    expect(service.search({ recordType: "spell", tradition: "primal", actionCost: 2 }).records[0]?.name).toBe("Sea Blessing");
    expect(service.search({ nameQuery: "Ghost Sailor", documentType: "Actor", excludeMissingDescription: true }).records).toHaveLength(1);
    expect(service.search({ nameQuery: "Ghost Sailor", documentType: "Actor", excludeAdventureContent: true }).records[0]?.sourceCategory).toBe("core");
    expect(service.search({ documentType: "Actor", coreOnly: true }).records.every((record) => record.sourceCategory === "core")).toBe(true);
    expect(() => service.search({ mode: "structured", themeQuery: "ghost ship" })).toThrow(/themeQuery requires mode lexical or hybrid/i);

    const cythnigot = service.lookup("Cythnigot", { documentType: "Actor" }).match;
    expect(cythnigot?.hasDescription).toBe(true);
    expect(cythnigot?.descriptionSnippet).toBe("Small aberration.");
    expect(cythnigot?.sourceCategory).toBe("core");
  });

  it("uses the recommendation-oriented ranking profile without suppressing described adventure content", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await Pf2eDataService.load(fixture.root, fixture.manifestPath);

    const ghostSailors = service.search({
      documentType: "Actor",
      nameQuery: "Ghost Sailor",
      rankingProfile: "preferReusableReferenceContent",
    }).records;
    expect(ghostSailors[0]?.sourceCategory).toBe("adventure");
    expect(ghostSailors[0]?.hasDescription).toBe(true);

    const bilgeSkeletons = service.search({
      documentType: "Actor",
      nameQuery: "Bilge Skeleton",
      rankingProfile: "preferReusableReferenceContent",
    }).records;
    expect(bilgeSkeletons[0]?.sourceCategory).toBe("core");
  });

  it("builds linked rules context from UUID references", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await Pf2eDataService.load(fixture.root, fixture.manifestPath);

    const firstHop = service.getRulesContext("Blinded", { recordType: "condition", referenceDepth: 1 });
    expect(firstHop?.record.name).toBe("Blinded");
    expect(firstHop?.references.map((record) => record.name)).toEqual(["Dazzled", "Seek"]);

    const secondHop = service.getRulesContext("Blinded", { recordType: "condition", referenceDepth: 2 });
    expect(secondHop?.references.map((record) => record.name)).toContain("Hidden");
    expect(secondHop?.edges.some((edge) => edge.toRecordKey === "conditionitems:Hidden" && edge.depth === 2)).toBe(true);
  });

  it("reuses an unchanged SQLite index and rebuilds when the source changes", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    const indexPath = path.join(fixture.root, ".cache", "pf2e-index.sqlite");

    const firstService = await Pf2eDataService.load(fixture.root, fixture.manifestPath, { indexPath });
    expect(firstService.getStats()).toEqual({ packCount: 5, recordCount: 11 });
    firstService.close();

    const firstMtime = (await import("node:fs/promises")).stat(indexPath).then((details) => details.mtimeMs);
    const unchangedService = await Pf2eDataService.load(fixture.root, fixture.manifestPath, { indexPath });
    expect(unchangedService.getStats()).toEqual({ packCount: 5, recordCount: 11 });
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

    const rebuiltService = await Pf2eDataService.load(fixture.root, fixture.manifestPath, { indexPath });
    expect(rebuiltService.getStats()).toEqual({ packCount: 5, recordCount: 12 });
    expect(rebuiltService.lookup("Sea Ghoul", { documentType: "Actor" }).match?.name).toBe("Sea Ghoul");
    rebuiltService.close();
  });
});
