import path from "node:path";

import { writeJson } from "./pf2e-fixture.js";

export async function writeCreatureFixtureData(packRoot: string): Promise<void> {
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "animated-armor.json"), {
    _id: "animated-armor-1",
    name: "Animated Armor",
    type: "npc",
    system: {
      details: {
        level: {
          value: 2,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>An empty suit of armor animated by magic to guard a tomb hallway.</p>",
      },
      traits: {
        rarity: "common",
        value: ["construct", "mindless"],
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
          value: 7,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A broken servant enthralled to a vampire household.</p>",
      },
      traits: {
        rarity: "uncommon",
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
        rarity: "uncommon",
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "ghost-pirate-captain.json"), {
    _id: "ghost-pirate-captain",
    name: "Ghost Pirate Captain",
    type: "npc",
    system: {
      details: {
        level: {
          value: 8,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>An undead pirate captain prowls the ocean aboard a derelict ship.</p>",
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "amelekana.json"), {
    _id: "amelekana",
    name: "Amelekana",
    type: "npc",
    system: {
      details: {
        level: {
          value: 8,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Amelekanas are amphibious ambush predators native to rivers and lakes across Castrovel.</p>",
      },
      traits: {
        rarity: "common",
        value: ["beast", "water"],
        size: {
          value: "lg",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "electric-eel.json"), {
    _id: "electric-eel",
    name: "Electric Eel",
    type: "npc",
    system: {
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Usually found in freshwater rivers and lakes, an electric eel is not particularly aggressive.</p>",
      },
      traits: {
        rarity: "common",
        value: ["animal", "water"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "water-orm.json"), {
    _id: "water-orm",
    name: "Water Orm",
    type: "npc",
    system: {
      details: {
        level: {
          value: 9,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>These legendary creatures lurking in remote lakes inhabit cool inland waters, spy upon the shores of their lakes, and occasionally rise near the beach or a silty lake bed.</p>",
      },
      traits: {
        rarity: "common",
        value: ["beast", "water"],
        size: {
          value: "lg",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "hooktongue.json"), {
    _id: "hooktongue",
    name: "Hooktongue",
    type: "npc",
    system: {
      details: {
        level: {
          value: 9,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Legendary creatures lurking in remote lakes, water orms often spy upon the shores of their lakes and surface near the beach when curiosity overtakes caution.</p>",
      },
      traits: {
        rarity: "common",
        value: ["beast", "water"],
        size: {
          value: "lg",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "old-herok.json"), {
    _id: "old-herok",
    name: "Old Herok",
    type: "npc",
    system: {
      details: {
        level: {
          value: 11,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Old Herok is a water orm that hides in deep lakes, watching the shores of its inland domain and surfacing near lonely beaches only when it must.</p>",
      },
      traits: {
        rarity: "rare",
        value: ["beast", "water"],
        size: {
          value: "lg",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "gathganara.json"), {
    _id: "gathganara",
    name: "Gathganara",
    type: "npc",
    system: {
      details: {
        level: {
          value: 7,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Naiads protect streams, ponds, springs, and other natural bodies of fresh water where river tributaries meet beneath forest canopies.</p>",
      },
      traits: {
        rarity: "common",
        value: ["fey", "water"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "defaced-naiad-queen.json"), {
    _id: "defaced-naiad-queen",
    name: "Defaced Naiad Queen",
    type: "npc",
    system: {
      details: {
        level: {
          value: 15,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Naiad queens rule over pristine wildernesses centered on untouched lakes or other bodies of fresh water.</p>",
      },
      traits: {
        rarity: "common",
        value: ["fey", "water"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "coldmire-pond.json"), {
    _id: "coldmire-pond",
    name: "Coldmire Pond",
    type: "npc",
    system: {
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Coldmire ponds are sentient bodies of living water that crawl along the ground or drift through still inland pools.</p>",
      },
      traits: {
        rarity: "common",
        value: ["elemental", "water"],
        size: {
          value: "lg",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "boiling-spring.json"), {
    _id: "boiling-spring",
    name: "Boiling Spring",
    type: "npc",
    system: {
      details: {
        level: {
          value: 7,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A boiling spring is a humanoid water elemental made of scalding steam and bubbling water from a geothermal spring.</p>",
      },
      traits: {
        rarity: "common",
        value: ["elemental", "water"],
        size: {
          value: "lg",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "castruccio-irovetti.json"), {
    _id: "castruccio-irovetti",
    name: "Castruccio Irovetti",
    type: "npc",
    system: {
      details: {
        level: {
          value: 15,
        },
        publication: {
          title: "Pathfinder Adventure Path",
        },
        publicNotes: "<p>Since his flight from Numeria, Castruccio Irovetti has ruled Pitax for years and remains a major player in this River Kingdom's political scene.</p>",
      },
      traits: {
        rarity: "unique",
        value: ["human", "humanoid"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "apothecary-bee.json"), {
    _id: "apothecary-bee",
    name: "Apothecary Bee",
    type: "npc",
    system: {
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Long-abandoned gardens still grow along the Sphinx River in Osirion, where many apothecary bees prowl for flowers that meet their standards.</p>",
      },
      traits: {
        rarity: "common",
        value: ["animal"],
        size: {
          value: "sm",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "astradaemon.json"), {
    _id: "astradaemon",
    name: "Astradaemon",
    type: "npc",
    system: {
      details: {
        level: {
          value: 16,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Astradaemons hunt the pathways between life and death and stalk the banks of the River of Souls in the Astral Plane.</p>",
      },
      traits: {
        rarity: "common",
        value: ["daemon", "fiend", "unholy"],
        size: {
          value: "lg",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "sea-drake.json"), {
    _id: "sea-drake",
    name: "Sea Drake",
    type: "npc",
    system: {
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Long and slender, sea drakes have fins down the length of their backs and webbing between their talons. Although most sea drakes make their roosts high on ocean-facing cliffs, it isn't unheard of for them to dwell in underwater caves.</p>",
      },
      traits: {
        rarity: "common",
        value: ["amphibious", "dragon", "evil", "water"],
        size: {
          value: "lg",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "blodeuwedd.json"), {
    _id: "blodeuwedd",
    name: "Blodeuwedd",
    type: "npc",
    system: {
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>The mysterious blodeuwedds dwell in those parts of the world where the boundaries between the Material Plane and the First World have worn thin, or around portals between the two planes.</p>",
      },
      traits: {
        rarity: "common",
        value: ["chaotic", "fey", "plant"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "catrina.json"), {
    _id: "catrina",
    name: "Catrina",
    type: "npc",
    system: {
      details: {
        level: {
          value: 5,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Catrinas meet souls in the Boneyard, patiently explaining the finality of death and guiding spirits toward a calm passage into the afterlife.</p>",
      },
      traits: {
        rarity: "common",
        value: ["monitor", "psychopomp"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "shrine-caretaker.json"), {
    _id: "shrine-caretaker",
    name: "Shrine Caretaker",
    type: "npc",
    system: {
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>This combusted haunts a shrine and sometimes throws itself into lakes or rivers, believing the water will quiet the flames consuming it.</p>",
      },
      traits: {
        rarity: "common",
        value: ["spirit", "undead", "unholy"],
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

  await writeJson(path.join(packRoot, "pathfinder-npc-core", "court-jester.json"), {
    _id: "court-jester",
    name: "Court Jester",
    type: "npc",
    system: {
      details: {
        level: {
          value: 5,
        },
        publication: {
          title: "Pathfinder NPC Core",
        },
        publicNotes: "<p>Though court jesters are often the targets of easy mockery and idle amusements, this jester hides malice behind painted smiles and a razor wit.</p>",
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "mechanical-carny.json"), {
    _id: "mechanical-carny",
    name: "Mechanical Carny",
    type: "npc",
    system: {
      details: {
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Mechanical carnies are constructs manufactured to serve as entertainers, cleaners, and guards at carnivals and circuses.</p>",
      },
      traits: {
        rarity: "uncommon",
        value: ["construct"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "soulbound-doll.json"), {
    _id: "soulbound-doll",
    name: "Soulbound Doll",
    type: "npc",
    system: {
      details: {
        level: {
          value: 2,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Soulbound dolls are eerie mannequins or playthings that have been imbued with a small piece of a deceased mortal's soul.</p>",
      },
      traits: {
        rarity: "uncommon",
        value: ["construct"],
        size: {
          value: "sm",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "fire-scamp.json"), {
    _id: "fire-scamp",
    name: "Fire Scamp",
    type: "npc",
    system: {
      details: {
        level: {
          value: 1,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Although arguably quite friendly, fire scamps delight in fire and playing pranks on everyone they befriend.</p>",
      },
      traits: {
        rarity: "uncommon",
        value: ["elemental", "fire"],
        size: {
          value: "sm",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "masked-mourner.json"), {
    _id: "masked-mourner",
    name: "Masked Mourner",
    type: "npc",
    system: {
      details: {
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A solemn creature wearing a ceremonial mask and veiled face to hide its identity from the living.</p>",
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "faceless-butcher.json"), {
    _id: "faceless-butcher",
    name: "Faceless Butcher",
    type: "npc",
    system: {
      details: {
        level: {
          value: 7,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>This faceless horror has a blank, featureless face and keeps stolen faces as trophies.</p>",
      },
      traits: {
        rarity: "uncommon",
        value: ["aberration"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "false-herald.json"), {
    _id: "false-herald",
    name: "False Herald",
    type: "npc",
    system: {
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>The herald assumes a false identity, infiltrates courts, and impersonates priests to replace them.</p>",
      },
      traits: {
        rarity: "uncommon",
        value: ["humanoid"],
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "jungle-stalker.json"), {
    _id: "jungle-stalker",
    name: "Jungle Stalker",
    type: "npc",
    system: {
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A patient ambusher that stalks the deep jungles and tangled woods.</p>",
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "boggard-mire-scout.json"), {
    _id: "boggard-mire-scout",
    name: "Boggard Mire Scout",
    type: "npc",
    system: {
      details: {
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A croaking scout that watches for intruders from a reed blind.</p>",
      },
      traits: {
        rarity: "common",
        value: ["amphibious", "boggard", "humanoid"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "cairn-wight.json"), {
    _id: "cairn-wight",
    name: "Cairn Wight",
    type: "npc",
    system: {
      details: {
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A jealous undead guardian of barrows and sepulchers.</p>",
      },
      traits: {
        rarity: "common",
        value: ["undead", "unholy", "wight"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "temple-scavenger.json"), {
    _id: "temple-scavenger",
    name: "Temple Scavenger",
    type: "npc",
    system: {
      details: {
        level: {
          value: 3,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>This scavenger lurks among the ruins of a collapsed temple.</p>",
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "coastal-prowler.json"), {
    _id: "coastal-prowler",
    name: "Coastal Prowler",
    type: "npc",
    system: {
      details: {
        level: {
          value: 5,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A vigilant hunter prowls rocky shores and coastal reefs.</p>",
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "island-watcher.json"), {
    _id: "island-watcher",
    name: "Island Watcher",
    type: "npc",
    system: {
      details: {
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A wary survivor keeps watch over a lonely island and its hidden paths.</p>",
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "plains-runner.json"), {
    _id: "plains-runner",
    name: "Plains Runner",
    type: "npc",
    system: {
      details: {
        level: {
          value: 3,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A swift hunter races across grassy plains and open savannas.</p>",
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "canyon-stalker.json"), {
    _id: "canyon-stalker",
    name: "Canyon Stalker",
    type: "npc",
    system: {
      details: {
        level: {
          value: 5,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A patient hunter glides through canyons and narrow gorges carved into the badlands.</p>",
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "wasteland-reclaimer.json"), {
    _id: "wasteland-reclaimer",
    name: "Wasteland Reclaimer",
    type: "npc",
    system: {
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A scarred scavenger roams barren wastelands and blasted wastes in search of salvage.</p>",
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "temple-custodian.json"), {
    _id: "temple-custodian",
    name: "Temple Custodian",
    type: "npc",
    system: {
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A divine construct tends an ancient temple shrine and its sacred relics.</p>",
      },
      traits: {
        rarity: "common",
        value: ["construct"],
        size: {
          value: "med",
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "fortress-warden.json"), {
    _id: "fortress-warden",
    name: "Fortress Warden",
    type: "npc",
    system: {
      details: {
        level: {
          value: 7,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A grim defender patrols the walls of a mountain fortress and ancient citadel.</p>",
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "caldera-oni.json"), {
    _id: "caldera-oni",
    name: "Caldera Oni",
    type: "npc",
    system: {
      details: {
        level: {
          value: 10,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>As hot-blooded as the lava that floods their homes, caldera oni hunger for battle.</p>",
      },
      traits: {
        rarity: "common",
        value: ["fiend", "oni"],
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

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "mental-assault.json"), {
    _id: "mental-assault",
    name: "Mental Assault",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 20,
        },
        publication: {
          title: "Pathfinder Adventure Path",
        },
        publicNotes: "<p>Magic sigils hidden in the grain of the wooden door trigger a magic trap that damages the mind of anyone attempting to pick the door's lock.</p>",
      },
      traits: {
        rarity: "common",
        value: ["magical", "mechanical", "trap"],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "images-of-failure.json"), {
    _id: "images-of-failure",
    name: "Images of Failure",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 12,
        },
        publication: {
          title: "Pathfinder Adventure Path",
        },
        publicNotes: "<p>Psychically enhanced illusions flood the minds of creatures in the hallway with memories of their past failures.</p>",
      },
      traits: {
        rarity: "common",
        value: ["magical", "trap"],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "explosive-barrels.json"), {
    _id: "explosive-barrels",
    name: "Explosive Barrels",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 3,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Wooden barrels marked with an oil-drop symbol catch fire and explode.</p>",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "acid-mist.json"), {
    _id: "acid-mist",
    name: "Acid Mist",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A cloud of caustic acid sprays across the chamber and corrodes exposed gear.</p>",
      },
      traits: {
        rarity: "common",
        value: ["mechanical", "trap"],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "gas-trap.json"), {
    _id: "gas-trap",
    name: "Gas Trap",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A spring slams and locks the room's door before four hidden gas vents begin pumping poison gas into the chamber.</p>",
      },
      traits: {
        rarity: "common",
        value: ["mechanical", "trap"],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "mask-summoning-rune.json"), {
    _id: "mask-summoning-rune",
    name: "Mask Summoning Rune",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 5,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A glowing rune inscribed across the threshold summons masked guardians when a creature crosses the ward.</p>",
      },
      traits: {
        rarity: "common",
        value: ["magical", "trap"],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "thin-ice.json"), {
    _id: "thin-ice",
    name: "Thin Ice",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 1,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>The freezing floor gives way into a sheet of thin ice above frigid water.</p>",
      },
      traits: {
        rarity: "common",
        value: ["environmental"],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "electric-latch-rune.json"), {
    _id: "electric-latch-rune",
    name: "Electric Latch Rune",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Lightning crackles through the rune and a shock surges across the metal latch.</p>",
      },
      traits: {
        rarity: "common",
        value: ["magical", "trap"],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "buzzing-latch-rune.json"), {
    _id: "buzzing-latch-rune",
    name: "Buzzing Latch Rune",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 6,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A shrieking resonance and deafening sound burst from the rune, rattling the chamber.</p>",
      },
      traits: {
        rarity: "common",
        value: ["magical", "trap"],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "drowning-pit.json"), {
    _id: "drowning-pit",
    name: "Drowning Pit",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 2,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A trapdoor covers a 10-foot-square pit that's 30 feet deep and has 5 feet of water at the bottom.</p>",
      },
      traits: {
        rarity: "common",
        value: ["mechanical", "trap"],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "collapsing-bridge.json"), {
    _id: "collapsing-bridge",
    name: "Collapsing Bridge",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 5,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>Metal supports twist and shear off the bridge, causing stretches of the structure to collapse.</p>",
      },
      traits: {
        rarity: "common",
        value: ["mechanical", "trap"],
      },
    },
  });

  await writeJson(path.join(packRoot, "pathfinder-monster-core", "rushing-wind.json"), {
    _id: "rushing-wind",
    name: "Rushing Wind",
    type: "hazard",
    system: {
      details: {
        level: {
          value: 4,
        },
        publication: {
          title: "Pathfinder Monster Core",
        },
        publicNotes: "<p>A raging wind sucks creatures in the area toward the aiudara.</p>",
      },
      traits: {
        rarity: "common",
        value: ["environmental"],
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
}
