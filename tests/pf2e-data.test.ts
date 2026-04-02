import { execFile } from "node:child_process";
import { mkdtemp, mkdir, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { promisify } from "node:util";

import { afterEach, describe, expect, it } from "vitest";

import { Pf2eDataService } from "../src/pf2e-data.js";
import { RankingConfigStore } from "../src/ranking-config.js";

const execFileAsync = promisify(execFile);
const TEST_HASH_EMBEDDING = {
  provider: "hash" as const,
  modelId: "feature-hash-192",
  modelRevision: null,
  cachePath: path.join(os.tmpdir(), "pf2e-test-hf-cache"),
  localModelPath: null,
};

async function writeJson(filePath: string, value: unknown): Promise<void> {
  await writeFile(filePath, JSON.stringify(value, null, 2));
}

async function loadTestService(
  fixture: { root: string; manifestPath: string },
  options: Parameters<typeof Pf2eDataService.load>[2] = {},
): Promise<Pf2eDataService> {
  return Pf2eDataService.rebuildIndex(fixture.root, fixture.manifestPath, {
    embedding: TEST_HASH_EMBEDDING,
    ...options,
  });
}

async function openPreparedTestService(
  fixture: { root: string; manifestPath: string },
  options: Parameters<typeof Pf2eDataService.load>[2] = {},
): Promise<Pf2eDataService> {
  return Pf2eDataService.load(fixture.root, fixture.manifestPath, {
    embedding: TEST_HASH_EMBEDDING,
    ...options,
  });
}

async function initializeGitFixture(root: string): Promise<void> {
  await execFileAsync("git", ["init", "-b", "main"], { cwd: root });
  await execFileAsync("git", ["config", "user.name", "PF2E Test"], { cwd: root });
  await execFileAsync("git", ["config", "user.email", "pf2e-test@example.com"], { cwd: root });
  await execFileAsync("git", ["add", "."], { cwd: root });
  await execFileAsync("git", ["commit", "-m", "Initial fixture"], { cwd: root });
}

function createFakeEmbeddingProviderFactory(
  identity: { provider: "hash" | "hf-local"; model: string; revision: string | null; dimensions: number },
  warnings: string[] = [],
): NonNullable<Parameters<typeof Pf2eDataService.load>[2]>["embeddingProviderFactory"] {
  return async () => ({
    provider: {
      identity,
      async embed(text: string): Promise<Float32Array> {
        const vector = new Float32Array(identity.dimensions);
        if (text.trim().length > 0) {
          vector[0] = 1;
        }
        return vector;
      },
    },
    warnings,
  });
}

function createCapturingEmbeddingProviderFactory(
  calls: string[],
  identity: { provider: "hash" | "hf-local"; model: string; revision: string | null; dimensions: number },
): NonNullable<Parameters<typeof Pf2eDataService.load>[2]>["embeddingProviderFactory"] {
  return async () => ({
    provider: {
      identity,
      async embed(text: string): Promise<Float32Array> {
        calls.push(text);
        const vector = new Float32Array(identity.dimensions);
        if (text.trim().length > 0) {
          vector[0] = 1;
        }
        return vector;
      },
    },
    warnings: [],
  });
}

async function createFixture(): Promise<{ root: string; manifestPath: string }> {
  const root = await mkdtemp(path.join(os.tmpdir(), "pf2e-mcp-test-"));
  const packRoot = path.join(root, "packs", "pf2e");
  const packNames = [
    "actions",
    "actionspf2e",
    "classfeatures",
    "conditionitems",
    "equipment",
    "equipment-srd",
    "feats-srd",
    "heritages",
    "journals",
    "pathfinder-monster-core",
    "pfs-season-1-bestiary",
    "quest-for-the-frozen-flame-bestiary",
    "spells",
    "spells-srd",
  ];

  await Promise.all(packNames.map(async (packName) => mkdir(path.join(packRoot, packName), { recursive: true })));
  await mkdir(path.join(root, "src", "module", "migration", "migrations"), { recursive: true });

  await writeJson(path.join(root, "system.pf2e.json"), {
    packs: [
      {
        name: "actions",
        label: "Actions",
        path: "packs/actions",
        type: "Item",
      },
      {
        name: "actionspf2e",
        label: "Actions SRD",
        path: "packs/actionspf2e",
        type: "Item",
      },
      {
        name: "classfeatures",
        label: "Class Features",
        path: "packs/classfeatures",
        type: "Item",
      },
      {
        name: "conditionitems",
        label: "Conditions",
        path: "packs/conditionitems",
        type: "Item",
      },
      {
        name: "equipment",
        label: "Equipment",
        path: "packs/equipment",
        type: "Item",
      },
      {
        name: "equipment-srd",
        label: "Equipment SRD",
        path: "packs/equipment-srd",
        type: "Item",
      },
      {
        name: "feats-srd",
        label: "Feats",
        path: "packs/feats-srd",
        type: "Item",
      },
      {
        name: "heritages",
        label: "Heritages",
        path: "packs/heritages",
        type: "Item",
      },
      {
        name: "journals",
        label: "Journals",
        path: "packs/journals",
        type: "JournalEntry",
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
        name: "quest-for-the-frozen-flame-bestiary",
        label: "Quest for the Frozen Flame",
        path: "packs/quest-for-the-frozen-flame-bestiary",
        type: "Actor",
      },
      {
        name: "spells",
        label: "Spells",
        path: "packs/spells",
        type: "Item",
      },
      {
        name: "spells-srd",
        label: "Spells SRD",
        path: "packs/spells-srd",
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

  await writeJson(path.join(packRoot, "actions", "refocus.json"), {
    _id: "action-refocus-1",
    name: "Refocus",
    type: "action",
    system: {
      description: {
        value: "<p>You spend 10 minutes restoring your magical connection.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["concentrate", "exploration"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actions", "reactive-strike.json"), {
    _id: "reactive1",
    name: "Reactive Strike",
    type: "action",
    system: {
      description: {
        value: "<p>You lash out when a foe drops their guard.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["fighter"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actions", "attack-of-opportunity.json"), {
    _id: "aoo1",
    name: "Attack of Opportunity",
    type: "action",
    system: {
      description: {
        value: "<p>You punish a nearby opening.</p>",
      },
      publication: {
        title: "Pathfinder Core Rulebook",
        remaster: false,
      },
      traits: {
        rarity: "common",
        value: ["fighter"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "impersonate.json"), {
    _id: "action-impersonate-1",
    name: "Impersonate",
    type: "action",
    system: {
      description: {
        value: "<p>You create a disguise to pass yourself off as someone else.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["skill"],
      },
    },
  });

  await writeJson(path.join(packRoot, "classfeatures", "meditative-well.json"), {
    _id: "classfeature1",
    name: "Meditative Well",
    type: "feat",
    system: {
      description: {
        value: "<p>Your training deepens whenever you @UUID[Compendium.pf2e.actions.Item.Refocus]{Refocus}.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        value: [],
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

  await writeJson(path.join(packRoot, "conditionitems", "off-guard.json"), {
    _id: "Off-Guard",
    name: "Off-Guard",
    type: "condition",
    system: {
      description: {
        value: "<p>You take a penalty to AC.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
        remaster: true,
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "conditionitems", "flat-footed.json"), {
    _id: "Flat-Footed",
    name: "Flat-Footed",
    type: "condition",
    system: {
      description: {
        value: "<p>You take a penalty to AC.</p>",
      },
      publication: {
        title: "Pathfinder Core Rulebook",
        remaster: false,
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "spacious-pouch-type-i.json"), {
    _id: "pouch1",
    name: "Spacious Pouch (Type I)",
    type: "backpack",
    system: {
      description: {
        value: "<p>A roomy extradimensional pouch.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: [],
      },
      price: {
        value: {
          gp: 75,
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "marvelous-miniature-chest.json"), {
    _id: "miniChest1",
    name: "Marvelous Miniature (Chest)",
    type: "consumable",
    system: {
      description: {
        value: "<p>A tiny chest that unfolds to full size.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "marvelous-miniature-ladder.json"), {
    _id: "miniLadder1",
    name: "Marvelous Miniature (Ladder)",
    type: "consumable",
    system: {
      description: {
        value: "<p>A tiny ladder that unfolds to full size.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "marvelous-miniature-boat.json"), {
    _id: "miniBoat1",
    name: "Marvelous Miniature (Boat)",
    type: "consumable",
    system: {
      description: {
        value: "<p>A tiny boat that unfolds on the water.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "surging-serum-lesser.json"), {
    _id: "serum1",
    name: "Surging Serum (Lesser)",
    type: "consumable",
    system: {
      description: {
        value: "<p>A crackling restorative elixir.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable", "elixir"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "sightless-tincture.json"), {
    _id: "tincture1",
    name: "Sightless Tincture",
    type: "consumable",
    system: {
      description: {
        value: "<p>A toxin that clouds the victim's vision.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault (Remastered)",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable", "poison"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "elixir-of-life-minor.json"), {
    _id: "elixirLife1",
    name: "Elixir of Life (Minor)",
    type: "consumable",
    system: {
      description: {
        value: "<p>A healing elixir that restores hit points.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable", "elixir", "healing"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "antidote-lesser.json"), {
    _id: "antidote1",
    name: "Antidote (Lesser)",
    type: "consumable",
    system: {
      description: {
        value: "<p>This antidote bolsters the drinker against poison.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "antiplague-lesser.json"), {
    _id: "antiplague1",
    name: "Antiplague (Lesser)",
    type: "consumable",
    system: {
      description: {
        value: "<p>This antiplague helps ward off disease.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "bottled-catharsis-serenity.json"), {
    _id: "catharsis1",
    name: "Bottled Catharsis (Serenity)",
    type: "consumable",
    system: {
      description: {
        value: "<p>This bottled catharsis steadies the emotions and helps recover from mental conditions.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault (Remastered)",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "climbing-kit.json"), {
    _id: "climbKit1",
    name: "Climbing Kit",
    type: "equipment",
    system: {
      description: {
        value: "<p>A compact climbing kit with rope and pitons for climbing sheer walls and rappelling safely.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "concealable-thieves-tools.json"), {
    _id: "concealTools1",
    name: "Concealable Thieves' Tools",
    type: "equipment",
    system: {
      description: {
        value: "<p>Slim lockpicks and hidden tools for bypassing locks without drawing attention.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "disguise-kit.json"), {
    _id: "equip-disguise-kit-1",
    name: "Disguise Kit",
    type: "equipment",
    system: {
      description: {
        value: "<p>You usually need a disguise kit to set up a disguise in order to @UUID[Compendium.pf2e.actionspf2e.Item.Impersonate]{Impersonate} someone.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "quick-change-outfit.json"), {
    _id: "equip-quick-change-1",
    name: "Quick-Change Outfit",
    type: "equipment",
    system: {
      description: {
        value: "<p>A quick-change outfit is in fact two separate outfits sewn together, allowing you to switch quickly between the two outfits.</p>",
      },
      publication: {
        title: "Pathfinder Lost Omens Firebrands",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "masquerade-scarf.json"), {
    _id: "equip-masquerade-scarf-1",
    name: "Masquerade Scarf",
    type: "equipment",
    system: {
      description: {
        value: "<p>You arrange the scarf over your lower face, and it casts @UUID[Compendium.pf2e.spells-srd.Item.Illusory Disguise]{Illusory Disguise} on you.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "heritages", "nephilim.json"), {
    _id: "neph1",
    name: "Nephilim",
    type: "heritage",
    system: {
      description: {
        value: "<p>You are touched by an outer plane.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["nephilim"],
      },
    },
  });

  await writeJson(path.join(packRoot, "heritages", "naari.json"), {
    _id: "naari1",
    name: "Naari",
    type: "heritage",
    system: {
      description: {
        value: "<p>You are touched by elemental fire.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["naari"],
      },
    },
  });

  await writeJson(path.join(packRoot, "journals", "remaster-changes.json"), {
    _id: "journal1",
    name: "Remaster Changes",
    pages: [
      {
        _id: "page1",
        name: "Remaster Changes",
        type: "text",
        text: {
          content: `
            <ul>
              <li>Aasimar, Aphorite, Ganzi, and Tiefling are merged into @UUID[Compendium.pf2e.heritages.Item.neph1]{Nephilim}.</li>
              <li>Ifrit are now @UUID[Compendium.pf2e.heritages.Item.naari1]{Naari}.</li>
            </ul>
          `,
          format: 1,
        },
      },
      {
        _id: "page2",
        name: "Class Features",
        type: "text",
        text: {
          content: `
            <table><tbody>
              <tr><td>Attack of Opportunity</td><td>Multiple</td><td>Renamed</td><td>@UUID[Compendium.pf2e.actions.Item.Reactive Strike]{Reactive Strike}</td></tr>
              <tr><td>Strike Back</td><td>Multiple</td><td>Replaced</td><td>@UUID[Compendium.pf2e.actions.Item.Reactive Strike]{Reactive Strike}</td></tr>
            </tbody></table>
          `,
          format: 1,
        },
      },
      {
        _id: "page3",
        name: "Equipment",
        type: "text",
        text: {
          content: `
            <table><tbody>
              <tr>
                <td>Feather Token (Chest, Ladder, Swan Boat)</td>
                <td>Multiple</td>
                <td>Renamed</td>
                <td>
                  Marvelous Miniatures (
                  @UUID[Compendium.pf2e.equipment.Item.miniChest1]{Chest},
                  @UUID[Compendium.pf2e.equipment.Item.miniLadder1]{Ladder},
                  @UUID[Compendium.pf2e.equipment.Item.miniBoat1]{Boat}
                  )
                </td>
              </tr>
              <tr>
                <td>Bag of Holding</td>
                <td>Multiple</td>
                <td>Renamed</td>
                <td>@UUID[Compendium.pf2e.equipment.Item.pouch1]{Spacious Pouch}</td>
              </tr>
              <tr>
                <td>Sight-Theft Grit</td>
                <td>Multiple</td>
                <td>Renamed</td>
                <td>@UUID[Compendium.pf2e.equipment.Item.serum1]{Sightless Tincture}</td>
              </tr>
            </tbody></table>
          `,
          format: 1,
        },
      },
    ],
  });

  await writeFile(
    path.join(root, "src", "module", "migration", "migrations", "850-flat-footed-to-off-guard.ts"),
    `/** Rename all uses and mentions of "flat-footed" to "off-guard" */\n`,
  );

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

  await writeJson(path.join(packRoot, "feats-srd", "deep-focus.json"), {
    _id: "feat1",
    name: "Deep Focus",
    type: "feat",
    system: {
      description: {
        value: "<p>Whenever you @UUID[Compendium.pf2e.actions.Item.Refocus]{Refocus}, your resolve sharpens.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "spells-srd", "illusory-disguise.json"), {
    _id: "spell-illusory-disguise-1",
    name: "Illusory Disguise",
    type: "spell",
    system: {
      description: {
        value: "<p>You create an illusion that disguises the target.</p>",
      },
      level: {
        value: 1,
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        traditions: ["arcane", "occult"],
        value: ["illusion"],
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "legacy-nephilim-host.json"), {
    _id: "legacy-neph",
    name: "Legacy Nephilim Host",
    type: "npc",
    items: [
      {
        _id: "embedded-neph",
        _stats: {
          compendiumSource: "Compendium.pf2e.heritages.Item.neph1",
        },
        name: "Aasimar",
        type: "heritage",
        system: {
          slug: "nephilim",
          publication: {
            title: "Pathfinder Core Rulebook",
            remaster: false,
          },
          traits: {
            value: ["nephilim"],
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
          title: "Pathfinder Core Rulebook",
          remaster: false,
        },
      },
      perception: {
        mod: 5,
      },
      traits: {
        rarity: "common",
        size: {
          value: "med",
        },
        value: ["humanoid"],
      },
      attributes: {
        hp: {
          value: 30,
          max: 30,
        },
        speed: {
          value: 25,
          otherSpeeds: [],
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "ghost-commoner-core.json"), {
    _id: "ghost-commoner-core",
    name: "Ghost Commoner",
    type: "npc",
    system: {
      details: {
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A lingering spirit trapped in an old manor, bound by grief and unfinished business.</p>",
      },
      traits: {
        rarity: "common",
        value: ["ghost", "incorporeal", "spirit", "undead", "unholy"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "ship-captain.json"), {
    _id: "ship-captain",
    name: "Ship Captain",
    type: "npc",
    system: {
      details: {
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A weathered ship captain who knows every reef and harbor on this coast.</p>",
      },
      traits: {
        rarity: "common",
        value: ["human", "water"],
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "pelagic-stalker.json"), {
    _id: "pelagic-stalker",
    name: "Pelagic Stalker",
    type: "npc",
    system: {
      details: {
        level: {
          value: 5,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A sleek predator built for sudden bursts of speed.</p>",
      },
      traits: {
        rarity: "common",
        value: ["aquatic", "beast"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "fungus-leshy-core.json"), {
    _id: "fungus-leshy-core",
    name: "Fungus Leshy",
    type: "npc",
    system: {
      details: {
        level: {
          value: 2,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A cave-dwelling leshy that tends damp gardens of moss, spores, and rot.</p>",
      },
      traits: {
        rarity: "common",
        value: ["fungus", "leshy", "plant"],
        size: {
          value: "sm",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "mournful-hallway.json"), {
    _id: "mournful-hallway",
    name: "Mournful Hallway",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 2,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A grief-soaked corridor where spirits claw at the living.</p>",
      },
      traits: {
        rarity: "common",
        value: ["haunt", "curse", "magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "spear-launcher.json"), {
    _id: "spear-launcher",
    name: "Spear Launcher",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 1,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A spring-loaded spear trap hidden in the wall.</p>",
      },
      traits: {
        rarity: "common",
        value: ["trap", "mechanical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "diver.json"), {
    _id: "diver",
    name: "Diver",
    type: "npc",
    system: {
      details: {
        level: {
          value: 2,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A salvage diver used to dark holds, drowned wrecks, and foggy coasts.</p>",
      },
      traits: {
        rarity: "common",
        value: ["human"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "lion.json"), {
    _id: "lion",
    name: "Lion",
    type: "npc",
    system: {
      details: {
        level: {
          value: 3,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A stalking predator that inspires fear.</p>",
      },
      traits: {
        rarity: "common",
        value: ["animal"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "amber-sentinel.json"), {
    _id: "amber-sentinel",
    name: "Amber Sentinel",
    type: "npc",
    system: {
      details: {
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A vigilant guardian that patrols ancient ruins and watches for intruders.</p>",
      },
      traits: {
        rarity: "common",
        value: ["construct", "guardian"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "azure-sentinel.json"), {
    _id: "azure-sentinel",
    name: "Azure Sentinel",
    type: "npc",
    system: {
      details: {
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A vigilant guardian that patrols ancient ruins and watches for intruders.</p>",
      },
      traits: {
        rarity: "uncommon",
        value: ["construct", "guardian"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "gloam-sentinel.json"), {
    _id: "gloam-sentinel",
    name: "Gloam Sentinel",
    type: "npc",
    system: {
      details: {
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A vigilant guardian that patrols ancient ruins and watches for intruders.</p>",
      },
      traits: {
        rarity: "rare",
        value: ["construct", "guardian"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "last-sentinel.json"), {
    _id: "last-sentinel",
    name: "Last Sentinel",
    type: "npc",
    system: {
      details: {
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A vigilant guardian that patrols ancient ruins and watches for intruders.</p>",
      },
      traits: {
        rarity: "unique",
        value: ["construct", "guardian"],
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

  await writeJson(path.join(packRoot, "pfs-season-1-bestiary", "ghost-of-diggen-thrune-3-4.json"), {
    _id: "ghost-of-diggen-thrune-3-4",
    name: "Ghost Of Diggen Thrune (3-4)",
    type: "npc",
    system: {
      details: {
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Society Scenario #1-11: Haunted Harbor",
        },
      },
      traits: {
        rarity: "common",
        value: ["ghost", "incorporeal", "spirit", "undead", "unholy"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pfs-season-1-bestiary", "blighted-fungus-leshy.json"), {
    _id: "blighted-fungus-leshy",
    name: "Blighted Fungus Leshy",
    type: "npc",
    system: {
      details: {
        level: {
          value: 3,
        },
        publication: {
          title: "Pathfinder Society Scenario #1-15: The Blooming Catastrophe",
        },
      },
      traits: {
        rarity: "uncommon",
        value: ["fungus", "leshy", "unholy"],
        size: {
          value: "sm",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "quest-for-the-frozen-flame-bestiary", "crawling-hand-swarm.json"), {
    _id: "crawling-hand-swarm",
    name: "Crawling Hand Swarm",
    type: "npc",
    system: {
      details: {
        level: {
          value: 5,
        },
        publication: {
          title: "Pathfinder #176: Lost Mammoth Valley",
        },
        publicNotes: "<p>A mass of severed hands writhing through a frozen burial ground.</p>",
      },
      traits: {
        rarity: "common",
        value: ["evil", "swarm", "undead", "unholy"],
        size: {
          value: "lg",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "quest-for-the-frozen-flame-bestiary", "bilge-skeleton-adventure-path.json"), {
    _id: "bilge-skeleton-ap",
    name: "Bilge Skeleton",
    type: "npc",
    system: {
      details: {
        level: {
          value: 2,
        },
        publication: {
          title: "Pathfinder #176: Lost Mammoth Valley",
        },
        publicNotes: "<p>A half-frozen skeleton dredged up from a long-buried wreck.</p>",
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

  await writeJson(path.join(packRoot, "spells", "focus-burst.json"), {
    _id: "spell2",
    name: "Focus Burst",
    type: "spell",
    system: {
      description: {
        value: "<p>You disrupt a creature as it tries to @UUID[Compendium.pf2e.actions.Item.Refocus]{Refocus}.</p>",
      },
      level: {
        value: 3,
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        traditions: ["occult"],
        value: ["concentrate", "focus"],
      },
    },
  });

  return {
    root,
    manifestPath: path.join(root, "system.pf2e.json"),
  };
}

async function createHardFilterFixture(): Promise<{ root: string; manifestPath: string }> {
  const root = await mkdtemp(path.join(os.tmpdir(), "pf2e-mcp-filter-test-"));
  const packRoot = path.join(root, "packs", "pf2e");
  const packNames = [
    "action-macros",
    "actions",
    "campaign-effects",
    "feats-srd",
    "macros",
    "pathfinder-bestiary",
    "pathfinder-bestiary-2",
    "pathfinder-monster-core-2",
    "pathfinder-society-boons",
    "pfs-introductions-bestiary",
    "pfs-season-3-bestiary",
  ];

  await Promise.all(packNames.map(async (packName) => mkdir(path.join(packRoot, packName), { recursive: true })));
  await Promise.all([
    mkdir(path.join(packRoot, "campaign-effects", "pathfinder-society"), { recursive: true }),
    mkdir(path.join(packRoot, "pfs-season-3-bestiary", "3-04"), { recursive: true }),
    mkdir(path.join(packRoot, "pfs-season-3-bestiary", "3-13"), { recursive: true }),
  ]);

  await writeJson(path.join(root, "system.pf2e.json"), {
    packs: [
      { name: "action-macros", label: "Action Macros", path: "packs/action-macros", type: "Macro" },
      { name: "actions", label: "Actions", path: "packs/actions", type: "Item" },
      { name: "campaign-effects", label: "Campaign Effects", path: "packs/campaign-effects", type: "Item" },
      { name: "feats-srd", label: "Feats", path: "packs/feats-srd", type: "Item" },
      { name: "macros", label: "Macros", path: "packs/macros", type: "Macro" },
      { name: "pathfinder-bestiary", label: "Pathfinder Bestiary", path: "packs/pathfinder-bestiary", type: "Actor" },
      { name: "pathfinder-bestiary-2", label: "Pathfinder Bestiary 2", path: "packs/pathfinder-bestiary-2", type: "Actor" },
      { name: "pathfinder-monster-core-2", label: "Pathfinder Monster Core 2", path: "packs/pathfinder-monster-core-2", type: "Actor" },
      { name: "pathfinder-society-boons", label: "Pathfinder Society Boons", path: "packs/pathfinder-society-boons", type: "Item" },
      { name: "pfs-introductions-bestiary", label: "Pathfinder Society Introductions", path: "packs/pfs-introductions-bestiary", type: "Actor" },
      { name: "pfs-season-3-bestiary", label: "Season 3", path: "packs/pfs-season-3-bestiary", type: "Actor" },
    ],
  });

  await writeJson(path.join(packRoot, "actions", "raise-a-shield.json"), {
    _id: "action-raise-a-shield",
    name: "Raise a Shield",
    type: "action",
    system: {
      description: {
        value: "<p>You position your shield to protect yourself.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        value: ["defensive"],
      },
    },
  });

  await writeJson(path.join(packRoot, "feats-srd", "proud-mentor.json"), {
    _id: "feat-proud-mentor",
    name: "Proud Mentor",
    type: "feat",
    system: {
      description: {
        value: "<p>You enjoy mentoring others and take pride in your students' accomplishments.</p>",
      },
      publication: {
        title: "Pathfinder Howl of the Wild",
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-bestiary", "ghoul.json"), {
    _id: "base-ghoul",
    name: "Ghoul",
    type: "npc",
    system: {
      details: {
        level: {
          value: 1,
        },
        publication: {
          title: "Pathfinder Bestiary",
        },
        publicNotes: "<p>A flesh-hungry undead scavenger.</p>",
      },
      traits: {
        rarity: "common",
        value: ["ghoul", "undead"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-bestiary-2", "grimstalker.json"), {
    _id: "base-grimstalker",
    name: "Grimstalker",
    type: "npc",
    system: {
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Bestiary 2",
        },
        publicNotes: "<p>A cunning forest predator.</p>",
      },
      traits: {
        rarity: "common",
        value: ["beast"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core-2", "zebub.json"), {
    _id: "base-zebub",
    name: "Zebub",
    type: "npc",
    system: {
      details: {
        level: {
          value: 1,
        },
        publication: {
          title: "Pathfinder Monster Core 2",
        },
        publicNotes: "<p>A buzzing fiend that delights in decay.</p>",
      },
      traits: {
        rarity: "common",
        value: ["fiend"],
        size: {
          value: "sm",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pfs-introductions-bestiary", "ghoul-pfs-intro-2.json"), {
    _id: "pfs-ghoul",
    name: "Ghoul (PFS Intro 2)",
    type: "npc",
    system: {
      details: {
        level: {
          value: 1,
        },
        publication: {
          title: "Pathfinder Society Intro #2: United in Purpose",
        },
      },
      traits: {
        rarity: "common",
        value: ["ghoul", "undead"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pfs-season-3-bestiary", "3-13", "grimstalker-pfs-3-13.json"), {
    _id: "pfs-grimstalker",
    name: "Grimstalker (PFS 3-13)",
    type: "npc",
    system: {
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Society Scenario #3-13: Guardian's Covenant",
        },
      },
      traits: {
        rarity: "common",
        value: ["beast"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pfs-season-3-bestiary", "3-04", "zebub-pfs.json"), {
    _id: "pfs-zebub",
    name: "Zebub (PFS)",
    type: "npc",
    system: {
      details: {
        level: {
          value: 1,
        },
        publication: {
          title: "Pathfinder Society Scenario #3-04: The Devil-Wrought Disappearance",
        },
      },
      traits: {
        rarity: "common",
        value: ["fiend"],
        size: {
          value: "sm",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-society-boons", "magical-mentor.json"), {
    _id: "boon-magical-mentor",
    name: "Magical Mentor",
    type: "feat",
    system: {
      category: "pfsboon",
      description: {
        value: "<p>While working with less experienced Pathfinder allies, you provide key spellcasting insights.</p>",
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "action-macros", "trip-athletics.json"), {
    _id: "macro-trip-athletics",
    name: "Trip: Athletics",
    type: "script",
    command: "game.pf2e.actions.trip({ event: event });",
  });

  await writeJson(path.join(packRoot, "action-macros", "raise-a-shield.json"), {
    _id: "macro-raise-a-shield",
    name: "Raise a Shield",
    type: "script",
    command: "game.pf2e.actions.raiseAShield({ actors: [token?.actor ?? actor ?? game.user.character].filter((actor) => actor) })",
  });

  await writeJson(path.join(packRoot, "macros", "treat-wounds.json"), {
    _id: "macro-treat-wounds",
    name: "Treat Wounds",
    type: "script",
    command: "game.pf2e.actions.treatWounds({ event, actors: [token?.actor ?? actor ?? game.user.character].filter((actor) => actor) })",
  });

  await writeJson(path.join(packRoot, "campaign-effects", "pathfinder-society", "effect-magical-mentor.json"), {
    _id: "effect-magical-mentor",
    name: "Effect: Magical Mentor",
    type: "effect",
    system: {
      description: {
        value: "<p>Granted by Magical Mentor.</p>",
      },
      publication: {
        title: "Pathfinder Society Boons",
      },
      traits: {
        value: [],
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

    const service = await loadTestService(fixture);

    expect(service.listPacks()).toHaveLength(13);
    expect(service.getStats()).toEqual({ packCount: 13, recordCount: 52 });
    expect(service.getPack("Actions")?.name).toBe("actions");
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

  it("supports lookup, listing, filtering, and derived metadata", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    expect(service.lookup("Raise Shield").match?.name).toBe("Raise a Shield");
    expect(service.listRecords({ pack: "actions" }).searchProfile).toBeNull();
    expect(service.listRecords({ pack: "actions" }).records).toHaveLength(4);
    expect(service.listRecords({ category: "creature", levelMin: 4, levelMax: 4, traitsAny: ["undead"] }).records.map((record) => record.name)).toEqual(["Ghost Commoner"]);
    expect((await service.search({ category: "creature", traitsAll: ["fiend"] })).records[0]?.name).toBe("Cythnigot");
    expect((await service.search({ category: "creature", traitsAll: ["fiend"] })).searchProfile).toBeNull();
    expect((await service.search({ category: "creature", size: "sm" })).records.every((record) => record.size === "sm")).toBe(true);
    expect((await service.search({ searchProfile: "lexical", query: "aberration", category: "creature" })).records[0]?.name).toBe("Cythnigot");
    expect((await service.search({ category: "spell", traditions: ["primal"], actionCost: 2 })).records[0]?.name).toBe("Sea Blessing");
    expect((await service.search({ category: "spell", spellKinds: ["focus"] })).records.map((record) => record.name)).toEqual(["Focus Burst"]);
    expect((await service.search({ category: "rule", subcategory: "condition" })).records.map((record) => record.name)).toEqual(
      expect.arrayContaining(["Blinded", "Dazzled", "Hidden"]),
    );
    expect((await service.search({ category: "hazard" })).records.map((record) => record.name)).toEqual(
      expect.arrayContaining(["Mournful Hallway", "Spear Launcher"]),
    );
    expect((await service.search({ category: "hazard", subcategory: "trap" })).records.map((record) => record.name)).toEqual(["Spear Launcher"]);
    expect(service.listRecords({ pack: "Pathfinder Monster Core", category: "hazard", subcategory: "trap" }).records.map((record) => record.name)).toEqual(["Spear Launcher"]);
    expect((await service.search({ category: "creature", excludeTraits: ["water"] })).records.some((record) => record.traits.includes("water"))).toBe(false);
    expect((await service.search({ category: "creature", nameQuery: "Ghost Sailor", excludeMissingDescription: true })).records.every((record) => record.hasDescription)).toBe(true);
    expect((await service.search({ category: "creature", nameQuery: "Ghost Sailor", excludeSources: ["adventure"] })).records[0]?.sourceCategory).toBe("core");
    expect((await service.search({ category: "creature", sources: ["core"] })).records.every((record) => record.sourceCategory === "core")).toBe(true);
    expect((await service.search({ query: "ghost ship", category: "creature" })).mode).toBe("hybrid");
    expect((await service.search({ query: "ghost ship", category: "creature" })).searchProfile).toBe("balanced");
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", derivedTagsAny: ["anti_poison"] }).records.map((record) => record.name)).toEqual(["Antidote (Lesser)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", derivedTagsAny: ["mental_recovery"] }).records.map((record) => record.name)).toEqual(["Bottled Catharsis (Serenity)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "gear", derivedTagsAny: ["lock_bypass"] }).records.map((record) => record.name)).toEqual(["Concealable Thieves' Tools"]);
    expect(service.listRecords({ category: "equipment", subcategory: "backpack", derivedTagsAny: ["carry_support"] }).records.map((record) => record.name)).toEqual(["Spacious Pouch (Type I)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", derivedTagsAll: ["beneficial", "anti_disease"] }).records.map((record) => record.name)).toEqual(["Antiplague (Lesser)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", excludeDerivedTags: ["offensive"] }).records.map((record) => record.name)).not.toContain("Sightless Tincture");
    expect(service.listRecords({ category: "equipment", subcategory: "gear", derivedTagsAny: ["social_infiltration"] }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Masquerade Scarf", "Quick-Change Outfit"]));
    expect(service.listRecords({ category: "creature", derivedTagsAny: ["aquatic_context"] }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Ghost Sailor", "Pelagic Stalker", "Ship Captain"]));

    const cythnigot = service.lookup("Cythnigot", { category: "creature" }).match;
    expect(cythnigot?.hasDescription).toBe(true);
    expect(cythnigot?.descriptionSnippet).toBe("Small aberration.");
    expect(cythnigot?.sourceCategory).toBe("core");
    expect(cythnigot?.category).toBe("creature");
    const seaBlessing = service.lookup("Sea Blessing", { category: "spell" }).match;
    expect(seaBlessing?.subcategory).toBeNull();
    expect(seaBlessing?.traditions).toEqual(["primal"]);
    const focusBurst = service.lookup("Focus Burst", { category: "spell" }).match;
    expect(focusBurst?.subcategory).toBeNull();
    expect(focusBurst?.spellKinds).toEqual(["focus"]);
    const antidote = service.lookup("Antidote (Lesser)", { category: "equipment" }).match;
    expect(antidote?.derivedTags).toEqual(expect.arrayContaining(["beneficial", "anti_poison"]));
    expect(antidote?.derivedTags).not.toEqual(expect.arrayContaining(["offensive", "thrown_offense"]));
    const bottledCatharsis = service.lookup("Bottled Catharsis (Serenity)", { category: "equipment" }).match;
    expect(bottledCatharsis?.derivedTags).toEqual(expect.arrayContaining(["beneficial", "condition_support", "mental_recovery"]));
    const shipCaptain = service.lookup("Ship Captain", { category: "creature" }).match;
    expect(shipCaptain?.derivedTags).toEqual(expect.arrayContaining(["nautical", "profession_npc", "scene_adjacent"]));
    const pelagicStalker = service.lookup("Pelagic Stalker", { category: "creature" }).match;
    expect(pelagicStalker?.derivedTags).toContain("aquatic_context");
    const spaciousPouch = service.lookup("Spacious Pouch (Type I)", { category: "equipment" }).match;
    expect(spaciousPouch?.derivedTags).toContain("carry_support");
    const masqueradeScarf = service.lookup("Masquerade Scarf", { category: "equipment" }).match;
    expect(masqueradeScarf?.derivedTags).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));
    const quickChangeOutfit = service.lookup("Quick-Change Outfit", { category: "equipment" }).match;
    expect(quickChangeOutfit?.derivedTags).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));
    expect(service.lookup("Blinded", { category: "rule", subcategory: "condition" }).match?.category).toBe("rule");
  });

  it("keeps deterministic listing distinct from structured ranked search", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const listed = service.listRecords({
      category: "creature",
      levelMin: 2,
      levelMax: 2,
    }).records.map((record) => `${record.name}::${record.packLabel}`);
    const searched = (await service.search({
      category: "creature",
      levelMin: 2,
      levelMax: 2,
    })).records.map((record) => `${record.name}::${record.packLabel}`);

    expect(listed).not.toEqual(searched);
    expect(listed.indexOf("Bilge Skeleton::Quest for the Frozen Flame")).toBeLessThan(
      listed.indexOf("Diver::Pathfinder Monster Core"),
    );
    expect(searched.indexOf("Bilge Skeleton::Quest for the Frozen Flame")).toBeGreaterThan(
      searched.indexOf("Diver::Pathfinder Monster Core"),
    );
  });

  it("normalizes legacy plural aliases and supports scoped mixed-family filters", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const legacyCategoryLookup = service.lookup("Cythnigot", { category: "creatures" }).match;
    expect(legacyCategoryLookup?.category).toBe("creature");

    const scopedResults = await service.search({
      scopes: [
        { category: "feats" },
        { category: "rules", subcategories: ["actions"] },
      ],
      limit: 20,
    });
    expect(scopedResults.records.some((record) => record.name === "Deep Focus" && record.category === "feat")).toBe(true);
    expect(scopedResults.records.some((record) => record.name === "Refocus" && record.category === "rule")).toBe(true);
    expect(scopedResults.records.some((record) => record.category === "creature")).toBe(false);

    await expect(service.search({
      scopes: [{ category: "feat", subcategories: ["action"] }],
    })).rejects.toThrow(/does not belong to category "feat"/i);
  });

  it("requires a text query or structured filters for ranked search", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    await expect(service.search({})).rejects.toThrow("pf2e_search requires search text and/or at least one structured filter.");
    await expect(service.search({ searchProfile: "concept" })).rejects.toThrow("pf2e_search requires search text and/or at least one structured filter.");
  });

  it("maps user-facing search profiles onto the underlying retrieval modes", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const lexicalResults = await service.search({
      searchProfile: "lexical",
      query: "aberration",
      category: "creature",
    });
    expect(lexicalResults.searchProfile).toBe("lexical");
    expect(lexicalResults.mode).toBe("lexical");
    expect(lexicalResults.records[0]?.name).toBe("Cythnigot");

    const balancedResults = await service.search({
      searchProfile: "balanced",
      query: "ghost ship",
      category: "creature",
    });
    expect(balancedResults.searchProfile).toBe("balanced");
    expect(balancedResults.mode).toBe("hybrid");

    const conceptResults = await service.search({
      searchProfile: "concept",
      query: "ghost ship",
      category: "creature",
      explain: true,
    });
    expect(conceptResults.searchProfile).toBe("concept");
    expect(conceptResults.mode).toBe("hybrid");
    expect(conceptResults.explain?.searchProfile).toBe("concept");
    expect(conceptResults.explain?.fusionMethod).toBe("weightedRrf");
    expect(conceptResults.explain?.fusionProfile).toBe("concept");
    expect(conceptResults.explain?.fusionConfig).toEqual({
      rrfK: 60,
      lexicalWeight: 0.3,
      semanticWeight: 0.7,
      lexicalTopK: 100,
      semanticTopK: 150,
    });
  });

  it("uses normalized text for lexical scoring and raw query text for embeddings", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    const embeddingCalls: string[] = [];

    const service = await loadTestService(fixture, {
      embeddingProviderFactory: createCapturingEmbeddingProviderFactory(embeddingCalls, {
        provider: "hash",
        model: "capture-model",
        revision: null,
        dimensions: 8,
      }),
    });

    const query = "  Ghost-ship: body horror?!  ";
    const result = await service.search({
      searchProfile: "concept",
      query,
      category: "creature",
      explain: true,
    });

    expect(embeddingCalls.at(-1)).toBe("Ghost-ship: body horror?!");
    expect(result.explain?.semanticQuery).toBe("Ghost-ship: body horror?!");
    expect(result.explain?.lexicalQuery).toBe("ghost ship body horror");
    expect(result.explain?.query?.normalizedQuery).toBe("ghost ship body horror");
  });

  it("surfaces haunted-ship swarm candidates in broad themed search", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const broadQuery =
      "ghost ship cursed voyage fear fog darkness possession maddening whispers vermin in the hold wrong-feeling stowaways body horror haunted physically unclean";
    const broadResults = await service.search({
      category: "creature",
      levelMin: 1,
      levelMax: 5,
      rarity: "common",
      query: broadQuery,
      limit: 20,
      explain: true,
    });
    const broadNames = broadResults.records.map((record) => record.name);
    const crawlingIndex = broadNames.indexOf("Crawling Hand Swarm");

    expect(broadResults.mode).toBe("hybrid");
    expect(crawlingIndex).toBeGreaterThanOrEqual(0);

    const crawlingExplain = broadResults.explain?.records.find((record) => record.name === "Crawling Hand Swarm");
    expect(broadResults.explain?.query?.queryTokens).toEqual(expect.arrayContaining(["ghost", "ship", "body", "horror"]));
    expect(Array.isArray(crawlingExplain?.matchedTraits)).toBe(true);
    expect(Array.isArray(crawlingExplain?.matchedNameTokens)).toBe(true);
    expect(typeof crawlingExplain?.lexicalRerankScore).toBe("number");
    expect(crawlingExplain?.fusionScore).not.toBeNull();
    expect(crawlingExplain?.rerankAdjustments.sourcePenalty ?? 0).toBe(0);

    const lexicalResults = await service.search({
      category: "creature",
      levelMin: 1,
      levelMax: 5,
      searchProfile: "lexical",
      query: "undead swarm body horror haunted ship crawling infestation severed limbs cursed voyage",
      limit: 20,
    });
    const lexicalNames = lexicalResults.records.map((record) => record.name);
    const lexicalCrawlingIndex = lexicalNames.indexOf("Crawling Hand Swarm");
    const lexicalDiverIndex = lexicalNames.indexOf("Diver");
    const lexicalLionIndex = lexicalNames.indexOf("Lion");
    expect(lexicalCrawlingIndex).toBeGreaterThanOrEqual(0);
    expect(lexicalDiverIndex).toSatisfy((index) => index === -1 || index > lexicalCrawlingIndex);
    expect(lexicalLionIndex).toSatisfy((index) => index === -1 || index > lexicalCrawlingIndex);
  });

  it("applies small source-quality preferences and stronger thematic unique penalties", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const bilgeResults = await service.search({
      category: "creature",
      nameQuery: "Bilge Skeleton",
      explain: true,
    });
    expect(bilgeResults.records[0]?.sourceCategory).toBe("core");

    const coreBilgeExplain = bilgeResults.explain?.records.find((record) => record.name === "Bilge Skeleton" && record.rerankAdjustments.sourceQuality > 0);
    const adventureBilgeExplain = bilgeResults.explain?.records.find((record) => record.name === "Bilge Skeleton" && record.rerankAdjustments.sourceQuality < 0);
    expect(coreBilgeExplain?.rerankAdjustments.sourceQuality).toBe(0.04);
    expect(adventureBilgeExplain?.rerankAdjustments.sourceQuality).toBe(-0.01);

    const sentinelResults = await service.search({
      category: "creature",
      query: "sentinel guardian ancient ruins watch intruders",
      limit: 10,
      explain: true,
    });
    const sentinelNames = sentinelResults.records.map((record) => record.name);
    const commonIndex = sentinelNames.indexOf("Amber Sentinel");
    const uncommonIndex = sentinelNames.indexOf("Azure Sentinel");
    const rareIndex = sentinelNames.indexOf("Gloam Sentinel");
    const uniqueIndex = sentinelNames.indexOf("Last Sentinel");

    expect(commonIndex).toBeGreaterThanOrEqual(0);
    expect(uncommonIndex).toBeGreaterThanOrEqual(0);
    expect(rareIndex).toBeGreaterThanOrEqual(0);
    expect(uniqueIndex).toBeGreaterThan(rareIndex);

    const uniqueExplain = sentinelResults.explain?.records.find((record) => record.name === "Last Sentinel");
    const rareExplain = sentinelResults.explain?.records.find((record) => record.name === "Gloam Sentinel");
    expect(uniqueExplain?.rerankAdjustments.rarityPreference).toBe(-0.2);
    expect(rareExplain?.rerankAdjustments.rarityPreference).toBe(0.01);

    const exactUniqueResults = await service.search({
      category: "creature",
      nameQuery: "Last Sentinel",
      explain: true,
    });
    expect(exactUniqueResults.records[0]?.name).toBe("Last Sentinel");
    const exactUniqueExplain = exactUniqueResults.explain?.records.find((record) => record.name === "Last Sentinel");
    expect(exactUniqueExplain?.rerankAdjustments.rarityPreference).toBe(-0.03);
  });

  it("hot-reloads ranking weights without rebuilding the service", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    const rankingConfigPath = path.join(fixture.root, "pf2e-ranking.json");
    const rankingConfigStore = await RankingConfigStore.create(rankingConfigPath, { watch: false });
    const service = await loadTestService(fixture, { rankingConfigStore });

    const baselineResults = await service.search({
      category: "creature",
      nameQuery: "Bilge Skeleton",
      explain: true,
    });
    expect(baselineResults.records[0]?.sourceCategory).toBe("core");
    expect(baselineResults.explain?.rankingConfig.source).toBe("default");

    const baselineRevision = service.getRankingConfigStatus().revision;
    await writeJson(rankingConfigPath, {
      sourceQuality: {
        core: -0.5,
        adventure: 0.5,
      },
    });
    await rankingConfigStore.reload();

    const updatedResults = await service.search({
      category: "creature",
      nameQuery: "Bilge Skeleton",
      explain: true,
    });
    expect(updatedResults.records[0]?.sourceCategory).toBe("adventure");
    expect(updatedResults.explain?.rankingConfig.source).toBe("file");
    expect(updatedResults.explain?.rankingConfig.revision).toBeGreaterThan(baselineRevision);
    expect(updatedResults.explain?.records.some((record) => record.rerankAdjustments.sourceQuality === 0.5)).toBe(true);
    service.close();
  });

  it("excludes dedicated Pathfinder Society content while retaining base equivalents", async () => {
    const fixture = await createHardFilterFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    expect(service.listPacks().map((pack) => pack.name)).not.toContain("macros");
    expect(service.listPacks().map((pack) => pack.name)).not.toContain("action-macros");
    expect(service.lookup("Grimstalker", { category: "creature" }).match?.name).toBe("Grimstalker");
    expect(service.lookup("Ghoul", { category: "creature" }).match?.name).toBe("Ghoul");
    expect(service.lookup("Zebub", { category: "creature" }).match?.name).toBe("Zebub");
    expect(service.lookup("Raise Shield", { category: "rule", subcategory: "action" }).match?.name).toBe("Raise a Shield");

    expect((await service.search({ nameQuery: "Grimstalker (PFS 3-13)", category: "creature" })).records.map((record) => record.name)).not.toContain("Grimstalker (PFS 3-13)");
    expect((await service.search({ nameQuery: "Ghoul (PFS Intro 2)", category: "creature" })).records.map((record) => record.name)).not.toContain("Ghoul (PFS Intro 2)");
    expect((await service.search({ nameQuery: "Zebub (PFS)", category: "creature" })).records.map((record) => record.name)).not.toContain("Zebub (PFS)");
    expect((await service.search({ nameQuery: "Magical Mentor" })).records.map((record) => record.name)).not.toContain("Magical Mentor");
    expect((await service.search({ nameQuery: "Effect: Magical Mentor" })).records.map((record) => record.name)).not.toContain("Effect: Magical Mentor");
    expect((await service.search({ nameQuery: "Treat Wounds" })).records.map((record) => record.name)).not.toContain("Treat Wounds");
    expect((await service.search({ nameQuery: "Trip: Athletics" })).records.map((record) => record.name)).not.toContain("Trip: Athletics");

    const featResults = (await service.search({
      category: "feat",
      query: "mentor training support teamwork guidance",
      limit: 10,
    })).records.map((record) => record.name);
    expect(featResults).toContain("Proud Mentor");
    expect(featResults).not.toContain("Magical Mentor");
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
        category: "creature",
        family: "context",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "swamp", description: expect.any(String) }),
          expect.objectContaining({ value: "graveyard", description: expect.any(String) }),
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
        { value: "mechanical", count: 1 },
        { value: "trap", count: 1 },
      ],
    });

    expect(service.listFilterValues({
      field: "derivedTags",
      category: "equipment",
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["beneficial", "offensive", "climbing", "lock_bypass", "mental_recovery", "carry_support"]));

    expect(service.listFilterValues({
      field: "subcategories",
      category: "hazard",
    })).toEqual({
      field: "subcategories",
      values: [
        { value: "haunt", count: 1 },
        { value: "trap", count: 1 },
      ],
    });

    expect(service.listFilterValues({
      field: "publicationTitle",
      category: "spell",
    })).toEqual({
      field: "publicationTitle",
      values: [
        { value: "Pathfinder Player Core", count: 3 },
      ],
    });

    expect(service.listFilterValues({
      field: "traditions",
      category: "spell",
    })).toEqual({
      field: "traditions",
      values: [
        { value: "occult", count: 2 },
        { value: "arcane", count: 1 },
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
      field: "rarity",
      category: "creature",
    }).values.map((entry) => entry.value)).toEqual(["common", "uncommon", "rare", "unique"]);

    expect(service.listFilterValues({
      field: "size",
      category: "creature",
    }).values.map((entry) => entry.value)).toEqual(["med", "sm", "lg"]);

    expect(service.listFilterValues({
      field: "sources",
      category: "creature",
    }).values.map((entry) => entry.value)).toEqual(["core", "adventure", "rules"]);

    expect(service.listFilterValues({
      field: "packs",
      category: "spell",
    })).toEqual({
      field: "packs",
      values: [
        { value: "Spells", count: 2 },
        { value: "Spells SRD", count: 1 },
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
        { value: "rule", count: 4 },
        { value: "hazard", count: 1 },
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

  it("loads an unchanged SQLite index and requires explicit rebuild when the source changes", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    const indexPath = path.join(fixture.root, ".cache", "pf2e-index.sqlite");

    const firstService = await loadTestService(fixture, { indexPath });
    expect(firstService.getStats()).toEqual({ packCount: 13, recordCount: 52 });
    firstService.close();

    const firstMtime = (await import("node:fs/promises")).stat(indexPath).then((details) => details.mtimeMs);
    const unchangedService = await openPreparedTestService(fixture, { indexPath });
    expect(unchangedService.getStats()).toEqual({ packCount: 13, recordCount: 52 });
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
    expect(rebuiltService.getStats()).toEqual({ packCount: 13, recordCount: 53 });
    expect(rebuiltService.lookup("Sea Ghoul", { category: "creature" }).match?.name).toBe("Sea Ghoul");
    rebuiltService.close();
  });

  it("treats untracked JSON files in git checkouts as stale source changes", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    await initializeGitFixture(fixture.root);
    const indexPath = path.join(fixture.root, ".cache", "pf2e-index.sqlite");

    const firstService = await loadTestService(fixture, { indexPath });
    expect(firstService.getStats()).toEqual({ packCount: 13, recordCount: 52 });
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

  it("indexes verified aliases onto remaster canonical records and exposes linked legacy records", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const embedCalls: string[] = [];
    const service = await loadTestService(fixture, {
      embeddingProviderFactory: createCapturingEmbeddingProviderFactory(embedCalls, {
        provider: "hash",
        model: "feature-hash-192",
        revision: null,
        dimensions: 4,
      }),
    });

    expect(service.lookup("Attack of Opportunity", { category: "rule", subcategory: "action" }).match?.name).toBe("Reactive Strike");
    expect(service.lookup("Strike Back", { category: "rule", subcategory: "action" }).match?.name).toBe("Reactive Strike");
    expect(service.lookup("flat-footed", { category: "rule", subcategory: "condition" }).match?.name).toBe("Off-Guard");
    expect(service.lookup("Aasimar").match?.name).toBe("Nephilim");
    expect(service.lookup("Ifrit").match?.name).toBe("Naari");
    expect(service.lookup("Feather Token (Swan Boat)").match?.name).toBe("Marvelous Miniature (Boat)");
    expect(service.lookup("Bag of Holding", { category: "equipment" }).match?.name).toBe("Spacious Pouch (Type I)");
    expect(service.lookup("Attack of Opportunity", { category: "rule", subcategory: "action" }).match?.aliases).toContain("Attack of Opportunity");
    expect(service.lookup("Strike Back", { category: "rule", subcategory: "action" }).match?.aliases).toContain("Strike Back");
    expect(service.lookup("flat-footed", { category: "rule", subcategory: "condition" }).match?.aliases).toContain("flat-footed");
    expect(service.lookup("Aasimar").match?.aliases).toContain("Aasimar");
    expect(service.lookup("Ifrit").match?.aliases).toContain("Ifrit");
    expect(service.lookup("Bag of Holding", { category: "equipment" }).match?.aliases).toContain("Bag of Holding");

    const attackSearch = await service.search({
      category: "rule",
      subcategory: "action",
      nameQuery: "Attack of Opportunity",
    });
    expect(attackSearch.records.map((record) => record.name)).toContain("Reactive Strike");
    expect(attackSearch.records.map((record) => record.name)).not.toContain("Attack of Opportunity");

    const offGuard = service.lookup("Off-Guard", { category: "rule", subcategory: "condition" }).match;
    expect(offGuard?.aliases).toContain("flat-footed");
    expect(offGuard?.legacyRecordLinks).toEqual([
      {
        recordKey: "conditionitems:Flat-Footed",
        name: "Flat-Footed",
      },
    ]);
    expect(service.getRecord(offGuard!.legacyRecordLinks[0]!.recordKey)?.name).toBe("Flat-Footed");

    expect(embedCalls.some((text) => text.includes("Attack of Opportunity") && text.includes("Reactive Strike"))).toBe(true);
    expect(embedCalls.some((text) => text.includes("flat-footed") && text.includes("Off-Guard"))).toBe(true);
    expect(embedCalls.some((text) => text.includes("Aasimar") && text.includes("Nephilim"))).toBe(true);

    const nephilim = service.lookup("Nephilim").match;
    expect(nephilim?.aliases).toContain("Tiefling");
    expect(nephilim?.aliases).not.toContain("and Tiefling");

    const naari = service.lookup("Naari").match;
    expect(naari?.aliases).toContain("Ifrit");
    expect(naari?.aliases.some((alias) => alias.includes("are now"))).toBe(false);

    const boat = service.lookup("Marvelous Miniature (Boat)").match;
    expect(boat?.aliases).toContain("Feather Token (Swan Boat)");

    expect(service.lookup("Sight-Theft Grit", { category: "equipment" }).match?.name).not.toBe("Surging Serum (Lesser)");
    expect(service.lookup("Surging Serum (Lesser)", { category: "equipment" }).match?.aliases).not.toContain("Sight-Theft Grit");
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
