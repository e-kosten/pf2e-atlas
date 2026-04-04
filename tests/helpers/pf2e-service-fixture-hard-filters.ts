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
