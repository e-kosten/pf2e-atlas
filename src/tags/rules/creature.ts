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

const CARNIVAL_SHOW_NAME_ANCHORS = [
  tokenAnchor("carnival", "name"),
  tokenAnchor("circus", "name"),
  tokenAnchor("clown", "name"),
  tokenAnchor("carny", "name"),
  tokenAnchor("jester", "name"),
];

const CARNIVAL_SHOW_TEXT_ANCHORS = [
  tokenAnchor("carnival"),
  tokenAnchor("carnivals"),
  tokenAnchor("circus"),
  tokenAnchor("circuses"),
  tokenAnchor("clown"),
  tokenAnchor("clowns"),
  tokenAnchor("carny"),
  tokenAnchor("carnies"),
  tokenAnchor("jester"),
  tokenAnchor("jesters"),
  tokenAnchor("barker"),
  tokenAnchor("barkers"),
  tokenAnchor("sideshow"),
  tokenAnchor("sideshows"),
  phraseAnchor("traveling show"),
  phraseAnchor("traveling shows"),
  phraseAnchor("traveling circus"),
  phraseAnchor("traveling carnival"),
  phraseAnchor("court jester"),
];

const CARNIVAL_SHOW_CONTEXT_TEXT_NEAR = [
  {
    terms: [
      tokenAnchor("performer"),
      tokenAnchor("performers"),
      tokenAnchor("entertainer"),
      tokenAnchor("entertainers"),
      tokenAnchor("circus"),
      tokenAnchor("circuses"),
      tokenAnchor("carnival"),
      tokenAnchor("carnivals"),
      tokenAnchor("traveling"),
      tokenAnchor("jester"),
      tokenAnchor("jesters"),
    ],
    window: 4,
    scope: "description" as const,
    minTermsMatched: 2,
  },
];

const LIVING_TOY_NAME_ANCHORS = [
  tokenAnchor("doll", "name"),
  tokenAnchor("dolls", "name"),
  tokenAnchor("puppet", "name"),
  tokenAnchor("puppets", "name"),
  tokenAnchor("mannequin", "name"),
  tokenAnchor("mannequins", "name"),
  tokenAnchor("marionette", "name"),
  tokenAnchor("marionettes", "name"),
];

const LIVING_TOY_TEXT_ANCHORS = [
  tokenAnchor("doll"),
  tokenAnchor("dolls"),
  tokenAnchor("puppet"),
  tokenAnchor("puppets"),
  tokenAnchor("mannequin"),
  tokenAnchor("mannequins"),
  tokenAnchor("marionette"),
  tokenAnchor("marionettes"),
  tokenAnchor("plaything"),
  tokenAnchor("playthings"),
  tokenAnchor("toylike"),
  phraseAnchor("toy like"),
  phraseAnchor("soulbound doll"),
  phraseAnchor("dressmaker s dummy"),
];

const MASK_MOTIF_NAME_ANCHORS = [
  tokenAnchor("mask", "name"),
  tokenAnchor("masked", "name"),
  tokenAnchor("veil", "name"),
  tokenAnchor("veiled", "name"),
];

const MASK_MOTIF_TEXT_ANCHORS = [
  phraseAnchor("wears a mask"),
  phraseAnchor("wearing a mask"),
  phraseAnchor("masked face"),
  phraseAnchor("ceremonial mask"),
  phraseAnchor("death mask"),
  phraseAnchor("stone mask"),
  phraseAnchor("ornate mask"),
  phraseAnchor("wears a veil"),
  phraseAnchor("wearing a veil"),
  phraseAnchor("veiled face"),
  phraseAnchor("face hidden"),
  phraseAnchor("hides its face"),
  phraseAnchor("hide its face"),
  phraseAnchor("conceals its face"),
  phraseAnchor("conceal its face"),
];

const FACELESS_HORROR_NAME_ANCHORS = [
  tokenAnchor("faceless", "name"),
  tokenAnchor("featureless", "name"),
];

const FACELESS_HORROR_TEXT_ANCHORS = [
  phraseAnchor("no face"),
  phraseAnchor("face gone"),
  phraseAnchor("stolen face"),
  phraseAnchor("stolen faces"),
  phraseAnchor("featureless face"),
  phraseAnchor("blank face"),
  phraseAnchor("empty face"),
  phraseAnchor("faceless horror"),
  phraseAnchor("faces are stolen"),
  phraseAnchor("faces were stolen"),
];

const DISGUISED_PRETENDER_NAME_ANCHORS = [
  tokenAnchor("pretender", "name"),
  tokenAnchor("impostor", "name"),
  tokenAnchor("imposter", "name"),
  tokenAnchor("masquerade", "name"),
  tokenAnchor("disguise", "name"),
];

