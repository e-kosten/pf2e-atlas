import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const creatureSettingProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.CREATURE.SETTING_HABITAT_SETTING, {
    aquatic_setting: {
      description: "Strongly associated with open water, underwater spaces, or aquatic environments.",
    },
    arctic_setting: {
      description: "Strongly associated with snow, ice, tundra, or frozen reaches.",
    },
    canyon_setting: {
      description: "Strongly associated with canyons, gorges, mesas, or badlands.",
    },
    coastal_setting: {
      description: "Strongly associated with coasts, shores, reefs, or littoral edges.",
    },
    desert_setting: {
      description: "Strongly associated with dunes, sand, or arid wastes.",
    },
    forest_setting: {
      description: "Strongly associated with forests, jungles, groves, or briar-choked wilds.",
    },
    freshwater_setting: {
      description: "Strongly associated with rivers, lakes, ponds, streams, springs, or other inland waters.",
    },
    island_setting: {
      description: "Strongly associated with islands, archipelagos, or isolated isles.",
    },
    jungle_setting: {
      description: "Strongly associated with jungles, rainforests, or dense tropical canopies.",
    },
    mountain_setting: {
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
    plains_setting: {
      description: "Strongly associated with open plains, grasslands, prairies, or savannas.",
    },
    sky_setting: {
      description: "Strongly associated with open skies, storm clouds, or high-altitude aerial habitats.",
    },
    swamp_setting: {
      description: "Strongly associated with bogs, marshes, fens, or mires.",
    },
    underground_setting: {
      description: "Strongly associated with caves, tunnels, crypts, or subterranean spaces.",
    },
    volcanic_setting: {
      description: "Strongly associated with volcanoes, calderas, lava, or magma.",
    },
    wasteland_setting: {
      description: "Strongly associated with barren wastes, blasted wastelands, or desolate badlands.",
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.CREATURE.SETTING_PLANAR_SETTING, {
    abaddon_setting: {
      description: "Strongly associated with Abaddon, daemons, or soul-devouring lower-planar wastelands.",
      nativeOntologyPolicy: "aggregates_native_signals",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    abyss_setting: {
      description: "Strongly associated with the Abyss, demon realms, or qlippoth-infested outer rifts.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    astral_setting: {
      description: "Strongly associated with Astral Plane scenes, silver-void travel, or stable portal routes.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    axis_setting: {
      description: "Strongly associated with Axis, the Eternal City, or its lawful planar order.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    boneyard_setting: {
      description: "Strongly associated with the Boneyard, psychopomp duties, or soul-processing afterlife scenes.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    cosmic_framework_setting: {
isComposite: true,
      description:
        "Strongly associated with the cosmic framework planes of Axis, the Boneyard, and the Maelstrom, which govern order, judgment, and transformative change.",
      nativeOntologyPolicy: "aggregates_native_signals",
      adjacentTags: ["axis_setting", "boneyard_setting", "maelstrom_setting"],
      compositeOfAnyTags: ["axis_setting", "boneyard_setting", "maelstrom_setting"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    dreamlands_setting: {
      description:
        "Strongly associated with the Dreamlands, Leng-linked dream roads, or iconic denizens that dwell there.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    elemental_plane_setting: {
isComposite: true,
      description: "Strongly associated with one of the elemental planes of Fire, Air, Water, or Earth.",
      nativeOntologyPolicy: "aggregates_native_signals",
      adjacentTags: [
        "plane_of_fire_setting",
        "plane_of_air_setting",
        "plane_of_water_setting",
        "plane_of_earth_setting",
      ],
      compositeOfAnyTags: [
        "plane_of_fire_setting",
        "plane_of_air_setting",
        "plane_of_water_setting",
        "plane_of_earth_setting",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    elysium_setting: {
      description: "Strongly associated with Elysium, azatas, or freedom-driven celestial heroism.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    ethereal_setting: {
      description:
        "Strongly associated with the Ethereal Plane, its native wildlife, or recurring hunting and travel routes through it.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    first_world_setting: {
      description: "Strongly associated with the First World, fey realms, or thin-boundary crossings into that plane.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    heaven_setting: {
      description: "Strongly associated with Heaven, archon hosts, or ordered celestial service.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hell_setting: {
      description: "Strongly associated with Hell, devil hosts, or infernal hierarchy.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    lower_plane_setting: {
isComposite: true,
      description: "Strongly associated with the lower planes of Hell, the Abyss, or Abaddon.",
      nativeOntologyPolicy: "aggregates_native_signals",
      adjacentTags: ["hell_setting", "abyss_setting", "abaddon_setting"],
      compositeOfAnyTags: ["hell_setting", "abyss_setting", "abaddon_setting"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    maelstrom_setting: {
      description: "Strongly associated with the Maelstrom, proteans, or its chaotic planar fringes.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    nirvana_setting: {
      description: "Strongly associated with Nirvana, enlightened celestials, or benevolent contemplative service.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    plane_of_air_setting: {
      description:
        "Strongly associated with the Plane of Air, its native denizens, or its endless winds and cloud realms.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    plane_of_earth_setting: {
      description:
        "Strongly associated with the Plane of Earth, its native denizens, or its crystal caverns and stonebound realms.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    plane_of_fire_setting: {
      description:
        "Strongly associated with the Plane of Fire, its native denizens, or its infernal-bright cityscapes and battlefields.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    plane_of_water_setting: {
      description:
        "Strongly associated with the Plane of Water, its native denizens, or its endless seas and oceanic realms.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    shadow_plane_setting: {
      description: "Strongly associated with the Shadow Plane or the Plane of Shadow.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    upper_plane_setting: {
isComposite: true,
      description: "Strongly associated with the upper planes of Heaven, Nirvana, or Elysium.",
      nativeOntologyPolicy: "aggregates_native_signals",
      adjacentTags: ["heaven_setting", "nirvana_setting", "elysium_setting"],
      compositeOfAnyTags: ["heaven_setting", "nirvana_setting", "elysium_setting"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.CREATURE.SETTING_REGIONAL_SETTING, {
    alien_technology_wasteland_setting: {
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
    darklands_setting: {
      description:
        "Strongly associated with the Darklands as a civilization-bearing underworld macro-region rather than generic underground terrain or one cave network.",
    },
    demonic_scar_frontier_setting: {
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
    gothic_horror_land_setting: {
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
    magic_blight_wasteland_setting: {
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
    mwangi_setting: {
      description:
        "Strongly associated with the Mwangi Expanse, its jungle polities, and Mwangi-rooted regional framing that materially affects creature planning and retrieval.",
    },
    organized_undead_society_setting: {
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
    tian_xia_setting: {
      description:
        "Strongly associated with Tian Xia and its major cultural subregions, where Tian-rooted regional framing materially affects creature planning and retrieval.",
    },
    undead_war_torn_region_setting: {
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
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.CREATURE.SETTING_SITE_SETTING, {
    battlefield_setting: {
      description:
        "Strongly associated with battlefields, war zones, organized military deployments, or mass-combat scenes.",
    },
    fortress_setting: {
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
    graveyard_setting: {
      description: "Strongly associated with cemeteries, tombs, barrows, or burial grounds.",
    },
    nautical_setting: {
      description: "Strongly associated with ships, sailors, wrecks, or harbors.",
    },
    ruins_setting: {
      description: "Strongly associated with ancient ruins or derelict structures.",
    },
    rural_setting: {
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
    small_settlement_setting: {
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
    temple_setting: {
      description: "Strongly associated with temples, shrines, monasteries, or other sacred encounter sites.",
    },
    urban_setting: {
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
] satisfies CategoryProjectionFamilyBlock<"creature">[];
