import { mkdir, readFile } from "node:fs/promises";
import path from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import { loadTestService, writeJson } from "../helpers/pf2e-fixture.js";
import { cleanupCreatedRoots, createFixture } from "../helpers/pf2e-service-fixture.js";

describe("Pf2eDataService / Creature explicit assignments, true seeds, and legacy seed migrations", () => {
  const createdRoots: string[] = [];

  afterEach(async () => {
    await cleanupCreatedRoots(createdRoots);
  });

  it("indexes explicit creature assignments, true seeds, and migrated legacy seed tags for live record keys", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const manifest = JSON.parse(await readFile(fixture.manifestPath, "utf8")) as {
      packs: Array<{ name: string; label: string; path: string; type: string }>;
    };
    manifest.packs.push({
      name: "season-of-ghosts-bestiary",
      label: "Season of Ghosts",
      path: "packs/season-of-ghosts-bestiary",
      type: "Actor",
    });
    await writeJson(fixture.manifestPath, manifest);

    const packRoot = path.join(fixture.root, "packs", "pf2e");
    await mkdir(path.join(packRoot, "season-of-ghosts-bestiary"), { recursive: true });

    await Promise.all([
      writeJson(path.join(packRoot, "pathfinder-npc-core", "departmental-chair.json"), {
        _id: "MxcprNbX7hcpAU8p",
        name: "Departmental Chair",
        type: "npc",
        system: {
          details: {
            level: { value: 7 },
            publication: { title: "Pathfinder NPC Core" },
            publicNotes: "<p>An overworked academic reluctantly handling university emergencies.</p>",
          },
          traits: {
            rarity: "common",
            value: ["human", "humanoid"],
            size: { value: "med" },
          },
        },
      }),
      writeJson(path.join(packRoot, "pathfinder-npc-core", "mage-knight.json"), {
        _id: "0cj7cQhgNnLxbUmR",
        name: "Mage Knight",
        type: "npc",
        system: {
          details: {
            level: { value: 10 },
            publication: { title: "Pathfinder NPC Core" },
            publicNotes: "<p>An armored spellcaster trained for frontline battle.</p>",
          },
          traits: {
            rarity: "common",
            value: ["human", "humanoid"],
            size: { value: "med" },
          },
        },
      }),
      writeJson(path.join(packRoot, "pathfinder-npc-core", "false-priest-seed.json"), {
        _id: "OAxxUyACpMlX3q1X",
        name: "False Priest",
        type: "npc",
        system: {
          details: {
            level: { value: 4 },
            publication: { title: "Pathfinder NPC Core" },
            publicNotes: "<p>A deceptive priest who weaponizes belief and hidden doctrine against the faithful.</p>",
          },
          traits: {
            rarity: "common",
            value: ["human", "humanoid"],
            size: { value: "med" },
          },
        },
      }),
      writeJson(path.join(packRoot, "season-of-ghosts-bestiary", "animated-axe-seed.json"), {
        _id: "V67VC975O8iC1Yq2",
        name: "Animated Axe",
        type: "npc",
        system: {
          details: {
            level: { value: 5 },
            publication: { title: "Pathfinder Adventure Path" },
            publicNotes: "<p>A cursed axe flies through the room under its own power.</p>",
          },
          traits: {
            rarity: "common",
            value: ["construct", "mindless"],
            size: { value: "med" },
          },
        },
      }),
      writeJson(path.join(packRoot, "season-of-ghosts-bestiary", "mercenary-enforcer-seed.json"), {
        _id: "wFcdatjBVDsVzbDO",
        name: "Mercenary Enforcer",
        type: "npc",
        system: {
          details: {
            level: { value: 6 },
            publication: { title: "Pathfinder Adventure Path" },
            publicNotes:
              "<p>A veteran mercenary who keeps order with steel and intimidation while serving as hired town muscle.</p>",
          },
          traits: {
            rarity: "common",
            value: ["human", "humanoid"],
            size: { value: "med" },
          },
        },
      }),
      writeJson(path.join(packRoot, "pathfinder-monster-core", "envyspawn-seed.json"), {
        _id: "3nUt7cW8fqE5IpyE",
        name: "Envyspawn",
        type: "npc",
        system: {
          details: {
            level: { value: 2 },
            publication: { title: "Pathfinder Monster Core" },
            publicNotes: "<p>A gaunt fleshwarped horror spawned from ancient runelord sin-magic.</p>",
          },
          traits: {
            rarity: "common",
            value: ["aberration", "evil"],
            size: { value: "med" },
          },
        },
      }),
      writeJson(path.join(packRoot, "pathfinder-monster-core", "conspirator-dragon-seed-negative.json"), {
        _id: "TGYELuImcTcuX0aH",
        name: "Conspirator Dragon (Adult)",
        type: "npc",
        system: {
          details: {
            level: { value: 12 },
            publication: { title: "Pathfinder Monster Core" },
            publicNotes:
              "<p>Hidden among the shadows and upper echelons of society are the conspirator dragons. However, as most conspirator dragons meet others while in disguise, they do their best to maintain their disguise.</p>",
          },
          traits: {
            rarity: "common",
            value: ["dragon", "occult"],
            size: { value: "huge" },
          },
        },
      }),
      writeJson(path.join(packRoot, "pathfinder-monster-core", "conspirator-dragon-spellcaster-seed.json"), {
        _id: "T0OAOkmk4xz0wvjJ",
        name: "Conspirator Dragon (Adult, Spellcaster)",
        type: "npc",
        system: {
          details: {
            level: { value: 12 },
            publication: { title: "Pathfinder Monster Core" },
            publicNotes:
              "<p>Hidden among the shadows and upper echelons of society are the conspirator dragons. Their spellcaster variants lean even harder into manipulation and intrigue.</p>",
          },
          traits: {
            rarity: "common",
            value: ["dragon", "occult"],
            size: { value: "huge" },
          },
        },
      }),
      writeJson(path.join(packRoot, "season-of-ghosts-bestiary", "noppera-bo-impersonator-arcane.json"), {
        _id: "QSa1PbcvbgDv8Zpr",
        name: "Noppera-Bo Impersonator (Arcane)",
        type: "npc",
        system: {
          details: {
            level: { value: 6 },
            publication: { title: "Pathfinder Adventure Path" },
            publicNotes:
              "<p>A faceless infiltrator trained to steal identities with practiced supernatural disguise.</p>",
          },
          traits: {
            rarity: "common",
            value: ["aberration", "chaotic", "evil"],
            size: { value: "med" },
          },
        },
      }),
    ]);

    const service = await loadTestService(fixture);

    expect(service.lookup("Departmental Chair", { category: "creature" }).match?.derivedTags).toEqual(
      expect.arrayContaining(["profession_npc", "civic_npc"]),
    );
    expect(service.lookup("False Priest", { category: "creature" }).match?.derivedTags).toEqual(
      expect.arrayContaining(["profession_npc", "enforcer_npc"]),
    );
    expect(service.lookup("Mage Knight", { category: "creature" }).match?.derivedTags).toContain("enforcer_npc");
    expect(service.lookup("Mercenary Enforcer", { category: "creature" }).match?.derivedTags).toEqual(
      expect.arrayContaining(["profession_npc", "civic_npc", "enforcer_npc"]),
    );
    expect(service.lookup("Animated Axe", { category: "creature" }).match?.derivedTags).toContain("animated_object");
    expect(service.lookup("Noppera-Bo Impersonator (Arcane)", { category: "creature" }).match?.derivedTags).toEqual(
      expect.arrayContaining(["disguised_pretender", "faceless_horror"]),
    );
    expect(service.lookup("Envyspawn", { category: "creature" }).match?.derivedTags).toContain("sinspawn_family");
    expect(service.lookup("Conspirator Dragon (Adult)", { category: "creature" }).match?.derivedTags).toEqual(
      expect.arrayContaining(["disguised_pretender", "urban_setting"]),
    );
    expect(
      service.lookup("Conspirator Dragon (Adult, Spellcaster)", { category: "creature" }).match?.derivedTags,
    ).toEqual(expect.arrayContaining(["disguised_pretender", "dragon_spellcaster", "urban_setting"]));
    expect(
      service
        .listRecords({
          category: "creature",
          metadata: { field: "derivedTags", op: "includesAny", values: ["sinspawn_family"] },
        })
        .records.map((record) => record.name),
    ).toContain("Envyspawn");
  });
});
