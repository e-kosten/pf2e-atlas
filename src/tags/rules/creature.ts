import {
  ASTRAL_SETTING_CONTEXT_TEXT_ANCHORS,
  ASTRAL_SETTING_TEXT_ANCHORS,
  AQUATIC_SETTING_STRONG_TEXT_ANCHORS,
  AQUATIC_SETTING_WEAK_TEXT_ANCHORS,
  BONEYARD_SETTING_CONTEXT_TEXT_ANCHORS,
  BONEYARD_SETTING_TEXT_ANCHORS,
  COASTAL_SETTING_NAME_ANCHORS,
  COASTAL_SETTING_STRONG_TEXT_ANCHORS,
  COASTAL_SETTING_WEAK_TEXT_ANCHORS,
  DerivedTagRule,
  FIRST_WORLD_SETTING_CONTEXT_TEXT_ANCHORS,
  FIRST_WORLD_SETTING_TEXT_ANCHORS,
  FRESHWATER_SETTING_BLOCKER_TEXT_ANCHORS,
  FRESHWATER_SETTING_HABITAT_TEXT_NEAR,
  FRESHWATER_SETTING_NAME_ANCHORS,
  FRESHWATER_SETTING_STRONG_TEXT_ANCHORS,
  SCENE_ADJACENT_BLOCKER_TRAITS,
  STRONG_PROFESSION_NAME_ANCHORS,
  UNDEAD_GLOSSARY_FAMILIES,
  WEAK_PROFESSION_NAME_ANCHORS,
  phraseAnchor,
  tokenAnchor,
} from "../shared.js";

