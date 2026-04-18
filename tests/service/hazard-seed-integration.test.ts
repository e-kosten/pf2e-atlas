import { mkdir, readFile } from "node:fs/promises";
import path from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import { writeJson, loadTestService } from "../helpers/pf2e-fixture.js";
import { cleanupCreatedRoots, createFixture } from "../helpers/pf2e-service-fixture.js";

describe("Pf2eDataService / Hazard manual seeds", () => {
  const createdRoots: string[] = [];

  afterEach(async () => {
    await cleanupCreatedRoots(createdRoots);
  });

  it("indexes manual hazard seed tags for live record keys", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const manifest = JSON.parse(await readFile(fixture.manifestPath, "utf8")) as {
      packs: Array<{ name: string; label: string; path: string; type: string }>;
    };
    manifest.packs.push(
      { name: "blood-lords-bestiary", label: "Blood Lords", path: "packs/blood-lords-bestiary", type: "Actor" },
      {
        name: "extinction-curse-bestiary",
        label: "Extinction Curse",
        path: "packs/extinction-curse-bestiary",
        type: "Actor",
      },
      {
        name: "outlaws-of-alkenstar-bestiary",
        label: "Outlaws of Alkenstar",
        path: "packs/outlaws-of-alkenstar-bestiary",
        type: "Actor",
      },
    );
    await writeJson(fixture.manifestPath, manifest);

    const packRoot = path.join(fixture.root, "packs", "pf2e");
    await Promise.all([
      mkdir(path.join(packRoot, "blood-lords-bestiary"), { recursive: true }),
      mkdir(path.join(packRoot, "extinction-curse-bestiary"), { recursive: true }),
      mkdir(path.join(packRoot, "outlaws-of-alkenstar-bestiary"), { recursive: true }),
    ]);

    await Promise.all([
      writeJson(path.join(packRoot, "blood-lords-bestiary", "time-rift.json"), {
        _id: "I83vD5fNYIC1s3Xg",
        name: "Time Rift",
        type: "hazard",
        system: {
          details: {
            level: { value: 20 },
            publication: { title: "Pathfinder Adventure Path" },
            publicNotes:
              "<p>A time rift focused on the sundial causes time to pass erratically for all creatures in the area.</p>",
          },
          traits: {
            rarity: "common",
            value: ["magical", "trap"],
          },
        },
      }),
      writeJson(path.join(packRoot, "extinction-curse-bestiary", "mukradi-summoning-runes.json"), {
        _id: "1CjTIaMYUvQUkQI2",
        name: "Mukradi Summoning Runes",
        type: "hazard",
        system: {
          details: {
            level: { value: 15 },
            publication: { title: "Pathfinder Adventure Path" },
            publicNotes: "<p>Barely visible runes are etched into the stone floor in a 20-foot diameter circle.</p>",
          },
          traits: {
            rarity: "common",
            value: ["magical", "trap"],
          },
        },
      }),
      writeJson(path.join(packRoot, "outlaws-of-alkenstar-bestiary", "subduing-gas-chamber.json"), {
        _id: "QQ2Ci8E2lkxG8QIV",
        name: "Subduing Gas Chamber",
        type: "hazard",
        system: {
          details: {
            level: { value: 5 },
            publication: { title: "Pathfinder Adventure Path" },
            publicNotes:
              "<p>A mechanical sensor in the desk drawer releases a counterweight in the wall, which slams the door shut and opens the sleeping gas tank under the bed, allowing gas to fill the air-tight room with a hissing sound.</p>",
          },
          traits: {
            rarity: "common",
            value: ["mechanical", "trap"],
          },
        },
      }),
    ]);

    const service = await loadTestService(fixture);

    expect(service.lookup("Mukradi Summoning Runes", { category: "hazard" }).match?.derivedTags).toEqual(
      expect.arrayContaining(["spawned_attackers", "ward_trigger"]),
    );
    expect(service.lookup("Time Rift", { category: "hazard" }).match?.derivedTags).toContain("planar_breach");
    expect(service.lookup("Subduing Gas Chamber", { category: "hazard" }).match?.derivedTags).toEqual(
      expect.arrayContaining(["barrier_lockdown", "poison_hazard", "respiratory_hazard"]),
    );
    expect(
      service
        .listRecords({
          category: "hazard",
          metadata: { field: "derivedTags", op: "includesAny", values: ["planar_breach"] },
        })
        .records.map((record) => record.name),
    ).toContain("Time Rift");
  });
});
