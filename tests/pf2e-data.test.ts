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
  const packNames = ["actions", "classfeatures", "conditionitems", "feats-srd", "pathfinder-monster-core", "pfs-season-1-bestiary", "quest-for-the-frozen-flame-bestiary", "spells"];

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
        name: "feats-srd",
        label: "Feats",
        path: "packs/feats-srd",
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
    _id: "Refocus",
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
        value: ["concentrate"],
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

    const service = await Pf2eDataService.load(fixture.root, fixture.manifestPath);

    expect(service.listPacks()).toHaveLength(7);
    expect(service.getStats()).toEqual({ packCount: 7, recordCount: 23 });
    expect(service.getPack("Actions")?.name).toBe("actions");
  });

  it("supports lookup, listing, filtering, and derived metadata", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await Pf2eDataService.load(fixture.root, fixture.manifestPath);

    expect(service.lookup("Raise Shield").match?.name).toBe("Raise a Shield");
    expect(service.listRecords({ pack: "actions" }).records).toHaveLength(3);
    expect(service.search({ documentType: "Actor", traitsAll: ["fiend"] }).records[0]?.name).toBe("Cythnigot");
    expect(service.search({ documentType: "Actor", size: "sm" }).records.every((record) => record.size === "sm")).toBe(true);
    expect(service.search({ mode: "lexical", themeQuery: "aberration", documentType: "Actor" }).records[0]?.name).toBe("Cythnigot");
    expect(service.search({ recordType: "spell", tradition: "primal", actionCost: 2 }).records[0]?.name).toBe("Sea Blessing");
    expect(service.search({ nameQuery: "Ghost Sailor", documentType: "Actor", excludeMissingDescription: true }).records.every((record) => record.hasDescription)).toBe(true);
    expect(service.search({ nameQuery: "Ghost Sailor", documentType: "Actor", excludeAdventureContent: true }).records[0]?.sourceCategory).toBe("core");
    expect(service.search({ documentType: "Actor", coreOnly: true }).records.every((record) => record.sourceCategory === "core")).toBe(true);
    expect(service.search({ themeQuery: "ghost ship", documentType: "Actor" }).mode).toBe("hybrid");
    expect(() => service.search({ mode: "structured", themeQuery: "ghost ship" })).toThrow(
      /omit mode to default to hybrid, or set mode to lexical or hybrid/i,
    );

    const cythnigot = service.lookup("Cythnigot", { documentType: "Actor" }).match;
    expect(cythnigot?.hasDescription).toBe(true);
    expect(cythnigot?.descriptionSnippet).toBe("Small aberration.");
    expect(cythnigot?.sourceCategory).toBe("core");
  });

  it("uses the recommendation-oriented ranking profile without suppressing described adventure content", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await Pf2eDataService.load(fixture.root, fixture.manifestPath);

    const crawlingHands = service.search({
      documentType: "Actor",
      nameQuery: "Crawling Hand Swarm",
      rankingProfile: "preferReusableReferenceContent",
    }).records;
    expect(crawlingHands[0]?.sourceCategory).toBe("adventure");
    expect(crawlingHands[0]?.hasDescription).toBe(true);

    const bilgeSkeletons = service.search({
      documentType: "Actor",
      nameQuery: "Bilge Skeleton",
      rankingProfile: "preferReusableReferenceContent",
    }).records;
    expect(bilgeSkeletons[0]?.sourceCategory).toBe("core");
  });

  it("surfaces metadata-only haunted-ship swarm candidates in broad themed search", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await Pf2eDataService.load(fixture.root, fixture.manifestPath);

    const broadQuery =
      "ghost ship cursed voyage fear fog darkness possession maddening whispers vermin in the hold wrong-feeling stowaways body horror haunted physically unclean";
    const broadResults = service.search({
      recordType: "npc",
      levelMin: 1,
      levelMax: 5,
      rarity: "common",
      themeQuery: broadQuery,
      limit: 20,
      explain: true,
    });
    const broadNames = broadResults.records.map((record) => record.name);
    const crawlingIndex = broadNames.indexOf("Crawling Hand Swarm");
    const diverIndex = broadNames.indexOf("Diver");
    const lionIndex = broadNames.indexOf("Lion");

    expect(broadResults.mode).toBe("hybrid");
    expect(crawlingIndex).toBeGreaterThanOrEqual(0);
    expect(diverIndex).toBeGreaterThan(crawlingIndex);
    expect(lionIndex).toBeGreaterThan(crawlingIndex);

    const crawlingExplain = broadResults.explain?.records.find((record) => record.name === "Crawling Hand Swarm");
    expect(broadResults.explain?.query?.matchedRules.map((rule) => rule.id)).toEqual(
      expect.arrayContaining(["spectral-undead", "maritime-depths", "body-horror"]),
    );
    expect(crawlingExplain?.matchedTraits).toEqual(expect.arrayContaining(["swarm", "undead"]));
    expect(crawlingExplain?.matchedNameTokens).toContain("crawling");
    expect(Array.isArray(crawlingExplain?.matchedMetadataTokens)).toBe(true);
    expect(crawlingExplain?.matchedRuleIds).toEqual(
      expect.arrayContaining(["spectral-undead", "maritime-depths", "body-horror"]),
    );
    expect(crawlingExplain?.components.metadataOnlyBoost ?? 0).toBe(0);
    expect(crawlingExplain?.components.sourcePenalty ?? 0).toBe(0);

    const withoutExpansion = service.search({
      recordType: "npc",
      levelMin: 1,
      levelMax: 5,
      rarity: "common",
      themeQuery: broadQuery,
      limit: 20,
      explain: true,
      expandQuery: false,
    });
    expect(withoutExpansion.explain?.query?.matchedRules).toEqual([]);
    expect(withoutExpansion.explain?.query?.skippedRules.map((rule) => rule.reason)).toContain("expansion_disabled");

    const lexicalResults = service.search({
      recordType: "npc",
      levelMin: 1,
      levelMax: 5,
      mode: "lexical",
      themeQuery: "undead swarm body horror haunted ship crawling infestation severed limbs cursed voyage",
      limit: 20,
    });
    const lexicalNames = lexicalResults.records.map((record) => record.name);
    const lexicalCrawlingIndex = lexicalNames.indexOf("Crawling Hand Swarm");
    expect(lexicalCrawlingIndex).toBeGreaterThanOrEqual(0);
    expect(lexicalNames.indexOf("Diver")).toSatisfy((index) => index === -1 || index > lexicalCrawlingIndex);
    expect(lexicalNames.indexOf("Lion")).toSatisfy((index) => index === -1 || index > lexicalCrawlingIndex);
  });

  it("applies small source-quality preferences and stronger thematic unique penalties", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await Pf2eDataService.load(fixture.root, fixture.manifestPath);

    const bilgeResults = service.search({
      documentType: "Actor",
      nameQuery: "Bilge Skeleton",
      explain: true,
    });
    expect(bilgeResults.records[0]?.sourceCategory).toBe("core");

    const coreBilgeExplain = bilgeResults.explain?.records.find((record) => record.name === "Bilge Skeleton" && record.components.sourceQuality > 0);
    const adventureBilgeExplain = bilgeResults.explain?.records.find((record) => record.name === "Bilge Skeleton" && record.components.sourceQuality < 0);
    expect(coreBilgeExplain?.components.sourceQuality).toBe(0.04);
    expect(adventureBilgeExplain?.components.sourceQuality).toBe(-0.01);

    const sentinelResults = service.search({
      recordType: "npc",
      themeQuery: "sentinel guardian ancient ruins watch intruders",
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
    expect(uniqueExplain?.components.rarityPreference).toBe(-0.2);
    expect(rareExplain?.components.rarityPreference).toBe(0.01);

    const exactUniqueResults = service.search({
      documentType: "Actor",
      nameQuery: "Last Sentinel",
      explain: true,
    });
    expect(exactUniqueResults.records[0]?.name).toBe("Last Sentinel");
    const exactUniqueExplain = exactUniqueResults.explain?.records.find((record) => record.name === "Last Sentinel");
    expect(exactUniqueExplain?.components.rarityPreference).toBe(-0.03);
  });

  it("excludes dedicated Pathfinder Society content while retaining base equivalents", async () => {
    const fixture = await createHardFilterFixture();
    createdRoots.push(fixture.root);

    const service = await Pf2eDataService.load(fixture.root, fixture.manifestPath);

    expect(service.listPacks().map((pack) => pack.name)).not.toContain("macros");
    expect(service.listPacks().map((pack) => pack.name)).not.toContain("action-macros");
    expect(service.lookup("Grimstalker", { documentType: "Actor" }).match?.name).toBe("Grimstalker");
    expect(service.lookup("Ghoul", { documentType: "Actor" }).match?.name).toBe("Ghoul");
    expect(service.lookup("Zebub", { documentType: "Actor" }).match?.name).toBe("Zebub");
    expect(service.lookup("Raise Shield", { documentType: "Item" }).match?.name).toBe("Raise a Shield");

    expect(service.search({ nameQuery: "Grimstalker (PFS 3-13)", documentType: "Actor" }).records.map((record) => record.name)).not.toContain("Grimstalker (PFS 3-13)");
    expect(service.search({ nameQuery: "Ghoul (PFS Intro 2)", documentType: "Actor" }).records.map((record) => record.name)).not.toContain("Ghoul (PFS Intro 2)");
    expect(service.search({ nameQuery: "Zebub (PFS)", documentType: "Actor" }).records.map((record) => record.name)).not.toContain("Zebub (PFS)");
    expect(service.search({ nameQuery: "Magical Mentor" }).records.map((record) => record.name)).not.toContain("Magical Mentor");
    expect(service.search({ nameQuery: "Effect: Magical Mentor" }).records.map((record) => record.name)).not.toContain("Effect: Magical Mentor");
    expect(service.search({ nameQuery: "Treat Wounds" }).records.map((record) => record.name)).not.toContain("Treat Wounds");
    expect(service.search({ nameQuery: "Trip: Athletics" }).records.map((record) => record.name)).not.toContain("Trip: Athletics");

    const featResults = service.search({
      recordType: "feat",
      themeQuery: "mentor training support teamwork guidance",
      limit: 10,
    }).records.map((record) => record.name);
    expect(featResults).toContain("Proud Mentor");
    expect(featResults).not.toContain("Magical Mentor");
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

  it("supports batch lookup, batch record fetch, and curated backlink retrieval", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await Pf2eDataService.load(fixture.root, fixture.manifestPath);

    const lookups = service.lookupMany([{ name: "Refocus" }, { name: "Deep Focus" }], { coreOnly: true });
    expect(lookups.map((result) => result.match?.name)).toEqual(["Refocus", "Deep Focus"]);
    expect(lookups.map((result) => result.matchType)).toEqual(["exact", "exact"]);

    const records = service.getRecordsByKeys(["actions:Refocus", "feats-srd:feat1"]);
    expect(records.map((record) => record.name)).toEqual(["Refocus", "Deep Focus"]);

    const outgoing = service.getLinkedRules(["conditionitems:Blinded"], { maxPerPrimary: 5 });
    expect(outgoing.records.map((record) => record.name)).toEqual(["Dazzled", "Seek"]);

    const backlinks = service.getBacklinks(["actions:Refocus"], { maxPerPrimary: 10 });
    expect(backlinks.records.map((record) => record.name)).toEqual(["Deep Focus", "Meditative Well"]);
    expect(backlinks.edges.every((edge) => edge.direction === "backlink")).toBe(true);
    expect(backlinks.records.some((record) => record.type === "spell")).toBe(false);
  });

  it("exposes indexed search vocabulary for agent planning", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await Pf2eDataService.load(fixture.root, fixture.manifestPath);

    const vocabulary = service.getSearchVocabulary({ traitLimitPerRecordType: 4 });
    expect(vocabulary.documentTypes.map((entry) => entry.value)).toEqual(expect.arrayContaining(["Actor", "Item"]));
    expect(vocabulary.recordTypes.map((entry) => entry.value)).toEqual(expect.arrayContaining(["npc", "spell", "action"]));
    expect(vocabulary.itemCategories.map((entry) => entry.value)).toEqual(expect.arrayContaining(["action", "feat", "spell"]));
    expect(vocabulary.traditions.map((entry) => entry.value)).toContain("primal");
    expect(vocabulary.commonTraitsByRecordType.find((entry) => entry.recordType === "npc")?.traits.length).toBeGreaterThan(0);
  });

  it("collects rule question context without synthesis", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await Pf2eDataService.load(fixture.root, fixture.manifestPath);

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

  it("reuses an unchanged SQLite index and rebuilds when the source changes", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    const indexPath = path.join(fixture.root, ".cache", "pf2e-index.sqlite");

    const firstService = await Pf2eDataService.load(fixture.root, fixture.manifestPath, { indexPath });
    expect(firstService.getStats()).toEqual({ packCount: 7, recordCount: 23 });
    firstService.close();

    const firstMtime = (await import("node:fs/promises")).stat(indexPath).then((details) => details.mtimeMs);
    const unchangedService = await Pf2eDataService.load(fixture.root, fixture.manifestPath, { indexPath });
    expect(unchangedService.getStats()).toEqual({ packCount: 7, recordCount: 23 });
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
    expect(rebuiltService.getStats()).toEqual({ packCount: 7, recordCount: 24 });
    expect(rebuiltService.lookup("Sea Ghoul", { documentType: "Actor" }).match?.name).toBe("Sea Ghoul");
    rebuiltService.close();
  });
});
