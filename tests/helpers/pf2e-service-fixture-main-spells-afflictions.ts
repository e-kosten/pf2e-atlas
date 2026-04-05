import path from "node:path";

import { writeJson } from "./pf2e-fixture.js";

export async function writeSpellAndAfflictionFixtureData(packRoot: string): Promise<void> {
  await writeJson(path.join(packRoot, "spells-srd", "alarm.json"), {
    _id: "spell-alarm-1",
    name: "Alarm",
    type: "spell",
    system: {
      description: {
        value: "<p>You ward an area to alert you when creatures enter without your permission.</p><p>Whenever a Small or larger corporeal creature enters the spell's area without speaking the password, alarm sends your choice of a mental alert or an audible alarm.</p>",
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
        value: ["concentrate", "manipulate"],
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

  await writeJson(path.join(packRoot, "spells", "web-of-eyes.json"), {
    _id: "spell-web-of-eyes-1",
    name: "Web of Eyes",
    type: "spell",
    system: {
      description: {
        value: "<p>You place an invisible scrying sensor on each target just above their eyes.</p><p>Each sensor looks where that target looks, and you can see what each target sees.</p>",
      },
      level: {
        value: 3,
      },
      publication: {
        title: "Pathfinder Dark Archive",
      },
      traits: {
        rarity: "common",
        traditions: ["occult"],
        value: ["concentrate", "manipulate", "scrying"],
      },
    },
  });

  await writeJson(path.join(packRoot, "spells", "painted-scout.json"), {
    _id: "spell-painted-scout-1",
    name: "Painted Scout",
    type: "spell",
    system: {
      description: {
        value: "<p>You press your hand to the stone, causing hand-drawn scouts to spread out from your fingers.</p><p>As long as you Sustain the Spell, you can see, hear, and smell through the scouts.</p>",
      },
      duration: {
        sustained: true,
        value: "10 minutes",
      },
      level: {
        value: 3,
      },
      publication: {
        title: "Pathfinder Secrets of Magic",
      },
      target: {
        value: "1 stone surface",
      },
      traits: {
        rarity: "common",
        traditions: ["arcane"],
        value: ["concentrate", "manipulate", "scrying"],
      },
    },
  });

  await writeJson(path.join(packRoot, "spells", "animal-form.json"), {
    _id: "spell-animal-form-1",
    name: "Animal Form",
    type: "spell",
    system: {
      description: {
        value: "<p>You transform into the battle form of a swift animal, taking on bestial attacks and movement.</p>",
      },
      level: {
        value: 2,
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        traditions: ["primal"],
        value: ["polymorph"],
      },
    },
  });

  await writeJson(path.join(packRoot, "spells-srd", "message-rune.json"), {
    _id: "spell-message-rune-1",
    name: "Message Rune",
    type: "spell",
    system: {
      description: {
        value: "<p>You record a message up to 5 minutes long and inscribe a special rune on any flat unattended surface or small object within reach.</p><p>You also specify a trigger that creatures must meet to hear the message.</p>",
      },
      level: {
        value: 2,
      },
      publication: {
        title: "Pathfinder Secrets of Magic",
      },
      traits: {
        rarity: "common",
        traditions: ["arcane", "occult"],
        value: ["concentrate", "linguistic", "manipulate", "mental"],
      },
    },
  });

  await writeJson(path.join(packRoot, "spells", "sending.json"), {
    _id: "spell-sending-1",
    name: "Sending",
    type: "spell",
    system: {
      description: {
        value: "<p>You send the creature a mental message of 25 words or fewer, and it can respond immediately with its own message of 25 words or fewer.</p>",
      },
      level: {
        value: 5,
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        traditions: ["arcane", "divine", "occult"],
        value: ["concentrate", "linguistic", "mental"],
      },
    },
  });

  await writeJson(path.join(packRoot, "spells", "sanctuary-circle.json"), {
    _id: "spell-sanctuary-circle-1",
    name: "Sanctuary Circle",
    type: "spell",
    system: {
      description: {
        value: "<p>A warding circle protects the target and grants a bonus to AC while they stand within the sanctified space.</p>",
      },
      level: {
        value: 3,
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        traditions: ["divine"],
        value: ["abjuration"],
      },
    },
  });

  await writeJson(path.join(packRoot, "spells", "breath-of-life.json"), {
    _id: "spell-breath-of-life-1",
    name: "Breath of Life",
    type: "spell",
    system: {
      description: {
        value: "<p>You stabilize the target and prevent it from dying.</p>",
      },
      level: {
        value: 5,
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        traditions: ["divine", "primal"],
        value: ["healing"],
      },
    },
  });

  await writeJson(path.join(packRoot, "spells", "teleport.json"), {
    _id: "spell-teleport-1",
    name: "Teleport",
    type: "spell",
    system: {
      description: {
        value: "<p>Targets are instantly transported to a destination you know well.</p>",
      },
      level: {
        value: 6,
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        traditions: ["arcane", "occult"],
        value: ["teleportation"],
      },
    },
  });

  await writeJson(path.join(packRoot, "spells", "water-walk.json"), {
    _id: "spell-water-walk-1",
    name: "Water Walk",
    type: "spell",
    system: {
      description: {
        value: "<p>The target can walk on the surface of water and move across liquid surfaces as if they were solid ground.</p>",
      },
      level: {
        value: 3,
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        traditions: ["divine", "primal"],
        value: ["transmutation"],
      },
    },
  });

  await writeJson(path.join(packRoot, "spells", "clear-mind.json"), {
    _id: "spell-clear-mind-1",
    name: "Clear Mind",
    type: "spell",
    system: {
      description: {
        value: "<p>You drive mental contamination from the target's mind and counteract an effect applying one of the following conditions.</p>",
      },
      level: {
        value: 4,
      },
      publication: {
        title: "Pathfinder Dark Archive",
      },
      traits: {
        rarity: "common",
        traditions: ["divine", "occult"],
        value: ["healing"],
      },
    },
  });

  await writeJson(path.join(packRoot, "spells", "fear.json"), {
    _id: "spell-fear-1",
    name: "Fear",
    type: "spell",
    system: {
      description: {
        value: "<p>You plant fear in the target; it must attempt a Will save.</p><p>The target becomes Frightened 2.</p>",
      },
      level: {
        value: 1,
      },
      defense: {
        save: {
          basic: false,
          statistic: "will",
        },
      },
      duration: {
        sustained: false,
        value: "1 minute",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      target: {
        value: "1 creature",
      },
      traits: {
        rarity: "common",
        traditions: ["arcane", "divine", "occult", "primal"],
        value: ["emotion", "fear", "mental"],
      },
      area: {
        value: 30,
        type: "cone",
      },
    },
  });

  await writeJson(path.join(packRoot, "spells", "blindness.json"), {
    _id: "spell-blindness-1",
    name: "Blindness",
    type: "spell",
    system: {
      description: {
        value: "<p>You blind the target. The target becomes Blinded until the spell ends.</p>",
      },
      level: {
        value: 3,
      },
      defense: {
        save: {
          basic: false,
          statistic: "fortitude",
        },
      },
      duration: {
        sustained: false,
        value: "unlimited",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      target: {
        value: "1 creature",
      },
      traits: {
        rarity: "common",
        traditions: ["arcane", "divine", "occult"],
        value: ["curse", "necromancy"],
      },
    },
  });

  await writeJson(path.join(packRoot, "spells", "hydraulic-push.json"), {
    _id: "spell-hydraulic-push-1",
    name: "Hydraulic Push",
    type: "spell",
    system: {
      description: {
        value: "<p>You call forth a powerful blast of pressurized water that bludgeons the target and knocks it back.</p>",
      },
      level: {
        value: 1,
      },
      defense: {
        save: {
          basic: true,
          statistic: "reflex",
        },
      },
      area: {
        value: 5,
        type: "burst",
      },
      duration: {
        sustained: false,
        value: "instantaneous",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      range: {
        value: "60 feet",
      },
      target: {
        value: "1 creature",
      },
      traits: {
        rarity: "common",
        traditions: ["arcane", "primal"],
        value: ["water"],
      },
    },
  });

  await writeJson(path.join(packRoot, "spells", "phantom-prison.json"), {
    _id: "spell-phantom-prison-1",
    name: "Phantom Prison",
    type: "spell",
    system: {
      description: {
        value: "<p>You completely surround a Large or smaller creature in immobile illusory walls, trapping it inside a false prison it can't escape.</p>",
      },
      level: {
        value: 5,
      },
      publication: {
        title: "Pathfinder Dark Archive",
      },
      traits: {
        rarity: "common",
        traditions: ["arcane", "occult"],
        value: ["illusion", "mental"],
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

  await writeJson(path.join(packRoot, "afflictions", "ghast-fever.json"), {
    _id: "affliction-ghast-fever-1",
    name: "Ghast Fever",
    type: "affliction",
    system: {
      description: {
        value: "<p>Stage 2 regains half as many Hit Points from all healing. Stage 4 gains no benefit from healing.</p>",
      },
      level: {
        value: 6,
      },
      publication: {
        title: "Pathfinder Monster Core",
      },
      traits: {
        rarity: "common",
        value: ["disease"],
      },
    },
  });

  await writeJson(path.join(packRoot, "afflictions", "bubonic-plague.json"), {
    _id: "affliction-bubonic-plague-1",
    name: "Bubonic Plague",
    type: "affliction",
    system: {
      description: {
        value: "<p>This widespread illness can sweep through entire communities, leaving few unaffected.</p><p><strong>Stage 1</strong> fatigued.</p>",
      },
      level: {
        value: 4,
      },
      publication: {
        title: "Pathfinder Bestiary 2",
      },
      traits: {
        rarity: "common",
        value: ["disease"],
      },
    },
  });

  await writeJson(path.join(packRoot, "afflictions", "reapers-shadow.json"), {
    _id: "affliction-reapers-shadow-1",
    name: "Reaper's Shadow",
    type: "affliction",
    system: {
      description: {
        value: "<p>This toxin erodes the connection between body and soul, tricking the latter into assuming the former has already died.</p><p><strong>Stage 1</strong> void damage and doomed 1.</p>",
      },
      level: {
        value: 13,
      },
      publication: {
        title: "Pathfinder #203: Shepherd of Decay",
      },
      traits: {
        rarity: "uncommon",
        value: ["poison", "void"],
      },
    },
  });

  await writeJson(path.join(packRoot, "afflictions", "endless-nightmare.json"), {
    _id: "affliction-endless-nightmare-1",
    name: "Endless Nightmare",
    type: "affliction",
    system: {
      description: {
        value: "<p>The victim's mind fills with terrifying visions.</p><p><strong>Stage 3</strong> The victim falls unconscious and can't be awakened.</p>",
      },
      level: {
        value: 9,
      },
      publication: {
        title: "Pathfinder Monster Core 2",
      },
      traits: {
        rarity: "common",
        value: ["curse", "emotion", "fear", "mental"],
      },
    },
  });

  await writeJson(path.join(packRoot, "afflictions", "mind-rotting-toxin.json"), {
    _id: "affliction-mind-rotting-toxin-1",
    name: "Mind-Rotting Toxin",
    type: "affliction",
    system: {
      description: {
        value: "<p>This toxin clouds the mind and leaves the victim stupefied with fractured thoughts.</p>",
      },
      publication: {
        title: "Pathfinder Lost Omens",
      },
      traits: {
        rarity: "common",
        value: ["poison"],
      },
    },
  });

  await writeJson(path.join(packRoot, "afflictions", "knockout-dram.json"), {
    _id: "affliction-knockout-dram-1",
    name: "Knockout Dram",
    type: "affliction",
    system: {
      description: {
        value: "<p>This soporific chemical quickly induces a deep unconsciousness and makes the victim sleep normally.</p>",
      },
      level: {
        value: 2,
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["poison"],
      },
    },
  });

  await writeJson(path.join(packRoot, "afflictions", "rotting-curse.json"), {
    _id: "affliction-rotting-curse-1",
    name: "Rotting Curse",
    type: "affliction",
    system: {
      description: {
        value: "<p>The victim's flesh rots and decays away, leaving necrotic wounds that worsen each day.</p>",
      },
      level: {
        value: 5,
      },
      publication: {
        title: "Pathfinder GM Core",
      },
      traits: {
        rarity: "common",
        value: ["curse"],
      },
    },
  });

  await writeJson(path.join(packRoot, "afflictions", "wasp-larva.json"), {
    _id: "affliction-wasp-larva-1",
    name: "Wasp Larva",
    type: "affliction",
    system: {
      description: {
        value: "<p>A parasitic larva burrows inside the host's body and hatches after several agonizing stages.</p>",
      },
      level: {
        value: 4,
      },
      publication: {
        title: "Pathfinder Monster Core",
      },
      traits: {
        rarity: "common",
        value: ["disease"],
      },
    },
  });

  await writeJson(path.join(packRoot, "afflictions", "liars-demise.json"), {
    _id: "affliction-liars-demise-1",
    name: "Liar's Demise",
    type: "affliction",
    system: {
      description: {
        value: "<p>The poison forces the victim to speak only the truth and can use no actions but to answer questions put to them.</p>",
      },
      level: {
        value: 5,
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["poison"],
      },
    },
  });

  await writeJson(path.join(packRoot, "afflictions", "black-apoxia.json"), {
    _id: "affliction-black-apoxia-1",
    name: "Black Apoxia",
    type: "affliction",
    system: {
      description: {
        value: "<p>The victim can't breathe properly as the disease leaves the lungs starved for air and the body breathless.</p>",
      },
      level: {
        value: 7,
      },
      publication: {
        title: "Pathfinder Adventure Path",
      },
      traits: {
        rarity: "common",
        value: ["disease"],
      },
    },
  });

  await writeJson(path.join(packRoot, "afflictions", "crystal-corruption.json"), {
    _id: "affliction-crystal-corruption-1",
    name: "Crystal Corruption",
    type: "affliction",
    system: {
      description: {
        value: "<p>Creatures afflicted by this curse slowly turn to solid crystal until they are petrified.</p>",
      },
      level: {
        value: 8,
      },
      publication: {
        title: "Pathfinder GM Core",
      },
      traits: {
        rarity: "common",
        value: ["curse"],
      },
    },
  });
}
