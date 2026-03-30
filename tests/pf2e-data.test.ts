import { mkdtemp, mkdir, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import { Pf2eDataService } from "../src/pf2e-data.js";

async function createFixture(): Promise<{ root: string; manifestPath: string }> {
  const root = await mkdtemp(path.join(os.tmpdir(), "pf2e-mcp-test-"));
  const packRoot = path.join(root, "packs", "pf2e");
  await mkdir(path.join(packRoot, "actions"), { recursive: true });
  await mkdir(path.join(packRoot, "pathfinder-monster-core"), { recursive: true });
  await mkdir(path.join(packRoot, "spells"), { recursive: true });

  await writeFile(
    path.join(root, "system.pf2e.json"),
    JSON.stringify(
      {
        packs: [
          {
            name: "actions",
            label: "Actions",
            path: "packs/actions",
            type: "Item",
          },
          {
            name: "pathfinder-monster-core",
            label: "Pathfinder Monster Core",
            path: "packs/pathfinder-monster-core",
            type: "Actor",
          },
          {
            name: "spells",
            label: "Spells",
            path: "packs/spells",
            type: "Item",
          },
        ],
      },
      null,
      2,
    ),
  );

  await writeFile(
    path.join(packRoot, "actions", "raise-a-shield.json"),
    JSON.stringify(
      {
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
            title: "Player Core",
          },
          traits: {
            rarity: "common",
            value: ["defensive"],
          },
        },
      },
      null,
      2,
    ),
  );

  await writeFile(
    path.join(packRoot, "pathfinder-monster-core", "cythnigot.json"),
    JSON.stringify(
      {
        _id: "monster1",
        name: "Cythnigot",
        type: "npc",
        system: {
          details: {
            level: {
              value: 1,
            },
            publication: {
              title: "Monster Core",
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
      },
      null,
      2,
    ),
  );

  await writeFile(
    path.join(packRoot, "spells", "sea-blessing.json"),
    JSON.stringify(
      {
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
            title: "Player Core",
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
      },
      null,
      2,
    ),
  );

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

    expect(service.listPacks()).toHaveLength(3);
    expect(service.getStats()).toEqual({ packCount: 3, recordCount: 3 });
    expect(service.getPack("Actions")?.name).toBe("actions");
  });

  it("supports lookup, listing, and filtering", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await Pf2eDataService.load(fixture.root, fixture.manifestPath);

    expect(service.lookup("Raise Shield").match?.name).toBe("Raise a Shield");
    expect(service.listRecords({ pack: "actions" }).records).toHaveLength(1);
    expect(service.search({ documentType: "Actor", traitsAll: ["fiend"] }).records[0]?.name).toBe("Cythnigot");
    expect(service.search({ documentType: "Actor", size: "sm" }).records[0]?.name).toBe("Cythnigot");
    expect(service.search({ mode: "lexical", themeQuery: "aberration", documentType: "Actor" }).records[0]?.name).toBe("Cythnigot");
    expect(service.search({ recordType: "spell", tradition: "primal", actionCost: 2 }).records[0]?.name).toBe("Sea Blessing");
    expect(() => service.search({ mode: "structured", themeQuery: "ghost ship" })).toThrow(/themeQuery requires mode lexical or hybrid/i);
  });

  it("reuses an unchanged SQLite index and rebuilds when the source changes", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    const indexPath = path.join(fixture.root, ".cache", "pf2e-index.sqlite");

    const firstService = await Pf2eDataService.load(fixture.root, fixture.manifestPath, { indexPath });
    expect(firstService.getStats()).toEqual({ packCount: 3, recordCount: 3 });
    firstService.close();

    const firstMtime = (await import("node:fs/promises")).stat(indexPath).then((details) => details.mtimeMs);
    const unchangedService = await Pf2eDataService.load(fixture.root, fixture.manifestPath, { indexPath });
    expect(unchangedService.getStats()).toEqual({ packCount: 3, recordCount: 3 });
    unchangedService.close();
    const secondMtime = (await import("node:fs/promises")).stat(indexPath).then((details) => details.mtimeMs);
    expect(await secondMtime).toBe(await firstMtime);

    await writeFile(
      path.join(fixture.root, "packs", "pf2e", "pathfinder-monster-core", "sea-ghoul.json"),
      JSON.stringify(
        {
          _id: "monster2",
          name: "Sea Ghoul",
          type: "npc",
          system: {
            details: {
              level: {
                value: 2,
              },
              publication: {
                title: "Monster Core",
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
        },
        null,
        2,
      ),
    );

    const rebuiltService = await Pf2eDataService.load(fixture.root, fixture.manifestPath, { indexPath });
    expect(rebuiltService.getStats()).toEqual({ packCount: 3, recordCount: 4 });
    expect(rebuiltService.lookup("Sea Ghoul", { documentType: "Actor" }).match?.name).toBe("Sea Ghoul");
    rebuiltService.close();
  });
});
