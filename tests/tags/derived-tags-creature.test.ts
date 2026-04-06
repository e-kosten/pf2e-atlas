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
    })).toEqual(expect.arrayContaining(["graveyard_setting", "underground_setting", "urban_setting", "profession_npc", "civic_npc"]));

    expect(deriveRecordTags({
      name: "Bog Wisp",
      category: "creature",
      subcategory: null,
      descriptionText: "A fey spirit that haunts marshy bogs and flooded mires.",
      traits: ["fey"],
    })).toContain("swamp_setting");

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
    })).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting", "forest_setting"]));

    expect(deriveRecordTags({
      name: "Defaced Naiad Queen",
      category: "creature",
      subcategory: null,
      descriptionText: "Naiad queens rule over pristine wildernesses centered on untouched lakes or other bodies of fresh water.",
      traits: ["fey", "water"],
    })).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));

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
    })).toEqual(expect.arrayContaining(["nautical_setting", "aquatic_setting", "undead_adjacent"]));

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
      name: "Divine Warden of Brigh",
      category: "creature",
      subcategory: null,
      descriptionText: "Imbued with the divine energy of the goddess of clockwork and invention, divine wardens of Brigh are prominent in Alkenstar as guardians of the Temple of Brigh there. Created through complex rituals performed by a faith's adherents, this divine spark allows the divine warden to serve as the protector for a temple, shrine, or other holy site. Such guardians aren't intrinsically bound to a fixed location, but they rarely leave the temple or site over which they watch.",
      traits: ["construct"],
    })).toContain("temple_setting");

    expect(deriveRecordTags({
      name: "Fortress Warden",
      category: "creature",
      subcategory: null,
      descriptionText: "A grim defender patrols the walls of a mountain fortress and ancient citadel.",
      traits: ["humanoid"],
    })).toEqual(expect.arrayContaining(["fortress_setting", "mountain_setting", "combatant_npc"]));

    expect(deriveRecordTags({
      name: "Rust Hag",
      category: "creature",
      subcategory: null,
      descriptionText: "Rust hags have a knack for technology. They make their homes in hollowed-out factories, abandoned tenements, and other sites of urban decay.",
      traits: ["hag"],
    })).toContain("urban_setting");

    expect(deriveRecordTags({
      name: "Furnerico",
      category: "creature",
      subcategory: null,
      descriptionText: "A mass of undulating tendrils and nerves, furnericos stalk sewer systems below large cities, reveling in the filth and rotting detritus created by those who dwell above the middens and sumps they call home.",
      traits: ["aberration"],
    })).toEqual(expect.arrayContaining(["urban_setting", "underground_setting"]));

    expect(deriveRecordTags({
      name: "Girtablilu Guardian",
      category: "creature",
      subcategory: null,
      descriptionText: "Sentries patrol the outskirts of girtablilu communities, watching for external threats set on reckless exploration of the girtablilus' sacred site. Girtablilus are desert-dwelling guardians most often found defending ancient temples and religious artifacts with zealous fervor.",
      traits: ["humanoid"],
    })).toContain("temple_setting");

    expect(deriveRecordTags({
      name: "Viscous Black Pudding",
      category: "creature",
      subcategory: null,
      descriptionText: "Most often found below ground, these oozes scour caves for objects to dissolve with their corrosive secretions.",
      traits: ["ooze"],
    })).toContain("underground_setting");

    expect(deriveRecordTags({
      name: "Onturat",
      category: "creature",
      subcategory: null,
      descriptionText: "Swirling down from misty peaks and through howling mountain passes like an evil wind, the vortex of bones known as a skulltaker is a terrible manifestation of the delirium and agony experienced by doomed climbers and lost trailblazers.",
      traits: ["undead"],
    })).toContain("mountain_setting");

    expect(deriveRecordTags({
      name: "Scarecrow",
      category: "creature",
      subcategory: null,
      descriptionText: "A cruel spirit haunts the countryside from its post beside a lonely farmstead and barn.",
      traits: ["construct"],
    })).toContain("rural_setting");

    expect(deriveRecordTags({
      name: "Wealthy Vigilante",
      category: "creature",
      subcategory: null,
      descriptionText: "By night, this member of the nobility dons a false identity to mete out extralegal justice.",
      traits: ["human", "humanoid"],
    })).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(deriveRecordTags({
      name: "Prophet",
      category: "creature",
      subcategory: null,
      descriptionText: "A wandering prophet shares divine dreams and advice with the faithful.",
      traits: ["human", "humanoid"],
    })).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

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
    })).toContain("first_world_setting");

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

  it("uses glossary family evidence and blocks redundant civic npc tags", () => {
    expect(deriveRecordTags({
      name: "Morlock Thrall",
      category: "creature",
      subcategory: null,
      descriptionText: "A thrall reshaped by a vampire master's curse.",
      traits: ["humanoid"],
      families: ["vampire"],
    })).toContain("undead_adjacent");

    expect(deriveRecordTags({
      name: "Manor Guard",
      category: "creature",
      subcategory: null,
      descriptionText: "A manor guard who patrols the estate grounds.",
      traits: ["human", "humanoid"],
      families: ["vampire"],
    })).toEqual(expect.arrayContaining(["profession_npc", "undead_adjacent"]));

    expect(deriveRecordTags({
      name: "Manor Guard",
      category: "creature",
      subcategory: null,
      descriptionText: "A manor guard who patrols the estate grounds.",
      traits: ["human", "humanoid"],
      families: ["vampire"],
    })).not.toContain("civic_npc");

    expect(deriveRecordTags({
      name: "Mythic Courtier",
      category: "creature",
      subcategory: null,
      descriptionText: "A courtier sustained by impossible necromancy.",
      traits: ["humanoid"],
      families: ["mythic", "lich"],
    })).toEqual(expect.arrayContaining(["profession_npc", "undead_adjacent"]));

    expect(deriveRecordTags({
      name: "Mythic Courtier",
      category: "creature",
      subcategory: null,
      descriptionText: "A courtier sustained by impossible necromancy.",
      traits: ["humanoid"],
      families: ["mythic", "lich"],
    })).not.toContain("civic_npc");

    expect(deriveRecordTags({
      name: "Bandit",
      category: "creature",
      subcategory: null,
      descriptionText: "Bandits waylay travelers and plunder their valuables before disappearing back to their wilderness hideouts.",
      traits: ["human", "humanoid"],
    })).toContain("combatant_npc");

    expect(deriveRecordTags({
      name: "Watch Officer",
      category: "creature",
      subcategory: null,
      descriptionText: "Often leading a small team of lower-ranking guards, watch officers patrol their assigned areas to maintain order and enforce laws.",
      traits: ["human", "humanoid"],
    })).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(deriveRecordTags({
      name: "Abbot of Abadar",
      category: "creature",
      subcategory: null,
      descriptionText: "The abbot runs the shrine and serves the community with counsel and order.",
      traits: ["human", "humanoid"],
    })).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(deriveRecordTags({
      name: "Traveling Apothecary",
      category: "creature",
      subcategory: null,
      descriptionText: "A respected apothecary and physician who serves the community by treating the sick.",
      traits: ["human", "humanoid"],
    })).toContain("profession_npc");

    expect(deriveRecordTags({
      name: "Adjutant Hellknight Armiger",
      category: "creature",
      subcategory: null,
      descriptionText: "An adjutant and armiger who fights alongside the order in disciplined battle formations.",
      traits: ["human", "humanoid"],
    })).toContain("combatant_npc");

    expect(deriveRecordTags({
      name: "Hellknight Sergeant",
      category: "creature",
      subcategory: null,
      descriptionText: "The sergeant leads disciplined enforcers in brutal close-quarters battle.",
      traits: ["human", "humanoid"],
    })).toContain("combatant_npc");

    expect(deriveRecordTags({
      name: "City Scribe",
      category: "creature",
      subcategory: null,
      descriptionText: "A human clerk who serves the community and keeps the peace while recording the daily business of the courthouse.",
      traits: ["human", "humanoid"],
    })).toContain("profession_npc");

    expect(deriveRecordTags({
      name: "Hryngar Forgepriest",
      category: "creature",
      subcategory: null,
      descriptionText: "The forgepriest proselytizes Droskar's teachings and punishes heretics for defiance.",
      traits: ["dwarf", "humanoid"],
    })).toContain("profession_npc");

    expect(deriveRecordTags({
      name: "Tree Singer",
      category: "creature",
      subcategory: null,
      descriptionText: "A humanoid primalist singer whose melodies stir trees and command plants.",
      traits: ["humanoid"],
    })).toContain("profession_npc");

    expect(deriveRecordTags({
      name: "Guild Engineer",
      category: "creature",
      subcategory: null,
      descriptionText: "A meticulous guild engineer keeps the city's lifts and aqueduct pumps working for the community.",
      traits: ["human", "humanoid"],
    })).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(deriveRecordTags({
      name: "Festival Lutenist",
      category: "creature",
      subcategory: null,
      descriptionText: "A celebrated lutenist performs at civic festivals and public ceremonies.",
      traits: ["human", "humanoid"],
    })).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(deriveRecordTags({
      name: "Hellknight Gaoler",
      category: "creature",
      subcategory: null,
      descriptionText: "A disciplined hellknight gaoler escorts prisoners and enforces the order's brutal routines.",
      traits: ["human", "humanoid"],
    })).toEqual(expect.arrayContaining(["profession_npc", "combatant_npc"]));

    expect(deriveRecordTags({
      name: "Hellknight Gaoler",
      category: "creature",
      subcategory: null,
      descriptionText: "A disciplined hellknight gaoler escorts prisoners and enforces the order's brutal routines.",
      traits: ["human", "humanoid"],
    })).not.toContain("civic_npc");

    expect(deriveRecordTags({
      name: "Veteran Noble",
      category: "creature",
      subcategory: null,
      descriptionText: "A veteran noble funds expeditions and stewards the city's public works.",
      traits: ["human", "humanoid"],
    })).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(deriveRecordTags({
      name: "Veteran Noble",
      category: "creature",
      subcategory: null,
      descriptionText: "A veteran noble funds expeditions and stewards the city's public works.",
      traits: ["human", "humanoid"],
    })).not.toContain("combatant_npc");

    expect(deriveRecordTags({
      name: "Gendarme",
      category: "creature",
      subcategory: null,
      descriptionText: "Powerful governments retain gendarmes to guard important magistrates and capture unusually dangerous criminals.",
      traits: ["human", "humanoid"],
    })).toContain("combatant_npc");

    expect(deriveRecordTags({
      name: "Line Infantry",
      category: "creature",
      subcategory: null,
      descriptionText: "Disciplined line infantry hold the battle line in ordered formations.",
      traits: ["human", "humanoid"],
    })).toContain("combatant_npc");

    expect(deriveRecordTags({
      name: "Unnamed Traveler",
      category: "creature",
      subcategory: null,
      descriptionText: "A traveler passes through town in search of lodging.",
      traits: ["human", "humanoid"],
    })).not.toEqual(expect.arrayContaining(["profession_npc", "civic_npc", "combatant_npc"]));
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
      name: "Living Mural",
      category: "creature",
      subcategory: null,
      descriptionText: "This two-dimensional mural has come to life as a mindless construct that peels itself from the wall to attack intruders.",
      traits: ["construct", "mindless"],
    })).toContain("living_artwork");

    expect(deriveRecordTags({
      name: "Masked Mourner",
      category: "creature",
      subcategory: null,
      descriptionText: "A solemn creature wearing a ceremonial mask and veiled face to hide its identity from the living.",
      traits: ["humanoid"],
    })).toContain("mask_motif");

    expect(deriveRecordTags({
      name: "Faceless Butcher",
      category: "creature",
      subcategory: null,
      descriptionText: "This faceless horror has a blank, featureless face and keeps stolen faces as trophies.",
      traits: ["aberration"],
    })).toContain("faceless_horror");

    expect(deriveRecordTags({
      name: "False Herald",
      category: "creature",
      subcategory: null,
      descriptionText: "The herald assumes a false identity, infiltrates courts, and impersonates priests to replace them.",
      traits: ["humanoid"],
    })).toContain("disguised_pretender");

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
      name: "Badger",
      category: "creature",
      subcategory: null,
      descriptionText: "A badger with dark facial markings and a striped muzzle.",
      traits: ["animal"],
    })).not.toContain("mask_motif");

    expect(deriveRecordTags({
      name: "Masked Duelist",
      category: "creature",
      subcategory: null,
      descriptionText: "A duelist wearing a bronze mask over a healthy and recognizable face.",
      traits: ["humanoid"],
    })).not.toContain("faceless_horror");

    expect(deriveRecordTags({
      name: "Taljjae",
      category: "creature",
      subcategory: null,
      descriptionText: "Taljjae is easily recognized due to its signature cloak and masks.",
      traits: ["fey"],
    })).toContain("mask_motif");

    expect(deriveRecordTags({
      name: "The Vanish Man",
      category: "creature",
      subcategory: null,
      descriptionText: "Variant faceless butcher.",
      traits: ["humanoid"],
    })).toContain("faceless_horror");

    expect(deriveRecordTags({
      name: "Chameleon Beast",
      category: "creature",
      subcategory: null,
      descriptionText: "A reptile that changes color to blend into its surroundings.",
      traits: ["animal"],
    })).not.toContain("disguised_pretender");

    expect(deriveRecordTags({
      name: "Goblin Igniter",
      category: "creature",
      subcategory: null,
      descriptionText: "Goblins think fire is a fun toy and admire anyone willing to burn down a barn for sport.",
      traits: ["goblin", "humanoid"],
    })).not.toContain("living_toy");

    expect(deriveRecordTags({
      name: "Blood Painter",
      category: "creature",
      subcategory: null,
      descriptionText: "An alien artist stalks battlefields in search of pigments and living canvases for its gruesome paintings.",
      traits: ["aberration"],
    })).not.toContain("living_artwork");

    expect(deriveRecordTags({
      name: "Animated Panoply",
      category: "creature",
      subcategory: null,
      descriptionText: "Granted a semblance of life through the use of rituals or other strange magic, animated objects take many forms and serve as guardians.",
      traits: ["construct", "mindless"],
    })).not.toContain("living_artwork");

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
      name: "Sakugami",
      category: "creature",
      subcategory: null,
      descriptionText: "Kami are divine nature spirits from the lands of Tian Xia, far to the east of the Inner Sea region.",
      traits: ["kami", "spirit", "wood"],
    })).not.toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting", "undead_adjacent"]));

    expect(deriveRecordTags({
      name: "Blossom Kami",
      category: "creature",
      subcategory: null,
      descriptionText: "The site of a new village might be chosen due to its proximity to an ancient wisteria, or a temple might be carefully constructed around a single young plum.",
      traits: ["kami", "spirit", "wood"],
    })).not.toEqual(expect.arrayContaining(["living_artwork", "mountain_setting", "rural_setting", "spawn_creator", "undead_adjacent"]));

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
    })).not.toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(deriveRecordTags({
      name: "Animated Armor",
      category: "creature",
      subcategory: null,
      descriptionText: "Animated armor serves as guardians and training partners in martial academies.",
      traits: ["construct", "mindless"],
    })).not.toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(deriveRecordTags({
      name: "Vanth Guardian Flock",
      category: "creature",
      subcategory: null,
      descriptionText: "Vanth psychopomps are eternal guardians of the cycle of life and death.",
      traits: ["monitor", "psychopomp", "troop"],
    })).not.toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));

    expect(deriveRecordTags({
      name: "Clockwork Soldier",
      category: "creature",
      subcategory: null,
      descriptionText: "These diligent machines guard their assigned posts tirelessly.",
      traits: ["clockwork", "construct", "mindless"],
    })).not.toContain("combatant_npc");

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

    expect(deriveRecordTags({
      name: "Generic Drake",
      category: "creature",
      subcategory: null,
      descriptionText: "Ravenous, bestial, and driven by instinct, drakes are draconic monsters. A single rampage of river drakes can quickly lay waste to a waterside village, and roving rampages of desert drakes are a plague to caravan traders. While it is generally easy for breeders to incubate the eggs of desert or jungle drakes or river drakes, many societies do not condone the trade of drake eggs and criminalize those who engage in it.",
      traits: ["dragon"],
    })).not.toEqual(expect.arrayContaining(["desert_setting", "forest_setting", "rural_setting", "spawn_creator", "urban_setting"]));

    expect(deriveRecordTags({
      name: "Divine Punisher",
      category: "creature",
      subcategory: null,
      descriptionText: "Such creatures are drawn to target not a single misbehaving follower, but entire villages, cities, or towns that have turned their backs on their gods.",
      traits: ["dragon"],
    })).not.toEqual(expect.arrayContaining(["rural_setting", "urban_setting"]));

    expect(deriveRecordTags({
      name: "Anugobu Apprentice",
      category: "creature",
      subcategory: null,
      descriptionText: "Renowned in Tengah folklore as divinely gifted artisans and crafters, anugobus are a curious group of tiny humanoids native to the central islands of Minata. Countless stories describe anugobus sneaking around Minatan cities to tinker with and improve upon other humanoids' structures and architecture. In reality, anugobus are as diverse as any species of humanoid, though they all have innate gifts for mending and construction, and they can walk on walls and ceilings. Nearly all verifiable anugobu encounters have taken place on the isles of Minata. Any project can be an anugobu's wonder, from repairing a majestic cathedral to hunting a particularly dangerous animal to exploring a mysterious cave.",
      traits: [],
    })).toEqual(expect.arrayContaining(["urban_setting"]));

    expect(deriveRecordTags({
      name: "Anugobu Apprentice",
      category: "creature",
      subcategory: null,
      descriptionText: "Renowned in Tengah folklore as divinely gifted artisans and crafters, anugobus are a curious group of tiny humanoids native to the central islands of Minata. Countless stories describe anugobus sneaking around Minatan cities to tinker with and improve upon other humanoids' structures and architecture. In reality, anugobus are as diverse as any species of humanoid, though they all have innate gifts for mending and construction, and they can walk on walls and ceilings. Nearly all verifiable anugobu encounters have taken place on the isles of Minata. Any project can be an anugobu's wonder, from repairing a majestic cathedral to hunting a particularly dangerous animal to exploring a mysterious cave.",
      traits: [],
    })).not.toEqual(expect.arrayContaining(["temple_setting", "underground_setting"]));

    expect(deriveRecordTags({
      name: "Troll King",
      category: "creature",
      subcategory: null,
      descriptionText: "Trolls are solitary hunters, for their wickedness is anathema even to other giants. They occasionally roam in small gangs of two to four, but only when prey is plentiful or a particularly strong counterforce has broached their hunting grounds. In rare instances, an old and powerful troll comes to lead small tribes of trolls. A wide variety of trolls exist, from the terrible monster traditionally associated with the name to the water-dwelling scrag and hybrid flood troll. Regional variations exist as well-mountain trolls among stony peaks, for instance, or moss trolls in swampy bayous-but all share the same trademark regenerative powers and insatiable thirst for blood.",
      traits: ["chaotic", "evil", "giant", "troll"],
    })).not.toEqual(expect.arrayContaining(["mountain_setting", "swamp_setting"]));

    expect(deriveRecordTags({
      name: "Watchtower Wraith",
      category: "creature",
      subcategory: null,
      descriptionText: "Wraiths may form packs with others of their kind in places where death and mayhem are commonplace-countrysides ravaged by war, metropolitan underworlds run by criminal overlords, or sites of fiendish cultic rituals. Ruins, sewers, and abandoned buildings provide sanctuary for wraiths during the day, as the creatures hunt exclusively at night or in dark places. Wraiths are smart enough to take advantage of their incorporeality in combat, so they keep to tortuous caverns or structures with hallways and avoid open areas.",
      traits: ["undead"],
    })).not.toEqual(expect.arrayContaining(["fortress_setting", "urban_setting", "underground_setting", "temple_setting"]));

    expect(deriveRecordTags({
      name: "Triton",
      category: "creature",
      subcategory: null,
      descriptionText: "Many tritons see themselves as defenders of the deep, dedicating their lives to protecting the inhabitants of the seas from evil creatures and intruders. Tritons live in natural-grown villages on the sea floor, forging dwellings out of colorful coral reefs, in rift valleys heated by volcanic activity, and even in underwater canyons. They like to decorate their homes with aquatic plants, bioluminescent fish, and attractive trinkets recovered from shipwrecks.",
      traits: ["amphibious", "humanoid", "water"],
    })).not.toEqual(expect.arrayContaining(["canyon_setting", "volcanic_setting"]));
  });

  it("derives animated object and animated statue tags without promoting the family name", () => {
    expect(deriveRecordTags({
      name: "Animated Armor",
      category: "creature",
      subcategory: null,
      descriptionText: "An armored construct animated by magic to guard a tomb.",
      traits: ["construct", "mindless"],
    })).toContain("animated_object");

    expect(deriveRecordTags({
      name: "Animated Cookware Swarm",
      category: "creature",
      subcategory: null,
      descriptionText: "A swarm of animated cookware clatters through the kitchen and lunges at intruders.",
      traits: ["construct", "swarm"],
    })).toContain("animated_object");

    expect(deriveRecordTags({
      name: "Animated Tea Cart",
      category: "creature",
      subcategory: null,
      descriptionText: "A construct tea cart rattles to life and careens through the parlor.",
      traits: ["construct", "mindless"],
    })).toContain("animated_object");

    expect(deriveRecordTags({
      name: "Scarecrow",
      category: "creature",
      subcategory: null,
      descriptionText: "This cruel construct is an animated scarecrow stuffed with spite and propped in a lonely field.",
      traits: ["construct", "mindless"],
    })).toContain("animated_object");

    expect(deriveRecordTags({
      name: "Giant Animated Statue",
      category: "creature",
      subcategory: null,
      descriptionText: "A giant statue animated to serve as a guardian at the gate.",
      traits: ["construct", "mindless"],
    })).toContain("animated_statue");

    expect(deriveRecordTags({
      name: "Shadowbound Monk Statue",
      category: "creature",
      subcategory: null,
      descriptionText: "A carved statue animated to stand watch like a patient monk.",
      traits: ["construct", "mindless"],
    })).toContain("animated_statue");

    expect(deriveRecordTags({
      name: "Old Man Statue",
      category: "creature",
      subcategory: null,
      descriptionText: "A divine warden of Irori disguised as an old statue.",
      traits: ["construct", "mindless"],
    })).toContain("animated_statue");
  });

  it("avoids figurative animation, mimics, and ordinary stone animals", () => {
    expect(deriveRecordTags({
      name: "Animated Debate",
      category: "creature",
      subcategory: null,
      descriptionText: "The animated debate between scholars grows louder, but nothing is literally brought to life.",
      traits: ["humanoid"],
    })).not.toEqual(expect.arrayContaining(["animated_object", "animated_statue"]));

    expect(deriveRecordTags({
      name: "Mimic Chest",
      category: "creature",
      subcategory: null,
      descriptionText: "A mimic disguised as a treasure chest waits for prey.",
      traits: ["aberration"],
    })).not.toEqual(expect.arrayContaining(["animated_object", "animated_statue"]));

    expect(deriveRecordTags({
      name: "Stone Lion Cub",
      category: "creature",
      subcategory: null,
      descriptionText: "A small stone lion cub prowls the sanctuary like an ornament come to life.",
      traits: ["animal"],
    })).not.toEqual(expect.arrayContaining(["animated_object", "animated_statue"]));
  });

  it("derives ontology clusters and threat profiles without mirroring native traits", () => {
    expect(deriveRecordTags({
      name: "Haunted Courtier",
      category: "creature",
      subcategory: null,
      descriptionText: "A courtier bound to unlife by a lich patron.",
      traits: ["humanoid"],
      families: ["lich"],
    })).toContain("undead_adjacent");

    expect(deriveRecordTags({
      name: "Body Snatcher",
      category: "creature",
      subcategory: null,
      descriptionText: "This parasite can possess a victim and take control of the victim's body from within.",
      traits: ["aberration"],
    })).toContain("possession_threat");

    expect(deriveRecordTags({
      name: "Soul Drinker",
      category: "creature",
      subcategory: null,
      descriptionText: "The fiend siphons souls and drains life from anyone trapped in its shadow.",
      traits: ["fiend"],
    })).toContain("life_drain_threat");

    expect(deriveRecordTags({
      name: "Vampiric Reaver",
      category: "creature",
      subcategory: null,
      descriptionText: "This predator drains blood and life force from victims, feeding on blood to renew itself.",
      traits: ["undead"],
    })).toContain("life_drain_threat");

    expect(deriveRecordTags({
      name: "Brood Mother",
      category: "creature",
      subcategory: null,
      descriptionText: "The brood mother implants eggs in living hosts, and fresh horrors burst from the host days later.",
      traits: ["aberration"],
    })).toContain("spawn_creator");

    expect(deriveRecordTags({
      name: "Carrion Brood Hatcher",
      category: "creature",
      subcategory: null,
      descriptionText: "The brood hatches inside living hosts and creates offspring that infest nearby victims.",
      traits: ["aberration"],
    })).toContain("spawn_creator");

    expect(deriveRecordTags({
      name: "Stone Gaze Basilisk",
      category: "creature",
      subcategory: null,
      descriptionText: "Its gaze can petrify trespassers and turn creatures to stone where they stand.",
      traits: ["beast"],
    })).toContain("petrification_threat");

    expect(deriveRecordTags({
      name: "Marsh Troll",
      category: "creature",
      subcategory: null,
      descriptionText: "The monster's regeneration can only be suppressed with acid or fire before it can be killed.",
      traits: ["giant"],
    })).toContain("regeneration_threat");

    expect(deriveRecordTags({
      name: "Ash Troll",
      category: "creature",
      subcategory: null,
      descriptionText: "Its fast healing lets it recover each round, and the creature can only be destroyed by acid or fire.",
      traits: ["giant"],
    })).toContain("regeneration_threat");

    expect(deriveRecordTags({
      name: "Web Lurker",
      category: "creature",
      subcategory: null,
      descriptionText: "An ambush predator that snares prey in sticky webs, leaves them webbed, and drags prey back to its lair.",
      traits: ["animal"],
    })).toContain("ambush_grabber");

    expect(deriveRecordTags({
      name: "Bog Ambusher",
      category: "creature",
      subcategory: null,
      descriptionText: "An ambush predator that constricts prey, swallows whole, and drags prey into the marsh.",
      traits: ["animal"],
    })).toContain("ambush_grabber");

    expect(deriveRecordTags({
      name: "Crypt Sentinel",
      category: "creature",
      subcategory: null,
      descriptionText: "A tireless guardian lurks within the tomb's lower crypts and watches over the buried dead.",
      traits: ["construct"],
    })).toContain("graveyard_setting");

    expect(deriveRecordTags({
      name: "Canopy Snatcher",
      category: "creature",
      subcategory: null,
      descriptionText: "Lying in wait among the branches, this predator snatches prey and carries off struggling victims into the canopy.",
      traits: ["animal"],
    })).toContain("ambush_grabber");

    expect(deriveRecordTags({
      name: "Playful Naiad",
      category: "creature",
      subcategory: null,
      descriptionText: "A playful fey guardian of clear pools and shaded brooks.",
      traits: ["fey", "water"],
    })).not.toEqual(expect.arrayContaining(["possession_threat", "life_drain_threat", "spawn_creator", "petrification_threat", "regeneration_threat", "ambush_grabber"]));

    expect(deriveRecordTags({
      name: "Courthouse Scribe",
      category: "creature",
      subcategory: null,
      descriptionText: "A human clerk who keeps records and works at the courthouse.",
      traits: ["human", "humanoid"],
    })).not.toContain("combatant_npc");

    expect(deriveRecordTags({
      name: "Stone Lion Cub",
      category: "creature",
      subcategory: null,
      descriptionText: "A small stone lion cub prowls the sanctuary like an ornament come to life.",
      traits: ["animal"],
    })).not.toContain("ambush_grabber");
  });
});
