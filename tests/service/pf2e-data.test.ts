import { mkdtemp, mkdir, writeFile } from "node:fs/promises";
import os from "node:os";
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

async function createFixture(): Promise<{ root: string; manifestPath: string }> {
  const root = await mkdtemp(path.join(os.tmpdir(), "pf2e-mcp-test-"));
  const packRoot = path.join(root, "packs", "pf2e");
  const packNames = [
    "actions",
    "actionspf2e",
    "afflictions",
    "bestiary-family-ability-glossary",
    "classfeatures",
    "conditionitems",
    "equipment",
    "equipment-srd",
    "feats-srd",
    "heritages",
    "journals",
    "pathfinder-monster-core",
    "pathfinder-npc-core",
    "pfs-season-1-bestiary",
    "quest-for-the-frozen-flame-bestiary",
    "spells",
    "spells-srd",
  ];

  await Promise.all(packNames.map(async (packName) => mkdir(path.join(packRoot, packName), { recursive: true })));
  await Promise.all([
    mkdir(path.join(packRoot, "bestiary-family-ability-glossary", "ghost"), { recursive: true }),
    mkdir(path.join(packRoot, "bestiary-family-ability-glossary", "lich"), { recursive: true }),
    mkdir(path.join(packRoot, "bestiary-family-ability-glossary", "mythic"), { recursive: true }),
    mkdir(path.join(packRoot, "bestiary-family-ability-glossary", "vampire"), { recursive: true }),
    mkdir(path.join(packRoot, "pathfinder-npc-core", "seafarer"), { recursive: true }),
  ]);
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
        name: "afflictions",
        label: "Afflictions",
        path: "packs/afflictions",
        type: "Item",
      },
      {
        name: "bestiary-family-ability-glossary",
        label: "Creature Family Ability Glossary",
        path: "packs/bestiary-family-ability-glossary",
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
        name: "pathfinder-npc-core",
        label: "Pathfinder NPC Core",
        path: "packs/pathfinder-npc-core",
        type: "Actor",
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

  await writeJson(path.join(packRoot, "actionspf2e", "escape.json"), {
    _id: "action-escape-1",
    name: "Escape",
    type: "action",
    system: {
      description: {
        value: "<p>You attempt to break free from being @UUID[Compendium.pf2e.conditionitems.Item.Grabbed]{Grabbed} or @UUID[Compendium.pf2e.conditionitems.Item.Restrained]{Restrained}.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["attack"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "grapple.json"), {
    _id: "action-grapple-1",
    name: "Grapple",
    type: "action",
    system: {
      description: {
        value: "<p>You grab a creature and hold it in place.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["attack"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "balance.json"), {
    _id: "action-balance-1",
    name: "Balance",
    type: "action",
    system: {
      description: {
        value: "<p>You move across a narrow surface or uneven ground.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["move"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "high-jump.json"), {
    _id: "action-high-jump-1",
    name: "High Jump",
    type: "action",
    system: {
      description: {
        value: "<p>You leap high into the air.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["move"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "long-jump.json"), {
    _id: "action-long-jump-1",
    name: "Long Jump",
    type: "action",
    system: {
      description: {
        value: "<p>You leap forward in a long arc.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["move"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "sense-direction.json"), {
    _id: "action-sense-direction-1",
    name: "Sense Direction",
    type: "action",
    system: {
      description: {
        value: "<p>You use your surroundings to determine direction.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["exploration"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "track.json"), {
    _id: "action-track-1",
    name: "Track",
    type: "action",
    system: {
      description: {
        value: "<p>You follow the trail left by a creature or group.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["exploration"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "cover-tracks.json"), {
    _id: "action-cover-tracks-1",
    name: "Cover Tracks",
    type: "action",
    system: {
      description: {
        value: "<p>You obscure the trail you leave behind.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["exploration"],
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

  await writeJson(path.join(packRoot, "conditionitems", "grabbed.json"), {
    _id: "Grabbed",
    name: "Grabbed",
    type: "condition",
    system: {
      description: {
        value: "<p>You are held in place and can use @UUID[Compendium.pf2e.actionspf2e.Item.Escape]{Escape} to break free.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "conditionitems", "restrained.json"), {
    _id: "Restrained",
    name: "Restrained",
    type: "condition",
    system: {
      description: {
        value: "<p>You are tightly bound and usually need to @UUID[Compendium.pf2e.actionspf2e.Item.Escape]{Escape}.</p>",
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

  await writeJson(path.join(packRoot, "equipment", "potion-cold-resistance-moderate.json"), {
    _id: "potion-cold-resistance-1",
    name: "Potion of Cold Resistance (Moderate)",
    type: "consumable",
    system: {
      description: {
        value: "<p>Drinking this thick, fortifying potion grants resistance 10 against cold damage for 1 hour.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["consumable", "magical", "potion"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "bloodhound-mask-greater.json"), {
    _id: "bloodhound-mask-1",
    name: "Bloodhound Mask (Greater)",
    type: "consumable",
    system: {
      description: {
        value: "<p>Once activated, the mask sharpens odors, giving you imprecise scent with a 60-foot range.</p><p>When you use Survival to @UUID[Compendium.pf2e.actionspf2e.Item.Track]{Track} a creature by its scent, the mask grants you a +3 item bonus to your Survival check.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "aroma-concealer.json"), {
    _id: "aroma-concealer-1",
    name: "Aroma Concealer",
    type: "consumable",
    system: {
      description: {
        value: "<p>This oily mix can be applied to a creature to reduce and cover any ordinary odors they produce. The creature receives a +2 item bonus to Stealth checks to @UUID[Compendium.pf2e.actionspf2e.Item.Hide]{Hide} or @UUID[Compendium.pf2e.actionspf2e.Item.Sneak]{Sneak} against creatures using primarily smell. This bonus also applies to the DC to @UUID[Compendium.pf2e.actionspf2e.Item.Track]{Track} the creature by scent.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "ichthyosis-mutagen.json"), {
    _id: "ichthyosis-mutagen-1",
    name: "Ichthyosis Mutagen",
    type: "consumable",
    system: {
      description: {
        value: "<p>Benefit You gain fast healing 2.</p><p>Drawback Any creature attempting to @UUID[Compendium.pf2e.actionspf2e.Item.Track]{Track} you in the next 24 hours gains a +4 circumstance bonus to their check.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable", "mutagen"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "escape-fulu.json"), {
    _id: "escape-fulu-1",
    name: "Escape Fulu",
    type: "consumable",
    system: {
      description: {
        value: "<p>Trigger You attempt to @UUID[Compendium.pf2e.actionspf2e.Item.Escape]{Escape}.</p><p>The escape fulu is a charm worn in case of kidnapping. When you activate this fulu, you gain a +2 status bonus to checks to Escape for 1 minute.</p>",
      },
      publication: {
        title: "Pathfinder Lost Omens Tian Xia Character Guide",
      },
      traits: {
        rarity: "common",
        value: ["consumable", "fulu", "magical", "talisman"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "travelers-fulu.json"), {
    _id: "travelers-fulu-1",
    name: "Traveler's Fulu",
    type: "consumable",
    system: {
      description: {
        value: "<p>Trigger You attempt to @UUID[Compendium.pf2e.actionspf2e.Item.Sense Direction]{Sense Direction}.</p><p>This fulu shows constellations and arrows across the night sky. Your attempt to Sense Direction functions as if you have a compass, and you use the outcome one degree of success better than the result of your Survival check.</p>",
      },
      publication: {
        title: "Pathfinder Lost Omens Tian Xia Character Guide",
      },
      traits: {
        rarity: "common",
        value: ["consumable", "fulu", "magical", "talisman"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "trackers-stew.json"), {
    _id: "trackers-stew-1",
    name: "Tracker's Stew",
    type: "consumable",
    system: {
      description: {
        value: "<p>Once you've eaten the stew, it improves your ability to sense and follow tracks for 24 hours. You gain a +1 item bonus to Survival checks to @UUID[Compendium.pf2e.actionspf2e.Item.Cover Tracks]{Cover Tracks} and @UUID[Compendium.pf2e.actionspf2e.Item.Track]{Track}.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "potion-of-disguise-moderate.json"), {
    _id: "potion-disguise-1",
    name: "Potion of Disguise (Moderate)",
    type: "consumable",
    system: {
      description: {
        value: "<p>Upon imbibing this potion, you take on the appearance of a specific type of creature for 2d12 hours.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["consumable", "magical", "polymorph", "potion"],
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

  await writeJson(path.join(packRoot, "equipment", "boots-of-free-running-greater.json"), {
    _id: "boots-free-running-1",
    name: "Boots of Free Running (Greater)",
    type: "equipment",
    system: {
      description: {
        value: "<p>These comfortable and practical boots slip on easily and fill you with boundless energy. The treads of these boots provide exceptional traction, with improved grip on surfaces you would traditionally have difficulty traversing. While wearing the boots, you gain a +3 item bonus to Acrobatics checks to @UUID[Compendium.pf2e.actionspf2e.Item.Balance]{Balance} and to Athletics checks to @UUID[Compendium.pf2e.actionspf2e.Item.High Jump]{High Jump} and @UUID[Compendium.pf2e.actionspf2e.Item.Long Jump]{Long Jump}.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["invested", "magical"],
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

  await writeJson(path.join(packRoot, "equipment-srd", "trackers-goggles.json"), {
    _id: "trackers-goggles-1",
    name: "Tracker's Goggles",
    type: "equipment",
    system: {
      description: {
        value: "<p>While wearing these goggles, you gain a +1 bonus to Survival checks to @UUID[Compendium.pf2e.actionspf2e.Item.Sense Direction]{Sense Direction} and @UUID[Compendium.pf2e.actionspf2e.Item.Track]{Track}. If you fail a check to Track, you can try again after 30 minutes rather than an hour.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["invested", "magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "tracking-tag.json"), {
    _id: "tracking-tag-1",
    name: "Tracking Tag",
    type: "equipment",
    system: {
      description: {
        value: "<p>Tracking tags are attached to wild animals to track their movements and identify individual creatures among a herd.</p>",
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

  await writeJson(path.join(packRoot, "equipment-srd", "trackless.json"), {
    _id: "trackless-1",
    name: "Trackless",
    type: "equipment",
    system: {
      description: {
        value: "<p>Trackless footwear is favored by anyone fleeing pursuit. While wearing it, you gain a +4 item bonus to the DC to track you.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["magical"],
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

  await writeJson(path.join(packRoot, "equipment-srd", "swallow-spike.json"), {
    _id: "equip-swallow-spike-1",
    name: "Swallow-Spike",
    type: "equipment",
    system: {
      description: {
        value: "<p>Your armor responds to your desire to break free of a creature grabbing you by growing spikes.</p><p>Trigger You become @UUID[Compendium.pf2e.conditionitems.Item.Grabbed]{Grabbed}, @UUID[Compendium.pf2e.conditionitems.Item.Restrained]{Restrained}, or another immobilizing effect that would hold you until you @UUID[Compendium.pf2e.actionspf2e.Item.Escape]{Escape}.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "shacklebreaker.json"), {
    _id: "equip-shacklebreaker-1",
    name: "Shacklebreaker",
    type: "equipment",
    system: {
      description: {
        value: "<p>This bracelet has three charms depicting a dagger, a shield, and a rose. Whenever you roll a success to free someone from manacles, it counts as two successes.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "handcuffs-average.json"), {
    _id: "equip-handcuffs-average-1",
    name: "Handcuffs (Average)",
    type: "equipment",
    system: {
      description: {
        value: "<p>These handcuffs possess a ratcheting lock system in each cuff that allows them to be quickly cinched down on a captive's limbs, even if they're actively resisting.</p>",
      },
      publication: {
        title: "Pathfinder GM Core",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "catch-pole.json"), {
    _id: "equip-catch-pole-1",
    name: "Catch Pole",
    type: "equipment",
    system: {
      description: {
        value: "<p>This sturdy pole has a rope attached to one end in a loop. You can pull the handle side of the rope to tighten the loop. Using this loop, you can @UUID[Compendium.pf2e.actionspf2e.Item.Grapple]{Grapple} without having a free hand.</p>",
      },
      publication: {
        title: "Pathfinder GM Core",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "lawbringers-lasso.json"), {
    _id: "equip-lawbringers-lasso-1",
    name: "Lawbringer's Lasso",
    type: "equipment",
    system: {
      description: {
        value: "<p>This enchanted lasso is a @UUID[Compendium.pf2e.equipment-srd.Item.Net]{Net} that can be used to @UUID[Compendium.pf2e.actionspf2e.Item.Grapple]{Grapple} creatures up to 30 feet away and has an @UUID[Compendium.pf2e.actionspf2e.Item.Escape]{Escape} DC of 18.</p>",
      },
      publication: {
        title: "Pathfinder Lost Omens Knights of Lastwall",
      },
      traits: {
        rarity: "common",
        value: ["lawful", "magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "injigos-loving-embrace.json"), {
    _id: "equip-injigos-loving-embrace-1",
    name: "Injigo's Loving Embrace",
    type: "equipment",
    system: {
      description: {
        value: "<p>Injigo's Loving Embrace functions as a typical net. You gain a +1 item bonus to Athletics checks to @UUID[Compendium.pf2e.actionspf2e.Item.Grapple]{Grapple} with the net.</p><p>The creature must succeed at a DC 25 check to @UUID[Compendium.pf2e.actionspf2e.Item.Escape]{Escape} the net.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "false-manacles.json"), {
    _id: "equip-false-manacles-1",
    name: "False Manacles",
    type: "equipment",
    system: {
      description: {
        value: "<p>These manacles are nearly indistinguishable from real manacles upon inspection, but contain a hidden release that enables a wearer who knows the location of the release to free themselves with a single Interact action.</p>",
      },
      publication: {
        title: "Pathfinder Lost Omens Impossible Lands",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "manacles-of-persuasion.json"), {
    _id: "equip-manacles-of-persuasion-1",
    name: "Manacles of Persuasion",
    type: "equipment",
    system: {
      description: {
        value: "<p>When the manacles are locked around an immobilized creature's wrists, they begin to sap the life out of the victim.</p>",
      },
      publication: {
        title: "Pathfinder Lost Omens Gods & Magic",
      },
      traits: {
        rarity: "common",
        value: ["magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "ghost-charge-prototype.json"), {
    _id: "weapon-bomb-1",
    name: "Ghost Charge Prototype",
    type: "weapon",
    system: {
      category: "martial",
      group: "bomb",
      damage: {
        damageType: "positive",
      },
      description: {
        value: "<p>A prototype bomb that bursts with cleansing light.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "bomb"],
      },
      usage: {
        value: "held-in-one-hand",
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "practice-sword.json"), {
    _id: "weapon-sword-1",
    name: "Practice Sword",
    type: "weapon",
    system: {
      category: "martial",
      group: "sword",
      damage: {
        damageType: "slashing",
      },
      description: {
        value: "<p>A balanced practice blade for drilling sword forms.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["martial"],
      },
      usage: {
        value: "held-in-one-hand",
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

  await writeJson(path.join(packRoot, "bestiary-family-ability-glossary", "ghost", "ghost-rejuvenation.json"), {
    _id: "ghost-rejuvenation-1",
    name: "Rejuvenation",
    type: "action",
    system: {
      description: {
        value: "<p>A destroyed ghost reforms unless its unfinished business is resolved.</p>",
      },
      publication: {
        title: "Pathfinder Monster Core",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "bestiary-family-ability-glossary", "lich", "lich-rejuvenation.json"), {
    _id: "lich-rejuvenation-1",
    name: "Rejuvenation",
    type: "action",
    system: {
      description: {
        value: "<p>A lich returns through its soul cage unless the cage is destroyed.</p>",
      },
      publication: {
        title: "Pathfinder Monster Core",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "bestiary-family-ability-glossary", "mythic", "recharge-spell.json"), {
    _id: "mythic-recharge-spell-1",
    name: "Recharge Spell",
    type: "action",
    system: {
      description: {
        value: "<p>The creature regains one expended spell.</p>",
      },
      publication: {
        title: "Pathfinder War of Immortals",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "bestiary-family-ability-glossary", "mythic", "mythic-power.json"), {
    _id: "mythic-power-1",
    name: "Mythic Power",
    type: "action",
    system: {
      description: {
        value: "<p>The creature can spend Mythic Points on extraordinary actions.</p>",
      },
      publication: {
        title: "Pathfinder War of Immortals",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "bestiary-family-ability-glossary", "vampire", "dominate.json"), {
    _id: "vampire-dominate-1",
    name: "Dominate",
    type: "action",
    system: {
      description: {
        value: "<p>The vampire bends a victim to its will.</p>",
      },
      publication: {
        title: "Pathfinder Monster Core",
      },
      traits: {
        rarity: "common",
        value: ["mental"],
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
    items: [
      {
        _id: "ghost-commoner-rejuvenation",
        _stats: {
          compendiumSource: "Compendium.pf2e.bestiary-family-ability-glossary.Item.ghost-rejuvenation-1",
        },
        name: "Rejuvenation",
        type: "action",
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "mythic-lich.json"), {
    _id: "mythic-lich-core",
    name: "Mythic Lich",
    type: "npc",
    items: [
      {
        _id: "mythic-lich-recharge-spell",
        _stats: {
          compendiumSource: "Compendium.pf2e.bestiary-family-ability-glossary.Item.mythic-recharge-spell-1",
        },
        name: "Recharge Spell",
        type: "action",
      },
      {
        _id: "mythic-lich-mythic-power",
        _stats: {
          compendiumSource: "Compendium.pf2e.bestiary-family-ability-glossary.Item.mythic-power-1",
        },
        name: "Mythic Power",
        type: "action",
      },
      {
        _id: "mythic-lich-rejuvenation",
        _stats: {
          compendiumSource: "Compendium.pf2e.bestiary-family-ability-glossary.Item.lich-rejuvenation-1",
        },
        name: "Rejuvenation",
        type: "action",
      },
    ],
    system: {
      details: {
        level: {
          value: 12,
        },
        publication: {
          title: "Pathfinder War of Immortals",
        },
        publicNotes: "<p>An undead spellcaster elevated by mythic power and sustained by a soul cage.</p>",
      },
      traits: {
        rarity: "rare",
        value: ["undead", "wizard"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "morlock-thrall.json"), {
    _id: "morlock-thrall-core",
    name: "Morlock Thrall",
    type: "npc",
    items: [
      {
        _id: "morlock-thrall-dominate",
        _stats: {
          compendiumSource: "Compendium.pf2e.bestiary-family-ability-glossary.Item.vampire-dominate-1",
        },
        name: "Dominate",
        type: "action",
      },
    ],
    system: {
      details: {
        level: {
          value: 5,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A broken servant enthralled to a vampire household.</p>",
      },
      traits: {
        rarity: "common",
        value: ["humanoid"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-npc-core", "_folders.json"), [
    {
      _id: "folder-seafarer",
      name: "Seafarer",
      folder: null,
    },
  ]);

  await writeJson(path.join(packRoot, "pathfinder-npc-core", "seafarer", "bosun.json"), {
    _id: "npc-core-bosun",
    folder: "folder-seafarer",
    name: "Bosun",
    type: "npc",
    system: {
      details: {
        level: {
          value: 3,
        },
        publication: {
          title: "Pathfinder NPC Core",
        },
        publicNotes: "<p>A seasoned deck officer responsible for shipboard labor and discipline.</p>",
      },
      traits: {
        rarity: "common",
        value: ["human", "humanoid"],
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
    items: Array.from({ length: 45 }, (_, index) => ({
      _id: `ship-captain-order-${index + 1}`,
      name: `Deck Order ${index + 1}`,
      type: "action",
      system: {
        description: {
          value: `<p>Deck order ${index + 1} coordinates the crew around starboard rigging routines and battle stations.</p>`,
        },
        traits: {
          value: ["auditory", "visual"],
        },
      },
    })),
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "wealthy-vigilante.json"), {
    _id: "wealthy-vigilante",
    name: "Wealthy Vigilante",
    type: "npc",
    system: {
      details: {
        level: {
          value: 8,
        },
        publication: {
          title: "Pathfinder NPC Core",
        },
        publicNotes: "<p>By night, this member of the nobility dons a false identity to mete out violent, extralegal justice.</p>",
      },
      traits: {
        rarity: "common",
        value: ["human", "humanoid"],
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "alarm-ward.json"), {
    _id: "alarm-ward",
    name: "Alarm Ward",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 1,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A silent ward flares when a creature crosses the threshold, raising the alarm and alerting nearby guards.</p>",
      },
      traits: {
        rarity: "common",
        value: ["magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "snaring-glyph.json"), {
    _id: "snaring-glyph",
    name: "Snaring Glyph",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 3,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A glowing sigil lashes out with force bands to bind intruders in place until they @UUID[Compendium.pf2e.actionspf2e.Item.Escape]{Escape}.</p><p>The creature becomes @UUID[Compendium.pf2e.conditionitems.Item.Restrained]{Restrained}.</p>",
      },
      traits: {
        rarity: "common",
        value: ["magical"],
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

  await writeJson(path.join(packRoot, "afflictions", "cackling-delirium.json"), {
    _id: "affliction-1",
    name: "Cackling Delirium",
    type: "affliction",
    system: {
      description: {
        value: "<p>Mocking whispers leave the victim confused, frightened, and unable to trust their own senses.</p>",
      },
      level: {
        value: 4,
      },
      publication: {
        title: "Pathfinder GM Core",
      },
      traits: {
        rarity: "common",
        value: ["curse", "mental"],
      },
    },
  });

  await writeJson(path.join(packRoot, "afflictions", "calcifying-rot.json"), {
    _id: "affliction-2",
    name: "Calcifying Rot",
    type: "affliction",
    system: {
      description: {
        value: "<p>The disease stiffens joints, reduces the victim's Speed, and can leave them immobilized.</p>",
      },
      level: {
        value: 3,
      },
      publication: {
        title: "Pathfinder GM Core",
      },
      traits: {
        rarity: "common",
        value: ["disease"],
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

    expect(service.listPacks()).toHaveLength(16);
    expect(service.getStats()).toEqual({ packCount: 16, recordCount: 97 });
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
    expect(service.listRecords({ category: "creature", levelMin: 4, levelMax: 4, metadata: { field: "traits", op: "includesAny", values: ["undead"] } }).records.map((record) => record.name)).toEqual(["Ghost Commoner"]);
    expect((await service.search({ category: "creature", metadata: { field: "traits", op: "includesAll", values: ["fiend"] } })).records[0]?.name).toBe("Cythnigot");
    expect((await service.search({ category: "creature", metadata: { field: "traits", op: "includesAll", values: ["fiend"] } })).searchProfile).toBeNull();
    expect((await service.search({ category: "creature", metadata: { field: "size", op: "eq", value: "sm" } })).records.every((record) => record.size === "sm")).toBe(true);
    expect((await service.search({ searchProfile: "lexical", query: "aberration", category: "creature" })).records[0]?.name).toBe("Cythnigot");
    expect((await service.search({ category: "spell", metadata: { field: "traditions", op: "includesAny", values: ["primal"] }, actionCost: 2 })).records[0]?.name).toBe("Sea Blessing");
    expect((await service.search({ category: "spell", metadata: { field: "spellKinds", op: "includesAny", values: ["focus"] } })).records.map((record) => record.name)).toEqual(["Focus Burst"]);
    expect((await service.search({ category: "rule", subcategory: "condition" })).records.map((record) => record.name)).toEqual(
      expect.arrayContaining(["Blinded", "Dazzled", "Hidden"]),
    );
    expect((await service.search({ category: "hazard" })).records.map((record) => record.name)).toEqual(
      expect.arrayContaining(["Mournful Hallway", "Spear Launcher"]),
    );
    expect((await service.search({ category: "hazard", subcategory: "trap" })).records.map((record) => record.name)).toEqual(["Spear Launcher"]);
    expect(service.listRecords({ pack: "Pathfinder Monster Core", category: "hazard", subcategory: "trap" }).records.map((record) => record.name)).toEqual(["Spear Launcher"]);
    expect((await service.search({ category: "affliction" })).records.map((record) => record.name)).toEqual(
      expect.arrayContaining(["Cackling Delirium", "Calcifying Rot"]),
    );
    expect((await service.search({ category: "creature", metadata: { field: "traits", op: "excludesAny", values: ["water"] } })).records.some((record) => record.traits.includes("water"))).toBe(false);
    expect((await service.search({ category: "creature", nameQuery: "Ghost Sailor", metadata: { field: "hasDescription", op: "eq", value: true } })).records.every((record) => record.hasDescription)).toBe(true);
    expect((await service.search({ category: "creature", nameQuery: "Ghost Sailor", metadata: { field: "sourceCategory", op: "notIn", values: ["adventure"] } })).records[0]?.sourceCategory).toBe("core");
    expect((await service.search({ category: "creature", metadata: { field: "sourceCategory", op: "eq", value: "core" } })).records.every((record) => record.sourceCategory === "core")).toBe(true);
    expect((await service.search({ query: "ghost ship", category: "creature" })).mode).toBe("hybrid");
    expect((await service.search({ query: "ghost ship", category: "creature" })).searchProfile).toBe("balanced");
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["anti_poison"] } }).records.map((record) => record.name)).toEqual(["Antidote (Lesser)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["mental_recovery"] } }).records.map((record) => record.name)).toEqual(["Bottled Catharsis (Serenity)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["energy_resistance"] } }).records.map((record) => record.name)).toEqual(["Potion of Cold Resistance (Moderate)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["escape_support"] } }).records.map((record) => record.name)).toEqual(["Escape Fulu"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["senses_support"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Bloodhound Mask (Greater)"]));
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["disguise"] } }).records.map((record) => record.name)).toEqual(["Potion of Disguise (Moderate)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["navigation"] } }).records.map((record) => record.name)).toEqual(["Traveler's Fulu"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["tracking"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Bloodhound Mask (Greater)", "Tracker's Stew"]));
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["anti_tracking"] } }).records.map((record) => record.name)).toEqual(["Aroma Concealer"]);
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["lock_bypass"] } }).records.map((record) => record.name)).toEqual(["Concealable Thieves' Tools"]);
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["mobility"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Boots of Free Running (Greater)", "Climbing Kit"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["tracking"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Tracker's Goggles", "Tracking Tag"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["anti_tracking"] } }).records.map((record) => record.name)).toEqual(["Trackless"]);
    expect(service.listRecords({ category: "equipment", subcategory: "backpack", metadata: { field: "derivedTags", op: "includesAny", values: ["carry_support"] } }).records.map((record) => record.name)).toEqual(["Spacious Pouch (Type I)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAll", values: ["beneficial", "anti_disease"] } }).records.map((record) => record.name)).toEqual(["Antiplague (Lesser)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "excludesAny", values: ["offensive"] } }).records.map((record) => record.name)).not.toContain("Sightless Tincture");
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["social_infiltration"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Masquerade Scarf", "Quick-Change Outfit"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_escape"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Shacklebreaker", "Swallow-Spike"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_escape"] } }).records.map((record) => record.name)).not.toEqual(expect.arrayContaining(["Catch Pole", "Handcuffs (Average)", "Lawbringer's Lasso", "Injigo's Loving Embrace"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_capture"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Catch Pole", "Handcuffs (Average)", "Lawbringer's Lasso", "Injigo's Loving Embrace", "False Manacles", "Manacles of Persuasion"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_capture"] } }).records.map((record) => record.name)).not.toContain("Shacklebreaker");
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["disguise"] } }).records.map((record) => record.name)).toEqual(["Illusory Disguise"]);
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["social_infiltration"] } }).records.map((record) => record.name)).toEqual(["Illusory Disguise"]);
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["alarm"] } }).records.map((record) => record.name)).toEqual(["Alarm Ward"]);
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_capture"] } }).records.map((record) => record.name)).toEqual(["Snaring Glyph"]);
    expect(service.listRecords({ category: "affliction", metadata: { field: "derivedTags", op: "includesAny", values: ["mental_impairment"] } }).records.map((record) => record.name)).toEqual(["Cackling Delirium"]);
    expect(service.listRecords({ category: "affliction", metadata: { field: "derivedTags", op: "includesAny", values: ["mobility_impairment"] } }).records.map((record) => record.name)).toEqual(["Calcifying Rot"]);
    expect(service.listRecords({
      category: "equipment",
      metadata: {
        and: [
          { field: "weaponGroup", op: "eq", value: "bomb" },
          { field: "hands", op: "eq", value: 1 },
        ],
      },
    }).records.map((record) => record.name)).toEqual(["Ghost Charge Prototype"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["aquatic_context"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Ghost Sailor", "Pelagic Stalker", "Ship Captain"]));
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["scene_adjacent"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Ship Captain", "Wealthy Vigilante"]));
    expect(service.listRecords({ category: "creature", metadata: { field: "families", op: "includesAny", values: ["ghost"] } }).records.map((record) => record.name)).toEqual(["Ghost Commoner"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "families", op: "includesAny", values: ["lich"] } }).records.map((record) => record.name)).toEqual(["Mythic Lich"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "families", op: "includesAny", values: ["seafarer"] } }).records.map((record) => record.name)).toEqual(["Bosun"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "families", op: "includesAll", values: ["mythic", "lich"] } }).records.map((record) => record.name)).toEqual(["Mythic Lich"]);
    expect(service.listRecords({ category: "creature", levelMin: 5, levelMax: 5, metadata: { field: "families", op: "excludesAny", values: ["vampire"] } }).records.map((record) => record.name)).not.toContain("Morlock Thrall");

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
    const illusoryDisguise = service.lookup("Illusory Disguise", { category: "spell" }).match;
    expect(illusoryDisguise?.derivedTags).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));
    const antidote = service.lookup("Antidote (Lesser)", { category: "equipment" }).match;
    expect(antidote?.derivedTags).toEqual(expect.arrayContaining(["beneficial", "anti_poison"]));
    expect(antidote?.derivedTags).not.toEqual(expect.arrayContaining(["offensive", "thrown_offense"]));
    const bottledCatharsis = service.lookup("Bottled Catharsis (Serenity)", { category: "equipment" }).match;
    expect(bottledCatharsis?.derivedTags).toEqual(expect.arrayContaining(["beneficial", "condition_support", "mental_recovery"]));
    const coldResistancePotion = service.lookup("Potion of Cold Resistance (Moderate)", { category: "equipment" }).match;
    expect(coldResistancePotion?.derivedTags).toEqual(expect.arrayContaining(["beneficial", "energy_resistance", "buff_support", "self_buff"]));
    const bloodhoundMask = service.lookup("Bloodhound Mask (Greater)", { category: "equipment" }).match;
    expect(bloodhoundMask?.derivedTags).toEqual(expect.arrayContaining(["beneficial", "senses_support", "self_buff", "tracking"]));
    expect(bloodhoundMask?.derivedTags).not.toContain("anti_tracking");
    const aromaConcealer = service.lookup("Aroma Concealer", { category: "equipment" }).match;
    expect(aromaConcealer?.derivedTags).toContain("anti_tracking");
    expect(aromaConcealer?.derivedTags).not.toContain("tracking");
    const ichthyosisMutagen = service.lookup("Ichthyosis Mutagen", { category: "equipment" }).match;
    expect(ichthyosisMutagen?.derivedTags).not.toContain("tracking");
    expect(ichthyosisMutagen?.derivedTags).not.toContain("anti_tracking");
    const escapeFulu = service.lookup("Escape Fulu", { category: "equipment" }).match;
    expect(escapeFulu?.derivedTags).toEqual(expect.arrayContaining(["beneficial", "escape_support", "buff_support", "self_buff"]));
    const travelersFulu = service.lookup("Traveler's Fulu", { category: "equipment" }).match;
    expect(travelersFulu?.derivedTags).toContain("navigation");
    const trackersStew = service.lookup("Tracker's Stew", { category: "equipment" }).match;
    expect(trackersStew?.derivedTags).toContain("tracking");
    const disguisePotion = service.lookup("Potion of Disguise (Moderate)", { category: "equipment" }).match;
    expect(disguisePotion?.derivedTags).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));
    const shipCaptain = service.lookup("Ship Captain", { category: "creature" }).match;
    expect(shipCaptain?.derivedTags).toEqual(expect.arrayContaining(["nautical", "profession_npc", "scene_adjacent"]));
    const wealthyVigilante = service.lookup("Wealthy Vigilante", { category: "creature" }).match;
    expect(wealthyVigilante?.derivedTags).toEqual(expect.arrayContaining(["profession_npc", "scene_adjacent"]));
    const ghostCommoner = service.lookup("Ghost Commoner", { category: "creature" }).match;
    expect(ghostCommoner?.families).toEqual(["ghost"]);
    const mythicLich = service.lookup("Mythic Lich", { category: "creature" }).match;
    expect(mythicLich?.families).toEqual(["lich", "mythic"]);
    const morlockThrall = service.lookup("Morlock Thrall", { category: "creature" }).match;
    expect(morlockThrall?.families).toEqual(["vampire"]);
    expect(morlockThrall?.derivedTags).toContain("undead_threat");
    const bosun = service.lookup("Bosun", { category: "creature" }).match;
    expect(bosun?.families).toEqual(["seafarer"]);
    const pelagicStalker = service.lookup("Pelagic Stalker", { category: "creature" }).match;
    expect(pelagicStalker?.derivedTags).toContain("aquatic_context");
    const spaciousPouch = service.lookup("Spacious Pouch (Type I)", { category: "equipment" }).match;
    expect(spaciousPouch?.derivedTags).toContain("carry_support");
    const bootsOfFreeRunning = service.lookup("Boots of Free Running (Greater)", { category: "equipment" }).match;
    expect(bootsOfFreeRunning?.derivedTags).toContain("mobility");
    expect(bootsOfFreeRunning?.derivedTags).not.toContain("climbing");
    const trackersGoggles = service.lookup("Tracker's Goggles", { category: "equipment" }).match;
    expect(trackersGoggles?.derivedTags).toEqual(expect.arrayContaining(["navigation", "survival", "tracking"]));
    const trackingTag = service.lookup("Tracking Tag", { category: "equipment" }).match;
    expect(trackingTag?.derivedTags).toContain("tracking");
    const trackless = service.lookup("Trackless", { category: "equipment" }).match;
    expect(trackless?.derivedTags).toContain("anti_tracking");
    expect(trackless?.derivedTags).not.toContain("tracking");
    const masqueradeScarf = service.lookup("Masquerade Scarf", { category: "equipment" }).match;
    expect(masqueradeScarf?.derivedTags).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));
    const quickChangeOutfit = service.lookup("Quick-Change Outfit", { category: "equipment" }).match;
    expect(quickChangeOutfit?.derivedTags).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));
    const shacklebreaker = service.lookup("Shacklebreaker", { category: "equipment" }).match;
    expect(shacklebreaker?.derivedTags).toContain("restraint_escape");
    expect(shacklebreaker?.derivedTags).not.toContain("restraint_capture");
    const swallowSpike = service.lookup("Swallow-Spike", { category: "equipment" }).match;
    expect(swallowSpike?.derivedTags).toContain("restraint_escape");
    const lawbringersLasso = service.lookup("Lawbringer's Lasso", { category: "equipment" }).match;
    expect(lawbringersLasso?.derivedTags).toContain("restraint_capture");
    expect(lawbringersLasso?.derivedTags).not.toContain("restraint_escape");
    const injigosLovingEmbrace = service.lookup("Injigo's Loving Embrace", { category: "equipment" }).match;
    expect(injigosLovingEmbrace?.derivedTags).toContain("restraint_capture");
    expect(injigosLovingEmbrace?.derivedTags).not.toContain("restraint_escape");
    const falseManacles = service.lookup("False Manacles", { category: "equipment" }).match;
    expect(falseManacles?.derivedTags).toContain("restraint_capture");
    expect(falseManacles?.derivedTags).not.toContain("restraint_escape");
    const manaclesOfPersuasion = service.lookup("Manacles of Persuasion", { category: "equipment" }).match;
    expect(manaclesOfPersuasion?.derivedTags).toContain("restraint_capture");
    const handcuffs = service.lookup("Handcuffs (Average)", { category: "equipment" }).match;
    expect(handcuffs?.derivedTags).toContain("restraint_capture");
    expect(handcuffs?.derivedTags).not.toContain("restraint_escape");
    const catchPole = service.lookup("Catch Pole", { category: "equipment" }).match;
    expect(catchPole?.derivedTags).toContain("restraint_capture");
    expect(catchPole?.derivedTags).not.toContain("restraint_escape");
    const alarmWard = service.lookup("Alarm Ward", { category: "hazard" }).match;
    expect(alarmWard?.derivedTags).toContain("alarm");
    const snaringGlyph = service.lookup("Snaring Glyph", { category: "hazard" }).match;
    expect(snaringGlyph?.derivedTags).toContain("restraint_capture");
    const cacklingDelirium = service.lookup("Cackling Delirium", { category: "affliction" }).match;
    expect(cacklingDelirium?.subcategory).toBe("curse");
    expect(cacklingDelirium?.derivedTags).toContain("mental_impairment");
    const calcifyingRot = service.lookup("Calcifying Rot", { category: "affliction" }).match;
    expect(calcifyingRot?.subcategory).toBe("disease");
    expect(calcifyingRot?.derivedTags).toContain("mobility_impairment");
    const ghostChargePrototype = service.lookup("Ghost Charge Prototype", { category: "equipment" }).match;
    expect(ghostChargePrototype?.weaponGroup).toBe("bomb");
    expect(ghostChargePrototype?.hands).toBe(1);
    expect(ghostChargePrototype?.damageTypes).toEqual(["positive"]);
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
    expect(row?.searchText).toContain("coordinates the crew around starboard rigging routines");
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
        category: "hazard",
        family: "function",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "alarm", description: expect.any(String) }),
          expect.objectContaining({ value: "restraint_capture", description: expect.any(String) }),
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
    }).values.map((entry) => entry.value)).toEqual(expect.arrayContaining(["beneficial", "offensive", "climbing", "lock_bypass", "mental_recovery", "carry_support", "tracking", "anti_tracking", "restraint_escape", "restraint_capture"]));
    expect(service.listFilterValues({
      field: "derivedTags",
      category: "spell",
    }).values.map((entry) => entry.value)).toEqual(["disguise", "social_infiltration"]);
    expect(service.listFilterValues({
      field: "derivedTags",
      category: "hazard",
    }).values.map((entry) => entry.value)).toEqual(["alarm", "restraint_capture"]);
    expect(service.listFilterValues({
      field: "derivedTags",
      category: "affliction",
    }).values.map((entry) => entry.value)).toEqual(["mental_impairment", "mobility_impairment"]);

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
    }).values.map((entry) => entry.value)).toEqual(["med", "sm", "lg"]);

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
        { value: "rule", count: 6 },
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
    expect(firstService.getStats()).toEqual({ packCount: 16, recordCount: 97 });
    firstService.close();

    const firstMtime = (await import("node:fs/promises")).stat(indexPath).then((details) => details.mtimeMs);
    const unchangedService = await openPreparedTestService(fixture, { indexPath });
    expect(unchangedService.getStats()).toEqual({ packCount: 16, recordCount: 97 });
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
    expect(rebuiltService.getStats()).toEqual({ packCount: 16, recordCount: 98 });
    expect(rebuiltService.lookup("Sea Ghoul", { category: "creature" }).match?.name).toBe("Sea Ghoul");
    rebuiltService.close();
  });

  it("treats untracked JSON files in git checkouts as stale source changes", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    await initializeGitFixture(fixture.root);
    const indexPath = path.join(fixture.root, ".cache", "pf2e-index.sqlite");

    const firstService = await loadTestService(fixture, { indexPath });
    expect(firstService.getStats()).toEqual({ packCount: 16, recordCount: 97 });
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