const DISGUISED_PRETENDER_TEXT_ANCHORS = [
  phraseAnchor("false identity"),
  phraseAnchor("false identities"),
  phraseAnchor("assume a false identity"),
  phraseAnchor("assumes a false identity"),
  phraseAnchor("impersonates"),
  phraseAnchor("impersonate"),
  phraseAnchor("infiltrates"),
  phraseAnchor("infiltrate"),
  phraseAnchor("wears another face"),
  phraseAnchor("wear another face"),
  phraseAnchor("take on the appearance of"),
  phraseAnchor("takes on the appearance of"),
  phraseAnchor("assume the form of"),
  phraseAnchor("assumes the form of"),
  phraseAnchor("replace the target"),
  phraseAnchor("replaces the target"),
  phraseAnchor("masquerades as"),
  phraseAnchor("shapeshift"),
  phraseAnchor("shapechange"),
  phraseAnchor("disguised as"),
];

const DISGUISED_PRETENDER_CONTEXT_TEXT_NEAR = [
  {
    terms: [
      tokenAnchor("disguise"),
      tokenAnchor("disguised"),
      tokenAnchor("disguises"),
      tokenAnchor("identity"),
      tokenAnchor("impersonate"),
      tokenAnchor("impersonates"),
      tokenAnchor("infiltrate"),
      tokenAnchor("infiltrates"),
      tokenAnchor("masquerade"),
      tokenAnchor("pretend"),
      tokenAnchor("pretender"),
      tokenAnchor("replace"),
      tokenAnchor("form"),
      tokenAnchor("guise"),
    ],
    window: 6,
    scope: "description" as const,
    minTermsMatched: 2,
  },
];

const BOUND_OBJECT_LIVING_BLOCKER_TRAITS = [
  "aberration",
  "animal",
  "beast",
  "dragon",
  "elemental",
  "fey",
  "fiend",
  "giant",
  "humanoid",
  "monitor",
  "plant",
  "undead",
];

const BOUND_OBJECT_MIMIC_BLOCKERS = [
  tokenAnchor("mimic"),
  tokenAnchor("mimics"),
  phraseAnchor("mimic chest"),
  phraseAnchor("mimic chests"),
];

const BOUND_OBJECT_FIGURATIVE_BLOCKERS = [
  phraseAnchor("animated discussion"),
  phraseAnchor("animated debate"),
  phraseAnchor("animated conversation"),
  phraseAnchor("animated expression"),
  phraseAnchor("animatedly"),
];

const BOUND_OBJECT_ANIMATION_TEXT_ANCHORS = [
  tokenAnchor("animated"),
  tokenAnchor("animate"),
  tokenAnchor("animates"),
  tokenAnchor("animation"),
  phraseAnchor("brought to life"),
  phraseAnchor("brought to unlife"),
  phraseAnchor("comes to life"),
];

const BOUND_OBJECT_OBJECT_TEXT_ANCHORS = [
  tokenAnchor("object"),
  tokenAnchor("objects"),
  tokenAnchor("armor"),
  tokenAnchor("armors"),
  tokenAnchor("broom"),
  tokenAnchor("brooms"),
  tokenAnchor("cookware"),
  tokenAnchor("silverware"),
  tokenAnchor("blade"),
  tokenAnchor("blades"),
  tokenAnchor("axe"),
  tokenAnchor("axes"),
  tokenAnchor("fireplace"),
  tokenAnchor("fireplaces"),
  tokenAnchor("vessel"),
  tokenAnchor("vessels"),
  tokenAnchor("cauldron"),
  tokenAnchor("cauldrons"),
  tokenAnchor("table"),
  tokenAnchor("tables"),
  tokenAnchor("chair"),
  tokenAnchor("chairs"),
  tokenAnchor("stool"),
  tokenAnchor("stools"),
  tokenAnchor("tool"),
  tokenAnchor("tools"),
  tokenAnchor("utensil"),
  tokenAnchor("utensils"),
  tokenAnchor("furniture"),
];

const BOUND_OBJECT_STATUE_TEXT_ANCHORS = [
  tokenAnchor("statue"),
  tokenAnchor("statues"),
  tokenAnchor("effigy"),
  tokenAnchor("effigies"),
  tokenAnchor("idol"),
  tokenAnchor("idols"),
  tokenAnchor("monument"),
  tokenAnchor("monuments"),
];

const BOUND_OBJECT_STATUE_CONTEXT_TEXT_ANCHORS = [
  tokenAnchor("animated"),
  tokenAnchor("animate"),
  tokenAnchor("animates"),
  tokenAnchor("animation"),
  tokenAnchor("guardian"),
  tokenAnchor("guardians"),
  tokenAnchor("sentinel"),
  tokenAnchor("sentinels"),
  tokenAnchor("watch"),
  tokenAnchor("watcher"),
  tokenAnchor("watchers"),
  tokenAnchor("protect"),
  tokenAnchor("protects"),
  phraseAnchor("comes to life"),
  phraseAnchor("brought to life"),
  phraseAnchor("stands guard"),
  phraseAnchor("standing guard"),
];

