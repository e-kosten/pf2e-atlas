import path from "node:path";

import { writeJson } from "./pf2e-fixture.js";

export async function writeSpellAndAfflictionFixtureData(packRoot: string): Promise<void> {
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
}
