import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const DescriptiveSettingProjectionDeclarations = [
  defineConceptProjections("abaddon_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Abaddon, daemons, or soul-devouring lower-planar wastelands.",
      nativeOntologyPolicy: "aggregates_native_signals",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("abyss_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Abyss, demon realms, or qlippoth-infested outer rifts.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("alien_technology_wasteland_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with alien technology, robots, mutants, strange metal ruins, and science-fantasy wastelands. In Golarion, this primarily corresponds to Numeria.",
      appliesWhen: [
        "Use when the creature is naturally retrieved for weird-tech wilderness, star-metal ruins, robot-haunted badlands, mutant frontiers, or barbarian-meets-super-science planning.",
        "The planning value comes from science-fantasy intrusion into the region's creature ecology rather than only from a single construct or technological gimmick.",
      ],
      doesNotApplyWhen: [
        "The creature only uses one technological item or construct-like ability without a broader weird-tech regional frame.",
        "The stronger fit is bound_object or a combat-role tag because the planning value is tactical rather than regional or thematic.",
      ],
      adjacentTags: ["magic_blight_wasteland_setting", "animated_object"],
    },
  }),
  defineConceptProjections("aquatic_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard strongly associated with flooded chambers, rivers, docks, ships, reefs, or underwater spaces.",
    },
  }),
  defineConceptProjections("aquatic_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with open water, underwater spaces, or aquatic environments.",
    },
  }),
  defineConceptProjections("arctic_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with snow, ice, tundra, or frozen reaches.",
    },
  }),
  defineConceptProjections("astral_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Astral Plane scenes, silver-void travel, or stable portal routes.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("axis_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Axis, the Eternal City, or its lawful planar order.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("battlefield_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard strongly associated with siegeworks, trenches, killing grounds, war engines, or other battlefield scenes.",
    },
  }),
  defineConceptProjections("battlefield_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with battlefields, war zones, organized military deployments, or mass-combat scenes.",
    },
  }),
  defineConceptProjections("boneyard_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Boneyard, psychopomp duties, or soul-processing afterlife scenes.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("bridge_passage_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard strongly associated with bridges, chokepoints, gates, stairwells, or forced passage bottlenecks.",
      appliesWhen: [
        "The hazard is naturally retrieved for narrow crossings, gates, stairwells, or other spaces where intruders must pass through a constrained route.",
        "Forced-passage geometry matters more than a broader dungeon, urban, or wilderness setting identity.",
      ],
      doesNotApplyWhen: [
        "The hazard only happens to sit near a doorway or bridge once without the chokepoint being central to its design.",
        "The stronger fit is threshold_lockdown or forced_movement because the route bottleneck is not the main retrieval hook.",
      ],
      adjacentTags: ["threshold_lockdown", "forced_movement"],
    },
  }),
  defineConceptProjections("canyon_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with canyons, gorges, mesas, or badlands.",
    },
  }),
  defineConceptProjections("coastal_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with coasts, shores, reefs, or littoral edges.",
    },
  }),
  defineConceptProjections("darklands_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with the Darklands as a civilization-bearing underworld macro-region rather than generic underground terrain or one cave network.",
    },
  }),
  defineConceptProjections("demonic_scar_frontier_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with demon-scarred frontiers, reclaimed homelands, abyss-tainted wilderness, and recovery after planar catastrophe. In Golarion, this primarily corresponds to the Sarkoris Scar.",
      appliesWhen: [
        "Use when the creature is naturally retrieved for post-invasion frontier planning: planar scars, reclaimed sacred sites, demon-torn wastelands, lingering abyssal corruption, or survival after a world-rending incursion.",
        "The planning value comes from a land still shaped by demonic devastation and reclamation rather than from generic fiendish wilderness or one demonic trait.",
      ],
      doesNotApplyWhen: [
        "The creature is merely demonic, corrupted, or extraplanar without a real frontier-of-reclamation regional frame.",
        "The stronger fit is planar_setting or corruption_profile because the retrieval hook is cosmological origin or body-horror taint rather than a scarred regional frontier.",
      ],
      adjacentTags: ["battlefield_setting", "void_tainted"],
    },
  }),
  defineConceptProjections("desert_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with dunes, sand, or arid wastes.",
    },
  }),
  defineConceptProjections("dreamlands_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with the Dreamlands, Leng-linked dream roads, or iconic denizens that dwell there.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("dungeon_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard strongly associated with dungeon corridors, chambers, trapped passages, or underground complexes.",
      appliesWhen: [
        "The hazard is naturally retrieved as a classic corridor, chamber, or trap-complex defense in a dungeon environment.",
        "Its encounter identity is more about underground built-space adventuring than a narrower tomb or temple context.",
      ],
      doesNotApplyWhen: [
        "The hazard is more specifically tomb-, temple-, bridge-, or aquatic-coded than generic dungeon-coded.",
        "The hazard merely happens to appear indoors once without dungeon-like scene identity.",
      ],
      adjacentTags: ["tomb_hazard", "temple_hazard"],
    },
  }),
  defineConceptProjections("elysium_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Elysium, azatas, or freedom-driven celestial heroism.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("ethereal_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with the Ethereal Plane, its native wildlife, or recurring hunting and travel routes through it.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("first_world_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the First World, fey realms, or thin-boundary crossings into that plane.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("forest_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with forests, jungles, groves, or briar-choked wilds.",
    },
  }),
  defineConceptProjections("fortress_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with castles, fortresses, citadels, watchtowers, or other fortified encounter sites.",
      appliesWhen: [
        "Fortified structures are part of the creature's recurring encounter identity.",
        "The creature is framed around castles, citadels, watchtowers, keeps, or defensive strongholds.",
      ],
      doesNotApplyWhen: [
        "The record only uses a fortress as a one-off location.",
        "The broader identity is urban_setting or temple_setting rather than fortified-site specific.",
      ],
      adjacentTags: ["urban_setting", "temple_setting"],
    },
  }),
  defineConceptProjections("freshwater_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with rivers, lakes, ponds, streams, springs, or other inland waters.",
    },
  }),
  defineConceptProjections("gothic_horror_land_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with Gothic-horror lands of mist, superstition, graveyard dread, cursed nobility, and classic night monsters. In Golarion, this primarily corresponds to Ustalav.",
      appliesWhen: [
        "Use when the creature is naturally retrieved for Gothic-horror encounter planning: decaying manors, haunted villages, graveyard menace, superstitious communities, or classic monster-fiction mood.",
        "The planning value comes from atmospheric horror identity, cursed lineage, or old-world dread rather than only from one undead, fiend, or shapeshifter trait.",
      ],
      doesNotApplyWhen: [
        "The creature is simply spooky, undead, or cursed without a real Gothic-horror social or atmospheric frame.",
        "The stronger fit is story_motif or genre_motif because the retrieval hook is a narrower narrative motif rather than a full regional horror lens.",
      ],
      adjacentTags: ["graveyard_setting", "folk_horror"],
    },
  }),
  defineConceptProjections("graveyard_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with cemeteries, tombs, barrows, or burial grounds.",
    },
  }),
  defineConceptProjections("heaven_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Heaven, archon hosts, or ordered celestial service.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("hell_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Hell, devil hosts, or infernal hierarchy.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("island_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with islands, archipelagos, or isolated isles.",
    },
  }),
  defineConceptProjections("jungle_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with jungles, rainforests, or dense tropical canopies.",
    },
  }),
  defineConceptProjections("maelstrom_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Maelstrom, proteans, or its chaotic planar fringes.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("magic_blight_wasteland_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with arcane devastation, dead-magic pockets, wild-magic scars, magical storms, and survival in a magically broken wasteland. In Golarion, this primarily corresponds to the Mana Wastes.",
      appliesWhen: [
        "Use when the creature is naturally retrieved for dead-magic deserts, wild-magic badlands, spellscar survival, mutation-causing arcane fallout, or regions where magic itself has wounded the landscape.",
        "The planning value comes from magical environmental breakage and wasteland adaptation rather than only from one magical trait or one regional nation.",
      ],
      doesNotApplyWhen: [
        "The creature is merely magical, arcane, or mutated without a meaningful tie to a magic-blasted regional wasteland.",
        "The stronger fit is alien_technology_wasteland_setting because the retrieval hook is weird technology, robots, or alien ruins rather than magical landscape scarring.",
      ],
      adjacentTags: ["alien_technology_wasteland_setting", "wasteland_setting"],
    },
  }),
  defineConceptProjections("mountain_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with cliffs, peaks, passes, or rocky heights.",
      appliesWhen: [
        "Mountain terrain is a recurring habitat or encounter frame.",
        "The creature is tied to peaks, passes, cliffs, or alpine strongholds.",
      ],
      doesNotApplyWhen: [
        "Rocky terrain is incidental rather than defining.",
        "The creature is better described as underground_setting or sky_setting.",
      ],
      adjacentTags: ["underground_setting", "sky_setting"],
    },
  }),
  defineConceptProjections("mwangi_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with the Mwangi Expanse, its jungle polities, and Mwangi-rooted regional framing that materially affects creature planning and retrieval.",
    },
  }),
  defineConceptProjections("nautical_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with ships, sailors, wrecks, or harbors.",
    },
  }),
  defineConceptProjections("nirvana_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Nirvana, enlightened celestials, or benevolent contemplative service.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("organized_undead_society_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with organized undead societies, corpse labor, necromantic bureaucracy, and courtly undead institutions. In Golarion, this primarily corresponds to Geb.",
      appliesWhen: [
        "Use when the creature is naturally retrieved for an undead-ruled state, necromantic civil order, corpse-backed labor system, or other organized deathless society rather than an isolated tomb or graveyard.",
        "The planning value comes from undead institutions, court politics, civic structure, or civilized undead social roles as much as from simple undead presence.",
      ],
      doesNotApplyWhen: [
        "The creature is merely undead, tomb-dwelling, or cemetery-haunting without a meaningful link to a broader undead social order.",
        "The stronger fit is undead_war_torn_region_setting because the retrieval hook is an undead occupation zone or shattered crusader frontier rather than a stable undead society.",
      ],
      adjacentTags: ["undead_war_torn_region_setting", "urban_setting"],
    },
  }),
  defineConceptProjections("plains_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with open plains, grasslands, prairies, or savannas.",
    },
  }),
  defineConceptProjections("plane_of_air_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with the Plane of Air, its native denizens, or its endless winds and cloud realms.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("plane_of_earth_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with the Plane of Earth, its native denizens, or its crystal caverns and stonebound realms.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("plane_of_fire_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with the Plane of Fire, its native denizens, or its infernal-bright cityscapes and battlefields.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("plane_of_water_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with the Plane of Water, its native denizens, or its endless seas and oceanic realms.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("ruins_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with ancient ruins or derelict structures.",
    },
  }),
  defineConceptProjections("rural_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with farms, pastures, croplands, countryside routes, mills, or other agricultural rural encounter scenes.",
      appliesWhen: [
        "The creature is repeatedly framed around farms, fields, mills, roadsides, or countryside scenes.",
        "Agricultural or open-country placement is a recurring part of its encounter identity.",
      ],
      doesNotApplyWhen: [
        "The record only implies generic overland travel.",
        "The creature is better modeled by small_settlement_setting or plains_setting.",
      ],
      adjacentTags: ["small_settlement_setting", "plains_setting"],
    },
  }),
  defineConceptProjections("shadow_plane_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Shadow Plane or the Plane of Shadow.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("sky_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with open skies, storm clouds, or high-altitude aerial habitats.",
    },
  }),
  defineConceptProjections("small_settlement_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with villages, hamlets, small towns, or other low-density community settlements.",
      appliesWhen: [
        "The creature is repeatedly framed around villages, hamlets, or small town life.",
        "Its encounter identity is tied to low-density community spaces rather than major urban districts.",
      ],
      doesNotApplyWhen: [
        "The record only says the creature can appear near people.",
        "The creature is more strongly urban, rural, or fortress-coded than village-coded.",
      ],
      adjacentTags: ["urban_setting", "rural_setting"],
    },
  }),
  defineConceptProjections("swamp_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with bogs, marshes, fens, or mires.",
    },
  }),
  defineConceptProjections("temple_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard strongly associated with shrines, sanctums, cathedrals, monasteries, or consecrated sites.",
      appliesWhen: [
        "Sacred architecture, ritual sanctums, or faith-site defense is central to the hazard's scene identity.",
        "The hazard is naturally retrieved for shrine, sanctum, or temple protection rather than generic indoor defense.",
      ],
      doesNotApplyWhen: [
        "The hazard is merely magical or haunted without specific sacred-site framing.",
        "The stronger setting is dungeon_hazard or urban_hazard rather than temple-focused.",
      ],
      adjacentTags: ["dungeon_hazard", "appeasement_countermeasure"],
    },
  }),
  defineConceptProjections("temple_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with temples, shrines, monasteries, or other sacred encounter sites.",
    },
  }),
  defineConceptProjections("tian_xia_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with Tian Xia and its major cultural subregions, where Tian-rooted regional framing materially affects creature planning and retrieval.",
    },
  }),
  defineConceptProjections("tomb_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard strongly associated with tombs, barrows, mausoleums, crypts, or burial defenses.",
      appliesWhen: [
        "Burial sites, funerary wards, grave robbing countermeasures, or undead-rest protections are central to the hazard.",
        "A user would retrieve it for crypts, tombs, barrows, or similar funerary scenes.",
      ],
      doesNotApplyWhen: [
        "The hazard is only underground or haunted without real funerary or burial-site identity.",
        "The stronger placement is temple_hazard or generic dungeon_hazard.",
      ],
      adjacentTags: ["dungeon_hazard", "temple_hazard"],
    },
  }),
  defineConceptProjections("undead_war_torn_region_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with undead occupation, shattered crusader lands, haunted battlefields, refugee pressure, and ruined strongholds under deathless siege. In Golarion, this primarily corresponds to the Gravelands.",
      appliesWhen: [
        "Use when the creature is naturally retrieved for undead war-frontier planning: ruined forts, haunted battlefields, occupied borderlands, refugee routes, or crusader-collapse aftermath.",
        "The planning value comes from a region still actively shaped by undead war, occupation, or collapse rather than by a stable undead social order.",
      ],
      doesNotApplyWhen: [
        "The creature is simply undead, martial, or grim without a meaningful tie to an undead-ravaged frontier or occupied war zone.",
        "The stronger fit is organized_undead_society_setting because the retrieval hook is necromantic civilization rather than a shattered war front.",
      ],
      adjacentTags: ["organized_undead_society_setting", "battlefield_setting"],
    },
  }),
  defineConceptProjections("underground_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with caves, tunnels, crypts, or subterranean spaces.",
    },
  }),
  defineConceptProjections("urban_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard strongly associated with civic districts, alleys, rooftops, sewers, shops, or other built urban spaces.",
    },
  }),
  defineConceptProjections("urban_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with urban encounter scenes such as cities, streets, alleys, dense buildings, markets, or sewers.",
      appliesWhen: [
        "The creature is primarily framed as belonging in city or sewer encounter spaces.",
        "Urban placement is a recurring part of its identity, role, or habitat.",
      ],
      doesNotApplyWhen: [
        "The record merely mentions a city once.",
        "The creature can appear in towns but is not specifically urban-coded.",
        "The creature is better modeled by fortress_setting or small_settlement_setting.",
      ],
      positiveSignals: [
        "city guard roles",
        "sewer habitat",
        "street or alley patrol scenes",
        "market or district anchoring",
      ],
      negativeSignals: ["generic settlement mention", "single adventure location", "fortress-only residence"],
      adjacentTags: ["small_settlement_setting", "fortress_setting"],
    },
  }),
  defineConceptProjections("volcanic_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with volcanoes, calderas, lava, or magma.",
    },
  }),
  defineConceptProjections("wasteland_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with barren wastes, blasted wastelands, or desolate badlands.",
    },
  }),
  defineConceptProjections("wilderness_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard strongly associated with forests, trails, camps, ruins in the wild, or other outdoor expedition scenes.",
    },
  }),
] satisfies ConceptProjectionDeclaration[];
