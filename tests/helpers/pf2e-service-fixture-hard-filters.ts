import { mkdtemp, mkdir } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { writeJson } from "./pf2e-fixture.js";
import type { ServiceTestFixture } from "./pf2e-service-fixture-runtime.js";

export async function createHardFilterFixture(): Promise<{ root: string; manifestPath: string }> {
  const root = await mkdtemp(path.join(os.tmpdir(), "pf2e-mcp-filter-test-"));
  const packRoot = path.join(root, "packs", "pf2e");
  const packNames = [
    "action-macros",
    "actions",
    "campaign-effects",
    "equipment",
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
      { name: "equipment", label: "Equipment", path: "packs/equipment", type: "Item" },
      { name: "feats-srd", label: "Feats", path: "packs/feats-srd", type: "Item" },
      { name: "macros", label: "Macros", path: "packs/macros", type: "Macro" },
      { name: "pathfinder-bestiary", label: "Pathfinder Bestiary", path: "packs/pathfinder-bestiary", type: "Actor" },
      {
        name: "pathfinder-bestiary-2",
        label: "Pathfinder Bestiary 2",
        path: "packs/pathfinder-bestiary-2",
        type: "Actor",
      },
      {
        name: "pathfinder-monster-core-2",
        label: "Pathfinder Monster Core 2",
        path: "packs/pathfinder-monster-core-2",
        type: "Actor",
      },
      {
        name: "pathfinder-society-boons",
        label: "Pathfinder Society Boons",
        path: "packs/pathfinder-society-boons",
        type: "Item",
      },
      {
        name: "pfs-introductions-bestiary",
        label: "Pathfinder Society Introductions",
        path: "packs/pfs-introductions-bestiary",
        type: "Actor",
      },
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core-2", "tactical-mastermind.json"), {
    _id: "tactical-mastermind",
    name: "Tactical Mastermind",
    type: "npc",
    system: {
      abilities: {
        str: { mod: 0 },
        dex: { mod: 3 },
        con: { mod: 2 },
        int: { mod: 5 },
        wis: { mod: 4 },
        cha: { mod: 2 },
      },
      details: {
        level: {
          value: 7,
        },
        publication: {
          title: "Pathfinder Monster Core 2",
        },
        publicNotes: "<p>A calculating battlefield coordinator.</p>",
      },
      perception: {
        mod: 15,
        senses: [{ type: "darkvision" }, { type: "scent", range: 30, acuity: "imprecise" }],
      },
      saves: {
        fortitude: { value: 12 },
        reflex: { value: 14 },
        will: { value: 17 },
      },
      skills: {
        arcana: { mod: 18, rank: 4 },
        society: { mod: 16, rank: 4 },
        stealth: { mod: 11, rank: 2 },
      },
      attributes: {
        speed: {
          value: 25,
          otherSpeeds: [{ type: "fly", value: 40 }],
        },
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core-2", "silver-tongue-duelist.json"), {
    _id: "silver-tongue-duelist",
    name: "Silver Tongue Duelist",
    type: "npc",
    system: {
      abilities: {
        str: { mod: 2 },
        dex: { mod: 4 },
        con: { mod: 2 },
        int: { mod: 2 },
        wis: { mod: 1 },
        cha: { mod: 5 },
      },
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Monster Core 2",
        },
        publicNotes: "<p>A charismatic duelist with a sharp tongue.</p>",
      },
      perception: {
        mod: 12,
      },
      saves: {
        fortitude: { value: 11 },
        reflex: { value: 16 },
        will: { value: 13 },
      },
      skills: {
        diplomacy: { mod: 17, rank: 4 },
        deception: { mod: 15, rank: 3 },
        arcana: { mod: 7, rank: 0 },
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core-2", "stubborn-brute.json"), {
    _id: "stubborn-brute",
    name: "Stubborn Brute",
    type: "npc",
    system: {
      abilities: {
        str: { mod: 5 },
        dex: { mod: 1 },
        con: { mod: 4 },
        int: { mod: 0 },
        wis: { mod: 1 },
        cha: { mod: 2 },
      },
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Monster Core 2",
        },
        publicNotes: "<p>A bruiser who relies on grit more than guile.</p>",
      },
      perception: {
        mod: 10,
      },
      saves: {
        fortitude: { value: 18 },
        reflex: { value: 9 },
        will: { value: 10 },
      },
      skills: {
        athletics: { mod: 16, rank: 4 },
        arcana: { mod: 4, rank: 0 },
      },
      traits: {
        rarity: "common",
        value: ["humanoid"],
        size: {
          value: "lg",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core-2", "clockwork-killbox.json"), {
    _id: "clockwork-killbox",
    name: "Clockwork Killbox",
    type: "hazard",
    system: {
      attributes: {
        ac: { value: 20 },
        hardness: 8,
        hp: {
          value: 32,
          max: 32,
          brokenThreshold: 16,
        },
        stealth: {
          value: 10,
        },
      },
      details: {
        disable: "<p>@Check[crafting|dc:18] (trained) or Thievery (trained) to jam the gearing.</p>",
        isComplex: true,
        level: {
          value: 5,
        },
        publication: {
          title: "Pathfinder Monster Core 2",
        },
        publicNotes: "<p>A nested ring of hidden gears launches razors at intruders.</p>",
      },
      saves: {
        fortitude: { value: 14 },
        reflex: { value: 11 },
        will: { value: 9 },
      },
      traits: {
        rarity: "common",
        value: ["mechanical", "trap"],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core-2", "haunting-choir.json"), {
    _id: "haunting-choir",
    name: "Haunting Choir",
    type: "hazard",
    system: {
      attributes: {
        ac: { value: 18 },
        hardness: 0,
        hp: {
          value: 24,
          max: 24,
          brokenThreshold: 12,
        },
        stealth: {
          value: 11,
        },
      },
      details: {
        disable: "<p>@Check[religion|dc:22] (expert) to quiet the restless dead.</p>",
        isComplex: false,
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Monster Core 2",
        },
        publicNotes: "<p>Whispering spirits erupt into a dissonant chorus that rattles the mind.</p>",
      },
      saves: {
        fortitude: { value: 8 },
        reflex: { value: 10 },
        will: { value: 16 },
      },
      traits: {
        rarity: "common",
        value: ["haunt", "magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "repeating-hand-crossbow.json"), {
    _id: "weapon-repeating-hand-crossbow",
    name: "Repeating Hand Crossbow",
    type: "weapon",
    system: {
      damage: {
        dice: 1,
        die: "d6",
        damageType: "piercing",
      },
      description: {
        value: "<p>A compact crossbow built for rapid follow-up shots.</p>",
      },
      group: "bow",
      level: {
        value: 2,
      },
      price: {
        value: {
          gp: 3,
        },
      },
      publication: {
        title: "Pathfinder Guns & Gears",
      },
      range: 60,
      reload: {
        value: "1",
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

  await writeJson(path.join(packRoot, "equipment", "siege-laser.json"), {
    _id: "weapon-siege-laser",
    name: "Siege Laser",
    type: "weapon",
    system: {
      damage: {
        dice: 2,
        die: "d10",
        damageType: "fire",
      },
      description: {
        value: "<p>A heavy experimental launcher that requires careful reloading.</p>",
      },
      group: "firearm",
      level: {
        value: 7,
      },
      price: {
        value: {
          gp: 25,
        },
      },
      publication: {
        title: "Pathfinder Guns & Gears",
      },
      range: 120,
      reload: {
        value: "2",
      },
      traits: {
        rarity: "common",
        value: ["martial"],
      },
      usage: {
        value: "held-in-two-hands",
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "silken-vest.json"), {
    _id: "armor-silken-vest",
    name: "Silken Vest",
    type: "armor",
    system: {
      acBonus: 1,
      checkPenalty: 0,
      description: {
        value: "<p>A flexible vest reinforced with hidden scales.</p>",
      },
      group: "cloth",
      dexCap: 5,
      level: {
        value: 1,
      },
      price: {
        value: {
          gp: 2,
        },
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      speedPenalty: 0,
      strength: 0,
      traits: {
        rarity: "common",
        value: ["light"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "fortress-plate.json"), {
    _id: "armor-fortress-plate",
    name: "Fortress Plate",
    type: "armor",
    system: {
      acBonus: 4,
      checkPenalty: -2,
      description: {
        value: "<p>Massive plate designed to absorb the worst battlefield punishment.</p>",
      },
      group: "plate",
      dexCap: 1,
      level: {
        value: 8,
      },
      price: {
        value: {
          gp: 45,
        },
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      speedPenalty: -10,
      strength: 4,
      traits: {
        rarity: "common",
        value: ["heavy"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "tower-bulwark.json"), {
    _id: "shield-tower-bulwark",
    name: "Tower Bulwark",
    type: "shield",
    system: {
      acBonus: 2,
      description: {
        value: "<p>A reinforced tower shield meant to lock down a corridor.</p>",
      },
      hardness: 10,
      hp: {
        value: 40,
        max: 40,
        brokenThreshold: 20,
      },
      level: {
        value: 6,
      },
      price: {
        value: {
          gp: 18,
        },
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["shield"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "buckler-aegis.json"), {
    _id: "shield-buckler-aegis",
    name: "Buckler Aegis",
    type: "shield",
    system: {
      acBonus: 1,
      description: {
        value: "<p>A compact shield for duelists who value mobility over staying power.</p>",
      },
      hardness: 3,
      hp: {
        value: 12,
        max: 12,
        brokenThreshold: 6,
      },
      level: {
        value: 1,
      },
      price: {
        value: {
          gp: 1,
        },
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["shield"],
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
    command:
      "game.pf2e.actions.raiseAShield({ actors: [token?.actor ?? actor ?? game.user.character].filter((actor) => actor) })",
  });

  await writeJson(path.join(packRoot, "macros", "treat-wounds.json"), {
    _id: "macro-treat-wounds",
    name: "Treat Wounds",
    type: "script",
    command:
      "game.pf2e.actions.treatWounds({ event, actors: [token?.actor ?? actor ?? game.user.character].filter((actor) => actor) })",
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
