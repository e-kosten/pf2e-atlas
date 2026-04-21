import type { DerivedTagLegacySeedMigrationCategory } from "../../domain/derived-tag-types.js";

// Temporary migration bucket for legacy carried-over "seed" records that are still
// applied live but need manual review to become either true exemplars or assignments.
export const CREATURE_DERIVED_TAG_LEGACY_SEED_MIGRATIONS = {
  category: "creature",
  tags: [
    {
      tag: "faceless_horror",
      includeRecords: [
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Arcane)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Divine)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Martial)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Occult)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Primal)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Skilled)",
        },
      ],
    },
    {
      tag: "island_setting",
      includeRecords: [
        {
          pack: "age-of-ashes-bestiary",
          name: "Hermean Mutant",
        },
        {
          pack: "age-of-ashes-bestiary",
          name: "Mengkare",
        },
        {
          pack: "age-of-ashes-bestiary",
          name: "Dragonshard Guardian",
        },
      ],
    },
    {
      tag: "nautical_setting",
      includeRecords: [
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Black Whale Guard (F3)",
        },
      ],
    },
    {
      tag: "organized_undead_society_setting",
      includeRecords: [
        {
          pack: "blood-lords-bestiary",
          name: "Bloodshroud",
        },
        {
          pack: "blood-lords-bestiary",
          name: "Charghar",
        },
        {
          pack: "blood-lords-bestiary",
          name: "Cobblebone Swarm",
        },
        {
          pack: "blood-lords-bestiary",
          name: "Necromunculus",
        },
      ],
    },
    {
      tag: "undead_war_torn_region_setting",
      includeRecords: [
        {
          pack: "book-of-the-dead-bestiary",
          name: "Gallowdead",
        },
        {
          pack: "claws-of-the-tyrant-bestiary",
          name: "Chernasardo Ranger",
        },
        {
          pack: "claws-of-the-tyrant-bestiary",
          name: "Commander Arsiella Dei",
        },
        {
          pack: "claws-of-the-tyrant-bestiary",
          name: "Knight Reclaimant",
        },
        {
          pack: "claws-of-the-tyrant-bestiary",
          name: "Raised Cavalry",
        },
      ],
    },
    {
      tag: "tian_xia_setting",
      includeRecords: [
        {
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Amihan",
        },
        {
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Caustic Monitor",
        },
        {
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Jaiban",
        },
        {
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Nai Yan Fei",
        },
      ],
    },
    {
      tag: "urban_setting",
      includeRecords: [
        {
          pack: "battlecry-bestiary",
          name: "Ofalth Stampede",
        },
        {
          pack: "blood-lords-bestiary",
          name: "Theater Phantasm",
        },
        {
          pack: "book-of-the-dead-bestiary",
          name: "Bone Croupier",
        },
      ],
    },
    {
      tag: "battlefield_setting",
      includeRecords: [
        {
          pack: "battlecry-bestiary",
          name: "Archer Regiment",
        },
        {
          pack: "battlecry-bestiary",
          name: "Dromaar Company",
        },
      ],
    },
    {
      tag: "small_settlement_setting",
      includeRecords: [
        {
          pack: "extinction-curse-bestiary",
          name: "Shoony Hierarch",
        },
        {
          pack: "extinction-curse-bestiary",
          name: "Shoony Militia Member",
        },
      ],
    },
    {
      tag: "desert_setting",
      includeRecords: [
        {
          pack: "battlecry-bestiary",
          name: "Qadiran Camel Corps",
        },
        {
          pack: "book-of-the-dead-bestiary",
          name: "Mummy Prophet Of Set",
        },
      ],
    },
    {
      tag: "rural_setting",
      includeRecords: [
        {
          pack: "fall-of-plaguestone-bestiary",
          name: "Drunken Farmer",
        },
        {
          pack: "book-of-the-dead-bestiary",
          name: "Death Coach",
        },
      ],
    },
    {
      tag: "profession_npc",
      includeRecords: [
        {
          pack: "pathfinder-npc-core",
          name: "Arms Dealer",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Artillerist",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Astronomer",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Beast Tamer",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Catfolk Name Collector",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Chronicler",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Construction Worker",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Court Historian",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Courtesan",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Dockhand",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Driver",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Dromaar Lorekeeper",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Envoy",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Expedition Leader",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Gunsmith",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Halfling Head Chef",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Halfling Yarnspinner",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Harrow Reader",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Innkeeper",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Local Herbalist",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Messenger",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Miner",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Natural Scientist",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Orc Agriculturist",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Orc Gamekeeper",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Dwarf Smith",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "Captain Lamperia Bane",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "False Hellbreaker",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "Hecatinia",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Changeling Hellknight",
        },
        {
          pack: "revenge-of-the-runelords-bestiary",
          name: "Lograsi",
        },
        {
          pack: "age-of-ashes-bestiary",
          name: "Bloody Blade Mercenary",
        },
        {
          pack: "age-of-ashes-bestiary",
          name: "Ekujae Guardian",
        },
        {
          pack: "age-of-ashes-bestiary",
          name: "Scarlet Triad Boss",
        },
        {
          pack: "age-of-ashes-bestiary",
          name: "Scarlet Triad Sniper",
        },
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Norgorberite Poisoner",
        },
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Skinsaw Murderer",
        },
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Starwatch Commando",
        },
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Sleepless Sun Veteran",
        },
        {
          pack: "blood-lords-bestiary",
          name: "Nwanyian Archer",
        },
        {
          pack: "blog-bestiary",
          name: "Urok",
        },
        {
          pack: "claws-of-the-tyrant-bestiary",
          name: "Commander Arsiella Dei",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "Adjutant Hellknight Armiger",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "Isgeri Slayer",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "Jordis Serelli",
        },
        {
          pack: "kingmaker-bestiary",
          name: "Cleansed Cultist",
        },
        {
          pack: "kingmaker-bestiary",
          name: "False Priestess",
        },
        {
          pack: "kingmaker-bestiary",
          name: "Thresholder Disciple",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Virtuous Defender",
        },
        {
          pack: "menace-under-otari-bestiary",
          name: "Drow Priestess (BB)",
        },
        {
          pack: "menace-under-otari-bestiary",
          name: "Kobold Boss Zolgran (BB)",
        },
        {
          pack: "outlaws-of-alkenstar-bestiary",
          name: "Corrupt Shieldmarshal (Clan Pistol)",
        },
        {
          pack: "outlaws-of-alkenstar-bestiary",
          name: "Gilded Gunner Assassin",
        },
        {
          pack: "outlaws-of-alkenstar-bestiary",
          name: "Gilded Gunner Goon",
        },
        {
          pack: "pathfinder-bestiary-3",
          name: "Android Infiltrator",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Cultist",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Demonbane Warrior",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Dwarf General",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Enigmatic Conspirant",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Exiled Revolutionary",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Infantry Soldier",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Knight",
        },
        {
          pack: "prey-for-death-bestiary",
          name: "Berserker Ordulf",
        },
        {
          pack: "prey-for-death-bestiary",
          name: "Ordulf Bladecaller",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Mercenary Enforcer",
        },
        {
          pack: "seven-dooms-for-sandpoint-bestiary",
          name: "Devil's Disciple",
        },
        {
          pack: "sky-kings-tomb-bestiary",
          name: "Hryngar King's Agent",
        },
        {
          pack: "strength-of-thousands-bestiary",
          name: "Sun Warrior Brigade",
        },
        {
          pack: "the-enmity-cycle-bestiary",
          name: "Scorching Sun Cultist",
        },
        {
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Weapon Master",
        },
      ],
    },
    {
      tag: "civic_npc",
      includeRecords: [
        {
          pack: "pathfinder-npc-core",
          name: "Astronomer",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Beast Tamer",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Catfolk Name Collector",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Chronicler",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Construction Worker",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Court Historian",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Courtesan",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Dockhand",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Driver",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Dromaar Lorekeeper",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Envoy",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Expedition Leader",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Gunsmith",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Halfling Head Chef",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Halfling Yarnspinner",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Harrow Reader",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Innkeeper",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Local Herbalist",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Messenger",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Miner",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Natural Scientist",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Orc Agriculturist",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Orc Gamekeeper",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Dwarf Smith",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "Captain Lamperia Bane",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "Hecatinia",
        },
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Starwatch Commando",
        },
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Sleepless Sun Veteran",
        },
        {
          pack: "blood-lords-bestiary",
          name: "Firebrand Bastion",
        },
        {
          pack: "blood-lords-bestiary",
          name: "Nwanyian Archer",
        },
        {
          pack: "claws-of-the-tyrant-bestiary",
          name: "Commander Arsiella Dei",
        },
        {
          pack: "gatewalkers-bestiary",
          name: "Oaksteward Enforcer",
        },
        {
          pack: "gatewalkers-bestiary",
          name: "Oaksteward Enforcer (Gatehouse)",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "Adjutant Hellknight Armiger",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "Jordis Serelli",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Changeling Hellknight",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Virtuous Defender",
        },
        {
          pack: "outlaws-of-alkenstar-bestiary",
          name: "Corrupt Shieldmarshal (Clan Pistol)",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Dwarf General",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Enigmatic Conspirant",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Exiled Revolutionary",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Knight",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Mercenary Enforcer",
        },
        {
          pack: "sky-kings-tomb-bestiary",
          name: "Hryngar King's Agent",
        },
        {
          pack: "age-of-ashes-bestiary",
          name: "Ekujae Guardian",
        },
      ],
    },
    {
      tag: "enforcer_npc",
      includeRecords: [
        {
          pack: "pathfinder-npc-core",
          name: "Artillerist",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Heavy Cavalry",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Hellknight Cavalry Brigade",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Hobgoblin Battalion",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Hobgoblin Spellbreaker",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Kholo Pragmatist",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Kobold Egg Guardian",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Legbreaker",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Mage Knight",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Mixed Martial Artist",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Musketeer",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Orc Commander",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Orc Veteran",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Orc Veteran Master",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Orc Skullcrushers",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Knight",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Dwarf Battalion",
        },
        {
          pack: "age-of-ashes-bestiary",
          name: "Scarlet Triad Boss",
        },
        {
          pack: "age-of-ashes-bestiary",
          name: "Scarlet Triad Enforcer",
        },
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Sleepless Sun Veteran",
        },
        {
          pack: "blood-lords-bestiary",
          name: "Nwanyian Archer",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "Captain Lamperia Bane",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "False Hellbreaker",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "Hecatinia",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Changeling Hellknight",
        },
        {
          pack: "kingmaker-bestiary",
          name: "Thresholder Disciple",
        },
        {
          pack: "prey-for-death-bestiary",
          name: "Berserker Ordulf",
        },
        {
          pack: "revenge-of-the-runelords-bestiary",
          name: "Lograsi",
        },
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Copper Hand Rogue",
        },
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Gref",
        },
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Kekker",
        },
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Starwatch Commando",
        },
        {
          pack: "age-of-ashes-bestiary",
          name: "Bloody Blade Mercenary",
        },
        {
          pack: "age-of-ashes-bestiary",
          name: "Dmiri Yoltosha",
        },
        {
          pack: "age-of-ashes-bestiary",
          name: "Ekujae Guardian",
        },
        {
          pack: "age-of-ashes-bestiary",
          name: "Laslunn",
        },
        {
          pack: "age-of-ashes-bestiary",
          name: "Scarlet Triad Sniper",
        },
        {
          pack: "claws-of-the-tyrant-bestiary",
          name: "Commander Arsiella Dei",
        },
        {
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Dark Talon Kobold",
        },
        {
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Foolish Hunter",
        },
        {
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Gurtlekep",
        },
        {
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Kapmek",
        },
        {
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Ygrik",
        },
        {
          pack: "fall-of-plaguestone-bestiary",
          name: "Graytusk",
        },
        {
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Archery Specialist",
        },
        {
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Master Xun",
        },
        {
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Ran-to (Level 20)",
        },
        {
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Troff Frostknuckles",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "Chelaxian Loyalist",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "Isgeri Slayer",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "Jordis Serelli",
        },
        {
          pack: "kingmaker-bestiary",
          name: "Ameon Trask",
        },
        {
          pack: "kingmaker-bestiary",
          name: "Villamor Koth",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Anadi Hunter",
        },
        {
          pack: "outlaws-of-alkenstar-bestiary",
          name: "Corrupt Shieldmarshal (Clan Pistol)",
        },
        {
          pack: "outlaws-of-alkenstar-bestiary",
          name: "Gilded Gunner Assassin",
        },
        {
          pack: "outlaws-of-alkenstar-bestiary",
          name: "Gilded Gunner Goon",
        },
        {
          pack: "outlaws-of-alkenstar-bestiary",
          name: "Gilded Gunner Safecracker",
        },
        {
          pack: "pathfinder-bestiary-3",
          name: "Android Infiltrator",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Cultist",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Gang Leader",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Hero Hunter",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Monster Hunter",
        },
        {
          pack: "pathfinder-npc-core",
          name: "Wealthy Vigilante",
        },
        {
          pack: "prey-for-death-bestiary",
          name: "Gorumite Veteran",
        },
        {
          pack: "quest-for-the-frozen-flame-bestiary",
          name: "Graylok Ambusher",
        },
        {
          pack: "rusthenge-bestiary",
          name: "Fallen Acolyte",
        },
        {
          pack: "rusthenge-bestiary",
          name: "Rustsworn Initiate",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Mercenary Enforcer",
        },
        {
          pack: "seven-dooms-for-sandpoint-bestiary",
          name: "Devil's Disciple",
        },
        {
          pack: "shadows-at-sundown-bestiary",
          name: "Yniesse Zenderholm",
        },
        {
          pack: "sky-kings-tomb-bestiary",
          name: "Blacknoon Apprentice",
        },
        {
          pack: "sky-kings-tomb-bestiary",
          name: "Hrungul Ironeye",
        },
        {
          pack: "sky-kings-tomb-bestiary",
          name: "Hryngar King's Agent",
        },
        {
          pack: "strength-of-thousands-bestiary",
          name: "Crimson Acolyte",
        },
        {
          pack: "strength-of-thousands-bestiary",
          name: "Cyclops Bully",
        },
        {
          pack: "the-enmity-cycle-bestiary",
          name: "Scorching Sun Cultist",
        },
        {
          pack: "the-slithering-bestiary",
          name: "Aspis Technician",
        },
        {
          pack: "troubles-in-otari-bestiary",
          name: "Orc Scrapper",
        },
        {
          pack: "triumph-of-the-tusk-bestiary",
          name: "Molog",
        },
        {
          pack: "triumph-of-the-tusk-bestiary",
          name: "Orc Hunter",
        },
        {
          pack: "triumph-of-the-tusk-bestiary",
          name: "Wingripper Wyvern Rider",
        },
        {
          pack: "wardens-of-wildwood-bestiary",
          name: "Hateful Logger",
        },
        {
          pack: "battlecry-bestiary",
          name: "Dromaar Company",
        },
        {
          pack: "blog-bestiary",
          name: "Urok",
        },
      ],
    },
    {
      tag: "sinspawn_family",
      includeRecords: [
        {
          pack: "pathfinder-monster-core",
          name: "Envyspawn",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Gluttonyspawn",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Greedspawn",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Lustspawn",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Pridespawn",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Slothspawn",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Wrathspawn",
        },
        {
          pack: "pathfinder-monster-core-2",
          name: "Atrixyl",
        },
        {
          pack: "battlecry-bestiary",
          name: "Sinswarm",
        },
        {
          pack: "revenge-of-the-runelords-bestiary",
          name: "Greedspawn",
        },
        {
          pack: "revenge-of-the-runelords-bestiary",
          name: "Pridespawn Sentinel",
        },
        {
          pack: "revenge-of-the-runelords-bestiary",
          name: "Knight of Malice",
        },
        {
          pack: "seven-dooms-for-sandpoint-bestiary",
          name: "Greedspawn",
        },
      ],
    },
    {
      tag: "dragon_spellcaster",
      includeRecords: [
        {
          pack: "lost-omens-bestiary",
          name: "Adamantine Archdragon (Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Adamantine Dragon (Adult, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Adamantine Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Adamantine Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Barrage Archdragon (Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Barrage Dragon (Adult, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Barrage Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Barrage Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Cloud Archdragon (Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Cloud Dragon (Adult, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Cloud Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Cloud Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Conspirator Archdragon (Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Conspirator Dragon (Adult, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Conspirator Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Conspirator Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Crystal Archdragon (Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Crystal Dragon (Adult, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Crystal Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Crystal Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Delight Archdragon (Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Delight Dragon (Adult, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Delight Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Delight Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Diabolic Archdragon (Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Diabolic Dragon (Adult, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Diabolic Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Diabolic Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Empyreal Archdragon (Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Empyreal Dragon (Adult, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Empyreal Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Empyreal Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Fortune Archdragon (Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Fortune Dragon (Adult, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Fortune Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Fortune Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Mocking Archdragon (Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Mocking Dragon (Adult, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Mocking Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Mocking Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Oath Archdragon (Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Oath Dragon (Adult, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Oath Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Oath Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Omen Archdragon (Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Omen Dragon (Adult, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Omen Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Omen Dragon (Young, Spellcaster)",
        },
        {
          pack: "pathfinder-bestiary",
          name: "Red Dragon (Adult, Spellcaster)",
        },
        {
          pack: "pathfinder-bestiary",
          name: "Red Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "pathfinder-bestiary",
          name: "Red Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Requiem Archdragon (Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core-2",
          name: "Requiem Dragon (Adult, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core-2",
          name: "Requiem Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core-2",
          name: "Requiem Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Rune Archdragon (Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core-2",
          name: "Rune Dragon (Adult, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core-2",
          name: "Rune Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core-2",
          name: "Rune Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Sage Archdragon (Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Sage Dragon (Adult, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Sage Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Sage Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Sky Archdragon (Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Sky Dragon (Adult, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Sky Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Sky Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Stormcrown Archdragon (Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Stormcrown Dragon (Adult, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Stormcrown Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Stormcrown Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Time Archdragon (Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Time Dragon (Adult, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Time Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Time Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Umbral Archdragon (Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Umbral Dragon (Adult, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Umbral Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Umbral Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Vizier Archdragon (Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Vizier Dragon (Adult, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Vizier Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Vizier Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Vorpal Archdragon (Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Vorpal Dragon (Adult, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Vorpal Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Vorpal Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Wailing Archdragon (Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Wailing Dragon (Adult, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Wailing Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Wailing Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Whisper Archdragon (Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core-2",
          name: "Whisper Dragon (Adult, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core-2",
          name: "Whisper Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core-2",
          name: "Whisper Dragon (Young, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Wish Archdragon (Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Wish Dragon (Adult, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Wish Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Wish Dragon (Young, Spellcaster)",
        },
      ],
    },
    {
      tag: "regeneration_threat",
      includeRecords: [
        {
          pack: "pathfinder-monster-core-2",
          name: "Cavern Troll",
        },
        {
          pack: "menace-under-otari-bestiary",
          name: "Forest Troll (BB)",
        },
        {
          pack: "stolen-fate-bestiary",
          name: "Gegnir",
        },
        {
          pack: "kingmaker-bestiary",
          name: "Gurija",
        },
        {
          pack: "kingmaker-bestiary",
          name: "Hargulka",
        },
        {
          pack: "pathfinder-monster-core-2",
          name: "Jotund Troll",
        },
        {
          pack: "triumph-of-the-tusk-bestiary",
          name: "Marguk Cleftneck",
        },
        {
          pack: "rage-of-elements-bestiary",
          name: "Nightwood Guardian",
        },
        {
          pack: "pathfinder-monster-core-2",
          name: "Two-Headed Troll",
        },
      ],
    },
    {
      tag: "disguised_pretender",
      includeRecords: [
        {
          pack: "pathfinder-npc-core",
          name: "Master of Disguise",
        },
        {
          pack: "book-of-the-dead-bestiary",
          name: "Ecorche",
        },
        {
          pack: "kingmaker-bestiary",
          name: "Virthad",
        },
        {
          pack: "kingmaker-bestiary",
          name: "Fetch Stalker",
        },
        {
          pack: "pathfinder-bestiary",
          name: "Doppelganger",
        },
        {
          pack: "pathfinder-bestiary",
          name: "Green Hag",
        },
        {
          pack: "pathfinder-bestiary",
          name: "Raja Rakshasa",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Cuckoo Hag",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Gimmerling",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-bo Grunt",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-bo Trickster",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Arcane)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Divine)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Martial)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Occult)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Primal)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Skilled)",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Conspirator Archdragon",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Conspirator Archdragon (Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Conspirator Dragon (Adult, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Conspirator Dragon (Ancient)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Conspirator Dragon (Ancient, Spellcaster)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Conspirator Dragon (Young)",
        },
        {
          pack: "pathfinder-monster-core",
          name: "Conspirator Dragon (Young, Spellcaster)",
        },
      ],
    },
    {
      tag: "animated_object",
      includeRecords: [
        {
          pack: "age-of-ashes-bestiary",
          name: "Animated Dragonstorm",
        },
        {
          pack: "blood-lords-bestiary",
          name: "Animated Fireplace",
        },
        {
          pack: "curtain-call-bestiary",
          name: "Animated Boiler",
        },
        {
          pack: "gatewalkers-bestiary",
          name: "Quarry Construct",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Animated Bamboo Figurine",
        },
        {
          pack: "lost-omens-bestiary",
          name: "Animated Kite",
        },
        {
          pack: "myth-speaker-bestiary",
          name: "Animated Panoply",
        },
        {
          pack: "one-shot-bestiary",
          name: "Soulbound Guardian",
        },
        {
          pack: "outlaws-of-alkenstar-bestiary",
          name: "Glass Elephant",
        },
        {
          pack: "pathfinder-monster-core-2",
          name: "Animated Trebuchet",
        },
        {
          pack: "quest-for-the-frozen-flame-bestiary",
          name: "Weykoward",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Animated Axe",
        },
      ],
    },
  ],
} satisfies DerivedTagLegacySeedMigrationCategory;
