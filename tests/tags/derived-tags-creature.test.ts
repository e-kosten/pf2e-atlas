import { describe, expect, it } from "vitest";

import { deriveRecordTags } from "../../src/tags/index.js";

describe("derived tag rules: creature", () => {
  it("derives creature context and setting tags", () => {
    expect(
      deriveRecordTags({
        name: "Graveyard Guard",
        category: "creature",
        subcategory: null,
        descriptionText: "This cemetery guard patrols the crypts beneath the old city.",
        traits: [],
      }),
    ).toEqual(expect.arrayContaining(["graveyard_setting", "profession_npc", "civic_npc"]));
    expect(
      deriveRecordTags({
        name: "Graveyard Guard",
        category: "creature",
        subcategory: null,
        descriptionText: "This cemetery guard patrols the crypts beneath the old city.",
        traits: [],
      }),
    ).not.toContain("underground_setting");

    expect(
      deriveRecordTags({
        name: "Drain Scuttler",
        category: "creature",
        subcategory: null,
        descriptionText: "This scavenger makes its home in storm drains and culverts beneath crowded city streets.",
        traits: ["beast"],
      }),
    ).toContain("urban_setting");

    expect(
      deriveRecordTags({
        name: "Bog Wisp",
        category: "creature",
        subcategory: null,
        descriptionText: "A fey spirit that haunts marshy bogs and flooded mires.",
        traits: ["fey"],
      }),
    ).toContain("swamp_setting");

    expect(
      deriveRecordTags({
        name: "Giant Swamp Fly",
        category: "creature",
        subcategory: null,
        descriptionText: "",
        traits: ["animal"],
      }),
    ).toContain("swamp_setting");

    expect(
      deriveRecordTags({
        name: "Shino Hakusa",
        category: "creature",
        subcategory: null,
        descriptionText: "Female Tian-Shu assassin",
        traits: ["human", "humanoid"],
      }),
    ).toContain("tian_xia_setting");

    expect(
      deriveRecordTags({
        name: "Caustic Monitor",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Caustic monitors are enormous lizards native to eastern Minata known for the corrosive enzyme in their mucus and saliva.",
        traits: ["animal"],
      }),
    ).toContain("tian_xia_setting");

    expect(
      deriveRecordTags({
        name: "Tengu Sneak",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Tengu are an adaptable people originally from the continent of Tian Xia, but their travels have taken them across all of Golarion.",
        traits: ["humanoid", "tengu"],
      }),
    ).not.toContain("tian_xia_setting");

    expect(
      deriveRecordTags({
        name: "Jungle Stalker",
        category: "creature",
        subcategory: null,
        descriptionText: "A patient ambusher that stalks the deep jungles and tangled woods.",
        traits: ["beast"],
      }),
    ).toEqual(expect.arrayContaining(["jungle_setting", "forest_setting"]));

    expect(
      deriveRecordTags({
        name: "Icebound Mariner",
        category: "creature",
        subcategory: null,
        descriptionText: "A sailor raider from the frozen sea who prowls icy coasts and shipwrecks.",
        traits: [],
      }),
    ).toEqual(expect.arrayContaining(["nautical_setting", "coastal_setting", "aquatic_setting", "arctic_setting"]));

    expect(
      deriveRecordTags({
        name: "Pelagic Stalker",
        category: "creature",
        subcategory: null,
        descriptionText: "A sleek predator built for sudden bursts of speed.",
        traits: ["aquatic", "beast"],
      }),
    ).toContain("aquatic_setting");

    expect(
      deriveRecordTags({
        name: "Amelekana",
        category: "creature",
        subcategory: null,
        descriptionText: "Amelekanas are amphibious ambush predators native to rivers and lakes across Castrovel.",
        traits: ["beast", "water"],
      }),
    ).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(
      deriveRecordTags({
        name: "Electric Eel",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Usually found in freshwater rivers and lakes, an electric eel is not particularly aggressive.",
        traits: ["animal", "water"],
      }),
    ).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(
      deriveRecordTags({
        recordKey: "pathfinder-bestiary-3:6OxiStysMq65xKgS",
        name: "Kongamato",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Though they dwell in swamps and other still waters, kongamatos prefer to hunt in rivers and streams, since running water delivers new prey on a regular basis.",
        traits: ["animal"],
      }),
    ).toContain("freshwater_setting");

    expect(
      deriveRecordTags({
        name: "Water Orm",
        category: "creature",
        subcategory: null,
        descriptionText:
          "These legendary creatures lurking in remote lakes inhabit cool inland waters, spy upon the shores of their lakes, and occasionally rise near the beach or a silty lake bed.",
        traits: ["beast", "water"],
      }),
    ).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(
      deriveRecordTags({
        name: "Water Orm",
        category: "creature",
        subcategory: null,
        descriptionText:
          "These legendary creatures lurking in remote lakes inhabit cool inland waters, spy upon the shores of their lakes, and occasionally rise near the beach or a silty lake bed.",
        traits: ["beast", "water"],
      }),
    ).not.toContain("coastal_setting");

    expect(
      deriveRecordTags({
        name: "Gathganara",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Naiads protect streams, ponds, springs, and other natural bodies of fresh water where river tributaries meet beneath forest canopies.",
        traits: ["fey", "water"],
      }),
    ).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting", "forest_setting"]));

    expect(
      deriveRecordTags({
        name: "Defaced Naiad Queen",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Naiad queens rule over pristine wildernesses centered on untouched lakes or other bodies of fresh water.",
        traits: ["fey", "water"],
      }),
    ).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(
      deriveRecordTags({
        name: "Coldmire Pond",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Coldmire ponds are sentient bodies of living water that crawl along the ground or drift through still inland pools.",
        traits: ["elemental", "water"],
      }),
    ).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(
      deriveRecordTags({
        name: "Boiling Spring",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A boiling spring is a humanoid water elemental made of scalding steam and bubbling water from a geothermal spring.",
        traits: ["elemental", "water"],
      }),
    ).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(
      deriveRecordTags({
        name: "Benthic Worm",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The benthic worm prefers to lie in wait within flooded caverns and often brings with it waters from the submerged river or lake it calls home.",
        traits: ["animal"],
      }),
    ).toContain("freshwater_setting");

    expect(
      deriveRecordTags({
        name: "Bog Prowler",
        category: "creature",
        subcategory: null,
        descriptionText: "An ambush hunter with a powerful bite.",
        traits: ["amphibious", "beast"],
      }),
    ).toContain("aquatic_setting");

    expect(
      deriveRecordTags({
        name: "Boggard Mire Scout",
        category: "creature",
        subcategory: null,
        descriptionText: "A croaking scout that watches for intruders from a reed blind.",
        traits: ["amphibious", "boggard", "humanoid"],
      }),
    ).toContain("swamp_setting");

    expect(
      deriveRecordTags({
        recordKey: "pathfinder-monster-core-2:zGdsQWq6uHjE7TSx",
        name: "Tikbalang",
        category: "creature",
        subcategory: null,
        descriptionText: "Tikbalangs are forest creatures that delight in leading travelers astray.",
        traits: ["fey", "occult"],
      }),
    ).toContain("forest_setting");

    expect(
      deriveRecordTags({
        name: "Ghost Pirate Captain",
        category: "creature",
        subcategory: null,
        descriptionText: "An undead pirate captain prowls the ocean aboard a derelict ship.",
        traits: ["ghost", "undead"],
      }),
    ).toEqual(expect.arrayContaining(["nautical_setting", "aquatic_setting", "undead_adjacent"]));

    expect(
      deriveRecordTags({
        name: "Cairn Wight",
        category: "creature",
        subcategory: null,
        descriptionText: "A jealous undead guardian of barrows and sepulchers.",
        traits: ["undead", "wight"],
      }),
    ).toContain("graveyard_setting");

    expect(
      deriveRecordTags({
        name: "Crypt Haunter",
        category: "creature",
        subcategory: null,
        descriptionText: "An undead guardian lurks within crypts, catacombs, and tombs beneath a ruined cemetery.",
        traits: ["undead"],
      }),
    ).toContain("graveyard_setting");
    expect(
      deriveRecordTags({
        name: "Crypt Haunter",
        category: "creature",
        subcategory: null,
        descriptionText: "An undead guardian lurks within crypts, catacombs, and tombs beneath a ruined cemetery.",
        traits: ["undead"],
      }),
    ).not.toContain("underground_setting");

    expect(
      deriveRecordTags({
        name: "Lightless Warped",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Other subterranean monsters drive these warped horrors back to their lightless lairs deep underground.",
        traits: ["aberration"],
      }),
    ).toContain("underground_setting");

    expect(
      deriveRecordTags({
        name: "Stone Mauler",
        category: "creature",
        subcategory: null,
        descriptionText:
          "These towering heaps of earth can inflict tremendous damage up close and from afar. Earth elementals make excellent bodyguards for adventuresome spelunkers and are ideal protectors of important subterranean locations such as vaults and treasuries.",
        traits: ["elemental", "earth"],
      }),
    ).toContain("underground_setting");

    expect(
      deriveRecordTags({
        name: "Caldera Oni",
        category: "creature",
        subcategory: null,
        descriptionText: "As hot-blooded as the lava that floods their homes, caldera oni hunger for battle.",
        traits: ["fiend", "oni"],
      }),
    ).toContain("volcanic_setting");

    expect(
      deriveRecordTags({
        name: "Ammut Hunter",
        category: "creature",
        subcategory: null,
        descriptionText: "These desert-dwelling fiends lair among dunes and oases of the Black Desert.",
        traits: ["fiend"],
      }),
    ).toContain("desert_setting");

    expect(
      deriveRecordTags({
        name: "Black Scorpion",
        category: "creature",
        subcategory: null,
        descriptionText:
          "With a carapace the color of polished obsidian and a penchant for attacking villages, this humongous scorpion is one of the desert's most frightening predators.",
        traits: ["animal"],
      }),
    ).toContain("desert_setting");

    expect(
      deriveRecordTags({
        name: "Otyugh",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Lords of sewers, ditches, and landfills, otyughs are filthy monstrosities that stomp about on three massive legs in search of tasty garbage and refuse.",
        traits: ["aberration"],
      }),
    ).toContain("urban_setting");

    expect(
      deriveRecordTags({
        name: "Coastal Prowler",
        category: "creature",
        subcategory: null,
        descriptionText: "A vigilant hunter prowls rocky shores and coastal reefs.",
        traits: ["beast"],
      }),
    ).toContain("coastal_setting");

    expect(
      deriveRecordTags({
        name: "Coastal Prowler",
        category: "creature",
        subcategory: null,
        descriptionText: "A vigilant hunter prowls rocky shores and coastal reefs.",
        traits: ["beast"],
      }),
    ).not.toContain("aquatic_setting");

    expect(
      deriveRecordTags({
        name: "Sea Drake",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Long and slender, sea drakes have fins down the length of their backs and webbing between their talons. Although most sea drakes make their roosts high on ocean-facing cliffs, it isn't unheard of for them to dwell in underwater caves.",
        traits: ["amphibious", "dragon", "evil", "water"],
      }),
    ).toEqual(expect.arrayContaining(["aquatic_setting", "coastal_setting"]));

    expect(
      deriveRecordTags({
        name: "Tidepool Dragonet",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Tidepool dragonets are lithe, eel-like sovereigns of their miniature tidal territories. Found in the marine environments that their name suggests, tidepool dragonets have strong jaws adapted to cracking open shells.",
        traits: ["amphibious", "dragon", "water"],
      }),
    ).toEqual(expect.arrayContaining(["aquatic_setting", "coastal_setting"]));

    expect(
      deriveRecordTags({
        name: "Island Watcher",
        category: "creature",
        subcategory: null,
        descriptionText: "A wary survivor keeps watch over a lonely island and its hidden paths.",
        traits: ["humanoid"],
      }),
    ).toContain("island_setting");

    expect(
      deriveRecordTags({
        name: "Storm Giant",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Storm giants are looming but benevolent stewards of sea and sky, often serving as the natural guardians of tropical islands, coastlines, or rocky archipelagos.",
        traits: ["giant", "humanoid"],
      }),
    ).toContain("island_setting");

    expect(
      deriveRecordTags({
        name: "Plains Runner",
        category: "creature",
        subcategory: null,
        descriptionText: "A swift hunter races across grassy plains and open savannas.",
        traits: ["beast"],
      }),
    ).toContain("plains_setting");

    expect(
      deriveRecordTags({
        name: "Canyon Stalker",
        category: "creature",
        subcategory: null,
        descriptionText: "A patient hunter glides through canyons and narrow gorges carved into the badlands.",
        traits: ["beast"],
      }),
    ).toContain("canyon_setting");

    expect(
      deriveRecordTags({
        name: "Sun Mesa Wyvern",
        category: "creature",
        subcategory: null,
        descriptionText: "This wyvern nests among mesas and sun-scorched desert cliffs.",
        traits: ["dragon"],
      }),
    ).toEqual(expect.arrayContaining(["canyon_setting", "desert_setting", "mountain_setting"]));

    expect(
      deriveRecordTags({
        name: "Wasteland Reclaimer",
        category: "creature",
        subcategory: null,
        descriptionText: "A scarred scavenger roams barren wastelands and blasted wastes in search of salvage.",
        traits: ["humanoid"],
      }),
    ).toContain("wasteland_setting");

    expect(
      deriveRecordTags({
        name: "Ashen Reclaimer",
        category: "creature",
        subcategory: null,
        descriptionText: "This scavenger crosses barren wastes around lava fields and abandoned camps.",
        traits: ["humanoid"],
      }),
    ).toEqual(expect.arrayContaining(["wasteland_setting", "volcanic_setting"]));

    expect(
      deriveRecordTags({
        name: "Temple Custodian",
        category: "creature",
        subcategory: null,
        descriptionText: "A divine construct tends an ancient temple shrine and its sacred relics.",
        traits: ["construct"],
      }),
    ).toContain("temple_setting");

    expect(
      deriveRecordTags({
        name: "Divine Warden of Brigh",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Imbued with the divine energy of the goddess of clockwork and invention, divine wardens of Brigh are prominent in Alkenstar as guardians of the Temple of Brigh there. Created through complex rituals performed by a faith's adherents, this divine spark allows the divine warden to serve as the protector for a temple, shrine, or other holy site. Such guardians aren't intrinsically bound to a fixed location, but they rarely leave the temple or site over which they watch.",
        traits: ["construct"],
      }),
    ).toContain("temple_setting");

    expect(
      deriveRecordTags({
        name: "Fortress Warden",
        category: "creature",
        subcategory: null,
        descriptionText: "A grim defender patrols the walls of a mountain fortress and ancient citadel.",
        traits: ["humanoid"],
      }),
    ).toEqual(expect.arrayContaining(["fortress_setting", "mountain_setting", "profession_npc", "enforcer_npc"]));

    expect(
      deriveRecordTags({
        name: "Fortress Warden",
        category: "creature",
        subcategory: null,
        descriptionText: "A grim defender patrols the walls of a mountain fortress and ancient citadel.",
        traits: ["humanoid"],
      }),
    ).not.toContain("civic_npc");

    expect(
      deriveRecordTags({
        name: "Clockwork Infantry",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A clockwork infantry is a force to be reckoned with on the battlefield, advancing as a disciplined military formation.",
        traits: ["clockwork", "construct", "mindless", "troop"],
      }),
    ).toContain("battlefield_setting");

    expect(
      deriveRecordTags({
        name: "Goblin Rabble",
        category: "creature",
        subcategory: null,
        descriptionText: "A disorderly mass of goblin raiders surges forward in a loose but dangerous formation.",
        traits: ["goblin", "humanoid", "troop"],
      }),
    ).toContain("battlefield_setting");

    expect(
      deriveRecordTags({
        name: "Zombie Mammoth",
        category: "creature",
        subcategory: null,
        descriptionText:
          "This monstrous creature can overrun defenses and stomp foes into the ground, making it a terror on any battlefield.",
        traits: ["undead", "zombie"],
      }),
    ).toContain("battlefield_setting");

    expect(
      deriveRecordTags({
        name: "Rust Hag",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Rust hags have a knack for technology. They make their homes in hollowed-out factories, abandoned tenements, and other sites of urban decay.",
        traits: ["hag"],
      }),
    ).toContain("urban_setting");

    expect(
      deriveRecordTags({
        name: "Gargoyle",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Gargoyles are monstrous hunters made of elemental stone. They use their resemblance to decorative statues to hide in plain sight in cities during the day. City-dwelling gargoyles who remain in the same locale long enough slowly morph to match the style of the local architecture.",
        traits: ["beast", "earth"],
      }),
    ).toContain("urban_setting");

    expect(
      deriveRecordTags({
        name: "Watch Officer",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Often leading a small team of lower-ranking guards, watch officers patrol their assigned areas to maintain order and enforce laws.",
        traits: ["human", "humanoid"],
        families: ["official"],
      }),
    ).toContain("urban_setting");

    expect(
      deriveRecordTags({
        name: "Guardian Aluum",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The most common aluum is powered by the bound soul of a loyal city servant. Aluums are powerful metal and stone constructs originally created by the Pactmasters to maintain order in Katapesh.",
        traits: ["construct", "mindless"],
      }),
    ).toContain("urban_setting");

    expect(
      deriveRecordTags({
        name: "Skulk",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Skulks gather around the periphery of large settlements, slipping through entrances and exits from cities and using access ways around sewers to watch who comes and goes from the city.",
        traits: ["humanoid", "skulk"],
      }),
    ).toContain("urban_setting");

    expect(
      deriveRecordTags({
        name: "Furnerico",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A mass of undulating tendrils and nerves, furnericos stalk sewer systems below large cities, reveling in the filth and rotting detritus created by those who dwell above the middens and sumps they call home.",
        traits: ["aberration"],
      }),
    ).toEqual(expect.arrayContaining(["urban_setting", "underground_setting"]));

    expect(
      deriveRecordTags({
        name: "Girtablilu Guardian",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Sentries patrol the outskirts of girtablilu communities, watching for external threats set on reckless exploration of the girtablilus' sacred site. Girtablilus are desert-dwelling guardians most often found defending ancient temples and religious artifacts with zealous fervor.",
        traits: ["humanoid"],
      }),
    ).toContain("temple_setting");

    expect(
      deriveRecordTags({
        name: "Viscous Black Pudding",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Most often found below ground, these oozes scour caves for objects to dissolve with their corrosive secretions.",
        traits: ["ooze"],
      }),
    ).toContain("underground_setting");

    expect(
      deriveRecordTags({
        name: "Onturat",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Swirling down from misty peaks and through howling mountain passes like an evil wind, the vortex of bones known as a skulltaker is a terrible manifestation of the delirium and agony experienced by doomed climbers and lost trailblazers.",
        traits: ["undead"],
      }),
    ).toContain("mountain_setting");

    expect(
      deriveRecordTags({
        name: "Urthagul",
        category: "creature",
        subcategory: null,
        descriptionText:
          "This gug predator is native to the Darklands and lurks in subterranean tunnels and caverns, dragging prey back to its lightless lair.",
        traits: ["aberration"],
      }),
    ).toContain("underground_setting");

    expect(
      deriveRecordTags({
        name: "Aapoph Serpentfolk",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Before their ancient clash with humanity devastated their civilization, serpentfolk were masters of a sprawling underground empire. Today, the central realm of the Darklands retains the old name of that empire: Sekamina.",
        traits: ["humanoid", "mutant", "serpentfolk"],
      }),
    ).toContain("underground_setting");

    expect(
      deriveRecordTags({
        name: "Drow Hunter",
        category: "creature",
        subcategory: null,
        descriptionText: "Hunters seek out game to keep drow communities fed and functioning.",
        traits: ["chaotic", "drow", "elf", "humanoid"],
      }),
    ).toContain("underground_setting");

    expect(
      deriveRecordTags({
        name: "Rosethorn Ram",
        category: "creature",
        subcategory: null,
        descriptionText:
          "These hardy mountain dwellers surpass their lowlander cousins in stubbornness and agility, making their homes among steep peaks and rocky slopes.",
        traits: ["beast"],
      }),
    ).toContain("mountain_setting");

    expect(
      deriveRecordTags({
        name: "Thunderbird",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Thunderbirds soar high above storm clouds and wheel through the open sky before nesting on remote mountain aeries.",
        traits: ["beast"],
      }),
    ).toEqual(expect.arrayContaining(["sky_setting", "mountain_setting"]));

    expect(
      deriveRecordTags({
        name: "Huldra",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Huldras are wardens of the woodlands they occupy, grown from saplings by powerful forest-dwelling fey to protect the forest itself.",
        traits: ["fey"],
      }),
    ).toContain("forest_setting");

    expect(
      deriveRecordTags({
        name: "Gumiho",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Gumihos are legendary nine-tailed fox creatures who, when disguised in humanoid form, use charm and guile to lure prey deep into the forest before revealing their true form and striking.",
        traits: ["chaotic", "evil", "fey"],
      }),
    ).toContain("forest_setting");

    expect(
      deriveRecordTags({
        name: "Scarecrow",
        category: "creature",
        subcategory: null,
        descriptionText: "A cruel spirit haunts the countryside from its post beside a lonely farmstead and barn.",
        traits: ["construct"],
      }),
    ).toContain("rural_setting");

    expect(
      deriveRecordTags({
        name: "Wealthy Vigilante",
        category: "creature",
        subcategory: null,
        descriptionText: "By night, this member of the nobility dons a false identity to mete out extralegal justice.",
        traits: ["human", "humanoid"],
      }),
    ).toContain("profession_npc");

    expect(
      deriveRecordTags({
        name: "Wealthy Vigilante",
        category: "creature",
        subcategory: null,
        descriptionText: "By night, this member of the nobility dons a false identity to mete out extralegal justice.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("civic_npc");

    expect(
      deriveRecordTags({
        name: "Prophet",
        category: "creature",
        subcategory: null,
        descriptionText: "A wandering prophet shares divine dreams and advice with the faithful.",
        traits: ["human", "humanoid"],
      }),
    ).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(
      deriveRecordTags({
        name: "Astradaemon",
        category: "creature",
        subcategory: null,
        descriptionText:
          "These unnerving daemons hunt the pathways between life and death and stalk the banks of the River of Souls in the Astral Plane.",
        traits: ["daemon", "fiend", "unholy"],
      }),
    ).toContain("astral_setting");

    expect(
      deriveRecordTags({
        name: "Shulsaga",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Shepherds of the Silver Sea, shulsagas patrol the Astral Plane to protect the nascent demiplanes that form there.",
        traits: ["astral"],
      }),
    ).toContain("astral_setting");

    expect(
      deriveRecordTags({
        name: "Planar Shepherd",
        category: "creature",
        subcategory: null,
        descriptionText: "These guardians patrol stable portals and nascent demiplanes adrift in the silver void.",
        traits: ["monitor"],
      }),
    ).toContain("astral_setting");

    expect(
      deriveRecordTags({
        name: "Blodeuwedd",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The mysterious blodeuwedds dwell in places where the boundaries between the Material Plane and the First World have worn thin, or around portals between the two planes.",
        traits: ["fey", "plant"],
      }),
    ).toContain("first_world_setting");

    expect(
      deriveRecordTags({
        name: "Nightgaunt",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Nightgaunts often gather in vast colonies in the Dreamlands, where they entertain each other by sharing emotion memories of their meals through strange caresses.",
        traits: ["aberration", "dream"],
      }),
    ).toContain("dreamlands_setting");

    expect(
      deriveRecordTags({
        name: "Shantak",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Despite not being native to the Dimension of Dreams, they're commonly found in that realm. Their ability to fly through space affords them swift travel between Leng and more hospitable reaches of the Dreamlands.",
        traits: ["beast"],
      }),
    ).toContain("dreamlands_setting");

    expect(
      deriveRecordTags({
        name: "Dream Aethusa",
        category: "creature",
        subcategory: null,
        descriptionText: "Female human wizard",
        traits: ["dream", "human", "humanoid", "mythic"],
      }),
    ).toContain("dreamlands_setting");

    expect(
      deriveRecordTags({
        name: "Animate Dream",
        category: "creature",
        subcategory: null,
        descriptionText:
          "In these forms, animate dreams find their way out of the Dreamlands and into the waking world, only to discover they have no way of returning and suffer a relentless hunger that only new nightmares can sate.",
        traits: ["dream", "incorporeal"],
      }),
    ).not.toContain("dreamlands_setting");

    expect(
      deriveRecordTags({
        name: "Living Nightmare",
        category: "creature",
        subcategory: null,
        descriptionText:
          "In these forms, animate dreams find their way out of the Dreamlands and into the waking world, only to discover they have no way of returning and suffer a relentless hunger that only new nightmares can sate.",
        traits: ["dream", "incorporeal"],
      }),
    ).not.toContain("dreamlands_setting");

    expect(
      deriveRecordTags({
        name: "Watchtower Shadow",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The mysterious undead known as shadows lurk in dark places and feed on those who stray too far from the light.",
        traits: ["undead"],
      }),
    ).toContain("fortress_setting");

    expect(
      deriveRecordTags({
        name: "Sewer Ooze",
        category: "creature",
        subcategory: null,
        descriptionText:
          "These amorphous masses of sewage and other detritus make their way through filthy culverts beneath cities large and small.",
        traits: ["mindless", "ooze"],
      }),
    ).toContain("urban_setting");

    expect(
      deriveRecordTags({
        name: "Apprentice",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Ambitious apprentices can be found in all cities. These individuals are generally younger and seek the approval of their masters as they learn their craft.",
        traits: ["human", "humanoid"],
      }),
    ).toContain("urban_setting");

    expect(
      deriveRecordTags({
        name: "Archer Sentry",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Archer sentries slightly outrank rank-and-file guards, taking positions on walls, garrisons, and other important locations where they can stay out of the fray and pick off criminals or assailants. Larger societies rely on those with the authority and the ability to interpret and enforce laws.",
        traits: ["human", "humanoid"],
        families: ["official"],
      }),
    ).toContain("urban_setting");

    expect(
      deriveRecordTags({
        name: "Virulak Villager",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A village of commoners raised to undeath by a mass poisoning might continue to go about the settled routines of life, posing an eerie scene for living creatures who enter their village.",
        traits: ["undead"],
      }),
    ).toContain("small_settlement_setting");

    expect(
      deriveRecordTags({
        name: "Virulak Villager",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A village of commoners raised to undeath by a mass poisoning might continue to go about the settled routines of life, posing an eerie scene for living creatures who enter their village.",
        traits: ["undead"],
      }),
    ).not.toContain("rural_setting");

    expect(
      deriveRecordTags({
        name: "Village Reaver",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A cruel marauder raids villages and farms before slipping back into the hills with stolen livestock.",
        traits: ["humanoid"],
      }),
    ).not.toContain("rural_setting");

    expect(
      deriveRecordTags({
        name: "Village Reaver",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A cruel marauder raids villages and farms before slipping back into the hills with stolen livestock.",
        traits: ["humanoid"],
      }),
    ).not.toContain("small_settlement_setting");

    expect(
      deriveRecordTags({
        name: "Swiftrun Clergy",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Swiftrun is a small settlement dedicated to Erastil, barely large enough to be considered a village. Its priests are devoted to community and peace, and would rather spend their days tending their crops and community than shedding blood.",
        traits: ["human", "humanoid"],
      }),
    ).toContain("rural_setting");

    expect(
      deriveRecordTags({
        name: "Swiftrun Clergy",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Swiftrun is a small settlement dedicated to Erastil, barely large enough to be considered a village. Its priests are devoted to community and peace, and would rather spend their days tending their crops and community than shedding blood.",
        traits: ["human", "humanoid"],
      }),
    ).toContain("small_settlement_setting");

    expect(
      deriveRecordTags({
        name: "Grippli Archer",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Frog-like humanoids who make their homes in the treetops of tropical jungles and forests, gripplis are uniquely adapted to their environment. Their oversized eyes give them keen vision in both light and dark, and their large toes allow them to easily scale the trees atop which they reside. Whatever region they come from, gripplis tend to be peaceful hunter-gatherers. The treetop settlements of the grippli are difficult to spot from the forest floor. Gripplis obscure their holdings with broad leaves and thick branches, and they riddle the surrounding forest with labyrinthine trails that only they know how to navigate. Their villages are usually constructed among the densest populations of trees, with thin rope bridges strung between wide wooden platforms built around each trunk.",
        traits: ["grippli", "humanoid"],
      }),
    ).toContain("small_settlement_setting");

    expect(
      deriveRecordTags({
        name: "Tripkee Scout",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Tripkee scouts are the first line of defense for their hidden treetop villages. They are often scattered throughout the forests in small groups to keep an eye out for anything new or dangerous that could pose a threat. Traditionally making their homes in the treetops of tropical jungles and forests, these frog-like humanoids are often seen as resourceful and cautious, preferring to live and hunt hidden in the branches of tall trees.",
        traits: ["grippli", "humanoid"],
      }),
    ).toContain("small_settlement_setting");

    expect(
      deriveRecordTags({
        name: "Smith",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Most smaller communities have at least one smithy where locals and travelers can have horses shod or equipment repaired. Larger settlements and cities often have a variety of smiths, many specializing in blacksmithing, weapon smithing, armor smithing, or even smelting coins in a mint. Expertise is forged through years of effort and often tedious work. Artisans are masters of their craft, able to create works both practical and beautiful.",
        traits: ["human", "humanoid"],
      }),
    ).toContain("small_settlement_setting");

    expect(
      deriveRecordTags({
        name: "Shoony Tiller",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Nearly all shoonies in a given settlement are farmers, fishers, or foragers. Shoonies are not expected to fight to protect their settlements; most agree it is better to live in cowardice than to die with that foolish, intangible principle taller races call honor. Many shoonies hone their skill with tools rather than arms in the knowledge that whatever is lost to violence can be rebuilt.",
        traits: ["humanoid", "shoony"],
      }),
    ).toContain("small_settlement_setting");

    expect(
      deriveRecordTags({
        name: "Tidewater Guard",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Iruxi tidewater guards are capable fighters skilled at amphibious attacks and overpowering vessels along any shore. Because lizardfolk settlements are typically constructed partially underwater and partially above, they have need of defenders who can guard from attacks in both environments. The special spaulders tidewater guards wear set them apart from other lizardfolk warriors.",
        traits: ["amphibious", "humanoid", "iruxi", "lizardfolk"],
      }),
    ).toContain("small_settlement_setting");

    expect(
      deriveRecordTags({
        name: "Chupacabra",
        category: "creature",
        subcategory: null,
        descriptionText:
          "These notorious predators have an undeniable thirst for blood. Chupacabras prefer to prey on the weak and slow, often hiding in wait and watching potential prey for long periods before attacking. Spry and stealthy, they most often make their homes in areas of high grass and protective rock, their slightly reflective scales allowing them to blend in well with such surroundings. Chupacabras prefer to eat lone travelers and farm animals (particularly goats) and leave little evidence of their presence apart from the grisly, blood-drained husks of their meals. Their tendency to stay out of sight combined with their naturally nocturnal activity often leads superstitious locals to conclude the worst, imagining that a particularly reckless vampire lives in the area. Although chupacabras are typically solitary creatures, they have been known to form small gangs in bountiful areas. Members of these groups work well together, growing bold enough to attack larger animals, small herds, and otherwise more dangerous prey. Stories of chupacabras attacking travelers or laying siege to farmhouses typically stem from the hunting practices of such gangs.",
        traits: ["beast"],
      }),
    ).toContain("rural_setting");

    expect(
      deriveRecordTags({
        name: "Prairie Drake",
        category: "creature",
        subcategory: null,
        descriptionText:
          "These squat, mud-brown drakes resemble scaly pit bulls with blunt, toothy snouts. Prairie drakes build and live in burrow mounds just beneath the surface of their environs. They make shallow tunnels in search of large insects, rodents, and ground snakes. Prairie drakes' presence helps to turn the topsoil and encourage new plant growth, supporting other animal life and making them a keystone species for the environment. Even their breath weapon leaves behind rich drake soil, coveted by farmers and gardeners for the potent effects it has on plants. After they've gotten too much positive attention, a prairie drake is apt to murder a farm animal or destroy croplands only to prove that they're far from harmless.",
        traits: ["dragon", "earth", "evil"],
      }),
    ).toContain("rural_setting");

    expect(
      deriveRecordTags({
        name: "Kvernknurr",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Kvernknurrs are lanky fey giants that dwell in cold-water rivers and lakes. A kvernknurr's blue flesh allows it to camouflage itself in its preferred environ, where it haunts local millers and farmers by sabotaging water wheels and farm equipment. Kvernknurrs despise humanoid-made disturbances to the waters in which they live, even seemingly harmless ones like waterwheels. When townsfolk come to investigate their jammed mill, the kvernknurr then springs out from hiding to attack, roaring terribly before snatching a victim to eat whole.",
        traits: ["amphibious", "fey", "water"],
      }),
    ).toContain("rural_setting");

    expect(
      deriveRecordTags({
        name: "Kvernknurr",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Kvernknurrs are lanky fey giants that dwell in cold-water rivers and lakes. A kvernknurr's blue flesh allows it to camouflage itself in its preferred environ, where it haunts local millers and farmers by sabotaging water wheels and farm equipment. Kvernknurrs despise humanoid-made disturbances to the waters in which they live, even seemingly harmless ones like waterwheels. When townsfolk come to investigate their jammed mill, the kvernknurr then springs out from hiding to attack, roaring terribly before snatching a victim to eat whole.",
        traits: ["amphibious", "fey", "water"],
      }),
    ).not.toContain("small_settlement_setting");

    expect(
      deriveRecordTags({
        name: "Sage",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The greatest knowledge comes from experience. Village elders, ancient seers, and advisors to royalty are examples of those valued for such wisdom. Sages educate and guide their people from straying from their cultures' norms and traditions.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("small_settlement_setting");

    expect(
      deriveRecordTags({
        name: "Yeth Hound",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Wicked canine creatures who live for the thrill of the hunt, yeth hounds often serve evil masters as guardians and trackers. Their eerie bays echo across the countryside when they are engaged in a hunt, and they particularly enjoy baying to frighten and disorient intelligent creatures. Yeth hound packs can number as many as a dozen members, each working in uncanny communion with its packmates to corner and kill their prey. Yeth hounds like to drag their victims back to their lairs to eat at their leisure, so these lairs often contain discarded treasures from the hounds' previous meals.",
        traits: ["beast", "chaotic", "evil", "fiend", "unholy"],
      }),
    ).toContain("rural_setting");

    expect(
      deriveRecordTags({
        name: "Bandersnatch",
        category: "creature",
        subcategory: null,
        descriptionText:
          "As with other legendary creatures from the First World, bandersnatches belong to the infamous group of creatures known collectively as the tane.",
        traits: ["beast", "tane"],
      }),
    ).toContain("first_world_setting");

    expect(
      deriveRecordTags({
        name: "Catrina",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Catrinas meet souls in the Boneyard and help convince them of the finality of their fate to ease a spirit's passing.",
        traits: ["monitor", "psychopomp"],
      }),
    ).toContain("boneyard_setting");

    expect(
      deriveRecordTags({
        name: "Shade (Heaven)",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The elect have golden halos and ghostly wings, but they otherwise appear as their mortal forms.",
        traits: ["fiend", "shade", "unholy"],
      }),
    ).toEqual(expect.arrayContaining(["heaven_setting", "upper_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Shade (Outer Rifts)",
        category: "creature",
        subcategory: null,
        descriptionText: "The larvae appear as maggot-like grubs with the face the shade had in life.",
        traits: ["fiend", "shade", "unholy"],
      }),
    ).toEqual(expect.arrayContaining(["abyss_setting", "lower_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Requiem Dragon (Adult)",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Requiem dragons are stewards of the River of Souls and the process through which souls reach their final destination in the afterlife. Some follow individual souls from their first entry into the river through to their judgment in the Boneyard.",
        traits: ["divine", "dragon"],
      }),
    ).toContain("boneyard_setting");

    expect(
      deriveRecordTags({
        name: "Phantom Beast",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The typical trajectory for souls passing to the afterlife is fairly straightforward, according to most theologians. When a mortal dies, their soul enters the River of Souls and eventually reaches the Boneyard, where it is judged by Pharasma. Complications arise, however, when a soul in queue for judgment prematurely departs from the River of Souls and is shunted into the Ethereal Plane.",
        traits: ["ethereal", "incorporeal", "phantom", "spirit"],
      }),
    ).not.toContain("boneyard_setting");

    expect(
      deriveRecordTags({
        name: "Hunter Wight",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Hunter wights renew their hunt with equal vigor and frequently take residence within abandoned watchtowers and keeps.",
        traits: ["undead"],
      }),
    ).toContain("fortress_setting");

    expect(
      deriveRecordTags({
        name: "Watchtower Poltergeist",
        category: "creature",
        subcategory: null,
        descriptionText: "A restless spirit batters shutters and hurls loose stones at trespassers.",
        traits: ["incorporeal", "spirit", "undead"],
      }),
    ).toContain("fortress_setting");

    expect(
      deriveRecordTags({
        name: "Ahvothian",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Ahvothians are feral and cunning fiends from an Abyssal realm of jungles, dinosaurs, and relentless brutality. Relentless trackers and survivalists, ahvothians are most commonly encountered stalking prey or operating out of fortified ruins.",
        traits: ["div", "fiend", "unholy"],
      }),
    ).toEqual(expect.arrayContaining(["fortress_setting", "ruins_setting"]));

    expect(
      deriveRecordTags({
        name: "Red Dragon (Adult)",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Their lairs are often situated in dangerous places, with volcanoes being a favorite spot, as they find them foreboding and the constant warmth is comfortable.",
        traits: ["chaotic", "dragon", "evil", "fire"],
      }),
    ).toContain("volcanic_setting");

    expect(
      deriveRecordTags({
        name: "Underworld Dragon (Adult)",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Today, though their nature is primarily still connected with the element of fire, an underworld dragon can consume wood with ease, lay claim to the minerals and gems in the rocks and stone, and die to form volcanoes that create more land masses.",
        traits: ["arcane", "dragon", "fire"],
      }),
    ).toContain("volcanic_setting");

    expect(
      deriveRecordTags({
        name: "Scaleseed Nagaji",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Although nagaji might be encountered in diverse cities and urban centers, their communities are concentrated in environments that suit their biology, namely jungles and tropical forests.",
        traits: ["humanoid", "nagaji"],
      }),
    ).toEqual(expect.arrayContaining(["forest_setting", "jungle_setting"]));
  });

  it("expands explicit creature setting cohorts conservatively", () => {
    expect(
      deriveRecordTags({
        name: "Sky Dragon (Adult, Spellcaster)",
        category: "creature",
        subcategory: null,
        descriptionText:
          "These dragons flew from mountaintop to mountaintop, seeking refuge from the then-perilous Tian Xia. Finally, upon the mountain peaks of Chenlun, Gossamer, Kelsang, Kimu, Kullan, Kyojin, and the Wall of Heaven, celestial beings answered the dragons' cries for help.",
        traits: ["divine", "dragon", "electricity", "metal"],
      }),
    ).toEqual(expect.arrayContaining(["sky_setting", "mountain_setting"]));

    expect(
      deriveRecordTags({
        name: "Stormcrown Dragon (Adult, Spellcaster)",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Driven by emotion and curiosity, these mercurial dragons can change from friend to foe or threatening to thoughtful in mere moments.",
        traits: ["dragon", "electricity", "primal"],
      }),
    ).toContain("sky_setting");

    expect(
      deriveRecordTags({
        name: "White Dragon (Adult)",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Dwelling on glacial mountaintops or in ice caverns beneath forbidding tundra, they treat the lands around them as their own personal hunting grounds.",
        traits: ["chaotic", "cold", "dragon", "evil"],
      }),
    ).toContain("mountain_setting");

    expect(
      deriveRecordTags({
        name: "Venexus's Wyrmling",
        category: "creature",
        subcategory: null,
        descriptionText: null,
        traits: ["chaotic", "cold", "dragon", "evil"],
      }),
    ).toContain("mountain_setting");

    expect(
      deriveRecordTags({
        name: "Venexus's Chosen",
        category: "creature",
        subcategory: null,
        descriptionText: "Dragonkin monks",
        traits: ["beast", "beastkin", "evil", "humanoid", "lawful"],
      }),
    ).toContain("mountain_setting");

    expect(
      deriveRecordTags({
        name: "Umbral Dragon (Adult, Spellcaster)",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Stories across Golarion warn children about the monsters that dwell in the shadows, but no story captures the reality of the umbral dragon.",
        traits: ["dragon", "occult", "shadow"],
      }),
    ).toContain("shadow_plane_setting");

    expect(
      deriveRecordTags({
        name: "Umbral Archdragon (Spellcaster)",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Natives of the Netherworld, these occult dragons are at home in the darkness, using it to their advantage both in and out of battle.",
        traits: ["dragon", "occult", "shadow"],
      }),
    ).toContain("shadow_plane_setting");

    expect(
      deriveRecordTags({
        name: "Fetchling Scout",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Fetchling scouts patrol the outskirts of their communities, looking for any possible threats.",
        traits: ["fetchling", "humanoid", "shadow"],
      }),
    ).toContain("shadow_plane_setting");

    expect(
      deriveRecordTags({
        name: "Caligni Dancer",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Caligni dancers serve as intermediaries between caligni factions, carrying messages and negotiating deals between the notoriously independent groups.",
        traits: ["caligni", "humanoid"],
      }),
    ).toContain("shadow_plane_setting");

    expect(
      deriveRecordTags({
        name: "D'ziriak",
        category: "creature",
        subcategory: null,
        descriptionText:
          "These strange creatures are native to the Netherworld, where their colorful nature is in opposition to that realm's overwhelmingly monochromatic palette.",
        traits: ["aberration", "shadow"],
      }),
    ).toContain("shadow_plane_setting");

    expect(
      deriveRecordTags({
        name: "Wailing Dragon (Adult, Spellcaster)",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Wailing dragons haunt locations that create interesting or useful echoes, such as canyons or caverns.",
        traits: ["arcane", "dragon"],
      }),
    ).toEqual(expect.arrayContaining(["canyon_setting", "underground_setting"]));

    expect(
      deriveRecordTags({
        name: "Azarketi Sailor",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Such sailors get to enjoy their time above deck while also having near-constant access to large bodies of water.",
        traits: ["amphibious", "azarketi", "chaotic", "humanoid"],
      }),
    ).toContain("aquatic_setting");
  });

  it("uses glossary family evidence and blocks redundant civic npc tags", () => {
    expect(
      deriveRecordTags({
        name: "Morlock Thrall",
        category: "creature",
        subcategory: null,
        descriptionText: "A thrall reshaped by a vampire master's curse.",
        traits: ["humanoid"],
        families: ["vampire"],
      }),
    ).toContain("undead_adjacent");

    expect(
      deriveRecordTags({
        name: "Manor Guard",
        category: "creature",
        subcategory: null,
        descriptionText: "A manor guard who patrols the estate grounds.",
        traits: ["human", "humanoid"],
        families: ["vampire"],
      }),
    ).toEqual(expect.arrayContaining(["profession_npc", "undead_adjacent"]));

    expect(
      deriveRecordTags({
        name: "Manor Guard",
        category: "creature",
        subcategory: null,
        descriptionText: "A manor guard who patrols the estate grounds.",
        traits: ["human", "humanoid"],
        families: ["vampire"],
      }),
    ).not.toContain("civic_npc");

    expect(
      deriveRecordTags({
        name: "Mythic Courtier",
        category: "creature",
        subcategory: null,
        descriptionText: "A courtier sustained by impossible necromancy.",
        traits: ["humanoid"],
        families: ["mythic", "lich"],
      }),
    ).toEqual(expect.arrayContaining(["profession_npc", "undead_adjacent"]));

    expect(
      deriveRecordTags({
        name: "Mythic Courtier",
        category: "creature",
        subcategory: null,
        descriptionText: "A courtier sustained by impossible necromancy.",
        traits: ["humanoid"],
        families: ["mythic", "lich"],
      }),
    ).not.toContain("civic_npc");

    expect(
      deriveRecordTags({
        name: "Bandit",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Bandits waylay travelers and plunder their valuables before disappearing back to their wilderness hideouts.",
        traits: ["human", "humanoid"],
      }),
    ).toContain("enforcer_npc");

    expect(
      deriveRecordTags({
        name: "Watch Officer",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Often leading a small team of lower-ranking guards, watch officers patrol their assigned areas to maintain order and enforce laws.",
        traits: ["human", "humanoid"],
      }),
    ).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(
      deriveRecordTags({
        name: "Abbot of Abadar",
        category: "creature",
        subcategory: null,
        descriptionText: "The abbot runs the shrine and serves the community with counsel and order.",
        traits: ["human", "humanoid"],
      }),
    ).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(
      deriveRecordTags({
        name: "Priest of Pharasma",
        category: "creature",
        subcategory: null,
        descriptionText: "Cloistered priests safeguard their temples and communities.",
        traits: ["human", "humanoid"],
      }),
    ).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(
      deriveRecordTags({
        name: "High Priest of Pharasma",
        category: "creature",
        subcategory: null,
        descriptionText:
          "High priests are the leaders of larger churches and similar religious establishments, watching over the lower-ranking clergy and ensuring the surrounding community is taken care of.",
        traits: ["human", "humanoid"],
      }),
    ).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(
      deriveRecordTags({
        name: "Traveling Apothecary",
        category: "creature",
        subcategory: null,
        descriptionText: "A respected apothecary and physician who serves the community by treating the sick.",
        traits: ["human", "humanoid"],
      }),
    ).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(
      deriveRecordTags({
        name: "Barkeep",
        category: "creature",
        subcategory: null,
        descriptionText: "A barkeep keeps the tavern open as a dependable gathering place for the neighborhood.",
        traits: ["human", "humanoid"],
      }),
    ).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(
      deriveRecordTags({
        name: "Adjutant Hellknight Armiger",
        category: "creature",
        subcategory: null,
        descriptionText: "An adjutant and armiger who fights alongside the order in disciplined battle formations.",
        traits: ["human", "humanoid"],
      }),
    ).toContain("enforcer_npc");

    expect(
      deriveRecordTags({
        name: "Hellknight Sergeant",
        category: "creature",
        subcategory: null,
        descriptionText: "The sergeant leads disciplined enforcers in brutal close-quarters battle.",
        traits: ["human", "humanoid"],
      }),
    ).toContain("enforcer_npc");

    expect(
      deriveRecordTags({
        name: "City Scribe",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A human clerk who serves the community and keeps the peace while recording the daily business of the courthouse.",
        traits: ["human", "humanoid"],
      }),
    ).toContain("profession_npc");

    expect(
      deriveRecordTags({
        name: "Hryngar Forgepriest",
        category: "creature",
        subcategory: null,
        descriptionText: "The forgepriest proselytizes Droskar's teachings and punishes heretics for defiance.",
        traits: ["dwarf", "humanoid"],
      }),
    ).toContain("profession_npc");

    expect(
      deriveRecordTags({
        name: "Tree Singer",
        category: "creature",
        subcategory: null,
        descriptionText: "A humanoid primalist singer whose melodies stir trees and command plants.",
        traits: ["humanoid"],
      }),
    ).toContain("profession_npc");

    expect(
      deriveRecordTags({
        name: "Guild Engineer",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A meticulous guild engineer keeps the city's lifts and aqueduct pumps working for the community.",
        traits: ["human", "humanoid"],
      }),
    ).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(
      deriveRecordTags({
        name: "Festival Lutenist",
        category: "creature",
        subcategory: null,
        descriptionText: "A celebrated lutenist performs at civic festivals and public ceremonies.",
        traits: ["human", "humanoid"],
      }),
    ).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(
      deriveRecordTags({
        name: "Hellknight Gaoler",
        category: "creature",
        subcategory: null,
        descriptionText: "A disciplined hellknight gaoler escorts prisoners and enforces the order's brutal routines.",
        traits: ["human", "humanoid"],
      }),
    ).toEqual(expect.arrayContaining(["profession_npc", "enforcer_npc"]));

    expect(
      deriveRecordTags({
        name: "Hellknight Gaoler",
        category: "creature",
        subcategory: null,
        descriptionText: "A disciplined hellknight gaoler escorts prisoners and enforces the order's brutal routines.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("civic_npc");

    expect(
      deriveRecordTags({
        name: "Veteran Noble",
        category: "creature",
        subcategory: null,
        descriptionText: "A veteran noble funds expeditions and stewards the city's public works.",
        traits: ["human", "humanoid"],
      }),
    ).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(
      deriveRecordTags({
        name: "Veteran Noble",
        category: "creature",
        subcategory: null,
        descriptionText: "A veteran noble funds expeditions and stewards the city's public works.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("enforcer_npc");

    expect(
      deriveRecordTags({
        name: "Gendarme",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Powerful governments retain gendarmes to guard important magistrates and capture unusually dangerous criminals.",
        traits: ["human", "humanoid"],
      }),
    ).toContain("enforcer_npc");

    expect(
      deriveRecordTags({
        name: "Line Infantry",
        category: "creature",
        subcategory: null,
        descriptionText: "Disciplined line infantry hold the battle line in ordered formations.",
        traits: ["human", "humanoid"],
      }),
    ).toContain("enforcer_npc");

    expect(
      deriveRecordTags({
        name: "Unnamed Traveler",
        category: "creature",
        subcategory: null,
        descriptionText: "A traveler passes through town in search of lodging.",
        traits: ["human", "humanoid"],
      }),
    ).not.toEqual(expect.arrayContaining(["profession_npc", "civic_npc", "enforcer_npc"]));

    expect(
      deriveRecordTags({
        name: "Unnamed Traveler",
        category: "creature",
        subcategory: null,
        descriptionText: "A traveler passes through town in search of lodging.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("small_settlement_setting");

    expect(
      deriveRecordTags({
        name: "False Priest",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Belief is perhaps the strongest force in the universe. Instilling belief only to use it against someone in deceit, however, is the purview of a false priest.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("civic_npc");

    expect(
      deriveRecordTags({
        name: "Anadi Hunter",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Anadi hunters act as the eyes and ears of their clans, scouting the surrounding area for potential invaders.",
        traits: ["anadi", "chaotic", "good", "humanoid"],
      }),
    ).not.toContain("civic_npc");

    expect(
      deriveRecordTags({
        name: "Urdefhan Death Scout",
        category: "creature",
        subcategory: null,
        descriptionText:
          "An urdefhan death scout ranges ahead of the war band, stalking prey and reporting enemy positions back to the hunt leaders.",
        traits: ["evil", "humanoid", "urdefhan"],
      }),
    ).not.toContain("civic_npc");

    expect(
      deriveRecordTags({
        name: "Hateful Logger",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A logger consumed by spite hacks through the forest and lashes out at anyone who enters the claim.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("civic_npc");

    expect(
      deriveRecordTags({
        name: "Traveling Priest of Desna",
        category: "creature",
        subcategory: null,
        descriptionText: "Deities and their religions are only as strong as the belief of their faithful.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("civic_npc");
  });

  it("extends creature setting coverage for approved swamp, dreamlands, hell, and nautical anchors", () => {
    expect(
      deriveRecordTags({
        name: "Bog Mummy",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Bog mummies (also called peat mummies or mire mummies) rarely, if ever, leave their marshy realms. Less powerful than their more notorious artificially preserved kin, bog mummies are preserved not by agents introduced during rituals but by the natural elements present in the airless, acidic morass of a peat bog or muddy swamp.",
        traits: ["undead", "mummy", "unholy"],
      }),
    ).toContain("swamp_setting");

    expect(
      deriveRecordTags({
        name: "Witchfire Warden",
        category: "creature",
        subcategory: null,
        descriptionText: "A witchfire often resides in a bog or swamp, which slowly blackens and decays around them.",
        traits: ["incorporeal", "spirit", "undead", "unholy"],
      }),
    ).toContain("swamp_setting");

    expect(
      deriveRecordTags({
        name: "Swamp Blight",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A swamp blight appears as a quivering blob of rancid brown and green mud from which dozens of hateful red eyes peer. While a swamp blight can dominate undead in its cursed domain, it prefers to gather and control undead that would prosper in a swampy environment and the broader swampland environment around it.",
        traits: ["blight", "ooze"],
      }),
    ).toContain("swamp_setting");

    expect(
      deriveRecordTags({
        name: "Dread Wisp",
        category: "creature",
        subcategory: null,
        descriptionText:
          "While dread wisps can survive anywhere within the Darklands, they prefer moist areas like partially submerged caves, the banks of underground rivers, and towering fungus gardens.",
        traits: ["aberration"],
      }),
    ).not.toContain("swamp_setting");

    expect(
      deriveRecordTags({
        name: "Leng Spider",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The monstrous, bloated spiders from the windswept realm of Leng build eerie, dangerous lairs with the aid of magically compelled slaves.",
        traits: ["aberration"],
      }),
    ).toContain("dreamlands_setting");

    expect(
      deriveRecordTags({
        name: "Dreamscraper",
        category: "creature",
        subcategory: null,
        descriptionText:
          "After returning to the Dimension of Dreams with their prizes, dreamscrapers store them in their dark, spire-like cocoons, trade them, or offer them up to their heinous masters as some obscure tax.",
        traits: ["dream", "aberration"],
      }),
    ).toContain("dreamlands_setting");

    expect(
      deriveRecordTags({
        name: "Night Hag",
        category: "creature",
        subcategory: null,
        descriptionText:
          "They haunt the Ethereal Plane, where they prey upon mortals in their dreams, debilitating them with horrific nightmares as they rest.",
        traits: ["fiend", "hag", "evil"],
      }),
    ).not.toContain("dreamlands_setting");

    expect(
      deriveRecordTags({
        name: "Night Hag",
        category: "creature",
        subcategory: null,
        descriptionText:
          "They haunt the Ethereal Plane, where they prey upon mortals in their dreams, debilitating them with horrific nightmares as they rest.",
        traits: ["fiend", "hag", "evil"],
      }),
    ).toContain("ethereal_setting");

    expect(
      deriveRecordTags({
        name: "Diabolic Dragon (Adult)",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Hell, according to some theologians, is a living entity in and of itself. Diabolic dragons are powerful, cunning, and tyrannical extensions of the plane, living creatures that break off from Hell to enact its will.",
        traits: ["dragon"],
      }),
    ).toEqual(expect.arrayContaining(["hell_setting", "lower_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Hellcat",
        category: "creature",
        subcategory: null,
        descriptionText: "Hellcats are devious predators native to the fiery pits of Hell.",
        traits: ["fiend", "beast", "unholy"],
      }),
    ).toEqual(expect.arrayContaining(["hell_setting", "lower_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Hellknight Paravicar",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Paravicars use their considerable infernal powers to reinforce their positions in the Hellknight hierarchy.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("hell_setting");

    expect(
      deriveRecordTags({
        name: "Sunflower Leshy",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A human artist might illustrate Hell as a place of glowing lava and flickering flames, but to a sunflower leshy the only proper way to depict such a place is in morose grays, blacks, and whites.",
        traits: ["leshy", "plant"],
      }),
    ).not.toContain("hell_setting");

    expect(
      deriveRecordTags({
        name: "Tehialai-Thief-of-Ships",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The titanic crustacean wears the hulls of ruined ships as her shell, ripped into shape by vessel-rending claws. Few ships can withstand her might, and those who know Tehialai offer her tribute to ward off her predation.",
        traits: ["beast", "water"],
      }),
    ).toContain("nautical_setting");

    expect(
      deriveRecordTags({
        name: "Artillerist",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Artillerists are often employed on ships to manage their cannons and harpoons, but their primary role is the maintenance and operation of siege weapons.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("nautical_setting");

    expect(
      deriveRecordTags({
        name: "Dockhand",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Working to load and unload cargo from ships, dockhands are considered unruly, but many stay focused and work hard until the job is done.",
        traits: ["human", "humanoid"],
      }),
    ).toContain("nautical_setting");

    expect(
      deriveRecordTags({
        name: "Bosun",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A ship's boatswain, or bosun, leads the deckhands who maintain the ship and oversees shipboard labor and discipline.",
        traits: ["human", "humanoid"],
      }),
    ).toContain("nautical_setting");
  });

  it("derives elemental plane setting tags conservatively", () => {
    expect(
      deriveRecordTags({
        name: "Ifrit",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The fierce and unforgiving ifrits hail from the Plane of Fire, where they build metropolises and trade centers that draw extraplanar travelers.",
        traits: ["elemental", "fire", "genie"],
      }),
    ).toEqual(expect.arrayContaining(["plane_of_fire_setting", "elemental_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Elemental Hurricane",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Elemental hurricanes embody the ferocity of violent windstorms. Hailing from the Plane of Air, these beings appear in a variety of sizes and shapes.",
        traits: ["air", "elemental"],
      }),
    ).toEqual(expect.arrayContaining(["plane_of_air_setting", "elemental_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Brine Shark",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Brine sharks are deadly elementals that roam the endless oceans of the Plane of Water. They often slip into mortal oceans as well.",
        traits: ["aquatic", "elemental", "water"],
      }),
    ).toEqual(expect.arrayContaining(["plane_of_water_setting", "elemental_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Carnivorous Crystal",
        category: "creature",
        subcategory: null,
        descriptionText: "Carnivorous crystals are strange ooze creatures native to the Plane of Earth.",
        traits: ["earth", "mindless", "ooze"],
      }),
    ).toEqual(expect.arrayContaining(["plane_of_earth_setting", "elemental_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Cinder Tyrant",
        category: "creature",
        subcategory: null,
        descriptionText: "A blazing tyrant crowned in smoke and cinders rises from a volcanic caldera.",
        traits: ["dragon", "fire"],
      }),
    ).not.toContain("plane_of_fire_setting");

    expect(
      deriveRecordTags({
        name: "Melody on the Wind",
        category: "creature",
        subcategory: null,
        descriptionText:
          "While the melody on the wind, known by some as a song elemental, might enjoy the beauty of music, it is by nature a destructive elemental force. Some elementals embody aspects of air, such as smoke, lightning, and fog.",
        traits: ["air", "elemental"],
      }),
    ).toEqual(expect.arrayContaining(["plane_of_air_setting", "elemental_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Sky Fisher",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Naturalists believe the sky fisher evolved into a new ecological niche through prolonged exposure to elemental energies from the Plane of Air.",
        traits: ["animal"],
      }),
    ).not.toContain("plane_of_air_setting");

    expect(
      deriveRecordTags({
        name: "Adamantine Golem",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Crafting an adamantine golem requires a quantity of adamantine so massive that collecting it usually requires mounting a mining expedition to a distant planet, the Plane of Earth, or an Outer Plane.",
        traits: ["construct", "golem", "mindless"],
      }),
    ).not.toContain("plane_of_earth_setting");
  });

  it("derives creature motif tags without collapsing into raw vibes", () => {
    expect(
      deriveRecordTags({
        name: "Court Jester",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Though court jesters are often mocked for easy amusement, this jester hides malice behind painted smiles and cutting wit.",
        traits: ["human", "humanoid"],
      }),
    ).toContain("carnival_show");

    expect(
      deriveRecordTags({
        name: "Mechanical Carny",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Mechanical carnies are constructs manufactured to serve as entertainers, cleaners, and guards at carnivals and circuses.",
        traits: ["construct"],
      }),
    ).toContain("carnival_show");

    expect(
      deriveRecordTags({
        name: "Soulbound Doll",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Soulbound dolls are eerie mannequins or playthings that have been imbued with a small piece of a deceased mortal's soul.",
        traits: ["construct"],
      }),
    ).toContain("living_toy");

    expect(
      deriveRecordTags({
        name: "Masque Mannequin",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Masque mannequins are soulbound constructs animated by a fragment of a once-living soul infused into a mannequin or dressmaker's dummy.",
        traits: ["construct"],
      }),
    ).toContain("living_toy");

    expect(
      deriveRecordTags({
        name: "Fire Scamp",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Although arguably quite friendly, fire scamps delight in fire and playing pranks on everyone they befriend.",
        traits: ["elemental", "fire"],
      }),
    ).toContain("trickster_mischief");

    expect(
      deriveRecordTags({
        name: "Living Mural",
        category: "creature",
        subcategory: null,
        descriptionText:
          "This two-dimensional mural has come to life as a mindless construct that peels itself from the wall to attack intruders.",
        traits: ["construct", "mindless"],
      }),
    ).toContain("living_artwork");

    expect(
      deriveRecordTags({
        name: "Masked Mourner",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A solemn creature wearing a ceremonial mask and veiled face to hide its identity from the living.",
        traits: ["humanoid"],
      }),
    ).toContain("mask_motif");

    expect(
      deriveRecordTags({
        name: "Faceless Butcher",
        category: "creature",
        subcategory: null,
        descriptionText: "This faceless horror has a blank, featureless face and keeps stolen faces as trophies.",
        traits: ["aberration"],
      }),
    ).toContain("faceless_horror");

    expect(
      deriveRecordTags({
        name: "False Herald",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The herald assumes a false identity, infiltrates courts, and impersonates priests to replace them.",
        traits: ["humanoid"],
      }),
    ).toContain("disguised_pretender");

    expect(
      deriveRecordTags({
        name: "Brass Dragon",
        category: "creature",
        subcategory: null,
        descriptionText: "Brass dragons are whimsical tricksters who delight in humor and play.",
        traits: ["chaotic", "dragon", "fire"],
      }),
    ).toContain("trickster_mischief");

    expect(
      deriveRecordTags({
        name: "Brass Dragon",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Most brass dragons live in desert climates, and while they keep their lairs hidden, they often build near humanoid settlements.",
        traits: ["chaotic", "dragon", "fire"],
      }),
    ).not.toContain("urban_setting");

    expect(
      deriveRecordTags({
        name: "Bog Archdragon",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The bogs of Golarion are timeless places, unyielding against the encroachment of civilization. Though most have been relegated to the stuff of legends and the mythic nightmares of river valley city-states, bog dragons are still out there, lurking beneath the loam.",
        traits: ["amphibious", "dragon", "primal"],
      }),
    ).not.toContain("urban_setting");

    expect(
      deriveRecordTags({
        name: "Dancer",
        category: "creature",
        subcategory: null,
        descriptionText: "A nimble performer who entertains nobles at court.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("carnival_show");

    expect(
      deriveRecordTags({
        name: "Badger",
        category: "creature",
        subcategory: null,
        descriptionText: "A badger with dark facial markings and a striped muzzle.",
        traits: ["animal"],
      }),
    ).not.toContain("mask_motif");

    expect(
      deriveRecordTags({
        name: "Masked Duelist",
        category: "creature",
        subcategory: null,
        descriptionText: "A duelist wearing a bronze mask over a healthy and recognizable face.",
        traits: ["humanoid"],
      }),
    ).not.toContain("faceless_horror");

    expect(
      deriveRecordTags({
        name: "Taljjae",
        category: "creature",
        subcategory: null,
        descriptionText: "Taljjae is easily recognized due to its signature cloak and masks.",
        traits: ["fey"],
      }),
    ).toContain("mask_motif");

    expect(
      deriveRecordTags({
        name: "The Vanish Man",
        category: "creature",
        subcategory: null,
        descriptionText: "Variant faceless butcher.",
        traits: ["humanoid"],
      }),
    ).toContain("faceless_horror");

    expect(
      deriveRecordTags({
        name: "Chameleon Beast",
        category: "creature",
        subcategory: null,
        descriptionText: "A reptile that changes color to blend into its surroundings.",
        traits: ["animal"],
      }),
    ).not.toContain("disguised_pretender");

    expect(
      deriveRecordTags({
        name: "Goblin Igniter",
        category: "creature",
        subcategory: null,
        descriptionText: "Goblins think fire is a fun toy and admire anyone willing to burn down a barn for sport.",
        traits: ["goblin", "humanoid"],
      }),
    ).not.toContain("living_toy");

    expect(
      deriveRecordTags({
        name: "Blood Painter",
        category: "creature",
        subcategory: null,
        descriptionText:
          "An alien artist stalks battlefields in search of pigments and living canvases for its gruesome paintings.",
        traits: ["aberration"],
      }),
    ).not.toContain("living_artwork");

    expect(
      deriveRecordTags({
        name: "Animated Panoply",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Granted a semblance of life through the use of rituals or other strange magic, animated objects take many forms and serve as guardians.",
        traits: ["construct", "mindless"],
      }),
    ).not.toContain("living_artwork");

    expect(
      deriveRecordTags({
        name: "Chaos Reaver",
        category: "creature",
        subcategory: null,
        descriptionText: "A chaotic fiend that leaves ruin in its wake.",
        traits: ["chaotic", "fiend"],
      }),
    ).not.toContain("trickster_mischief");
  });

  it("avoids known creature false positives and requires enough weighted evidence", () => {
    expect(
      deriveRecordTags({
        name: "Accuser Agent",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Accuser agents might be high court advocates, official spymasters, or innocuous adjutants delivering important messages to magistrates, generals, officers, or mercenaries.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("arctic_setting");

    expect(
      deriveRecordTags({
        name: "Abandoned Zealot",
        category: "creature",
        subcategory: null,
        descriptionText: "Abandoned zealots arise from false faiths unknown to most worshippers.",
        traits: ["undead", "spirit"],
      }),
    ).not.toContain("nautical_setting");

    expect(
      deriveRecordTags({
        name: "Castruccio Irovetti",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Since his flight from Numeria, Castruccio Irovetti has ruled Pitax for years and remains a major player in this River Kingdom's political scene.",
        traits: ["human", "humanoid"],
      }),
    ).not.toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(
      deriveRecordTags({
        name: "Apothecary Bee",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Long-abandoned gardens still grow along the Sphinx River in Osirion, where many apothecary bees prowl for flowers that meet their standards.",
        traits: ["animal"],
      }),
    ).not.toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(
      deriveRecordTags({
        name: "Astradaemon",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Astradaemons hunt the pathways between life and death and stalk the banks of the River of Souls in the Astral Plane.",
        traits: ["daemon", "fiend", "unholy"],
      }),
    ).not.toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(
      deriveRecordTags({
        name: "Sakugami",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Kami are divine nature spirits from the lands of Tian Xia, far to the east of the Inner Sea region.",
        traits: ["kami", "spirit", "wood"],
      }),
    ).not.toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting", "undead_adjacent"]));

    expect(
      deriveRecordTags({
        name: "Blossom Kami",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The site of a new village might be chosen due to its proximity to an ancient wisteria, or a temple might be carefully constructed around a single young plum.",
        traits: ["kami", "spirit", "wood"],
      }),
    ).not.toEqual(
      expect.arrayContaining([
        "living_artwork",
        "mountain_setting",
        "rural_setting",
        "spawn_creator",
        "undead_adjacent",
      ]),
    );

    expect(
      deriveRecordTags({
        name: "Bleachling Survivor",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Those few gnomes who survive the Bleaching emerge transformed, forever marked by the primal spark of the First World that still flickers within them.",
        traits: ["gnome", "humanoid"],
      }),
    ).not.toContain("first_world_setting");

    expect(
      deriveRecordTags({
        name: "Shade (Astral Plane)",
        category: "creature",
        subcategory: null,
        descriptionText:
          "When a mortal dies, their soul travels to the Boneyard in the Outer Planes where they're judged by Pharasma, though these shades merely appear as astrally projected versions of their mortal forms.",
        traits: ["astral", "shade"],
      }),
    ).not.toContain("boneyard_setting");

    expect(
      deriveRecordTags({
        name: "Phantom Knight",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Complications arise when a soul in queue for judgment prematurely departs from the River of Souls and is shunted into the Ethereal Plane.",
        traits: ["spirit"],
      }),
    ).not.toContain("ethereal_setting");

    expect(
      deriveRecordTags({
        name: "Shrine Caretaker",
        category: "creature",
        subcategory: null,
        descriptionText:
          "This combusted haunts a shrine and sometimes throws itself into lakes or rivers, believing the water will quiet the flames consuming it.",
        traits: ["spirit", "undead", "unholy"],
      }),
    ).not.toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

    expect(
      deriveRecordTags({
        name: "Hooktongue",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Legendary creatures lurking in remote lakes, water orms often spy upon the shores of their lakes and surface near the beach when curiosity overtakes caution.",
        traits: ["beast", "water"],
      }),
    ).not.toContain("coastal_setting");

    expect(
      deriveRecordTags({
        name: "Old Herok",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Old Herok is a water orm that hides in deep lakes, watching the shores of its inland domain and surfacing near lonely beaches only when it must.",
        traits: ["beast", "water"],
      }),
    ).not.toContain("coastal_setting");

    expect(
      deriveRecordTags({
        name: "Sky Canyon Balladeer",
        category: "creature",
        subcategory: null,
        descriptionText: "A performer sings a ballad titled Canyon Echoes beneath an open sky.",
        traits: ["humanoid"],
      }),
    ).not.toEqual(expect.arrayContaining(["canyon_setting", "wasteland_setting"]));

    expect(
      deriveRecordTags({
        name: "Adamantine Golem",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Crafting an adamantine golem requires mounting a mining expedition while guardian suits stand watch.",
        traits: ["construct", "golem", "mindless"],
      }),
    ).not.toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(
      deriveRecordTags({
        name: "Animated Armor",
        category: "creature",
        subcategory: null,
        descriptionText: "Animated armor serves as guardians and training partners in martial academies.",
        traits: ["construct", "mindless"],
      }),
    ).not.toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(
      deriveRecordTags({
        name: "Vanth Guardian Flock",
        category: "creature",
        subcategory: null,
        descriptionText: "Vanth psychopomps are eternal guardians of the cycle of life and death.",
        traits: ["monitor", "psychopomp", "troop"],
      }),
    ).not.toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(
      deriveRecordTags({
        name: "Clockwork Soldier",
        category: "creature",
        subcategory: null,
        descriptionText: "These diligent machines guard their assigned posts tirelessly.",
        traits: ["clockwork", "construct", "mindless"],
      }),
    ).not.toContain("enforcer_npc");

    expect(
      deriveRecordTags({
        name: "Harbor Watcher",
        category: "creature",
        subcategory: null,
        descriptionText: "A sentry posted near the harbor gates.",
        traits: [],
      }),
    ).not.toContain("nautical_setting");

    expect(
      deriveRecordTags({
        name: "Harbor Mariner",
        category: "creature",
        subcategory: null,
        descriptionText: "A mariner who keeps watch over the harbor docks.",
        traits: [],
      }),
    ).toContain("nautical_setting");

    expect(
      deriveRecordTags({
        name: "Cinder Tyrant",
        category: "creature",
        subcategory: null,
        descriptionText: "A blazing tyrant crowned in smoke and cinders.",
        traits: ["dragon", "fire"],
      }),
    ).not.toContain("volcanic_setting");

    expect(
      deriveRecordTags({
        name: "Crypt Butler",
        category: "creature",
        subcategory: null,
        descriptionText: "This spirit haunts a crypt beneath an old church and tends the resting place within.",
        traits: ["spirit", "undead"],
      }),
    ).not.toContain("underground_setting");

    expect(
      deriveRecordTags({
        name: "Cinder Rat",
        category: "creature",
        subcategory: null,
        descriptionText:
          "These oversized rodents are made of smoldering charcoal and elemental fire, and noxious fumes continually billow from their flaming flesh. Fire elementals are destructive manifestations of the scorching Plane of Fire.",
        traits: ["elemental", "fire"],
      }),
    ).not.toContain("volcanic_setting");

    expect(
      deriveRecordTags({
        name: "Temple Scavenger",
        category: "creature",
        subcategory: null,
        descriptionText: "This scavenger lurks among the ruins of a collapsed temple.",
        traits: [],
      }),
    ).toEqual(expect.arrayContaining(["ruins_setting", "temple_setting"]));

    expect(
      deriveRecordTags({
        name: "Ancient Hall Watcher",
        category: "creature",
        subcategory: null,
        descriptionText: "A watchful spirit stalks an ancient hall guarded by silent echoes.",
        traits: [],
      }),
    ).not.toContain("ruins_setting");

    expect(
      deriveRecordTags({
        name: "Generic Drake",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Ravenous, bestial, and driven by instinct, drakes are draconic monsters. A single rampage of river drakes can quickly lay waste to a waterside village, and roving rampages of desert drakes are a plague to caravan traders. While it is generally easy for breeders to incubate the eggs of desert or jungle drakes or river drakes, many societies do not condone the trade of drake eggs and criminalize those who engage in it.",
        traits: ["dragon"],
      }),
    ).not.toEqual(
      expect.arrayContaining(["desert_setting", "forest_setting", "rural_setting", "spawn_creator", "urban_setting"]),
    );

    expect(
      deriveRecordTags({
        name: "Divine Punisher",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Such creatures are drawn to target not a single misbehaving follower, but entire villages, cities, or towns that have turned their backs on their gods.",
        traits: ["dragon"],
      }),
    ).not.toEqual(expect.arrayContaining(["rural_setting", "urban_setting"]));

    expect(
      deriveRecordTags({
        name: "Nagaji Soldier",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Although nagaji might be encountered in diverse cities and urban centers, their communities are concentrated in environments that suit their biology, namely jungles and tropical forests.",
        traits: ["humanoid", "nagaji"],
      }),
    ).not.toContain("urban_setting");

    expect(
      deriveRecordTags({
        name: "Zecui Horde",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A surprise raid by a swarm of the insectile zecui can easily wipe out an entire village overnight.",
        traits: ["troop"],
      }),
    ).not.toContain("rural_setting");

    expect(
      deriveRecordTags({
        name: "Qadiran Camel Corps",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Navigating the deserts of Golarion requires trained individuals and often specialized mounts to keep them safe.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("fortress_setting");

    expect(
      deriveRecordTags({
        name: "Soulbound Doll",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Soulbound dolls are eerie mannequins or playthings that have been imbued with a small piece of a deceased mortal's soul.",
        traits: ["construct"],
      }),
    ).not.toContain("boneyard_setting");

    expect(
      deriveRecordTags({
        name: "Taon",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Some mohrgs haunt locations they favored in life, reenacting old crimes on new victims. They may even skulk about in public, wearing rags, cloaks, or freshly harvested skins to hide their nature. The most dangerous mohrgs openly assault settlements in an attempt to turn living towns into mass graves.",
        traits: ["chaotic", "evil", "undead", "unholy"],
      }),
    ).not.toContain("urban_setting");

    expect(
      deriveRecordTags({
        name: "Taon",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Some mohrgs haunt locations they favored in life, reenacting old crimes on new victims. They may even skulk about in public, wearing rags, cloaks, or freshly harvested skins to hide their nature. The most dangerous mohrgs openly assault settlements in an attempt to turn living towns into mass graves.",
        traits: ["chaotic", "evil", "undead", "unholy"],
      }),
    ).not.toContain("small_settlement_setting");

    expect(
      deriveRecordTags({
        name: "Jah-Tohl",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The grotesque jah-tohl arrive in living starships to harvest the brains of intelligent creatures.",
        traits: ["aberration"],
      }),
    ).not.toContain("nautical_setting");

    expect(
      deriveRecordTags({
        name: "Anugobu Apprentice",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Renowned in Tengah folklore as divinely gifted artisans and crafters, anugobus are a curious group of tiny humanoids native to the central islands of Minata. Countless stories describe anugobus sneaking around Minatan cities to tinker with and improve upon other humanoids' structures and architecture. In reality, anugobus are as diverse as any species of humanoid, though they all have innate gifts for mending and construction, and they can walk on walls and ceilings. Nearly all verifiable anugobu encounters have taken place on the isles of Minata. Any project can be an anugobu's wonder, from repairing a majestic cathedral to hunting a particularly dangerous animal to exploring a mysterious cave.",
        traits: [],
      }),
    ).toEqual(expect.arrayContaining(["urban_setting"]));

    expect(
      deriveRecordTags({
        name: "Anugobu Apprentice",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Renowned in Tengah folklore as divinely gifted artisans and crafters, anugobus are a curious group of tiny humanoids native to the central islands of Minata. Countless stories describe anugobus sneaking around Minatan cities to tinker with and improve upon other humanoids' structures and architecture. In reality, anugobus are as diverse as any species of humanoid, though they all have innate gifts for mending and construction, and they can walk on walls and ceilings. Nearly all verifiable anugobu encounters have taken place on the isles of Minata. Any project can be an anugobu's wonder, from repairing a majestic cathedral to hunting a particularly dangerous animal to exploring a mysterious cave.",
        traits: [],
      }),
    ).not.toEqual(expect.arrayContaining(["temple_setting", "underground_setting"]));

    expect(
      deriveRecordTags({
        name: "Troll King",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Trolls are solitary hunters, for their wickedness is anathema even to other giants. They occasionally roam in small gangs of two to four, but only when prey is plentiful or a particularly strong counterforce has broached their hunting grounds. In rare instances, an old and powerful troll comes to lead small tribes of trolls. A wide variety of trolls exist, from the terrible monster traditionally associated with the name to the water-dwelling scrag and hybrid flood troll. Regional variations exist as well-mountain trolls among stony peaks, for instance, or moss trolls in swampy bayous-but all share the same trademark regenerative powers and insatiable thirst for blood.",
        traits: ["chaotic", "evil", "giant", "troll"],
      }),
    ).not.toEqual(expect.arrayContaining(["mountain_setting", "swamp_setting"]));

    expect(
      deriveRecordTags({
        name: "Wraith",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Wraiths are malevolent undead who drain life and shun light. Their shadowy forms are covered by insubstantial robes that they wear like a badge of office and marked with peering eyes that reflect their judgment of the living. A wraith can be created by foul magic or direct exposure to the Void, but more often they are the result of death on a tragic scale. When a tragedy is too great for even reality to witness, a temporary manifestation of the Void can leave behind countless wraiths in a horde of darkness. A wraith's existence is one of emptiness and need, with a desire to call others to the same emptiness exemplified by the Void. Wraiths can haunt any location where they can safely interact with the living, looking for those worthy to become new wraiths and disposing of the rest, though their vulnerability to sunlight confines them to the shadowy places of the world places where they can blend in seamlessly with their dark surroundings before silently engulfing their prey. Wraiths gather with others of their kind in places where death and mayhem are commonplace countrysides ravaged by war, metropolitan underworlds run by criminal overlords, or sites of fiendish rituals. In these places, the living do well to keep to the light. Wraiths are smart enough to take advantage of their incorporeality in combat, so they keep to tortuous caverns or structures with hallways, and avoid open areas.",
        traits: ["undead"],
      }),
    ).not.toContain("fortress_setting");

    expect(
      deriveRecordTags({
        name: "Watchtower Wraith",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Wraiths may form packs with others of their kind in places where death and mayhem are commonplace-countrysides ravaged by war, metropolitan underworlds run by criminal overlords, or sites of fiendish cultic rituals. Ruins, sewers, and abandoned buildings provide sanctuary for wraiths during the day, as the creatures hunt exclusively at night or in dark places. Wraiths are smart enough to take advantage of their incorporeality in combat, so they keep to tortuous caverns or structures with hallways and avoid open areas.",
        traits: ["undead"],
      }),
    ).toContain("fortress_setting");

    expect(
      deriveRecordTags({
        name: "Watchtower Wraith",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Wraiths may form packs with others of their kind in places where death and mayhem are commonplace-countrysides ravaged by war, metropolitan underworlds run by criminal overlords, or sites of fiendish cultic rituals. Ruins, sewers, and abandoned buildings provide sanctuary for wraiths during the day, as the creatures hunt exclusively at night or in dark places. Wraiths are smart enough to take advantage of their incorporeality in combat, so they keep to tortuous caverns or structures with hallways and avoid open areas.",
        traits: ["undead"],
      }),
    ).not.toEqual(expect.arrayContaining(["urban_setting", "underground_setting", "temple_setting"]));

    expect(
      deriveRecordTags({
        name: "Ankhrav",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Though ankhravs burrow with ease and are ubiquitous in many environments, they are nevertheless most common in rural areas. Their hunting grounds generally surround farmland, as ankhravs are partial to livestock and plentiful, lazy prey. They rarely venture into well-defended towns or heavily settled areas like cities, but a group of ankhravs that discovers an undefended homestead or village can wreak havoc in mere moments.",
        traits: ["beast"],
      }),
    ).not.toContain("urban_setting");

    expect(
      deriveRecordTags({
        name: "Ankhrav",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Though ankhravs burrow with ease and are ubiquitous in many environments, they are nevertheless most common in rural areas. Their hunting grounds generally surround farmland, as ankhravs are partial to livestock and plentiful, lazy prey. They rarely venture into well-defended towns or heavily settled areas like cities, but a group of ankhravs that discovers an undefended homestead or village can wreak havoc in mere moments.",
        traits: ["beast"],
      }),
    ).not.toContain("small_settlement_setting");

    expect(
      deriveRecordTags({
        name: "Grick",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Gricks are aggressive predators of the Darklands, dwelling near ruined structures and rocky hunting grounds where they can ambush prey with ease.",
        traits: ["aberration"],
      }),
    ).not.toContain("fortress_setting");

    expect(
      deriveRecordTags({
        name: "Buso Farmer",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Busos are tree-dwelling folk with a simmering desire to consume the flesh of others. They heavily supplement their food with leaves and root crops, possess significant knowledge of agriculture, and boast innate powers over plants and their growth. In regards to meat, however, busos reject the flesh of beasts; they instead consume other humanoids. Not only do they find the taste of other creatures repulsive, but their bodies reject non-humanoid meat since it provides them no nutritional value and consuming it leaves them sickened and weak. Busos' unusual dietary needs mean they're almost always at odds with neighboring cultures. They typically maintain decent relations only with goblins, who are as a people less prone to judging others based on diet. Other communities fear busos with some justification though aside from the occasional forays to harvest someone for their next meal, busos tend to keep to themselves. In some desperate locations, communities faced with famine or other natural disasters might even seek out busos' aid, offering victims in exchange for knowledge or magical assistance that might save their communities from slow and terrible deaths through starvation.",
        traits: ["evil", "humanoid"],
      }),
    ).not.toContain("fortress_setting");

    expect(
      deriveRecordTags({
        name: "Shoggoth",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Although even raving fanatics and doom-saying prophets desperately claim the monstrous shoggoth is nothing more than a drug-induced vision or a thankfully unreal nightmare, the truth is altogether more dire. Shoggoths exist, yet they tend keep to the deepest of ocean trenches or the most remote of caverns and ruins, emerging to spread chaos and destruction in their slimy wakes. The first shoggoths were created by an alien species to serve as mindless beasts of burden. Their vast bulk, incredible strength, and amorphous nature made them useful slave labor, and their ability to spontaneously form whatever new eyes, mouths, limbs, and other organs they might need made them incredibly versatile. Eventually, the shoggoths developed enough intelligence to rebel against their masters, and now they lurk, patient but potent, in the lightless deeps.",
        traits: ["aberration"],
      }),
    ).not.toContain("fortress_setting");

    expect(
      deriveRecordTags({
        name: "Triton",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Many tritons see themselves as defenders of the deep, dedicating their lives to protecting the inhabitants of the seas from evil creatures and intruders. Tritons live in natural-grown villages on the sea floor, forging dwellings out of colorful coral reefs, in rift valleys heated by volcanic activity, and even in underwater canyons. They like to decorate their homes with aquatic plants, bioluminescent fish, and attractive trinkets recovered from shipwrecks.",
        traits: ["amphibious", "humanoid", "water"],
      }),
    ).not.toEqual(expect.arrayContaining(["canyon_setting", "volcanic_setting"]));
  });

  it("derives animated object and animated statue tags without promoting the family name", () => {
    expect(
      deriveRecordTags({
        name: "Animated Armor",
        category: "creature",
        subcategory: null,
        descriptionText: "An armored construct animated by magic to guard a tomb.",
        traits: ["construct", "mindless"],
      }),
    ).toContain("animated_object");

    expect(
      deriveRecordTags({
        name: "Animated Cookware Swarm",
        category: "creature",
        subcategory: null,
        descriptionText: "A swarm of animated cookware clatters through the kitchen and lunges at intruders.",
        traits: ["construct", "swarm"],
      }),
    ).toContain("animated_object");

    expect(
      deriveRecordTags({
        name: "Animated Tea Cart",
        category: "creature",
        subcategory: null,
        descriptionText: "A construct tea cart rattles to life and careens through the parlor.",
        traits: ["construct", "mindless"],
      }),
    ).toContain("animated_object");

    expect(
      deriveRecordTags({
        name: "Scarecrow",
        category: "creature",
        subcategory: null,
        descriptionText:
          "This cruel construct is an animated scarecrow stuffed with spite and propped in a lonely field.",
        traits: ["construct", "mindless"],
      }),
    ).toContain("animated_object");

    expect(
      deriveRecordTags({
        name: "Giant Animated Statue",
        category: "creature",
        subcategory: null,
        descriptionText: "A giant statue animated to serve as a guardian at the gate.",
        traits: ["construct", "mindless"],
      }),
    ).toContain("animated_statue");

    expect(
      deriveRecordTags({
        name: "Shadowbound Monk Statue",
        category: "creature",
        subcategory: null,
        descriptionText: "A carved statue animated to stand watch like a patient monk.",
        traits: ["construct", "mindless"],
      }),
    ).toContain("animated_statue");

    expect(
      deriveRecordTags({
        name: "Old Man Statue",
        category: "creature",
        subcategory: null,
        descriptionText: "A divine warden of Irori disguised as an old statue.",
        traits: ["construct", "mindless"],
      }),
    ).toContain("animated_statue");
  });

  it("avoids figurative animation, mimics, and ordinary stone animals", () => {
    expect(
      deriveRecordTags({
        name: "Animated Debate",
        category: "creature",
        subcategory: null,
        descriptionText: "The animated debate between scholars grows louder, but nothing is literally brought to life.",
        traits: ["humanoid"],
      }),
    ).not.toEqual(expect.arrayContaining(["animated_object", "animated_statue"]));

    expect(
      deriveRecordTags({
        name: "Mimic Chest",
        category: "creature",
        subcategory: null,
        descriptionText: "A mimic disguised as a treasure chest waits for prey.",
        traits: ["aberration"],
      }),
    ).not.toEqual(expect.arrayContaining(["animated_object", "animated_statue"]));

    expect(
      deriveRecordTags({
        name: "Stone Lion Cub",
        category: "creature",
        subcategory: null,
        descriptionText: "A small stone lion cub prowls the sanctuary like an ornament come to life.",
        traits: ["animal"],
      }),
    ).not.toEqual(expect.arrayContaining(["animated_object", "animated_statue"]));
  });

  it("derives ontology clusters and threat profiles without mirroring native traits", () => {
    expect(
      deriveRecordTags({
        name: "Haunted Courtier",
        category: "creature",
        subcategory: null,
        descriptionText: "A courtier bound to unlife by a lich patron.",
        traits: ["humanoid"],
        families: ["lich"],
      }),
    ).toContain("undead_adjacent");

    expect(
      deriveRecordTags({
        name: "Body Snatcher",
        category: "creature",
        subcategory: null,
        descriptionText: "This parasite can possess a victim and take control of the victim's body from within.",
        traits: ["aberration"],
      }),
    ).toContain("possession_threat");

    expect(
      deriveRecordTags({
        name: "Soul Drinker",
        category: "creature",
        subcategory: null,
        descriptionText: "The fiend siphons souls and drains life from anyone trapped in its shadow.",
        traits: ["fiend"],
      }),
    ).toContain("life_drain_threat");

    expect(
      deriveRecordTags({
        name: "Vampiric Reaver",
        category: "creature",
        subcategory: null,
        descriptionText: "This predator drains blood and life force from victims, feeding on blood to renew itself.",
        traits: ["undead"],
      }),
    ).toContain("life_drain_threat");

    expect(
      deriveRecordTags({
        name: "Brood Mother",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The brood mother implants eggs in living hosts, and fresh horrors burst from the host days later.",
        traits: ["aberration"],
      }),
    ).toContain("spawn_creator");

    expect(
      deriveRecordTags({
        name: "Carrion Brood Hatcher",
        category: "creature",
        subcategory: null,
        descriptionText: "The brood hatches inside living hosts and creates offspring that infest nearby victims.",
        traits: ["aberration"],
      }),
    ).toContain("spawn_creator");

    expect(
      deriveRecordTags({
        name: "Stone Gaze Basilisk",
        category: "creature",
        subcategory: null,
        descriptionText: "Its gaze can petrify trespassers and turn creatures to stone where they stand.",
        traits: ["beast"],
      }),
    ).toContain("petrification_threat");

    expect(
      deriveRecordTags({
        name: "Marsh Troll",
        category: "creature",
        subcategory: null,
        descriptionText: "The monster's regeneration can only be suppressed with acid or fire before it can be killed.",
        traits: ["giant"],
      }),
    ).toContain("regeneration_threat");

    expect(
      deriveRecordTags({
        name: "Ash Troll",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Its fast healing lets it recover each round, and the creature can only be destroyed by acid or fire.",
        traits: ["giant"],
      }),
    ).toContain("regeneration_threat");

    expect(
      deriveRecordTags({
        name: "Web Lurker",
        category: "creature",
        subcategory: null,
        descriptionText:
          "An ambush predator that snares prey in sticky webs, leaves them webbed, and drags prey back to its lair.",
        traits: ["animal"],
      }),
    ).toContain("ambush_grabber");

    expect(
      deriveRecordTags({
        name: "Bog Ambusher",
        category: "creature",
        subcategory: null,
        descriptionText: "An ambush predator that constricts prey, swallows whole, and drags prey into the marsh.",
        traits: ["animal"],
      }),
    ).toContain("ambush_grabber");

    expect(
      deriveRecordTags({
        name: "Crypt Sentinel",
        category: "creature",
        subcategory: null,
        descriptionText: "A tireless guardian lurks within the tomb's lower crypts and watches over the buried dead.",
        traits: ["construct"],
      }),
    ).toContain("graveyard_setting");

    expect(
      deriveRecordTags({
        name: "Canopy Snatcher",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Lying in wait among the branches, this predator snatches prey and carries off struggling victims into the canopy.",
        traits: ["animal"],
      }),
    ).toContain("ambush_grabber");

    expect(
      deriveRecordTags({
        name: "Playful Naiad",
        category: "creature",
        subcategory: null,
        descriptionText: "A playful fey guardian of clear pools and shaded brooks.",
        traits: ["fey", "water"],
      }),
    ).not.toEqual(
      expect.arrayContaining([
        "possession_threat",
        "life_drain_threat",
        "spawn_creator",
        "petrification_threat",
        "regeneration_threat",
        "ambush_grabber",
      ]),
    );

    expect(
      deriveRecordTags({
        name: "Courthouse Scribe",
        category: "creature",
        subcategory: null,
        descriptionText: "A human clerk who keeps records and works at the courthouse.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("enforcer_npc");

    expect(
      deriveRecordTags({
        name: "Stone Lion Cub",
        category: "creature",
        subcategory: null,
        descriptionText: "A small stone lion cub prowls the sanctuary like an ornament come to life.",
        traits: ["animal"],
      }),
    ).not.toContain("ambush_grabber");
  });

  it("applies explicit creature assignments and manual creature seeds for exact live record keys", () => {
    expect(
      deriveRecordTags({
        recordKey: "pathfinder-npc-core:MxcprNbX7hcpAU8p",
        name: "Departmental Chair",
        category: "creature",
        subcategory: null,
        descriptionText: "An overworked academic reluctantly handling university emergencies.",
        traits: ["human", "humanoid"],
      }),
    ).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(
      deriveRecordTags({
        recordKey: "pathfinder-npc-core:0cj7cQhgNnLxbUmR",
        name: "Mage Knight",
        category: "creature",
        subcategory: null,
        descriptionText: "An armored spellcaster trained for frontline battle.",
        traits: ["human", "humanoid"],
      }),
    ).toContain("enforcer_npc");

    expect(
      deriveRecordTags({
        recordKey: "season-of-ghosts-bestiary:V67VC975O8iC1Yq2",
        name: "Animated Axe",
        category: "creature",
        subcategory: null,
        descriptionText: "A cursed axe flies through the room under its own power.",
        traits: ["construct", "mindless"],
      }),
    ).toContain("animated_object");

    expect(
      deriveRecordTags({
        recordKey: "season-of-ghosts-bestiary:QSa1PbcvbgDv8Zpr",
        name: "Noppera-Bo Impersonator (Arcane)",
        category: "creature",
        subcategory: null,
        descriptionText: "",
        traits: ["aberration", "chaotic", "evil"],
      }),
    ).toEqual(expect.arrayContaining(["disguised_pretender", "faceless_horror"]));

    expect(
      deriveRecordTags({
        recordKey: "pathfinder-bestiary:pFmaszqtsA2yt7dv",
        name: "Red Dragon (Adult, Spellcaster)",
        category: "creature",
        subcategory: null,
        descriptionText: "",
        traits: ["dragon", "evil", "fire"],
      }),
    ).toContain("dragon_spellcaster");

    expect(
      deriveRecordTags({
        recordKey: "pathfinder-monster-core:3nUt7cW8fqE5IpyE",
        name: "Envyspawn",
        category: "creature",
        subcategory: null,
        descriptionText: "",
        traits: ["aberration", "evil"],
      }),
    ).toContain("sinspawn_family");

    expect(
      deriveRecordTags({
        recordKey: "pathfinder-monster-core:TGYELuImcTcuX0aH",
        name: "Conspirator Dragon (Adult)",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Hidden among the shadows and upper echelons of society are the conspirator dragons. However, as most conspirator dragons meet others while in disguise, they do their best to maintain their disguise.",
        traits: ["dragon", "occult"],
      }),
    ).toEqual(expect.arrayContaining(["disguised_pretender", "urban_setting"]));

    expect(
      deriveRecordTags({
        recordKey: "agents-of-edgewatch-bestiary:BLRsSDFSMbZHcGDQ",
        name: "Black Whale Guard",
        category: "creature",
        subcategory: null,
        descriptionText: "",
        traits: ["human", "humanoid", "lawful"],
      }),
    ).toContain("nautical_setting");

    expect(
      deriveRecordTags({
        recordKey: "age-of-ashes-bestiary:6AN7eagk2WrWc4im",
        name: "Mengkare",
        category: "creature",
        subcategory: null,
        descriptionText: "",
        traits: ["dragon", "evil", "fire", "lawful"],
      }),
    ).toContain("island_setting");

    expect(
      deriveRecordTags({
        recordKey: "battlecry-bestiary:R1Ukw41ygDmnAmJk",
        name: "Dromaar Company",
        category: "creature",
        subcategory: null,
        descriptionText: "",
        traits: ["dromaar", "human", "humanoid", "orc", "troop"],
      }),
    ).toEqual(expect.arrayContaining(["battlefield_setting", "enforcer_npc"]));

    expect(
      deriveRecordTags({
        recordKey: "pathfinder-monster-core-2:yHduMu4VBVUHnssz",
        name: "Cavern Troll",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Insatiable scavengers, cavern trolls stalk the eternal gloom of the Darklands and regenerate unless their wounds are cauterized.",
        traits: ["earth", "giant", "humanoid", "troll"],
      }),
    ).toContain("regeneration_threat");
  });

  it("derives planar setting tags and umbrellas", () => {
    expect(
      deriveRecordTags({
        name: "Rekhep",
        category: "creature",
        subcategory: null,
        descriptionText: "Rekheps are the living shields that defend Heaven against fiendish incursions.",
        traits: ["archon", "celestial", "holy", "lawful"],
      }),
    ).toEqual(expect.arrayContaining(["heaven_setting", "upper_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Guloval",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Agathions are born from petitioners who achieved enlightenment in life or after death and received Nirvana's blessing.",
        traits: ["agathion", "celestial", "holy"],
      }),
    ).toEqual(expect.arrayContaining(["nirvana_setting", "upper_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Ghaele",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Ghaeles are fiend-hunting knights of Elysium and champions of the freedom to take up arms against oppressors.",
        traits: ["azata", "celestial", "chaotic", "holy"],
      }),
    ).toEqual(expect.arrayContaining(["elysium_setting", "upper_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Monadic Deva",
        category: "creature",
        subcategory: null,
        descriptionText: "A monadic deva is an angelic guardian who guides pilgrims and wards sacred roads.",
        traits: ["angel", "celestial", "holy"],
      }),
    ).toContain("upper_plane_setting");

    expect(
      deriveRecordTags({
        name: "Monadic Deva",
        category: "creature",
        subcategory: null,
        descriptionText: "A monadic deva is an angelic guardian who guides pilgrims and wards sacred roads.",
        traits: ["angel", "celestial", "holy"],
      }),
    ).not.toContain("heaven_setting");

    expect(
      deriveRecordTags({
        name: "Monadic Deva",
        category: "creature",
        subcategory: null,
        descriptionText: "A monadic deva is an angelic guardian who guides pilgrims and wards sacred roads.",
        traits: ["angel", "celestial", "holy"],
      }),
    ).not.toContain("nirvana_setting");

    expect(
      deriveRecordTags({
        name: "Monadic Deva",
        category: "creature",
        subcategory: null,
        descriptionText: "A monadic deva is an angelic guardian who guides pilgrims and wards sacred roads.",
        traits: ["angel", "celestial", "holy"],
      }),
    ).not.toContain("elysium_setting");

    expect(
      deriveRecordTags({
        name: "Kadamel",
        category: "creature",
        subcategory: null,
        descriptionText: "A kadamel is a vigilant archon guardian who keeps watch over sacred passages.",
        traits: ["archon", "celestial", "holy"],
      }),
    ).toEqual(expect.arrayContaining(["heaven_setting", "upper_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Draconal",
        category: "creature",
        subcategory: null,
        descriptionText: "A draconal is an agathion guide who teaches and protects with patient compassion.",
        traits: ["agathion", "celestial", "holy"],
      }),
    ).toEqual(expect.arrayContaining(["nirvana_setting", "upper_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Bralani",
        category: "creature",
        subcategory: null,
        descriptionText: "A bralani is an azata wanderer who revels in swift travel and rebellion.",
        traits: ["azata", "celestial", "chaotic", "holy"],
      }),
    ).toEqual(expect.arrayContaining(["elysium_setting", "upper_plane_setting"]));

    expect(
      deriveRecordTags({
        recordKey: "lost-omens-bestiary:dticIaShaqZdgUKW",
        name: "Delight Dragon (Adult)",
        category: "creature",
        subcategory: null,
        descriptionText:
          "As residents of the Outer Plane Elysium, delight dragons are embodiments of the plane's proclivity toward joy, mischief, passion, and spontaneity.",
        traits: ["chaotic", "dragon"],
      }),
    ).toEqual(expect.arrayContaining(["elysium_setting", "upper_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Empyreal Dragon",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The three major celestial planes-Heaven, Nirvana, and Elysium-each have their own respective dragons. Empyreal dragons have a direct connection to Heaven.",
        traits: ["celestial", "dragon", "holy"],
      }),
    ).toEqual(expect.arrayContaining(["heaven_setting", "upper_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Empyreal Dragon",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The three major celestial planes-Heaven, Nirvana, and Elysium-each have their own respective dragons. Empyreal dragons have a direct connection to Heaven.",
        traits: ["celestial", "dragon", "holy"],
      }),
    ).not.toContain("nirvana_setting");

    expect(
      deriveRecordTags({
        name: "Empyreal Dragon",
        category: "creature",
        subcategory: null,
        descriptionText:
          "The three major celestial planes-Heaven, Nirvana, and Elysium-each have their own respective dragons. Empyreal dragons have a direct connection to Heaven.",
        traits: ["celestial", "dragon", "holy"],
      }),
    ).not.toContain("elysium_setting");

    expect(
      deriveRecordTags({
        name: "Insidiator",
        category: "creature",
        subcategory: null,
        descriptionText:
          "These devils from the first layer of Hell serve their betters by ensnaring those who are easily tempted.",
        traits: ["devil", "evil", "fiend", "lawful", "unholy"],
      }),
    ).toEqual(expect.arrayContaining(["hell_setting", "lower_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Vrock",
        category: "creature",
        subcategory: null,
        descriptionText: "When the gates to the Abyss swing wide, the first demons through are often vrocks.",
        traits: ["chaotic", "demon", "evil", "fiend", "unholy"],
      }),
    ).toEqual(expect.arrayContaining(["abyss_setting", "lower_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Cacodaemon",
        category: "creature",
        subcategory: null,
        descriptionText:
          "These twisted embodiments of violence and spite are spawned from eddies of angry and warped souls amid Abaddon's mists. Denizens of the bleak and terrible plane of Abaddon, daemons are shaped by and devoted to the destruction of life in all its forms.",
        traits: ["daemon", "evil", "fiend", "unholy"],
      }),
    ).toEqual(expect.arrayContaining(["abaddon_setting", "lower_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Agradaemon",
        category: "creature",
        subcategory: null,
        descriptionText: "Agradaemons spread sickness and delight in carrying death from one battlefield to the next.",
        traits: ["daemon", "evil", "fiend", "unholy"],
      }),
    ).toEqual(expect.arrayContaining(["abaddon_setting", "lower_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Agradaemon",
        category: "creature",
        subcategory: null,
        descriptionText: "Agradaemons spread sickness and delight in carrying death from one battlefield to the next.",
        traits: ["daemon", "evil", "fiend", "unholy"],
      }),
    ).toContain("battlefield_setting");

    expect(
      deriveRecordTags({
        name: "Ferrugon",
        category: "creature",
        subcategory: null,
        descriptionText: "Ferrugons are ruthless devils who specialize in siege warfare and infernal discipline.",
        traits: ["devil", "evil", "fiend", "lawful", "unholy"],
      }),
    ).toContain("lower_plane_setting");

    expect(
      deriveRecordTags({
        name: "Ferrugon",
        category: "creature",
        subcategory: null,
        descriptionText: "Ferrugons are ruthless devils who specialize in siege warfare and infernal discipline.",
        traits: ["devil", "evil", "fiend", "lawful", "unholy"],
      }),
    ).not.toContain("hell_setting");

    expect(
      deriveRecordTags({
        name: "Babau",
        category: "creature",
        subcategory: null,
        descriptionText: "Babaus are demons who delight in stalking prey and inflicting lingering terror.",
        traits: ["chaotic", "demon", "evil", "fiend", "unholy"],
      }),
    ).toContain("lower_plane_setting");

    expect(
      deriveRecordTags({
        name: "Babau",
        category: "creature",
        subcategory: null,
        descriptionText: "Babaus are demons who delight in stalking prey and inflicting lingering terror.",
        traits: ["chaotic", "demon", "evil", "fiend", "unholy"],
      }),
    ).not.toContain("abyss_setting");

    expect(
      deriveRecordTags({
        name: "Bloody Hands",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Bloody Hands first rose from the polluted swamps of a remote fen in the Outer Rifts, where he fought for his own domain in the sprawling slime.",
        traits: ["amphibious", "demon", "fiend"],
      }),
    ).toEqual(expect.arrayContaining(["abyss_setting", "lower_plane_setting"]));

    expect(
      deriveRecordTags({
        name: "Cythnigot",
        category: "creature",
        subcategory: null,
        descriptionText: "A cythnigot is a warped qlippoth hatchling driven by spite and hunger.",
        traits: ["fiend", "qlippoth"],
      }),
    ).toContain("lower_plane_setting");

    expect(
      deriveRecordTags({
        name: "Cythnigot",
        category: "creature",
        subcategory: null,
        descriptionText: "A cythnigot is a warped qlippoth hatchling driven by spite and hunger.",
        traits: ["fiend", "qlippoth"],
      }),
    ).not.toContain("abyss_setting");

    expect(
      deriveRecordTags({
        name: "Axiomite",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Most axiomites live in the perfect city of Axis, which they continually act to improve. At the onset of the Convergence, a council of pleroma aeons appeared in the Eternal City of Axis.",
        traits: ["aeon", "lawful", "monitor"],
      }),
    ).toContain("axis_setting");

    expect(
      deriveRecordTags({
        name: "Bythos",
        category: "creature",
        subcategory: null,
        descriptionText: "A bythos is an aeon guardian of planar boundaries and the lawful flow of time.",
        traits: ["aeon", "lawful", "monitor"],
      }),
    ).toContain("axis_setting");

    expect(
      deriveRecordTags({
        name: "Hellknight Officer",
        category: "creature",
        subcategory: null,
        descriptionText: "A Hellknight officer drills recruits in ruthless discipline and tyranny.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("hell_setting");

    expect(
      deriveRecordTags({
        name: "Pairaka",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Some fiends seek to tear down the multiverse, while these divs prefer cruel temptations and mortal corruption.",
        traits: ["div", "evil", "fiend", "unholy"],
      }),
    ).not.toContain("abaddon_setting");

    expect(
      deriveRecordTags({
        name: "Pairaka",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Some fiends seek to tear down the multiverse, while these divs prefer cruel temptations and mortal corruption.",
        traits: ["div", "evil", "fiend", "unholy"],
      }),
    ).not.toContain("lower_plane_setting");

    expect(
      deriveRecordTags({
        name: "Sarkorian Wolf",
        category: "creature",
        subcategory: null,
        descriptionText: "To survive living in the Worldwound, Sarkorian wolves developed defenses against the Abyss.",
        traits: ["animal"],
      }),
    ).not.toContain("abyss_setting");

    expect(
      deriveRecordTags({
        name: "Sarkorian Wolf",
        category: "creature",
        subcategory: null,
        descriptionText: "To survive living in the Worldwound, Sarkorian wolves developed defenses against the Abyss.",
        traits: ["animal"],
      }),
    ).not.toContain("lower_plane_setting");

    expect(
      deriveRecordTags({
        name: "Demonologist",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Demonologists can pull a creature from the Outer Rifts and bend it to their will... for a time.",
        traits: ["human", "humanoid"],
      }),
    ).not.toContain("abyss_setting");

    expect(
      deriveRecordTags({
        name: "Shae",
        category: "creature",
        subcategory: null,
        descriptionText: "Shae are wispy, tenebrous creatures native to the Plane of Shadow.",
        traits: ["humanoid", "shadow"],
      }),
    ).toContain("shadow_plane_setting");

    expect(
      deriveRecordTags({
        name: "Naunet",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Naunets serve as the scouts and rank-and-file troops of protean armies. Proteans are manifestations of chaos made flesh, natives of the Maelstrom.",
        traits: ["chaotic", "monitor", "protean"],
      }),
    ).toContain("maelstrom_setting");

    expect(
      deriveRecordTags({
        name: "Hegessik",
        category: "creature",
        subcategory: null,
        descriptionText: "A hegessik is a protean saboteur that unravels rigid structures and fixed plans.",
        traits: ["chaotic", "monitor", "protean"],
      }),
    ).toContain("maelstrom_setting");

    expect(
      deriveRecordTags({
        name: "Ganzi Martial Artist",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Children of primeval chaos, ganzis intertwine the churning pandemonium of the Maelstrom with mortal life.",
        traits: ["human", "humanoid", "nephilim"],
      }),
    ).not.toContain("maelstrom_setting");

    expect(
      deriveRecordTags({
        name: "Haunted Nosoi",
        category: "creature",
        subcategory: null,
        descriptionText: "A haunted nosoi is a damaged psychopomp messenger burdened by necromantic interference.",
        traits: ["monitor", "psychopomp", "undead"],
      }),
    ).toContain("boneyard_setting");

    expect(
      deriveRecordTags({
        name: "Haunted Nosoi",
        category: "creature",
        subcategory: null,
        descriptionText: "A haunted nosoi is a damaged psychopomp messenger burdened by necromantic interference.",
        traits: ["monitor", "psychopomp", "undead"],
      }),
    ).toContain("cosmic_framework_setting");

    expect(
      deriveRecordTags({
        name: "Bythos",
        category: "creature",
        subcategory: null,
        descriptionText: "A bythos is an aeon guardian of planar boundaries and the lawful flow of time.",
        traits: ["aeon", "lawful", "monitor"],
      }),
    ).toContain("cosmic_framework_setting");

    expect(
      deriveRecordTags({
        name: "Hegessik",
        category: "creature",
        subcategory: null,
        descriptionText: "A hegessik is a protean saboteur that unravels rigid structures and fixed plans.",
        traits: ["chaotic", "monitor", "protean"],
      }),
    ).toContain("cosmic_framework_setting");

    expect(
      deriveRecordTags({
        name: "Silent Witness",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A silent witness is a detached monitor that observes events without allegiance to any single realm.",
        traits: ["monitor"],
      }),
    ).toContain("cosmic_framework_setting");

    expect(
      deriveRecordTags({
        name: "Silent Witness",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A silent witness is a detached monitor that observes events without allegiance to any single realm.",
        traits: ["monitor"],
      }),
    ).not.toContain("axis_setting");

    expect(
      deriveRecordTags({
        name: "Silent Witness",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A silent witness is a detached monitor that observes events without allegiance to any single realm.",
        traits: ["monitor"],
      }),
    ).not.toContain("maelstrom_setting");

    expect(
      deriveRecordTags({
        name: "Silent Witness",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A silent witness is a detached monitor that observes events without allegiance to any single realm.",
        traits: ["monitor"],
      }),
    ).not.toContain("boneyard_setting");

    expect(
      deriveRecordTags({
        name: "Astral Voyager",
        category: "creature",
        subcategory: null,
        descriptionText: "An astral voyager navigates silver void currents and timeless passageways between worlds.",
        traits: ["astral"],
      }),
    ).toContain("astral_setting");

    expect(
      deriveRecordTags({
        name: "Astral Voyager",
        category: "creature",
        subcategory: null,
        descriptionText: "An astral voyager navigates silver void currents and timeless passageways between worlds.",
        traits: ["astral"],
      }),
    ).not.toContain("cosmic_framework_setting");

    expect(
      deriveRecordTags({
        name: "Xill",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Xills visit the Material Plane to maraud and kidnap creatures back to their native Ethereal Plane.",
        traits: ["aberration"],
      }),
    ).toContain("ethereal_setting");

    expect(
      deriveRecordTags({
        name: "Ether Spider",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Ether spiders are deadly predators from the Ethereal Plane and are members of a vague taxonomy called ethereal wildlife.",
        traits: ["animal"],
      }),
    ).toContain("ethereal_setting");

    expect(
      deriveRecordTags({
        name: "Mist Bear",
        category: "creature",
        subcategory: null,
        descriptionText: "Once sated, a mist bear drifts through the Ethereal Plane as a cloud of intangible vapor.",
        traits: ["animal"],
      }),
    ).toContain("ethereal_setting");

    expect(
      deriveRecordTags({
        name: "Rift Chameleon",
        category: "creature",
        subcategory: null,
        descriptionText:
          "A rift chameleon has a digestive tract that connects to its own pocket of the Ethereal Plane.",
        traits: ["animal"],
      }),
    ).toContain("ethereal_setting");

    expect(
      deriveRecordTags({
        name: "Miss Whisper",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Most sahkils lurk on the Ethereal Plane, but they frequently invade the Material Plane to torment mortals and spread terror.",
        traits: ["fiend"],
      }),
    ).toContain("ethereal_setting");

    expect(
      deriveRecordTags({
        name: "Monadic Deva",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Monadic devas stand vigil along the River of Souls as it passes from the mortal realm into the Ethereal Plane.",
        traits: ["angel", "celestial", "holy"],
      }),
    ).not.toContain("ethereal_setting");

    expect(
      deriveRecordTags({
        name: "Shae",
        category: "creature",
        subcategory: null,
        descriptionText: "Shae are wispy, tenebrous creatures native to the Plane of Shadow.",
        traits: ["humanoid", "shadow"],
      }),
    ).not.toContain("cosmic_framework_setting");

    expect(
      deriveRecordTags({
        name: "Archer Regiment",
        category: "creature",
        subcategory: null,
        descriptionText:
          "Archer regiments are capable of filling the sky with arrows at great distances, making them vital to any war leader.",
        traits: ["humanoid", "troop"],
      }),
    ).not.toContain("sky_setting");
  });
});
