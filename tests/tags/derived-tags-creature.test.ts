import { describe, expect, it } from "vitest";

import { deriveRecordTags } from "../../src/tags/index.js";

describe("derived tag rules: creature", () => {
  it("derives creature context and setting tags", () => {
    expect(deriveRecordTags({
      name: "Graveyard Guard",
      category: "creature",
      subcategory: null,
      descriptionText: "This cemetery guard patrols the crypts beneath the old city.",
      traits: [],
    })).toEqual(expect.arrayContaining(["graveyard_setting", "underground_setting", "urban_setting", "profession_npc", "scene_adjacent"]));

    expect(deriveRecordTags({
      name: "Bog Wisp",
      category: "creature",
      subcategory: null,
      descriptionText: "A fey spirit that haunts marshy bogs and flooded mires.",
      traits: ["fey"],
    })).toEqual(expect.arrayContaining(["fey_threat", "swamp_setting"]));

    expect(deriveRecordTags({
      name: "Jungle Stalker",
      category: "creature",
      subcategory: null,
      descriptionText: "A patient ambusher that stalks the deep jungles and tangled woods.",
      traits: ["beast"],
    })).toContain("forest_setting");

    expect(deriveRecordTags({
      name: "Icebound Mariner",
      category: "creature",
      subcategory: null,
      descriptionText: "A sailor raider from the frozen sea who prowls icy coasts and shipwrecks.",
      traits: [],
    })).toEqual(expect.arrayContaining(["nautical_setting", "coastal_setting", "aquatic_setting", "arctic_setting"]));

    expect(deriveRecordTags({
      name: "Pelagic Stalker",
      category: "creature",
      subcategory: null,
      descriptionText: "A sleek predator built for sudden bursts of speed.",
      traits: ["aquatic", "beast"],
    })).toContain("aquatic_setting");

    expect(deriveRecordTags({
      name: "Amelekana",
      category: "creature",
      subcategory: null,
      descriptionText: "Amelekanas are amphibious ambush predators native to rivers and lakes across Castrovel.",
      traits: ["beast", "water"],
    })).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(deriveRecordTags({
      name: "Electric Eel",
      category: "creature",
      subcategory: null,
      descriptionText: "Usually found in freshwater rivers and lakes, an electric eel is not particularly aggressive.",
      traits: ["animal", "water"],
    })).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(deriveRecordTags({
      name: "Water Orm",
      category: "creature",
      subcategory: null,
      descriptionText: "These legendary creatures lurking in remote lakes inhabit cool inland waters, spy upon the shores of their lakes, and occasionally rise near the beach or a silty lake bed.",
      traits: ["beast", "water"],
    })).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(deriveRecordTags({
      name: "Water Orm",
      category: "creature",
      subcategory: null,
      descriptionText: "These legendary creatures lurking in remote lakes inhabit cool inland waters, spy upon the shores of their lakes, and occasionally rise near the beach or a silty lake bed.",
      traits: ["beast", "water"],
    })).not.toContain("coastal_setting");

    expect(deriveRecordTags({
      name: "Gathganara",
      category: "creature",
      subcategory: null,
      descriptionText: "Naiads protect streams, ponds, springs, and other natural bodies of fresh water where river tributaries meet beneath forest canopies.",
      traits: ["fey", "water"],
    })).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting", "forest_setting", "fey_threat"]));

    expect(deriveRecordTags({
      name: "Defaced Naiad Queen",
      category: "creature",
      subcategory: null,
      descriptionText: "Naiad queens rule over pristine wildernesses centered on untouched lakes or other bodies of fresh water.",
      traits: ["fey", "water"],
    })).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting", "fey_threat"]));

    expect(deriveRecordTags({
      name: "Coldmire Pond",
      category: "creature",
      subcategory: null,
      descriptionText: "Coldmire ponds are sentient bodies of living water that crawl along the ground or drift through still inland pools.",
      traits: ["elemental", "water"],
    })).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(deriveRecordTags({
      name: "Boiling Spring",
      category: "creature",
      subcategory: null,
      descriptionText: "A boiling spring is a humanoid water elemental made of scalding steam and bubbling water from a geothermal spring.",
      traits: ["elemental", "water"],
    })).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(deriveRecordTags({
      name: "Bog Prowler",
      category: "creature",
      subcategory: null,
      descriptionText: "An ambush hunter with a powerful bite.",
      traits: ["amphibious", "beast"],
    })).toContain("aquatic_setting");

    expect(deriveRecordTags({
      name: "Boggard Mire Scout",
      category: "creature",
      subcategory: null,
      descriptionText: "A croaking scout that watches for intruders from a reed blind.",
      traits: ["amphibious", "boggard", "humanoid"],
    })).toContain("swamp_setting");

    expect(deriveRecordTags({
      name: "Ghost Pirate Captain",
      category: "creature",
      subcategory: null,
      descriptionText: "An undead pirate captain prowls the ocean aboard a derelict ship.",
      traits: ["ghost", "undead"],
    })).toEqual(expect.arrayContaining(["nautical_setting", "aquatic_setting", "undead_threat"]));

    expect(deriveRecordTags({
      name: "Cairn Wight",
      category: "creature",
      subcategory: null,
      descriptionText: "A jealous undead guardian of barrows and sepulchers.",
      traits: ["undead", "wight"],
    })).toContain("graveyard_setting");

    expect(deriveRecordTags({
      name: "Caldera Oni",
      category: "creature",
      subcategory: null,
      descriptionText: "As hot-blooded as the lava that floods their homes, caldera oni hunger for battle.",
      traits: ["fiend", "oni"],
    })).toContain("volcanic_setting");

    expect(deriveRecordTags({
      name: "Coastal Prowler",
      category: "creature",
      subcategory: null,
      descriptionText: "A vigilant hunter prowls rocky shores and coastal reefs.",
      traits: ["beast"],
    })).toContain("coastal_setting");

    expect(deriveRecordTags({
      name: "Coastal Prowler",
      category: "creature",
      subcategory: null,
      descriptionText: "A vigilant hunter prowls rocky shores and coastal reefs.",
      traits: ["beast"],
    })).not.toContain("aquatic_setting");

    expect(deriveRecordTags({
      name: "Sea Drake",
      category: "creature",
      subcategory: null,
      descriptionText: "Long and slender, sea drakes have fins down the length of their backs and webbing between their talons. Although most sea drakes make their roosts high on ocean-facing cliffs, it isn't unheard of for them to dwell in underwater caves.",
      traits: ["amphibious", "dragon", "evil", "water"],
    })).toEqual(expect.arrayContaining(["aquatic_setting", "coastal_setting"]));

    expect(deriveRecordTags({
      name: "Tidepool Dragonet",
      category: "creature",
      subcategory: null,
      descriptionText: "Tidepool dragonets are lithe, eel-like sovereigns of their miniature tidal territories. Found in the marine environments that their name suggests, tidepool dragonets have strong jaws adapted to cracking open shells.",
      traits: ["amphibious", "dragon", "water"],
    })).toEqual(expect.arrayContaining(["aquatic_setting", "coastal_setting"]));

    expect(deriveRecordTags({
      name: "Island Watcher",
      category: "creature",
      subcategory: null,
      descriptionText: "A wary survivor keeps watch over a lonely island and its hidden paths.",
      traits: ["humanoid"],
    })).toContain("island_setting");

    expect(deriveRecordTags({
      name: "Plains Runner",
      category: "creature",
      subcategory: null,
      descriptionText: "A swift hunter races across grassy plains and open savannas.",
      traits: ["beast"],
    })).toContain("plains_setting");

    expect(deriveRecordTags({
      name: "Canyon Stalker",
      category: "creature",
      subcategory: null,
      descriptionText: "A patient hunter glides through canyons and narrow gorges carved into the badlands.",
      traits: ["beast"],
    })).toContain("canyon_setting");

    expect(deriveRecordTags({
      name: "Sun Mesa Wyvern",
      category: "creature",
      subcategory: null,
      descriptionText: "This wyvern nests among mesas and sun-scorched desert cliffs.",
      traits: ["dragon"],
    })).toEqual(expect.arrayContaining(["canyon_setting", "desert_setting", "mountain_setting"]));

    expect(deriveRecordTags({
      name: "Wasteland Reclaimer",
      category: "creature",
      subcategory: null,
      descriptionText: "A scarred scavenger roams barren wastelands and blasted wastes in search of salvage.",
      traits: ["humanoid"],
    })).toContain("wasteland_setting");

    expect(deriveRecordTags({
      name: "Ashen Reclaimer",
      category: "creature",
      subcategory: null,
      descriptionText: "This scavenger crosses barren wastes around lava fields and abandoned camps.",
      traits: ["humanoid"],
    })).toEqual(expect.arrayContaining(["wasteland_setting", "volcanic_setting"]));

    expect(deriveRecordTags({
      name: "Temple Custodian",
      category: "creature",
      subcategory: null,
      descriptionText: "A divine construct tends an ancient temple shrine and its sacred relics.",
      traits: ["construct"],
    })).toContain("temple_setting");

    expect(deriveRecordTags({
      name: "Fortress Warden",
      category: "creature",
      subcategory: null,
      descriptionText: "A grim defender patrols the walls of a mountain fortress and ancient citadel.",
      traits: ["humanoid"],
    })).toEqual(expect.arrayContaining(["fortress_setting", "mountain_setting"]));

    expect(deriveRecordTags({
      name: "Wealthy Vigilante",
      category: "creature",
      subcategory: null,
      descriptionText: "By night, this member of the nobility dons a false identity to mete out extralegal justice.",
      traits: ["human", "humanoid"],
    })).toEqual(expect.arrayContaining(["profession_npc", "scene_adjacent"]));

    expect(deriveRecordTags({
      name: "Prophet",
      category: "creature",
      subcategory: null,
      descriptionText: "A wandering prophet shares divine dreams and advice with the faithful.",
      traits: ["human", "humanoid"],
    })).toEqual(expect.arrayContaining(["profession_npc", "scene_adjacent"]));

    expect(deriveRecordTags({
      name: "Astradaemon",
      category: "creature",
      subcategory: null,
      descriptionText: "These unnerving daemons hunt the pathways between life and death and stalk the banks of the River of Souls in the Astral Plane.",
      traits: ["daemon", "fiend", "unholy"],
    })).toContain("astral_setting");

    expect(deriveRecordTags({
      name: "Shulsaga",
      category: "creature",
      subcategory: null,
      descriptionText: "Shepherds of the Silver Sea, shulsagas patrol the Astral Plane to protect the nascent demiplanes that form there.",
      traits: ["astral"],
    })).toContain("astral_setting");

    expect(deriveRecordTags({
      name: "Blodeuwedd",
      category: "creature",
      subcategory: null,
      descriptionText: "The mysterious blodeuwedds dwell in places where the boundaries between the Material Plane and the First World have worn thin, or around portals between the two planes.",
      traits: ["fey", "plant"],
    })).toEqual(expect.arrayContaining(["first_world_setting", "fey_threat", "plant_threat"]));

    expect(deriveRecordTags({
      name: "Bandersnatch",
      category: "creature",
      subcategory: null,
      descriptionText: "As with other legendary creatures from the First World, bandersnatches belong to the infamous group of creatures known collectively as the tane.",
      traits: ["beast", "tane"],
    })).toContain("first_world_setting");

    expect(deriveRecordTags({
      name: "Catrina",
      category: "creature",
      subcategory: null,
      descriptionText: "Catrinas meet souls in the Boneyard and help convince them of the finality of their fate to ease a spirit's passing.",
      traits: ["monitor", "psychopomp"],
    })).toContain("boneyard_setting");
  });

  it("uses glossary family evidence and blocks redundant scene-adjacent tags", () => {
    expect(deriveRecordTags({
      name: "Morlock Thrall",
      category: "creature",
      subcategory: null,
      descriptionText: "A thrall reshaped by a vampire master's curse.",
      traits: ["humanoid"],
      families: ["vampire"],
    })).toContain("undead_threat");

    expect(deriveRecordTags({
      name: "Manor Guard",
      category: "creature",
      subcategory: null,
      descriptionText: "A manor guard who patrols the estate grounds.",
      traits: ["human", "humanoid"],
      families: ["vampire"],
    })).toEqual(expect.arrayContaining(["profession_npc", "undead_threat"]));

    expect(deriveRecordTags({
      name: "Manor Guard",
      category: "creature",
      subcategory: null,
      descriptionText: "A manor guard who patrols the estate grounds.",
      traits: ["human", "humanoid"],
      families: ["vampire"],
    })).not.toContain("scene_adjacent");

    expect(deriveRecordTags({
      name: "Mythic Courtier",
      category: "creature",
      subcategory: null,
      descriptionText: "A courtier sustained by impossible necromancy.",
      traits: ["humanoid"],
      families: ["mythic", "lich"],
    })).toEqual(expect.arrayContaining(["profession_npc", "undead_threat"]));

    expect(deriveRecordTags({
      name: "Mythic Courtier",
      category: "creature",
      subcategory: null,
      descriptionText: "A courtier sustained by impossible necromancy.",
      traits: ["humanoid"],
      families: ["mythic", "lich"],
    })).not.toContain("scene_adjacent");
  });

  it("derives creature motif tags without collapsing into raw vibes", () => {
    expect(deriveRecordTags({
      name: "Court Jester",
      category: "creature",
      subcategory: null,
      descriptionText: "Though court jesters are often mocked for easy amusement, this jester hides malice behind painted smiles and cutting wit.",
      traits: ["human", "humanoid"],
    })).toContain("carnival_show");

    expect(deriveRecordTags({
      name: "Mechanical Carny",
      category: "creature",
      subcategory: null,
      descriptionText: "Mechanical carnies are constructs manufactured to serve as entertainers, cleaners, and guards at carnivals and circuses.",
      traits: ["construct"],
    })).toContain("carnival_show");

    expect(deriveRecordTags({
      name: "Soulbound Doll",
      category: "creature",
      subcategory: null,
      descriptionText: "Soulbound dolls are eerie mannequins or playthings that have been imbued with a small piece of a deceased mortal's soul.",
      traits: ["construct"],
    })).toContain("living_toy");

    expect(deriveRecordTags({
      name: "Masque Mannequin",
      category: "creature",
      subcategory: null,
      descriptionText: "Masque mannequins are soulbound constructs animated by a fragment of a once-living soul infused into a mannequin or dressmaker's dummy.",
      traits: ["construct"],
    })).toContain("living_toy");

    expect(deriveRecordTags({
      name: "Fire Scamp",
      category: "creature",
      subcategory: null,
      descriptionText: "Although arguably quite friendly, fire scamps delight in fire and playing pranks on everyone they befriend.",
      traits: ["elemental", "fire"],
    })).toContain("trickster_chaos");

    expect(deriveRecordTags({
      name: "Brass Dragon",
      category: "creature",
      subcategory: null,
      descriptionText: "Brass dragons are whimsical tricksters who delight in humor and play.",
      traits: ["chaotic", "dragon", "fire"],
    })).toContain("trickster_chaos");

    expect(deriveRecordTags({
      name: "Dancer",
      category: "creature",
      subcategory: null,
      descriptionText: "A nimble performer who entertains nobles at court.",
      traits: ["human", "humanoid"],
    })).not.toContain("carnival_show");

    expect(deriveRecordTags({
      name: "Goblin Igniter",
      category: "creature",
      subcategory: null,
      descriptionText: "Goblins think fire is a fun toy and admire anyone willing to burn down a barn for sport.",
      traits: ["goblin", "humanoid"],
    })).not.toContain("living_toy");

    expect(deriveRecordTags({
      name: "Chaos Reaver",
      category: "creature",
      subcategory: null,
      descriptionText: "A chaotic fiend that leaves ruin in its wake.",
      traits: ["chaotic", "fiend"],
    })).not.toContain("trickster_chaos");
  });

  it("avoids known creature false positives and requires enough weighted evidence", () => {
    expect(deriveRecordTags({
      name: "Accuser Agent",
      category: "creature",
      subcategory: null,
      descriptionText: "Accuser agents might be high court advocates, official spymasters, or innocuous adjutants delivering important messages to magistrates, generals, officers, or mercenaries.",
      traits: ["human", "humanoid"],
    })).not.toContain("arctic_setting");

    expect(deriveRecordTags({
      name: "Abandoned Zealot",
      category: "creature",
      subcategory: null,
      descriptionText: "Abandoned zealots arise from false faiths unknown to most worshippers.",
      traits: ["undead", "spirit"],
    })).not.toContain("nautical_setting");

    expect(deriveRecordTags({
      name: "Castruccio Irovetti",
      category: "creature",
      subcategory: null,
      descriptionText: "Since his flight from Numeria, Castruccio Irovetti has ruled Pitax for years and remains a major player in this River Kingdom's political scene.",
      traits: ["human", "humanoid"],
    })).not.toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(deriveRecordTags({
      name: "Apothecary Bee",
      category: "creature",
      subcategory: null,
      descriptionText: "Long-abandoned gardens still grow along the Sphinx River in Osirion, where many apothecary bees prowl for flowers that meet their standards.",
      traits: ["animal"],
    })).not.toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(deriveRecordTags({
      name: "Astradaemon",
      category: "creature",
      subcategory: null,
      descriptionText: "Astradaemons hunt the pathways between life and death and stalk the banks of the River of Souls in the Astral Plane.",
      traits: ["daemon", "fiend", "unholy"],
    })).not.toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(deriveRecordTags({
      name: "Bleachling Survivor",
      category: "creature",
      subcategory: null,
      descriptionText: "Those few gnomes who survive the Bleaching emerge transformed, forever marked by the primal spark of the First World that still flickers within them.",
      traits: ["gnome", "humanoid"],
    })).not.toContain("first_world_setting");

    expect(deriveRecordTags({
      name: "Shade (Astral Plane)",
      category: "creature",
      subcategory: null,
      descriptionText: "When a mortal dies, their soul travels to the Boneyard in the Outer Planes where they're judged by Pharasma, though these shades merely appear as astrally projected versions of their mortal forms.",
      traits: ["astral", "shade"],
    })).not.toContain("boneyard_setting");

    expect(deriveRecordTags({
      name: "Shrine Caretaker",
      category: "creature",
      subcategory: null,
      descriptionText: "This combusted haunts a shrine and sometimes throws itself into lakes or rivers, believing the water will quiet the flames consuming it.",
      traits: ["spirit", "undead", "unholy"],
    })).not.toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(deriveRecordTags({
      name: "Hooktongue",
      category: "creature",
      subcategory: null,
      descriptionText: "Legendary creatures lurking in remote lakes, water orms often spy upon the shores of their lakes and surface near the beach when curiosity overtakes caution.",
      traits: ["beast", "water"],
    })).not.toContain("coastal_setting");

    expect(deriveRecordTags({
      name: "Old Herok",
      category: "creature",
      subcategory: null,
      descriptionText: "Old Herok is a water orm that hides in deep lakes, watching the shores of its inland domain and surfacing near lonely beaches only when it must.",
      traits: ["beast", "water"],
    })).not.toContain("coastal_setting");

    expect(deriveRecordTags({
      name: "Sky Canyon Balladeer",
      category: "creature",
      subcategory: null,
      descriptionText: "A performer sings a ballad titled Canyon Echoes beneath an open sky.",
      traits: ["humanoid"],
    })).not.toEqual(expect.arrayContaining(["canyon_setting", "wasteland_setting"]));

    expect(deriveRecordTags({
      name: "Adamantine Golem",
      category: "creature",
      subcategory: null,
      descriptionText: "Crafting an adamantine golem requires mounting a mining expedition while guardian suits stand watch.",
      traits: ["construct", "golem", "mindless"],
    })).not.toEqual(expect.arrayContaining(["profession_npc", "scene_adjacent"]));

    expect(deriveRecordTags({
      name: "Animated Armor",
      category: "creature",
      subcategory: null,
      descriptionText: "Animated armor serves as guardians and training partners in martial academies.",
      traits: ["construct", "mindless"],
    })).not.toEqual(expect.arrayContaining(["profession_npc", "scene_adjacent"]));

    expect(deriveRecordTags({
      name: "Vanth Guardian Flock",
      category: "creature",
      subcategory: null,
      descriptionText: "Vanth psychopomps are eternal guardians of the cycle of life and death.",
      traits: ["monitor", "psychopomp", "troop"],
    })).not.toEqual(expect.arrayContaining(["profession_npc", "scene_adjacent"]));

    expect(deriveRecordTags({
      name: "Harbor Watcher",
      category: "creature",
      subcategory: null,
      descriptionText: "A sentry posted near the harbor gates.",
      traits: [],
    })).not.toContain("nautical_setting");

    expect(deriveRecordTags({
      name: "Harbor Mariner",
      category: "creature",
      subcategory: null,
      descriptionText: "A mariner who keeps watch over the harbor docks.",
      traits: [],
    })).toContain("nautical_setting");

    expect(deriveRecordTags({
      name: "Cinder Tyrant",
      category: "creature",
      subcategory: null,
      descriptionText: "A blazing tyrant crowned in smoke and cinders.",
      traits: ["dragon", "fire"],
    })).not.toContain("volcanic_setting");

    expect(deriveRecordTags({
      name: "Temple Scavenger",
      category: "creature",
      subcategory: null,
      descriptionText: "This scavenger lurks among the ruins of a collapsed temple.",
      traits: [],
    })).toEqual(expect.arrayContaining(["ruins_setting", "temple_setting"]));

    expect(deriveRecordTags({
      name: "Ancient Hall Watcher",
      category: "creature",
      subcategory: null,
      descriptionText: "A watchful spirit stalks an ancient hall guarded by silent echoes.",
      traits: [],
    })).not.toContain("ruins_setting");
  });
});
