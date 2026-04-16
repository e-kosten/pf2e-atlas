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
          assignmentMode: "hybrid"
        },
        {
          tag: "nautical_setting",
          description: "Strongly associated with ships, sailors, wrecks, or harbors.",
          assignmentMode: "hybrid"
        },
        {
          tag: "geb_setting",
          description: "Strongly associated with Geb, its necromantic society, or iconic locales such as Graydirge.",
          assignmentMode: "hybrid"
        },
        {
          tag: "gravelands_setting",
          description: "Strongly associated with the Gravelands, Lastwall's shattered front, or iconic sites such as Fort Ozem and Gallowspire.",
          assignmentMode: "hybrid"
        },
        {
          tag: "tian_xia_setting",
          description: "Strongly associated with Tian Xia regions such as Minata, Bonmu, or Tian cultural subregions like Tian-Shu, Tian-Hwan, Tian-Sing, and Tian-La.",
          assignmentMode: "hybrid"
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
          ]
        },
        {
          tag: "battlefield_setting",
          description: "Strongly associated with battlefields, war zones, organized military deployments, or mass-combat scenes.",
          assignmentMode: "hybrid"
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
          assignmentMode: "hybrid"
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
          nativeOntologyPolicy: "aggregates_native_signals"
        }
      ]
    },
    casting_profile: {
      description: "Creature prep-driving casting tags for encounter planning and shortlist searches.",
      tags: [
        {
          tag: "dragon_spellcaster",
          description: "Dragon or archdragon variant with an explicit spellcaster stat block or named spellcaster presentation.",
          assignmentMode: "hybrid"
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
          assignmentMode: "hybrid"
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
          assignmentMode: "hybrid"
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