export const CREATURE_DERIVED_TAG_RULES: DerivedTagRule[] = [
  {
    tag: "undead_threat",
    category: "creature",
    anyOf: [
      { traitsAny: ["undead", "ghost", "spirit", "skeleton", "ghoul"] },
      { familiesAny: UNDEAD_GLOSSARY_FAMILIES },
    ],
  },
  {
    tag: "fey_threat",
    category: "creature",
    anyOf: [
      { traitsAny: ["fey"] },
    ],
  },
  {
    tag: "plant_threat",
    category: "creature",
    anyOf: [
      { traitsAny: ["plant", "fungus", "leshy"] },
    ],
  },
  {
    tag: "freshwater_setting",
    category: "creature",
    threshold: 2,
    noneOf: [
      { textAny: FRESHWATER_SETTING_BLOCKER_TEXT_ANCHORS },
    ],
    anyOf: [
      { score: 2, textAny: FRESHWATER_SETTING_STRONG_TEXT_ANCHORS },
      { score: 2, textNear: FRESHWATER_SETTING_HABITAT_TEXT_NEAR },
      { score: 1, textAny: FRESHWATER_SETTING_NAME_ANCHORS },
      { score: 1, traitsAny: ["water"] },
    ],
  },
  {
    tag: "aquatic_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 3, traitsAny: ["water", "aquatic", "amphibious"] },
      { score: 2, textAny: AQUATIC_SETTING_STRONG_TEXT_ANCHORS },
      { score: 1, textAny: AQUATIC_SETTING_WEAK_TEXT_ANCHORS },
    ],
  },
  {
    tag: "aquatic_setting",
    category: "creature",
    requiresTags: ["freshwater_setting"],
  },
  {
    tag: "coastal_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: COASTAL_SETTING_STRONG_TEXT_ANCHORS },
      { score: 1, textAny: COASTAL_SETTING_NAME_ANCHORS },
      { score: 1, textAny: COASTAL_SETTING_WEAK_TEXT_ANCHORS },
    ],
  },
  {
    tag: "astral_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["astral"] },
    ],
  },
  {
    tag: "astral_setting",
    category: "creature",
    anyOf: [
      { textAny: ASTRAL_SETTING_TEXT_ANCHORS },
    ],
  },
  {
    tag: "astral_setting",
    category: "creature",
    allOf: [
      { textAny: [tokenAnchor("astral")] },
      { textAny: ASTRAL_SETTING_CONTEXT_TEXT_ANCHORS },
    ],
  },
  {
    tag: "first_world_setting",
    category: "creature",
    allOf: [
      { textAny: FIRST_WORLD_SETTING_TEXT_ANCHORS },
    ],
    threshold: 1,
    anyOf: [
      { score: 1, traitsAny: ["fey", "tane"] },
      { score: 1, textAny: FIRST_WORLD_SETTING_CONTEXT_TEXT_ANCHORS },
    ],
  },
  {
    tag: "boneyard_setting",
    category: "creature",
    allOf: [
      { textAny: BONEYARD_SETTING_TEXT_ANCHORS },
    ],
    threshold: 1,
    anyOf: [
      { score: 1, traitsAny: ["psychopomp"] },
      { score: 1, textAny: BONEYARD_SETTING_CONTEXT_TEXT_ANCHORS },
    ],
  },
  {
    tag: "island_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: [tokenAnchor("island"), tokenAnchor("islands"), tokenAnchor("archipelago"), tokenAnchor("archipelagos"), tokenAnchor("atoll"), tokenAnchor("atolls")] },
      { score: 1, textAny: [tokenAnchor("isle"), tokenAnchor("isles")] },
    ],
  },
  {
    tag: "nautical_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          tokenAnchor("sailor"),
          tokenAnchor("sailors"),
          tokenAnchor("mariner"),
          tokenAnchor("mariners"),
          tokenAnchor("pirate"),
          tokenAnchor("pirates"),
          tokenAnchor("corsair"),
          tokenAnchor("corsairs"),
          tokenAnchor("dock"),
          tokenAnchor("docks"),
          tokenAnchor("bilge"),
          tokenAnchor("wreck"),
          tokenAnchor("crew"),
          tokenAnchor("crews"),
          tokenAnchor("rigging"),
          tokenAnchor("mast"),
          tokenAnchor("masts"),
          tokenAnchor("shipwreck"),
          tokenAnchor("shipwrecks"),
          phraseAnchor("shipwreck"),
        ],
      },
      { score: 1, textAny: [tokenAnchor("ship"), tokenAnchor("ships"), tokenAnchor("captain"), tokenAnchor("vessel"), tokenAnchor("vessels"), tokenAnchor("deck"), tokenAnchor("decks")] },
      { score: 1, textAny: [tokenAnchor("harbor"), tokenAnchor("harbors")] },
    ],
  },
  {
    tag: "forest_setting",
    category: "creature",
    anyOf: [
      {
        textAny: [
          tokenAnchor("forest"),
          tokenAnchor("forests"),
          tokenAnchor("woodland"),
          tokenAnchor("woodlands"),
          tokenAnchor("woods"),
          tokenAnchor("grove"),
          tokenAnchor("groves"),
          tokenAnchor("briar"),
          tokenAnchor("jungle"),
          tokenAnchor("jungles"),
          tokenAnchor("rainforest"),
        ],
      },
    ],
  },
  {
    tag: "plains_setting",
    category: "creature",
    anyOf: [
      {
        textAny: [
          tokenAnchor("plains"),
          tokenAnchor("grassland"),
          tokenAnchor("grasslands"),
          tokenAnchor("prairie"),
          tokenAnchor("prairies"),
          tokenAnchor("savanna"),
          tokenAnchor("savannah"),
          tokenAnchor("steppe"),
          tokenAnchor("steppes"),
        ],
      },
    ],
  },
  {
    tag: "canyon_setting",
    category: "creature",
    anyOf: [
      {
        textAny: [
          tokenAnchor("canyon"),
          tokenAnchor("canyons"),
          tokenAnchor("gorge"),
          tokenAnchor("gorges"),
          tokenAnchor("mesa"),
          tokenAnchor("mesas"),
          tokenAnchor("badlands"),
        ],
      },
    ],
  },
  {
    tag: "swamp_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["boggard"] },
      {
        textAny: [
          tokenAnchor("swamp"),
          tokenAnchor("swamps"),
          tokenAnchor("bog"),
          tokenAnchor("bogs"),
          tokenAnchor("marsh"),
          tokenAnchor("marshes"),
          tokenAnchor("fen"),
          tokenAnchor("fens"),
          tokenAnchor("mire"),
          tokenAnchor("mires"),
        ],
      },
    ],
  },
  {
    tag: "underground_setting",
    category: "creature",
    anyOf: [
      {
        textAny: [
          tokenAnchor("cave"),
          tokenAnchor("caves"),
          tokenAnchor("cavern"),
          tokenAnchor("caverns"),
          tokenAnchor("underground"),
          tokenAnchor("tunnel"),
          tokenAnchor("tunnels"),
          tokenAnchor("subterranean"),
          tokenAnchor("underworld"),
          tokenAnchor("depths"),
          tokenAnchor("crypt"),
          tokenAnchor("crypts"),
        ],
      },
    ],
  },
  {
    tag: "urban_setting",
    category: "creature",
    anyOf: [
      {
        textAny: [
          tokenAnchor("city"),
          tokenAnchor("cities"),
          tokenAnchor("urban"),
          tokenAnchor("street"),
          tokenAnchor("streets"),
          tokenAnchor("alley"),
          tokenAnchor("alleys"),
          tokenAnchor("market"),
          tokenAnchor("markets"),
          tokenAnchor("sewer"),
          tokenAnchor("sewers"),
          tokenAnchor("town"),
          tokenAnchor("towns"),
        ],
      },
    ],
  },
  {
    tag: "arctic_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: [tokenAnchor("arctic"), tokenAnchor("snow"), tokenAnchor("snows"), tokenAnchor("tundra"), tokenAnchor("frozen"), tokenAnchor("glacier"), tokenAnchor("glaciers"), tokenAnchor("icebound")] },
      { score: 1, textAny: [tokenAnchor("ice"), tokenAnchor("icy")] },
    ],
  },
  {
    tag: "desert_setting",
    category: "creature",
    anyOf: [
      { textAny: [tokenAnchor("desert"), tokenAnchor("deserts"), tokenAnchor("dune"), tokenAnchor("dunes"), tokenAnchor("sand"), tokenAnchor("arid")] },
    ],
  },
  {
    tag: "wasteland_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          tokenAnchor("wasteland"),
          tokenAnchor("wastelands"),
          tokenAnchor("wastes"),
          tokenAnchor("barren"),
        ],
      },
      { score: 1, textAny: [tokenAnchor("blasted")] },
    ],
  },
  {
    tag: "mountain_setting",
    category: "creature",
    anyOf: [
      { textAny: [tokenAnchor("mountain"), tokenAnchor("mountains"), tokenAnchor("cliff"), tokenAnchor("cliffs"), tokenAnchor("peak"), tokenAnchor("peaks"), tokenAnchor("crag"), tokenAnchor("crags"), tokenAnchor("alp"), tokenAnchor("alpine"), tokenAnchor("pass"), tokenAnchor("passes")] },
    ],
  },
  {
    tag: "graveyard_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          tokenAnchor("graveyard"),
          tokenAnchor("graveyards"),
          tokenAnchor("cemetery"),
          tokenAnchor("cemeteries"),
          tokenAnchor("mausoleum"),
          tokenAnchor("mausoleums"),
          tokenAnchor("barrow"),
          tokenAnchor("barrows"),
          tokenAnchor("sepulcher"),
          tokenAnchor("sepulchers"),
          tokenAnchor("cairn"),
          tokenAnchor("cairns"),
          phraseAnchor("burial ground"),
          phraseAnchor("burial grounds"),
        ],
      },
      { score: 1, textAny: [tokenAnchor("crypt"), tokenAnchor("crypts")] },
      { score: 1, textAny: [tokenAnchor("tomb"), tokenAnchor("tombs")] },
    ],
  },
  {
    tag: "ruins_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textNear: [
          {
            terms: [
              tokenAnchor("ruin"),
              tokenAnchor("ruins"),
              tokenAnchor("derelict"),
              tokenAnchor("crumbling"),
              tokenAnchor("fallen"),
              tokenAnchor("collapsed"),
              tokenAnchor("hall"),
              tokenAnchor("temple"),
            ],
            window: 3,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
      {
        score: 2,
        textAny: [
          phraseAnchor("ancient ruins"),
          phraseAnchor("ruins of"),
          phraseAnchor("collapsed temple"),
          phraseAnchor("ruined temple"),
          phraseAnchor("ruined city"),
          phraseAnchor("lost city"),
          phraseAnchor("derelict structures"),
          phraseAnchor("derelict structure"),
        ],
      },
      { score: 1, textAny: [tokenAnchor("ruins"), tokenAnchor("ruin"), tokenAnchor("derelict")] },
    ],
  },
  {
    tag: "temple_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          tokenAnchor("temple"),
          tokenAnchor("temples"),
          tokenAnchor("shrine"),
          tokenAnchor("shrines"),
          tokenAnchor("cathedral"),
          tokenAnchor("cathedrals"),
          tokenAnchor("monastery"),
          tokenAnchor("monasteries"),
          tokenAnchor("chapel"),
          tokenAnchor("chapels"),
        ],
      },
      { score: 1, textAny: [phraseAnchor("place of worship"), phraseAnchor("house of worship")] },
    ],
  },
  {
    tag: "fortress_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          tokenAnchor("fortress"),
          tokenAnchor("fortresses"),
          tokenAnchor("castle"),
          tokenAnchor("castles"),
          tokenAnchor("citadel"),
          tokenAnchor("citadels"),
          tokenAnchor("stronghold"),
          tokenAnchor("strongholds"),
        ],
      },
      { score: 2, textAny: [phraseAnchor("sky citadel")] },
      { score: 1, textAny: [tokenAnchor("fort"), tokenAnchor("forts")] },
    ],
  },
  {
    tag: "volcanic_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          tokenAnchor("volcanic"),
          tokenAnchor("volcano"),
          tokenAnchor("volcanoes"),
          tokenAnchor("lava"),
          tokenAnchor("magma"),
          tokenAnchor("caldera"),
        ],
      },
      { score: 1, textAny: [tokenAnchor("ash"), tokenAnchor("ashen"), tokenAnchor("cinder"), tokenAnchor("cinders")] },
    ],
  },
  {
    tag: "profession_npc",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: STRONG_PROFESSION_NAME_ANCHORS },
      { score: 1, textAny: WEAK_PROFESSION_NAME_ANCHORS },
      { score: 1, traitsAny: ["humanoid", "human"] },
    ],
  },
  {
    tag: "scene_adjacent",
    category: "creature",
    requiresTags: ["profession_npc"],
    noneOf: [
      { traitsAny: SCENE_ADJACENT_BLOCKER_TRAITS },
      { familiesAny: UNDEAD_GLOSSARY_FAMILIES },
    ],
  },
];
