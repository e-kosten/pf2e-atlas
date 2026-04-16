import type { DerivedTagAuthoredCategoryOntology } from "../../types.js";

export const CREATURE_DERIVED_TAG_ONTOLOGY = {
  category: "creature",
  families: {
    setting: {
      description: "Creature environment and encounter-setting tags, covering habitats, hunting grounds, patrol zones, and other scenes where a creature predictably appears.",
      variantInheritance: true,
      tags: [
        {
          tag: "aquatic_setting",
          description: "Strongly associated with open water, underwater spaces, or aquatic environments.",
          assignmentMode: "hybrid"
        },
        {
          tag: "freshwater_setting",
          description: "Strongly associated with rivers, lakes, ponds, streams, springs, or other inland waters.",
          assignmentMode: "hybrid"
        },
        {
          tag: "coastal_setting",
          description: "Strongly associated with coasts, shores, reefs, or littoral edges.",
          assignmentMode: "hybrid"
        },
        {
          tag: "astral_setting",
          description: "Strongly associated with Astral Plane scenes, silver-void travel, or stable portal routes.",
          assignmentMode: "hybrid"
        },
        {
          tag: "ethereal_setting",
          description: "Strongly associated with the Ethereal Plane, its native wildlife, or recurring hunting and travel routes through it.",
          assignmentMode: "hybrid"
        },
        {
          tag: "plane_of_fire_setting",
          description: "Strongly associated with the Plane of Fire, its native denizens, or its infernal-bright cityscapes and battlefields.",
          assignmentMode: "hybrid"
        },
        {
          tag: "plane_of_air_setting",
          description: "Strongly associated with the Plane of Air, its native denizens, or its endless winds and cloud realms.",
          assignmentMode: "hybrid"
        },
        {
          tag: "plane_of_water_setting",
          description: "Strongly associated with the Plane of Water, its native denizens, or its endless seas and oceanic realms.",
          assignmentMode: "hybrid"
        },
        {
          tag: "plane_of_earth_setting",
          description: "Strongly associated with the Plane of Earth, its native denizens, or its crystal caverns and stonebound realms.",
          assignmentMode: "hybrid"
        },
        {
          tag: "elemental_plane_setting",
          description: "Strongly associated with one of the elemental planes of Fire, Air, Water, or Earth.",
          assignmentMode: "composite",
          nativeOntologyPolicy: "aggregates_native_signals",
          adjacentTags: [
            "plane_of_fire_setting",
            "plane_of_air_setting",
            "plane_of_water_setting",
            "plane_of_earth_setting"
          ],
          compositeOfAnyTags: [
            "plane_of_fire_setting",
            "plane_of_air_setting",
            "plane_of_water_setting",
            "plane_of_earth_setting"
          ]
        },
        {
          tag: "first_world_setting",
          description: "Strongly associated with the First World, fey realms, or thin-boundary crossings into that plane.",
          assignmentMode: "hybrid"
        },
        {
          tag: "dreamlands_setting",
          description: "Strongly associated with the Dreamlands, Leng-linked dream roads, or iconic denizens that dwell there.",
          assignmentMode: "hybrid"
        },
        {
          tag: "boneyard_setting",
          description: "Strongly associated with the Boneyard, psychopomp duties, or soul-processing afterlife scenes.",
          assignmentMode: "hybrid"
        },
        {
          tag: "heaven_setting",
          description: "Strongly associated with Heaven, archon hosts, or ordered celestial service.",
          assignmentMode: "hybrid"
        },
        {
          tag: "nirvana_setting",
          description: "Strongly associated with Nirvana, enlightened celestials, or benevolent contemplative service.",
          assignmentMode: "hybrid"
        },
        {
          tag: "elysium_setting",
          description: "Strongly associated with Elysium, azatas, or freedom-driven celestial heroism.",
          assignmentMode: "hybrid"
        },
        {
          tag: "upper_plane_setting",
          description: "Strongly associated with the upper planes of Heaven, Nirvana, or Elysium.",
          assignmentMode: "composite",
          nativeOntologyPolicy: "aggregates_native_signals",
          adjacentTags: [
            "heaven_setting",
            "nirvana_setting",
            "elysium_setting"
          ],
          compositeOfAnyTags: [
            "heaven_setting",
            "nirvana_setting",
            "elysium_setting"
          ]
        },
        {
          tag: "hell_setting",
          description: "Strongly associated with Hell, devil hosts, or infernal hierarchy.",
          assignmentMode: "hybrid"
        },
        {
          tag: "abyss_setting",
          description: "Strongly associated with the Abyss, demon realms, or qlippoth-infested outer rifts.",
          assignmentMode: "hybrid"
        },
        {
          tag: "abaddon_setting",
          description: "Strongly associated with Abaddon, daemons, or soul-devouring lower-planar wastelands.",
          assignmentMode: "hybrid",
          nativeOntologyPolicy: "aggregates_native_signals"
        },
        {
          tag: "lower_plane_setting",
          description: "Strongly associated with the lower planes of Hell, the Abyss, or Abaddon.",
          assignmentMode: "composite",
          nativeOntologyPolicy: "aggregates_native_signals",
          adjacentTags: [
            "hell_setting",
            "abyss_setting",
            "abaddon_setting"
          ],
          compositeOfAnyTags: [
            "hell_setting",
            "abyss_setting",
            "abaddon_setting"
          ]
        },
        {
          tag: "axis_setting",
          description: "Strongly associated with Axis, the Eternal City, or its lawful planar order.",
          assignmentMode: "hybrid"
        },
        {
          tag: "shadow_plane_setting",
          description: "Strongly associated with the Shadow Plane or the Plane of Shadow.",
          assignmentMode: "hybrid"
        },
        {
          tag: "maelstrom_setting",
          description: "Strongly associated with the Maelstrom, proteans, or its chaotic planar fringes.",
          assignmentMode: "hybrid"
        },
        {
          tag: "cosmic_framework_setting",
          description: "Strongly associated with the cosmic framework planes of Axis, the Boneyard, and the Maelstrom, which govern order, judgment, and transformative change.",
          assignmentMode: "composite",
          nativeOntologyPolicy: "aggregates_native_signals",
          adjacentTags: [
            "axis_setting",
            "boneyard_setting",
            "maelstrom_setting"
          ],
          compositeOfAnyTags: [
            "axis_setting",
            "boneyard_setting",
            "maelstrom_setting"
          ]
        },
        {
          tag: "island_setting",
          description: "Strongly associated with islands, archipelagos, or isolated isles.",
          assignmentMode: "hybrid",
          seedRecords: [
            {
              pack: "age-of-ashes-bestiary",
              name: "Hermean Mutant"
            },
            {
              pack: "age-of-ashes-bestiary",
              name: "Mengkare"
            },
            {
              pack: "age-of-ashes-bestiary",
              name: "Dragonshard Guardian"
            }
          ]
        },
        {
          tag: "nautical_setting",
          description: "Strongly associated with ships, sailors, wrecks, or harbors.",
          assignmentMode: "hybrid",
          seedRecords: [
            {
              pack: "agents-of-edgewatch-bestiary",
              name: "Black Whale Guard (F3)"
            }
          ]
        },
        {
          tag: "geb_setting",
          description: "Strongly associated with Geb, its necromantic society, or iconic locales such as Graydirge.",
          assignmentMode: "hybrid",
          seedRecords: [
            {
              pack: "blood-lords-bestiary",
              name: "Bloodshroud"
            },
            {
              pack: "blood-lords-bestiary",
              name: "Charghar"
            },
            {
              pack: "blood-lords-bestiary",
              name: "Cobblebone Swarm"
            },
            {
              pack: "blood-lords-bestiary",
              name: "Necromunculus"
            }
          ]
        },
        {
          tag: "gravelands_setting",
          description: "Strongly associated with the Gravelands, Lastwall's shattered front, or iconic sites such as Fort Ozem and Gallowspire.",
          assignmentMode: "hybrid",
          seedRecords: [
            {
              pack: "book-of-the-dead-bestiary",
              name: "Gallowdead"
            },
            {
              pack: "claws-of-the-tyrant-bestiary",
              name: "Chernasardo Ranger"
            },
            {
              pack: "claws-of-the-tyrant-bestiary",
              name: "Commander Arsiella Dei"
            },
            {
              pack: "claws-of-the-tyrant-bestiary",
              name: "Knight Reclaimant"
            },
            {
              pack: "claws-of-the-tyrant-bestiary",
              name: "Raised Cavalry"
            }
          ]
        },
        {
          tag: "tian_xia_setting",
          description: "Strongly associated with Tian Xia regions such as Minata, Bonmu, or Tian cultural subregions like Tian-Shu, Tian-Hwan, Tian-Sing, and Tian-La.",
          assignmentMode: "hybrid",
          seedRecords: [
            {
              pack: "fists-of-the-ruby-phoenix-bestiary",
              name: "Amihan"
            },
            {
              pack: "fists-of-the-ruby-phoenix-bestiary",
              name: "Caustic Monitor"
            },
            {
              pack: "fists-of-the-ruby-phoenix-bestiary",
              name: "Jaiban"
            },
            {
              pack: "fists-of-the-ruby-phoenix-bestiary",
              name: "Nai Yan Fei"
            }
          ]
        },
        {
          tag: "jungle_setting",
          description: "Strongly associated with jungles, rainforests, or dense tropical canopies.",
          assignmentMode: "hybrid"
        },
        {
          tag: "forest_setting",
          description: "Strongly associated with forests, jungles, groves, or briar-choked wilds.",
          assignmentMode: "hybrid"
        },
        {
          tag: "plains_setting",
          description: "Strongly associated with open plains, grasslands, prairies, or savannas.",
          assignmentMode: "hybrid"
        },
        {
          tag: "canyon_setting",
          description: "Strongly associated with canyons, gorges, mesas, or badlands.",
          assignmentMode: "hybrid"
        },
        {
          tag: "swamp_setting",
          description: "Strongly associated with bogs, marshes, fens, or mires.",
          assignmentMode: "hybrid"
        },
        {
          tag: "underground_setting",
          description: "Strongly associated with caves, tunnels, crypts, or subterranean spaces.",
          assignmentMode: "hybrid"
        },
        {
          tag: "urban_setting",
          description: "Strongly associated with urban encounter scenes such as cities, streets, alleys, dense buildings, markets, or sewers.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is primarily framed as belonging in city or sewer encounter spaces.",
            "Urban placement is a recurring part of its identity, role, or habitat."
          ],
          doesNotApplyWhen: [
            "The record merely mentions a city once.",
            "The creature can appear in towns but is not specifically urban-coded.",
            "The creature is better modeled by fortress_setting or small_settlement_setting."
          ],
          positiveSignals: [
            "city guard roles",
            "sewer habitat",
            "street or alley patrol scenes",
            "market or district anchoring"
          ],
          negativeSignals: [
            "generic settlement mention",
            "single adventure location",
            "fortress-only residence"
          ],
          adjacentTags: [
            "small_settlement_setting",
            "fortress_setting"
          ],
          seedRecords: [
            {
              pack: "battlecry-bestiary",
              name: "Ofalth Stampede"
            },
            {
              pack: "blood-lords-bestiary",
              name: "Theater Phantasm"
            },
            {
              pack: "book-of-the-dead-bestiary",
              name: "Bone Croupier"
            }
          ]
        },
        {
          tag: "battlefield_setting",
          description: "Strongly associated with battlefields, war zones, organized military deployments, or mass-combat scenes.",
          assignmentMode: "hybrid",
          seedRecords: [
            {
              pack: "battlecry-bestiary",
              name: "Archer Regiment"
            },
            {
              pack: "battlecry-bestiary",
              name: "Dromaar Company"
            }
          ]
        },
        {
          tag: "small_settlement_setting",
          description: "Strongly associated with villages, hamlets, small towns, or other low-density community settlements.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is repeatedly framed around villages, hamlets, or small town life.",
            "Its encounter identity is tied to low-density community spaces rather than major urban districts."
          ],
          doesNotApplyWhen: [
            "The record only says the creature can appear near people.",
            "The creature is more strongly urban, rural, or fortress-coded than village-coded."
          ],
          adjacentTags: [
            "urban_setting",
            "rural_setting"
          ],
          seedRecords: [
            {
              pack: "extinction-curse-bestiary",
              name: "Shoony Hierarch"
            },
            {
              pack: "extinction-curse-bestiary",
              name: "Shoony Militia Member"
            }
          ]
        },
        {
          tag: "arctic_setting",
          description: "Strongly associated with snow, ice, tundra, or frozen reaches.",
          assignmentMode: "hybrid"
        },
        {
          tag: "desert_setting",
          description: "Strongly associated with dunes, sand, or arid wastes.",
          assignmentMode: "hybrid",
          seedRecords: [
            {
              pack: "battlecry-bestiary",
              name: "Qadiran Camel Corps"
            },
            {
              pack: "book-of-the-dead-bestiary",
              name: "Mummy Prophet Of Set"
            }
          ]
        },
        {
          tag: "wasteland_setting",
          description: "Strongly associated with barren wastes, blasted wastelands, or desolate badlands.",
          assignmentMode: "hybrid"
        },
        {
          tag: "mountain_setting",
          description: "Strongly associated with cliffs, peaks, passes, or rocky heights.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Mountain terrain is a recurring habitat or encounter frame.",
            "The creature is tied to peaks, passes, cliffs, or alpine strongholds."
          ],
          doesNotApplyWhen: [
            "Rocky terrain is incidental rather than defining.",
            "The creature is better described as underground_setting or sky_setting."
          ],
          adjacentTags: [
            "underground_setting",
            "sky_setting"
          ]
        },
        {
          tag: "sky_setting",
          description: "Strongly associated with open skies, storm clouds, or high-altitude aerial habitats.",
          assignmentMode: "hybrid"
        },
        {
          tag: "graveyard_setting",
          description: "Strongly associated with cemeteries, tombs, barrows, or burial grounds.",
          assignmentMode: "hybrid"
        },
        {
          tag: "ruins_setting",
          description: "Strongly associated with ancient ruins or derelict structures.",
          assignmentMode: "hybrid"
        },
        {
          tag: "temple_setting",
          description: "Strongly associated with temples, shrines, monasteries, or other sacred encounter sites.",
          assignmentMode: "hybrid"
        },
        {
          tag: "fortress_setting",
          description: "Strongly associated with castles, fortresses, citadels, watchtowers, or other fortified encounter sites.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Fortified structures are part of the creature's recurring encounter identity.",
            "The creature is framed around castles, citadels, watchtowers, keeps, or defensive strongholds."
          ],
          doesNotApplyWhen: [
            "The record only uses a fortress as a one-off location.",
            "The broader identity is urban_setting or temple_setting rather than fortified-site specific."
          ],
          adjacentTags: [
            "urban_setting",
            "temple_setting"
          ]
        },
        {
          tag: "volcanic_setting",
          description: "Strongly associated with volcanoes, calderas, lava, or magma.",
          assignmentMode: "hybrid"
        },
        {
          tag: "rural_setting",
          description: "Strongly associated with farms, pastures, croplands, countryside routes, mills, or other agricultural rural encounter scenes.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is repeatedly framed around farms, fields, mills, roadsides, or countryside scenes.",
            "Agricultural or open-country placement is a recurring part of its encounter identity."
          ],
          doesNotApplyWhen: [
            "The record only implies generic overland travel.",
            "The creature is better modeled by small_settlement_setting or plains_setting."
          ],
          adjacentTags: [
            "small_settlement_setting",
            "plains_setting"
          ],
          seedRecords: [
            {
              pack: "fall-of-plaguestone-bestiary",
              name: "Drunken Farmer"
            },
            {
              pack: "book-of-the-dead-bestiary",
              name: "Death Coach"
            }
          ]
        }
      ]
    },
    encounter_role: {
      description: "Creature practical-fit tags for socially embedded NPCs and role-defined humanoid combatants.",
      tags: [
        {
          tag: "profession_npc",
          description: "Role-defined NPC such as a captain, guard, merchant, or commoner.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is primarily presented through a social role, job, office, or profession label.",
            "Retrieval value comes from it being a role-defined NPC rather than a species-driven monster."
          ],
          doesNotApplyWhen: [
            "The role label is incidental to a stronger monster or combat identity.",
            "The creature is better modeled only as combatant_npc or civic_npc."
          ],
          adjacentTags: [
            "civic_npc",
            "combatant_npc"
          ],
          seedRecords: [
            {
              pack: "pathfinder-npc-core",
              name: "Arms Dealer"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Artillerist"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Astronomer"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Beast Tamer"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Catfolk Name Collector"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Chronicler"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Construction Worker"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Court Historian"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Courtesan"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Dockhand"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Driver"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Dromaar Lorekeeper"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Envoy"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Expedition Leader"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Gunsmith"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Halfling Head Chef"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Halfling Yarnspinner"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Harrow Reader"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Innkeeper"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Local Herbalist"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Messenger"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Miner"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Natural Scientist"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Orc Agriculturist"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Orc Gamekeeper"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Dwarf Smith"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "Captain Lamperia Bane"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "False Hellbreaker"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "Hecatinia"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Changeling Hellknight"
            },
            {
              pack: "revenge-of-the-runelords-bestiary",
              name: "Lograsi"
            },
            {
              pack: "age-of-ashes-bestiary",
              name: "Bloody Blade Mercenary"
            },
            {
              pack: "age-of-ashes-bestiary",
              name: "Ekujae Guardian"
            },
            {
              pack: "age-of-ashes-bestiary",
              name: "Scarlet Triad Boss"
            },
            {
              pack: "age-of-ashes-bestiary",
              name: "Scarlet Triad Sniper"
            },
            {
              pack: "agents-of-edgewatch-bestiary",
              name: "Norgorberite Poisoner"
            },
            {
              pack: "agents-of-edgewatch-bestiary",
              name: "Skinsaw Murderer"
            },
            {
              pack: "agents-of-edgewatch-bestiary",
              name: "Starwatch Commando"
            },
            {
              pack: "agents-of-edgewatch-bestiary",
              name: "Sleepless Sun Veteran"
            },
            {
              pack: "blood-lords-bestiary",
              name: "Nwanyian Archer"
            },
            {
              pack: "blog-bestiary",
              name: "Urok"
            },
            {
              pack: "claws-of-the-tyrant-bestiary",
              name: "Commander Arsiella Dei"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "Adjutant Hellknight Armiger"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "Isgeri Slayer"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "Jordis Serelli"
            },
            {
              pack: "kingmaker-bestiary",
              name: "Cleansed Cultist"
            },
            {
              pack: "kingmaker-bestiary",
              name: "False Priestess"
            },
            {
              pack: "kingmaker-bestiary",
              name: "Thresholder Disciple"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Virtuous Defender"
            },
            {
              pack: "menace-under-otari-bestiary",
              name: "Drow Priestess (BB)"
            },
            {
              pack: "menace-under-otari-bestiary",
              name: "Kobold Boss Zolgran (BB)"
            },
            {
              pack: "outlaws-of-alkenstar-bestiary",
              name: "Corrupt Shieldmarshal (Clan Pistol)"
            },
            {
              pack: "outlaws-of-alkenstar-bestiary",
              name: "Gilded Gunner Assassin"
            },
            {
              pack: "outlaws-of-alkenstar-bestiary",
              name: "Gilded Gunner Goon"
            },
            {
              pack: "pathfinder-bestiary-3",
              name: "Android Infiltrator"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Cultist"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Demonbane Warrior"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Dwarf General"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Enigmatic Conspirant"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Exiled Revolutionary"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Infantry Soldier"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Knight"
            },
            {
              pack: "prey-for-death-bestiary",
              name: "Berserker Ordulf"
            },
            {
              pack: "prey-for-death-bestiary",
              name: "Ordulf Bladecaller"
            },
            {
              pack: "season-of-ghosts-bestiary",
              name: "Mercenary Enforcer"
            },
            {
              pack: "seven-dooms-for-sandpoint-bestiary",
              name: "Devil's Disciple"
            },
            {
              pack: "sky-kings-tomb-bestiary",
              name: "Hryngar King's Agent"
            },
            {
              pack: "strength-of-thousands-bestiary",
              name: "Sun Warrior Brigade"
            },
            {
              pack: "the-enmity-cycle-bestiary",
              name: "Scorching Sun Cultist"
            },
            {
              pack: "fists-of-the-ruby-phoenix-bestiary",
              name: "Weapon Master"
            }
          ]
        },
        {
          tag: "civic_npc",
          description: "Fits the civic or social fabric of a scene and usually isn't the primary monster answer.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature belongs to the civic, domestic, institutional, or everyday social fabric of the scene.",
            "A GM would plausibly retrieve it as a socially embedded NPC rather than as a combat-forward foe."
          ],
          doesNotApplyWhen: [
            "The record is primarily a hostile combatant or raider.",
            "The creature is only profession-labeled without clear social embeddedness."
          ],
          adjacentTags: [
            "profession_npc",
            "combatant_npc"
          ],
          seedRecords: [
            {
              pack: "pathfinder-npc-core",
              name: "Astronomer"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Beast Tamer"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Catfolk Name Collector"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Chronicler"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Construction Worker"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Court Historian"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Courtesan"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Dockhand"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Driver"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Dromaar Lorekeeper"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Envoy"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Expedition Leader"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Gunsmith"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Halfling Head Chef"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Halfling Yarnspinner"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Harrow Reader"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Innkeeper"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Local Herbalist"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Messenger"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Miner"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Natural Scientist"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Orc Agriculturist"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Orc Gamekeeper"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Dwarf Smith"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "Captain Lamperia Bane"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "Hecatinia"
            },
            {
              pack: "agents-of-edgewatch-bestiary",
              name: "Starwatch Commando"
            },
            {
              pack: "agents-of-edgewatch-bestiary",
              name: "Sleepless Sun Veteran"
            },
            {
              pack: "blood-lords-bestiary",
              name: "Firebrand Bastion"
            },
            {
              pack: "blood-lords-bestiary",
              name: "Nwanyian Archer"
            },
            {
              pack: "claws-of-the-tyrant-bestiary",
              name: "Commander Arsiella Dei"
            },
            {
              pack: "gatewalkers-bestiary",
              name: "Oaksteward Enforcer"
            },
            {
              pack: "gatewalkers-bestiary",
              name: "Oaksteward Enforcer (Gatehouse)"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "Adjutant Hellknight Armiger"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "Jordis Serelli"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Changeling Hellknight"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Virtuous Defender"
            },
            {
              pack: "outlaws-of-alkenstar-bestiary",
              name: "Corrupt Shieldmarshal (Clan Pistol)"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Dwarf General"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Enigmatic Conspirant"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Exiled Revolutionary"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Knight"
            },
            {
              pack: "season-of-ghosts-bestiary",
              name: "Mercenary Enforcer"
            },
            {
              pack: "sky-kings-tomb-bestiary",
              name: "Hryngar King's Agent"
            },
            {
              pack: "age-of-ashes-bestiary",
              name: "Ekujae Guardian"
            }
          ]
        },
        {
          tag: "combatant_npc",
          description: "Role-defined humanoid combatant such as a soldier, bandit, mercenary, or cultist.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is presented as a role-defined humanoid adversary or martial operative.",
            "Retrieval value comes from combat function rather than civic embedding."
          ],
          doesNotApplyWhen: [
            "The record is mainly a social or occupational NPC.",
            "The creature is better captured by profession_npc or civic_npc without a strong combat-forward role."
          ],
          adjacentTags: [
            "profession_npc",
            "civic_npc"
          ],
          seedRecords: [
            {
              pack: "pathfinder-npc-core",
              name: "Artillerist"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Heavy Cavalry"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Hellknight Cavalry Brigade"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Hobgoblin Battalion"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Hobgoblin Spellbreaker"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Kholo Pragmatist"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Kobold Egg Guardian"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Legbreaker"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Mage Knight"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Mixed Martial Artist"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Musketeer"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Orc Commander"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Orc Veteran"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Orc Veteran Master"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Orc Skullcrushers"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Knight"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Dwarf Battalion"
            },
            {
              pack: "age-of-ashes-bestiary",
              name: "Scarlet Triad Boss"
            },
            {
              pack: "age-of-ashes-bestiary",
              name: "Scarlet Triad Enforcer"
            },
            {
              pack: "agents-of-edgewatch-bestiary",
              name: "Sleepless Sun Veteran"
            },
            {
              pack: "blood-lords-bestiary",
              name: "Nwanyian Archer"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "Captain Lamperia Bane"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "False Hellbreaker"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "Hecatinia"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Changeling Hellknight"
            },
            {
              pack: "kingmaker-bestiary",
              name: "Thresholder Disciple"
            },
            {
              pack: "prey-for-death-bestiary",
              name: "Berserker Ordulf"
            },
            {
              pack: "revenge-of-the-runelords-bestiary",
              name: "Lograsi"
            },
            {
              pack: "agents-of-edgewatch-bestiary",
              name: "Copper Hand Rogue"
            },
            {
              pack: "agents-of-edgewatch-bestiary",
              name: "Gref"
            },
            {
              pack: "agents-of-edgewatch-bestiary",
              name: "Kekker"
            },
            {
              pack: "agents-of-edgewatch-bestiary",
              name: "Starwatch Commando"
            },
            {
              pack: "age-of-ashes-bestiary",
              name: "Bloody Blade Mercenary"
            },
            {
              pack: "age-of-ashes-bestiary",
              name: "Dmiri Yoltosha"
            },
            {
              pack: "age-of-ashes-bestiary",
              name: "Ekujae Guardian"
            },
            {
              pack: "age-of-ashes-bestiary",
              name: "Laslunn"
            },
            {
              pack: "age-of-ashes-bestiary",
              name: "Scarlet Triad Sniper"
            },
            {
              pack: "claws-of-the-tyrant-bestiary",
              name: "Commander Arsiella Dei"
            },
            {
              pack: "crown-of-the-kobold-king-bestiary",
              name: "Dark Talon Kobold"
            },
            {
              pack: "crown-of-the-kobold-king-bestiary",
              name: "Foolish Hunter"
            },
            {
              pack: "crown-of-the-kobold-king-bestiary",
              name: "Gurtlekep"
            },
            {
              pack: "crown-of-the-kobold-king-bestiary",
              name: "Kapmek"
            },
            {
              pack: "crown-of-the-kobold-king-bestiary",
              name: "Ygrik"
            },
            {
              pack: "fall-of-plaguestone-bestiary",
              name: "Graytusk"
            },
            {
              pack: "fists-of-the-ruby-phoenix-bestiary",
              name: "Archery Specialist"
            },
            {
              pack: "fists-of-the-ruby-phoenix-bestiary",
              name: "Master Xun"
            },
            {
              pack: "fists-of-the-ruby-phoenix-bestiary",
              name: "Ran-to (Level 20)"
            },
            {
              pack: "fists-of-the-ruby-phoenix-bestiary",
              name: "Troff Frostknuckles"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "Chelaxian Loyalist"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "Isgeri Slayer"
            },
            {
              pack: "hellbreakers-bestiary",
              name: "Jordis Serelli"
            },
            {
              pack: "kingmaker-bestiary",
              name: "Ameon Trask"
            },
            {
              pack: "kingmaker-bestiary",
              name: "Villamor Koth"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Anadi Hunter"
            },
            {
              pack: "outlaws-of-alkenstar-bestiary",
              name: "Corrupt Shieldmarshal (Clan Pistol)"
            },
            {
              pack: "outlaws-of-alkenstar-bestiary",
              name: "Gilded Gunner Assassin"
            },
            {
              pack: "outlaws-of-alkenstar-bestiary",
              name: "Gilded Gunner Goon"
            },
            {
              pack: "outlaws-of-alkenstar-bestiary",
              name: "Gilded Gunner Safecracker"
            },
            {
              pack: "pathfinder-bestiary-3",
              name: "Android Infiltrator"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Cultist"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Gang Leader"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Hero Hunter"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Monster Hunter"
            },
            {
              pack: "pathfinder-npc-core",
              name: "Wealthy Vigilante"
            },
            {
              pack: "prey-for-death-bestiary",
              name: "Gorumite Veteran"
            },
            {
              pack: "quest-for-the-frozen-flame-bestiary",
              name: "Graylok Ambusher"
            },
            {
              pack: "rusthenge-bestiary",
              name: "Fallen Acolyte"
            },
            {
              pack: "rusthenge-bestiary",
              name: "Rustsworn Initiate"
            },
            {
              pack: "season-of-ghosts-bestiary",
              name: "Mercenary Enforcer"
            },
            {
              pack: "seven-dooms-for-sandpoint-bestiary",
              name: "Devil's Disciple"
            },
            {
              pack: "shadows-at-sundown-bestiary",
              name: "Yniesse Zenderholm"
            },
            {
              pack: "sky-kings-tomb-bestiary",
              name: "Blacknoon Apprentice"
            },
            {
              pack: "sky-kings-tomb-bestiary",
              name: "Hrungul Ironeye"
            },
            {
              pack: "sky-kings-tomb-bestiary",
              name: "Hryngar King's Agent"
            },
            {
              pack: "strength-of-thousands-bestiary",
              name: "Crimson Acolyte"
            },
            {
              pack: "strength-of-thousands-bestiary",
              name: "Cyclops Bully"
            },
            {
              pack: "the-enmity-cycle-bestiary",
              name: "Scorching Sun Cultist"
            },
            {
              pack: "the-slithering-bestiary",
              name: "Aspis Technician"
            },
            {
              pack: "troubles-in-otari-bestiary",
              name: "Orc Scrapper"
            },
            {
              pack: "triumph-of-the-tusk-bestiary",
              name: "Molog"
            },
            {
              pack: "triumph-of-the-tusk-bestiary",
              name: "Orc Hunter"
            },
            {
              pack: "triumph-of-the-tusk-bestiary",
              name: "Wingripper Wyvern Rider"
            },
            {
              pack: "wardens-of-wildwood-bestiary",
              name: "Hateful Logger"
            },
            {
              pack: "battlecry-bestiary",
              name: "Dromaar Company"
            },
            {
              pack: "blog-bestiary",
              name: "Urok"
            }
          ]
        }
      ]
    },
    ontology_cluster: {
      description: "Creature semantic groupings that aggregate fragmented native ontology when exact traits are too narrow.",
      tags: [
        {
          tag: "undead_adjacent",
          description: "Groups undead and closely undead-coded native signals into one retrieval bucket.",
          assignmentMode: "hybrid",
          nativeOntologyPolicy: "aggregates_native_signals"
        },
        {
          tag: "sinspawn_family",
          description: "Groups sinspawn and close runelord-bred sinspawn offshoots into one retrieval bucket.",
          assignmentMode: "hybrid",
          nativeOntologyPolicy: "aggregates_native_signals",
          seedRecords: [
            {
              pack: "pathfinder-monster-core",
              name: "Envyspawn"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Gluttonyspawn"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Greedspawn"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Lustspawn"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Pridespawn"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Slothspawn"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Wrathspawn"
            },
            {
              pack: "pathfinder-monster-core-2",
              name: "Atrixyl"
            },
            {
              pack: "battlecry-bestiary",
              name: "Sinswarm"
            },
            {
              pack: "revenge-of-the-runelords-bestiary",
              name: "Greedspawn"
            },
            {
              pack: "revenge-of-the-runelords-bestiary",
              name: "Pridespawn Sentinel"
            },
            {
              pack: "revenge-of-the-runelords-bestiary",
              name: "Knight of Malice"
            },
            {
              pack: "seven-dooms-for-sandpoint-bestiary",
              name: "Greedspawn"
            }
          ]
        }
      ]
    },
    casting_profile: {
      description: "Creature prep-driving casting tags for encounter planning and shortlist searches.",
      tags: [
        {
          tag: "dragon_spellcaster",
          description: "Dragon or archdragon variant with an explicit spellcaster stat block or named spellcaster presentation.",
          assignmentMode: "hybrid",
          seedRecords: [
            {
              pack: "lost-omens-bestiary",
              name: "Adamantine Archdragon (Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Adamantine Dragon (Adult, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Adamantine Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Adamantine Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Barrage Archdragon (Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Barrage Dragon (Adult, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Barrage Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Barrage Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Cloud Archdragon (Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Cloud Dragon (Adult, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Cloud Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Cloud Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Conspirator Archdragon (Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Conspirator Dragon (Adult, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Conspirator Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Conspirator Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Crystal Archdragon (Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Crystal Dragon (Adult, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Crystal Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Crystal Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Delight Archdragon (Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Delight Dragon (Adult, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Delight Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Delight Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Diabolic Archdragon (Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Diabolic Dragon (Adult, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Diabolic Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Diabolic Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Empyreal Archdragon (Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Empyreal Dragon (Adult, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Empyreal Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Empyreal Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Fortune Archdragon (Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Fortune Dragon (Adult, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Fortune Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Fortune Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Mocking Archdragon (Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Mocking Dragon (Adult, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Mocking Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Mocking Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Oath Archdragon (Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Oath Dragon (Adult, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Oath Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Oath Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Omen Archdragon (Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Omen Dragon (Adult, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Omen Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Omen Dragon (Young, Spellcaster)"
            },
            {
              pack: "pathfinder-bestiary",
              name: "Red Dragon (Adult, Spellcaster)"
            },
            {
              pack: "pathfinder-bestiary",
              name: "Red Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "pathfinder-bestiary",
              name: "Red Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Requiem Archdragon (Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core-2",
              name: "Requiem Dragon (Adult, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core-2",
              name: "Requiem Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core-2",
              name: "Requiem Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Rune Archdragon (Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core-2",
              name: "Rune Dragon (Adult, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core-2",
              name: "Rune Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core-2",
              name: "Rune Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Sage Archdragon (Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Sage Dragon (Adult, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Sage Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Sage Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Sky Archdragon (Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Sky Dragon (Adult, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Sky Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Sky Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Stormcrown Archdragon (Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Stormcrown Dragon (Adult, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Stormcrown Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Stormcrown Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Time Archdragon (Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Time Dragon (Adult, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Time Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Time Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Umbral Archdragon (Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Umbral Dragon (Adult, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Umbral Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Umbral Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Vizier Archdragon (Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Vizier Dragon (Adult, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Vizier Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Vizier Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Vorpal Archdragon (Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Vorpal Dragon (Adult, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Vorpal Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Vorpal Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Wailing Archdragon (Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Wailing Dragon (Adult, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Wailing Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Wailing Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Whisper Archdragon (Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core-2",
              name: "Whisper Dragon (Adult, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core-2",
              name: "Whisper Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core-2",
              name: "Whisper Dragon (Young, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Wish Archdragon (Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Wish Dragon (Adult, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Wish Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Wish Dragon (Young, Spellcaster)"
            }
          ]
        }
      ]
    },
    threat_profile: {
      description: "Creature prep-driving threat patterns based on behavior, counterplay, or encounter consequence rather than type.",
      tags: [
        {
          tag: "possession_threat",
          description: "Can possess, body-snatch, or take control of a victim from within.",
          assignmentMode: "hybrid"
        },
        {
          tag: "life_drain_threat",
          description: "Threat defined by draining blood, vitality, life force, or souls from victims.",
          assignmentMode: "hybrid"
        },
        {
          tag: "spawn_creator",
          description: "Creates additional threats through infestation, spawn-making, conversion, or implanted offspring.",
          assignmentMode: "hybrid"
        },
        {
          tag: "petrification_threat",
          description: "Threat defined by petrifying victims or turning them to stone.",
          assignmentMode: "hybrid"
        },
        {
          tag: "regeneration_threat",
          description: "Regenerates or requires special suppression or finishing countermeasures.",
          assignmentMode: "hybrid",
          seedRecords: [
            {
              pack: "pathfinder-monster-core-2",
              name: "Cavern Troll"
            },
            {
              pack: "menace-under-otari-bestiary",
              name: "Forest Troll (BB)"
            },
            {
              pack: "stolen-fate-bestiary",
              name: "Gegnir"
            },
            {
              pack: "kingmaker-bestiary",
              name: "Gurija"
            },
            {
              pack: "kingmaker-bestiary",
              name: "Hargulka"
            },
            {
              pack: "pathfinder-monster-core-2",
              name: "Jotund Troll"
            },
            {
              pack: "triumph-of-the-tusk-bestiary",
              name: "Marguk Cleftneck"
            },
            {
              pack: "rage-of-elements-bestiary",
              name: "Nightwood Guardian"
            },
            {
              pack: "pathfinder-monster-core-2",
              name: "Two-Headed Troll"
            }
          ]
        },
        {
          tag: "ambush_grabber",
          description: "Captures prey through grabbing, constriction, swallowing whole, webbing, or drag-off ambush tactics.",
          assignmentMode: "hybrid"
        }
      ]
    },
    motif: {
      description: "Creature motif tags for recurring scene and presentation themes not captured by native traits.",
      tags: [
        {
          tag: "carnival_show",
          description: "Strongly associated with carnivals, circuses, clowns, jesters, or sideshow-style presentation.",
          assignmentMode: "hybrid"
        },
        {
          tag: "living_toy",
          description: "Strongly associated with dolls, puppets, mannequins, or other animated playthings.",
          assignmentMode: "hybrid"
        },
        {
          tag: "living_artwork",
          description: "Strongly associated with paintings, graffiti, murals, portraits, or other artworks brought to life.",
          assignmentMode: "hybrid"
        },
        {
          tag: "trickster_chaos",
          description: "Strongly associated with pranks, mischief, whimsy, or explicit trickster behavior.",
          assignmentMode: "hybrid"
        },
        {
          tag: "mask_motif",
          description: "Strongly associated with masks, veils, ceremonial face-coverings, or deliberately obscured presentation.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Masks, veils, or deliberate face-covering are a salient presentation motif.",
            "The obscured face is part of the creature's recurring visual identity."
          ],
          doesNotApplyWhen: [
            "A mask appears once as minor equipment.",
            "The stronger semantic is faceless_horror or disguised_pretender rather than mask imagery."
          ],
          adjacentTags: [
            "disguised_pretender",
            "faceless_horror"
          ]
        },
        {
          tag: "faceless_horror",
          description: "Strongly associated with missing, hidden, stolen, or featureless faces.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Missing, stolen, hidden, or featureless faces are central to the creature's horror identity.",
            "Face absence or erasure is a recurring motif, not just a disguise tactic."
          ],
          doesNotApplyWhen: [
            "The creature is mainly about impersonation or disguise rather than facial absence.",
            "A covered face appears only as costume or gear."
          ],
          adjacentTags: [
            "disguised_pretender",
            "mask_motif"
          ]
        },
        {
          tag: "disguised_pretender",
          description: "Strongly associated with false identities, impersonation, infiltration, shapeshifting, or replacement.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Impersonation, replacement, disguise, or false identity is a central retrieval hook.",
            "The creature is framed around passing as someone or something else."
          ],
          doesNotApplyWhen: [
            "It merely uses stealth or deception without identity substitution.",
            "The stronger motif is mask_motif or faceless_horror rather than impersonation."
          ],
          adjacentTags: [
            "faceless_horror",
            "mask_motif"
          ],
          seedRecords: [
            {
              pack: "pathfinder-npc-core",
              name: "Master of Disguise"
            },
            {
              pack: "book-of-the-dead-bestiary",
              name: "Ecorche"
            },
            {
              pack: "kingmaker-bestiary",
              name: "Virthad"
            },
            {
              pack: "kingmaker-bestiary",
              name: "Fetch Stalker"
            },
            {
              pack: "pathfinder-bestiary",
              name: "Doppelganger"
            },
            {
              pack: "pathfinder-bestiary",
              name: "Green Hag"
            },
            {
              pack: "pathfinder-bestiary",
              name: "Raja Rakshasa"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Cuckoo Hag"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Gimmerling"
            },
            {
              pack: "season-of-ghosts-bestiary",
              name: "Noppera-bo Grunt"
            },
            {
              pack: "season-of-ghosts-bestiary",
              name: "Noppera-bo Trickster"
            },
            {
              pack: "season-of-ghosts-bestiary",
              name: "Noppera-Bo Impersonator (Arcane)"
            },
            {
              pack: "season-of-ghosts-bestiary",
              name: "Noppera-Bo Impersonator (Divine)"
            },
            {
              pack: "season-of-ghosts-bestiary",
              name: "Noppera-Bo Impersonator (Martial)"
            },
            {
              pack: "season-of-ghosts-bestiary",
              name: "Noppera-Bo Impersonator (Occult)"
            },
            {
              pack: "season-of-ghosts-bestiary",
              name: "Noppera-Bo Impersonator (Primal)"
            },
            {
              pack: "season-of-ghosts-bestiary",
              name: "Noppera-Bo Impersonator (Skilled)"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Conspirator Archdragon"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Conspirator Archdragon (Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Conspirator Dragon (Adult, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Conspirator Dragon (Ancient)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Conspirator Dragon (Ancient, Spellcaster)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Conspirator Dragon (Young)"
            },
            {
              pack: "pathfinder-monster-core",
              name: "Conspirator Dragon (Young, Spellcaster)"
            }
          ]
        }
      ]
    },
    bound_object: {
      description: "Creature tags for animated objects, statues, and other bound physical forms.",
      tags: [
        {
          tag: "animated_object",
          description: "Strongly associated with animated objects, furniture, tools, or other constructed items.",
          assignmentMode: "hybrid",
          seedRecords: [
            {
              pack: "age-of-ashes-bestiary",
              name: "Animated Dragonstorm"
            },
            {
              pack: "blood-lords-bestiary",
              name: "Animated Fireplace"
            },
            {
              pack: "curtain-call-bestiary",
              name: "Animated Boiler"
            },
            {
              pack: "gatewalkers-bestiary",
              name: "Quarry Construct"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Animated Bamboo Figurine"
            },
            {
              pack: "lost-omens-bestiary",
              name: "Animated Kite"
            },
            {
              pack: "myth-speaker-bestiary",
              name: "Animated Panoply"
            },
            {
              pack: "one-shot-bestiary",
              name: "Soulbound Guardian"
            },
            {
              pack: "outlaws-of-alkenstar-bestiary",
              name: "Glass Elephant"
            },
            {
              pack: "pathfinder-monster-core-2",
              name: "Animated Trebuchet"
            },
            {
              pack: "quest-for-the-frozen-flame-bestiary",
              name: "Weykoward"
            },
            {
              pack: "season-of-ghosts-bestiary",
              name: "Animated Axe"
            }
          ]
        },
        {
          tag: "animated_statue",
          description: "Strongly associated with animated statues, effigies, idols, or monuments.",
          assignmentMode: "hybrid"
        }
      ]
    }
  }
} satisfies DerivedTagAuthoredCategoryOntology;
