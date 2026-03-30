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

    expect(service.listPacks()).toHaveLength(2);
    expect(service.getStats()).toEqual({ packCount: 2, recordCount: 2 });
    expect(service.getPack("Actions")?.name).toBe("actions");
  });

  it("supports lookup, listing, and filtering", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await Pf2eDataService.load(fixture.root, fixture.manifestPath);

    expect(service.lookup("Raise Shield").match?.name).toBe("Raise a Shield");
    expect(service.listRecords({ pack: "actions" }).records).toHaveLength(1);
    expect(service.search({ documentType: "Actor", traitsAll: ["fiend"] }).records[0]?.name).toBe("Cythnigot");
  });
});
