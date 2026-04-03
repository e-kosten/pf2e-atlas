import { mkdtemp, mkdir, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { writeJson } from "./pf2e-fixture.js";

export type ServiceTestFixture = {
  root: string;
  manifestPath: string;
};

export async function createFixture(): Promise<{ root: string; manifestPath: string }> {
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

export async function createHardFilterFixture(): Promise<{ root: string; manifestPath: string }> {
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

export async function cleanupCreatedRoots(createdRoots: string[]): Promise<void> {
  await Promise.all(
    createdRoots.splice(0).map(async (root) => {
      await import("node:fs/promises").then(({ rm }) => rm(root, { recursive: true, force: true }));
    }),
  );
}