const TRICKSTER_CHAOS_NAME_ANCHORS = [
  tokenAnchor("trickster", "name"),
  tokenAnchor("tricksters", "name"),
];

const TRICKSTER_CHAOS_TEXT_ANCHORS = [
  tokenAnchor("trickster"),
  tokenAnchor("tricksters"),
  tokenAnchor("prank"),
  tokenAnchor("pranks"),
  tokenAnchor("prankster"),
  tokenAnchor("pranksters"),
  tokenAnchor("mischievous"),
  tokenAnchor("whimsical"),
  tokenAnchor("playful"),
  phraseAnchor("delight in humor"),
  phraseAnchor("harmless pranks"),
  phraseAnchor("playing pranks"),
  phraseAnchor("play tricks"),
];

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
  {
    tag: "carnival_show",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: CARNIVAL_SHOW_NAME_ANCHORS },
      { score: 2, textAny: CARNIVAL_SHOW_TEXT_ANCHORS },
      { score: 1, textNear: CARNIVAL_SHOW_CONTEXT_TEXT_NEAR },
    ],
  },
  {
    tag: "living_toy",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: LIVING_TOY_NAME_ANCHORS },
      { score: 2, textAny: LIVING_TOY_TEXT_ANCHORS },
    ],
  },
  {
    tag: "mask_motif",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: MASK_MOTIF_NAME_ANCHORS },
      { score: 2, textAny: MASK_MOTIF_TEXT_ANCHORS },
      { score: 1, textNear: [
        {
          terms: [
            tokenAnchor("mask"),
            tokenAnchor("masked"),
            tokenAnchor("veil"),
            tokenAnchor("veiled"),
            tokenAnchor("face"),
            tokenAnchor("faces"),
            tokenAnchor("head"),
            tokenAnchor("features"),
            tokenAnchor("conceal"),
            tokenAnchor("conceals"),
          ],
          window: 5,
          scope: "description" as const,
          minTermsMatched: 2,
        },
      ] },
    ],
  },
  {
    tag: "faceless_horror",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: FACELESS_HORROR_NAME_ANCHORS },
      { score: 2, textAny: FACELESS_HORROR_TEXT_ANCHORS },
      { score: 1, textNear: [
        {
          terms: [
            tokenAnchor("face"),
            tokenAnchor("faces"),
            tokenAnchor("faceless"),
            tokenAnchor("featureless"),
            tokenAnchor("blank"),
            tokenAnchor("empty"),
            tokenAnchor("stolen"),
            tokenAnchor("gone"),
            tokenAnchor("hidden"),
          ],
          window: 5,
          scope: "description" as const,
          minTermsMatched: 2,
        },
      ] },
    ],
  },
  {
    tag: "disguised_pretender",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: DISGUISED_PRETENDER_NAME_ANCHORS },
      { score: 2, textAny: DISGUISED_PRETENDER_TEXT_ANCHORS },
      { score: 1, textNear: DISGUISED_PRETENDER_CONTEXT_TEXT_NEAR },
    ],
  },
  {
    tag: "animated_object",
    category: "creature",
    threshold: 2,
    noneOf: [
      { traitsAny: BOUND_OBJECT_LIVING_BLOCKER_TRAITS },
      { textAny: BOUND_OBJECT_MIMIC_BLOCKERS },
      { textAny: BOUND_OBJECT_FIGURATIVE_BLOCKERS },
    ],
    anyOf: [
      {
        score: 2,
        textNear: [
          {
            terms: [...BOUND_OBJECT_ANIMATION_TEXT_ANCHORS, ...BOUND_OBJECT_OBJECT_TEXT_ANCHORS],
            window: 4,
            scope: "either",
            minTermsMatched: 2,
          },
        ],
      },
    ],
  },
  {
    tag: "animated_statue",
    category: "creature",
    threshold: 2,
    noneOf: [
      { textAny: BOUND_OBJECT_MIMIC_BLOCKERS },
      { textAny: BOUND_OBJECT_FIGURATIVE_BLOCKERS },
      { textAny: [phraseAnchor("stone animal"), phraseAnchor("stone animals"), phraseAnchor("stone beast"), phraseAnchor("stone beasts"), phraseAnchor("stone creature"), phraseAnchor("stone creatures")] },
    ],
    anyOf: [
      {
        score: 2,
        textNear: [
          {
            terms: [...BOUND_OBJECT_STATUE_TEXT_ANCHORS, ...BOUND_OBJECT_STATUE_CONTEXT_TEXT_ANCHORS],
            window: 5,
            scope: "either",
            minTermsMatched: 2,
          },
        ],
      },
    ],
  },
  {
    tag: "trickster_chaos",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: TRICKSTER_CHAOS_NAME_ANCHORS },
      { score: 2, textAny: TRICKSTER_CHAOS_TEXT_ANCHORS },
    ],
  },
];
