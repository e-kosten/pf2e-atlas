import {
  ABYSS_SETTING_TEXT_ANCHORS,
  ASTRAL_SETTING_CONTEXT_TEXT_ANCHORS,
  ASTRAL_SETTING_TEXT_ANCHORS,
  AQUATIC_SETTING_STRONG_TEXT_ANCHORS,
  AQUATIC_SETTING_WEAK_TEXT_ANCHORS,
  BONEYARD_SETTING_CONTEXT_TEXT_ANCHORS,
  BONEYARD_SETTING_TEXT_ANCHORS,
  CIVIC_NPC_BLOCKER_TRAITS,
  COASTAL_SETTING_NAME_ANCHORS,
  COASTAL_SETTING_STRONG_TEXT_ANCHORS,
  COASTAL_SETTING_WEAK_TEXT_ANCHORS,
  DerivedTagRule,
  DREAMLANDS_SETTING_NAME_ANCHORS,
  DREAMLANDS_SETTING_TEXT_ANCHORS,
  ETHEREAL_SETTING_TEXT_ANCHORS,
  ELYSIUM_SETTING_TEXT_ANCHORS,
  FIRST_WORLD_SETTING_CONTEXT_TEXT_ANCHORS,
  FIRST_WORLD_SETTING_TEXT_ANCHORS,
  FRESHWATER_SETTING_BLOCKER_TEXT_ANCHORS,
  FRESHWATER_SETTING_NAME_ANCHORS,
  FRESHWATER_SETTING_STRONG_TEXT_ANCHORS,
  FRESHWATER_SETTING_TEXT_ANCHORS,
  HEAVEN_SETTING_TEXT_ANCHORS,
  HELL_SETTING_TEXT_ANCHORS,
  MAELSTROM_SETTING_TEXT_ANCHORS,
  NIRVANA_SETTING_TEXT_ANCHORS,
  SHADOW_PLANE_SETTING_TEXT_ANCHORS,
  STRONG_PROFESSION_NAME_ANCHORS,
  UNDEAD_GLOSSARY_FAMILIES,
  WEAK_PROFESSION_NAME_ANCHORS,
  patternAltAnchor,
  patternAnchor,
} from "../shared.js";

const CARNIVAL_SHOW_NAME_ANCHORS = [
  patternAnchor("carnival", "name"),
  patternAnchor("circus", "name"),
  patternAnchor("clown", "name"),
  patternAnchor("carny", "name"),
  patternAnchor("jester", "name"),
];

const CARNIVAL_SHOW_TEXT_ANCHORS = [
  patternAnchor("carnival"),
  patternAnchor("carnivals"),
  patternAnchor("circus"),
  patternAnchor("circuses"),
  patternAnchor("clown"),
  patternAnchor("clowns"),
  patternAnchor("carny"),
  patternAnchor("carnies"),
  patternAnchor("jester"),
  patternAnchor("jesters"),
  patternAnchor("barker"),
  patternAnchor("barkers"),
  patternAnchor("sideshow"),
  patternAnchor("sideshows"),
  patternAnchor("traveling show"),
  patternAnchor("traveling shows"),
  patternAnchor("traveling circus"),
  patternAnchor("traveling carnival"),
  patternAnchor("court jester"),
];

const CARNIVAL_SHOW_CONTEXT_TEXT_NEAR = [
  {
    all: [
      patternAltAnchor(["performer", "performers", "entertainer", "entertainers", "barker", "barkers"]),
      patternAltAnchor(["circus", "circuses", "carnival", "carnivals", "traveling show", "traveling circus", "traveling carnival", "jester", "jesters"]),
    ],
    window: 6,
    scope: "description" as const,
  },
];

const LIVING_TOY_NAME_ANCHORS = [
  patternAnchor("doll", "name"),
  patternAnchor("dolls", "name"),
  patternAnchor("puppet", "name"),
  patternAnchor("puppets", "name"),
  patternAnchor("mannequin", "name"),
  patternAnchor("mannequins", "name"),
  patternAnchor("marionette", "name"),
  patternAnchor("marionettes", "name"),
];

const LIVING_TOY_TEXT_ANCHORS = [
  patternAnchor("doll"),
  patternAnchor("dolls"),
  patternAnchor("puppet"),
  patternAnchor("puppets"),
  patternAnchor("mannequin"),
  patternAnchor("mannequins"),
  patternAnchor("marionette"),
  patternAnchor("marionettes"),
  patternAnchor("plaything"),
  patternAnchor("playthings"),
  patternAnchor("toylike"),
  patternAnchor("toy like"),
  patternAnchor("soulbound doll"),
  patternAnchor("dressmaker s dummy"),
];

const LIVING_ARTWORK_NAME_ANCHORS = [
  patternAnchor("living graffiti", "name"),
  patternAnchor("living mural", "name"),
];

const LIVING_ARTWORK_TEXT_ANCHORS = [
  patternAnchor("living graffiti"),
  patternAnchor("living mural"),
  patternAnchor("painting or drawing that has come to life"),
  patternAnchor("painting has come to life"),
  patternAnchor("drawing has come to life"),
  patternAnchor("two dimensional portrait given life"),
  patternAnchor("illustration painted in three dimensions"),
  patternAnchor("artwork brought to life"),
];

const MASK_MOTIF_NAME_ANCHORS = [
  patternAnchor("mask", "name"),
  patternAnchor("masked", "name"),
  patternAnchor("veil", "name"),
  patternAnchor("veiled", "name"),
];

const MASK_MOTIF_TEXT_ANCHORS = [
  patternAnchor("masks"),
  patternAnchor("wears a mask"),
  patternAnchor("wearing a mask"),
  patternAnchor("masked face"),
  patternAnchor("ceremonial mask"),
  patternAnchor("death mask"),
  patternAnchor("stone mask"),
  patternAnchor("ornate mask"),
  patternAnchor("wears a veil"),
  patternAnchor("wearing a veil"),
  patternAnchor("veiled face"),
  patternAnchor("face hidden"),
  patternAnchor("hides its face"),
  patternAnchor("hide its face"),
  patternAnchor("conceals its face"),
  patternAnchor("conceal its face"),
];

const FACELESS_HORROR_NAME_ANCHORS = [
  patternAnchor("faceless", "name"),
  patternAnchor("featureless", "name"),
];

const FACELESS_HORROR_TEXT_ANCHORS = [
  patternAnchor("no face"),
  patternAnchor("face gone"),
  patternAnchor("stolen face"),
  patternAnchor("stolen faces"),
  patternAnchor("variant faceless butcher"),
  patternAnchor("featureless face"),
  patternAnchor("blank face"),
  patternAnchor("empty face"),
  patternAnchor("faceless horror"),
  patternAnchor("faces are stolen"),
  patternAnchor("faces were stolen"),
];

const DISGUISED_PRETENDER_NAME_ANCHORS = [
  patternAnchor("pretender", "name"),
  patternAnchor("impostor", "name"),
  patternAnchor("imposter", "name"),
  patternAnchor("masquerade", "name"),
  patternAnchor("disguise", "name"),
];

const DISGUISED_PRETENDER_TEXT_ANCHORS = [
  patternAnchor("false identity"),
  patternAnchor("false identities"),
  patternAnchor("assume a false identity"),
  patternAnchor("assumes a false identity"),
  patternAnchor("impersonates"),
  patternAnchor("impersonate"),
  patternAnchor("infiltrates"),
  patternAnchor("infiltrate"),
  patternAnchor("wears another face"),
  patternAnchor("wear another face"),
  patternAnchor("take on the appearance of"),
  patternAnchor("takes on the appearance of"),
  patternAnchor("assume the form of"),
  patternAnchor("assumes the form of"),
  patternAnchor("replace the target"),
  patternAnchor("replaces the target"),
  patternAnchor("masquerades as"),
  patternAnchor("shapeshift"),
  patternAnchor("shapechange"),
  patternAnchor("disguised as"),
];

const DISGUISED_PRETENDER_CONTEXT_TEXT_NEAR = [
  {
    all: [
      patternAnchor("{{alt(disguise,disguised,disguises,masquerade,pretend,pretender,guise,false identity)}}"),
      patternAnchor("{{alt(identity,impersonate,impersonates,infiltrate,infiltrates,replace,form)}}"),
    ],
    window: 6,
    scope: "description" as const,
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
  patternAnchor("mimic"),
  patternAnchor("mimics"),
  patternAnchor("mimic chest"),
  patternAnchor("mimic chests"),
];

const BOUND_OBJECT_FIGURATIVE_BLOCKERS = [
  patternAnchor("animated discussion"),
  patternAnchor("animated debate"),
  patternAnchor("animated conversation"),
  patternAnchor("animated expression"),
  patternAnchor("animatedly"),
];

const BOUND_OBJECT_ANIMATION_TEXT_ANCHORS = [
  patternAnchor("animated"),
  patternAnchor("animate"),
  patternAnchor("animates"),
  patternAnchor("animation"),
  patternAnchor("brought to life"),
  patternAnchor("brought to unlife"),
  patternAnchor("comes to life"),
];

const BOUND_OBJECT_OBJECT_TEXT_ANCHORS = [
  patternAnchor("object"),
  patternAnchor("objects"),
  patternAnchor("armor"),
  patternAnchor("armors"),
  patternAnchor("doll"),
  patternAnchor("dolls"),
  patternAnchor("cart"),
  patternAnchor("carts"),
  patternAnchor("broom"),
  patternAnchor("brooms"),
  patternAnchor("cookware"),
  patternAnchor("door"),
  patternAnchor("doors"),
  patternAnchor("figurine"),
  patternAnchor("figurines"),
  patternAnchor("mannequin"),
  patternAnchor("mannequins"),
  patternAnchor("dummy"),
  patternAnchor("dummies"),
  patternAnchor("scarecrow"),
  patternAnchor("scarecrows"),
  patternAnchor("silverware"),
  patternAnchor("blade"),
  patternAnchor("blades"),
  patternAnchor("axe"),
  patternAnchor("axes"),
  patternAnchor("fireplace"),
  patternAnchor("fireplaces"),
  patternAnchor("vessel"),
  patternAnchor("vessels"),
  patternAnchor("cauldron"),
  patternAnchor("cauldrons"),
  patternAnchor("table"),
  patternAnchor("tables"),
  patternAnchor("chair"),
  patternAnchor("chairs"),
  patternAnchor("stool"),
  patternAnchor("stools"),
  patternAnchor("tool"),
  patternAnchor("tools"),
  patternAnchor("utensil"),
  patternAnchor("utensils"),
  patternAnchor("furniture"),
  patternAnchor("plaything"),
  patternAnchor("playthings"),
];

const BOUND_OBJECT_STATUE_TEXT_ANCHORS = [
  patternAnchor("statue"),
  patternAnchor("statues"),
  patternAnchor("effigy"),
  patternAnchor("effigies"),
  patternAnchor("idol"),
  patternAnchor("idols"),
  patternAnchor("monument"),
  patternAnchor("monuments"),
];

const BOUND_OBJECT_STATUE_CONTEXT_TEXT_ANCHORS = [
  patternAnchor("animated"),
  patternAnchor("animate"),
  patternAnchor("animates"),
  patternAnchor("animation"),
  patternAnchor("guardian"),
  patternAnchor("guardians"),
  patternAnchor("sentinel"),
  patternAnchor("sentinels"),
  patternAnchor("watch"),
  patternAnchor("watcher"),
  patternAnchor("watchers"),
  patternAnchor("protect"),
  patternAnchor("protects"),
  patternAnchor("comes to life"),
  patternAnchor("brought to life"),
  patternAnchor("stands guard"),
  patternAnchor("standing guard"),
];

const TRICKSTER_CHAOS_NAME_ANCHORS = [
  patternAnchor("trickster", "name"),
  patternAnchor("tricksters", "name"),
];

const TRICKSTER_CHAOS_TEXT_ANCHORS = [
  patternAnchor("trickster"),
  patternAnchor("tricksters"),
  patternAnchor("prank"),
  patternAnchor("pranks"),
  patternAnchor("prankster"),
  patternAnchor("pranksters"),
  patternAnchor("mischievous"),
  patternAnchor("whimsical"),
  patternAnchor("playful"),
  patternAnchor("delight in humor"),
  patternAnchor("harmless pranks"),
  patternAnchor("playing pranks"),
  patternAnchor("play tricks"),
];

const BONEYARD_SETTING_STEWARDSHIP_TEXT_ANCHORS = [
  patternAnchor("river of souls"),
  patternAnchor("call of the boneyard"),
  patternAnchor("safe journey of a soul"),
  patternAnchor("safe journey of souls"),
  patternAnchor("natural flow of life"),
  patternAnchor("guide the dead"),
  patternAnchor("guides the dead"),
  patternAnchor("guided the dead to the boneyard"),
  patternAnchor("final destination in the afterlife"),
  patternAnchor("souls reach their final destination"),
  patternAnchor("judgment in the boneyard"),
  patternAnchor("their judgment in the boneyard"),
  patternAnchor("follow individual souls"),
];

const ABADDON_SETTING_TEXT_ANCHORS = [
  patternAnchor("abaddon"),
  patternAnchor("abaddon s mists"),
  patternAnchor("plane of abaddon"),
];

const AXIS_SETTING_TEXT_ANCHORS = [
  patternAnchor("eternal city of axis"),
  patternAnchor("perfect city of axis"),
  patternAnchor("planar city of axis"),
  patternAnchor("city of axis"),
  patternAnchor("in axis"),
  patternAnchor("of axis"),
];

const PLANE_OF_FIRE_SETTING_TEXT_ANCHORS = [
  patternAnchor("native to the plane of fire"),
  patternAnchor("hail from the plane of fire"),
  patternAnchor("hailing from the plane of fire"),
  patternAnchor("summoned away from the plane of fire"),
];

const PLANE_OF_AIR_SETTING_TEXT_ANCHORS = [
  patternAnchor("native to the plane of air"),
  patternAnchor("hail from the plane of air"),
  patternAnchor("hailing from the plane of air"),
  patternAnchor("on the plane of air"),
];

const PLANE_OF_WATER_SETTING_TEXT_ANCHORS = [
  patternAnchor("native to the plane of water"),
  patternAnchor("hail from the plane of water"),
  patternAnchor("hailing from the plane of water"),
  patternAnchor("endless oceans of the plane of water"),
];

const PLANE_OF_EARTH_SETTING_TEXT_ANCHORS = [
  patternAnchor("native to the plane of earth"),
  patternAnchor("hail from the plane of earth"),
  patternAnchor("hailing from the plane of earth"),
  patternAnchor("coming from the plane of earth"),
];

const JUNGLE_SETTING_NAME_ANCHORS = [
  patternAnchor("jungle", "name"),
  patternAnchor("rainforest", "name"),
];

const JUNGLE_SETTING_TEXT_NEAR = [
  {
    all: [
      patternAltAnchor(["jungle", "jungles", "rainforest", "rainforests", "canopy", "canopies", "tropical forest", "tropical forests"], "description"),
      patternAltAnchor(["native", "native to", "found in", "found among", "live", "lives", "living", "dwell", "dwells", "dwelling", "hunt", "hunts", "hunting", "prowl", "prowls", "stalk", "stalks", "stalking", "haunt", "haunts", "roost", "roosts", "home", "homes"], "description"),
    ],
    window: 8,
    scope: "description" as const,
  },
];

const FOREST_SETTING_NAME_ANCHORS = [
  patternAnchor("forest", "name"),
  patternAnchor("jungle", "name"),
  patternAnchor("grove", "name"),
  patternAnchor("woodland", "name"),
];

const FOREST_SETTING_CORE_TEXT_ANCHORS = [
  patternAnchor("forest"),
  patternAnchor("forests"),
  patternAnchor("woodland"),
  patternAnchor("woodlands"),
  patternAnchor("woods"),
  patternAnchor("grove"),
  patternAnchor("groves"),
  patternAnchor("briar"),
  patternAnchor("jungle"),
  patternAnchor("jungles"),
  patternAnchor("rainforest"),
  patternAnchor("thicket"),
  patternAnchor("thickets"),
  patternAnchor("underbrush"),
];

const FOREST_SETTING_SUPPORT_TEXT_ANCHORS = [
  patternAnchor("canopy"),
  patternAnchor("canopies"),
  patternAnchor("treetop"),
  patternAnchor("treetops"),
  patternAnchor("arboreal"),
  patternAnchor("wooded area"),
  patternAnchor("wooded areas"),
  patternAnchor("forest dwelling"),
  patternAnchor("forest dwellers"),
  patternAnchor("forest floor"),
  patternAnchor("forest floors"),
  patternAnchor("within the woods"),
  patternAnchor("deep within the woods"),
  patternAnchor("old growth"),
  patternAnchor("bough"),
  patternAnchor("boughs"),
];

const SKY_SETTING_TEXT_NEAR = [
  {
    all: [
      patternAltAnchor(["sky", "skies", "open sky", "open skies", "storm cloud", "storm clouds", "cloud top", "cloud tops", "wind current", "wind currents", "high altitude", "high altitudes"], "description"),
      patternAltAnchor(["soar", "soars", "soaring", "glide", "glides", "gliding", "hover", "hovers", "hovering", "circle", "circles", "circling", "wheel", "wheels", "wheeling", "nest", "nests", "nesting", "roost", "roosts", "roosting"], "description"),
    ],
    window: 8,
    scope: "description" as const,
  },
];

const SKY_SETTING_NAME_ANCHORS = [
  patternAnchor("{{alt(sky,stormcrown)}} {{alt(dragon,archdragon)}}", "name"),
];

const DESERT_SETTING_NAME_ANCHORS = [
  patternAnchor("desert", "name"),
  patternAnchor("dune", "name"),
  patternAnchor("sand", "name"),
];

const DESERT_SETTING_STRONG_TEXT_ANCHORS = [
  patternAnchor("desert-dwelling"),
  patternAnchor("desert dwelling"),
  patternAnchor("desert {{alt(denizen,denizens,creature,creatures)}}"),
  patternAnchor("black desert"),
];

const DESERT_SETTING_EXACT_TEXT_ANCHORS = [
  patternAnchor("across the desert"),
  patternAnchor("desert sands"),
  patternAnchor("through the vast deserts"),
  patternAnchor("wander the deserts"),
  patternAnchor("deepest deserts"),
  patternAnchor("desert traders"),
  patternAnchor("desert's most frightening predators"),
];

const DESERT_SETTING_CORE_TEXT_ANCHORS = [
  patternAnchor("desert"),
  patternAnchor("deserts"),
  patternAnchor("dune"),
  patternAnchor("dunes"),
  patternAnchor("sand"),
  patternAnchor("arid"),
];

const DESERT_SETTING_SUPPORT_TEXT_ANCHORS = [
  patternAnchor("cliff"),
  patternAnchor("cliffs"),
  patternAnchor("mesa"),
  patternAnchor("mesas"),
  patternAnchor("oasis"),
  patternAnchor("oases"),
  patternAnchor("nest"),
  patternAnchor("nests"),
  patternAnchor("lair"),
  patternAnchor("lairs"),
  patternAnchor("burrow"),
  patternAnchor("burrows"),
  patternAnchor("burrowing"),
  patternAnchor("roam"),
  patternAnchor("roams"),
  patternAnchor("roaming"),
  patternAnchor("perch"),
  patternAnchor("perches"),
  patternAnchor("perched"),
  patternAnchor("home"),
  patternAnchor("homes"),
];

const RURAL_SETTING_NAME_ANCHORS = [
  patternAnchor("scarecrow", "name"),
  patternAnchor("villager", "name"),
  patternAnchor("villagers", "name"),
];

const RURAL_SETTING_TEXT_ANCHORS = [
  patternAnchor("hamlet"),
  patternAnchor("hamlets"),
  patternAnchor("farm"),
  patternAnchor("farms"),
  patternAnchor("farmstead"),
  patternAnchor("farmsteads"),
  patternAnchor("barn"),
  patternAnchor("barns"),
  patternAnchor("pasture"),
  patternAnchor("pastures"),
  patternAnchor("countryside"),
  patternAnchor("rural areas"),
  patternAnchor("outlying settlement"),
];

const RURAL_SETTING_CONTEXT_TEXT_ANCHOR = patternAltAnchor(["found in", "dwell", "dwells", "dwelling", "live", "lives", "living", "haunt", "haunts", "hunt", "hunts", "hunting", "prey on", "preys on", "patrol", "patrols", "prowl", "prowls", "protect", "protects", "sabotage", "sabotages", "siege", "sieges", "stalk", "stalks", "stalking", "terrorize", "terrorizes"], "description");

const RURAL_SETTING_COMMUNITY_CONTEXT_TEXT_ANCHOR = patternAltAnchor(["community", "communities", "home", "homes", "crop", "crops", "farming techniques", "tending crops", "tending their crops", "tending the crops", "tending their community"], "description");

const RURAL_SETTING_BLOCKER_TEXT_NEAR = [
  {
    all: [
      patternAltAnchor(["village", "villages", "farm", "farms", "hamlet", "hamlets", "countryside"]),
      patternAltAnchor(["raid", "raids", "raiding", "plunder", "waylay", "waylays", "wipe out", "lay waste"]),
    ],
    window: 6,
    scope: "description" as const,
  },
];

const URBAN_SETTING_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("market price"),
  patternAnchor("encountered in diverse cities and urban centers"),
];

const URBAN_SETTING_NAME_ANCHORS = [
  patternAnchor("sewer", "name"),
  patternAnchor("gutter", "name"),
  patternAnchor("gutters", "name"),
];

const FORTRESS_SETTING_NAME_ANCHORS = [
  patternAnchor("fortress", "name"),
  patternAnchor("citadel", "name"),
  patternAnchor("castle", "name"),
  patternAnchor("keep", "name"),
  patternAnchor("bastion", "name"),
  patternAnchor("watchtower", "name"),
];

const FORTRESS_SETTING_RESIDENCE_CONTEXT_TEXT_ANCHOR = patternAltAnchor(["within", "inside", "among", "take residence", "takes residence", "taking residence", "reside", "resides", "residing", "stationed", "posted", "quartered"], "description");
const FORTRESS_SETTING_STRUCTURE_TEXT_ANCHORS = [
  patternAltAnchor(["fortress", "fortresses", "castle", "castles", "citadel", "citadels", "stronghold", "strongholds", "bastion", "bastions", "watchtower", "watchtowers", "rampart", "ramparts", "battlement", "battlements"], "description"),
  patternAnchor("keep", "description", { pos: ["NOUN"] }),
  patternAnchor("keeps", "description", { pos: ["NOUN"] }),
];
const FORTRESS_SETTING_SITE_TEXT_ANCHORS = [
  patternAltAnchor(["fortress", "fortresses", "castle", "castles", "citadel", "citadels", "stronghold", "strongholds", "bastion", "bastions", "watchtower", "watchtowers", "fort", "forts", "garrison"], "description"),
  patternAnchor("keep", "description", { pos: ["NOUN"] }),
  patternAnchor("keeps", "description", { pos: ["NOUN"] }),
];

const DREAMLANDS_SETTING_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("find their way out of the dreamlands"),
  patternAnchor("out of the dreamlands and into the waking world"),
];

const DREAMLANDS_SETTING_EXTRA_NAME_ANCHORS = [
  patternAnchor("leng", "name"),
  patternAnchor("dreamscraper", "name"),
];

const DREAMLANDS_SETTING_EXTRA_TEXT_ANCHORS = [
  patternAnchor("dimension of dreams"),
  patternAnchor("dimension of leng"),
  patternAnchor("nightmare realm"),
];

const HELL_SETTING_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("hellknight"),
  patternAnchor("hellspawn"),
];

const HELL_SETTING_NAME_ANCHORS = [
  patternAnchor("diabolic", "name"),
  patternAnchor("hell hound", "name"),
  patternAnchor("hellcat", "name"),
  patternAnchor("hellwasp", "name"),
];

const ABYSS_SETTING_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("against the abyss"),
];

const SWAMP_SETTING_NAME_ANCHORS = [
  patternAnchor("bog", "name"),
  patternAnchor("fen", "name"),
  patternAnchor("mire", "name"),
];

const SWAMP_SETTING_TEXT_ANCHORS = [
  patternAnchor("peat bog"),
  patternAnchor("marshy realms"),
  patternAnchor("muddy swamp"),
  patternAnchor("bog or swamp"),
  patternAnchor("swamp or bog"),
  patternAnchor("bogs or swamps"),
  patternAnchor("swampy environment"),
  patternAnchor("swampland environment"),
  patternAnchor("tropical swamps"),
];

const NAUTICAL_SETTING_NAME_ANCHORS = [
  patternAnchor("dockhand", "name"),
  patternAnchor("thief of ships", "name"),
];

const NAUTICAL_SETTING_TEXT_ANCHORS = [
  patternAnchor("aboard ship"),
  patternAnchor("aboard a ship"),
  patternAnchor("aboard ships"),
  patternAnchor("black ships"),
  patternAnchor("load and unload cargo from ships"),
  patternAnchor("ship s boatswain"),
  patternAnchor("shipboard"),
  patternAnchor("shipboard labor"),
  patternAnchor("aboard a derelict ship"),
];

const URBAN_SETTING_SETTLEMENT_LIST_TEXT_ANCHORS = [
  patternAnchor("village"),
  patternAnchor("villages"),
  patternAnchor("city"),
  patternAnchor("cities"),
  patternAnchor("town"),
  patternAnchor("towns"),
];

const CREATURE_TERRAIN_HABITAT_CONTEXT_TEXT_ANCHOR = patternAltAnchor(["native", "native to", "found in", "found among", "found near", "inhabit", "inhabits", "inhabiting", "dwell", "dwells", "dwelling", "dweller", "dwellers", "live", "lives", "living", "lair", "lairs", "haunt", "haunts", "lurk", "lurks", "lurking", "roam", "roams", "roaming", "hunt", "hunts", "hunting", "prowl", "prowls", "stalk", "stalks", "stalking", "nest", "nests", "nesting", "roost", "roosts", "patrol", "patrols", "home", "homes", "watch over", "watches over", "keep watch over", "keeps watch over"], "description");

const CREATURE_SITE_HABITAT_CONTEXT_TEXT_ANCHOR = patternAltAnchor(["found in", "dwell", "dwells", "dwelling", "live", "lives", "living", "haunt", "haunts", "lurk", "lurks", "lurking", "patrol", "patrols", "guard", "guards", "guarding", "tend", "tends", "watch over", "watches over", "keep watch over", "keeps watch over", "home", "homes"], "description");

const CREATURE_CANYON_HABITAT_CONTEXT_TEXT_ANCHOR = patternAltAnchor(["found in", "dwell", "dwells", "dwelling", "haunt", "haunts", "lurk", "lurks", "lurking", "prowl", "prowls", "stalk", "stalks", "stalking", "glide through", "glides through", "hunt", "hunts", "hunting", "nest", "nests", "nesting", "home", "homes"], "description");

const CREATURE_SITE_FUNCTION_CONTEXT_TEXT_ANCHOR = patternAltAnchor(["found defending", "defend", "defends", "defending", "protect", "protects", "protector", "protectors", "guardians of", "serve as the protector", "serves as the protector", "rarely leave", "rarely leaves", "watch over", "watches over"], "description");

const CREATURE_URBAN_ACTIVITY_CONTEXT_TEXT_ANCHOR = patternAltAnchor(["make their home", "make their homes", "makes its home", "call home", "calls home", "sneak around", "sneaks around", "sneaking around", "stalk", "stalks", "stalking", "prominent in", "visible in", "below"], "description");

const CREATURE_UNDERGROUND_ACTIVITY_CONTEXT_TEXT_ANCHOR = patternAltAnchor(["below ground", "below large cities", "make their home", "make their homes", "makes its home", "call home", "calls home", "stalk", "stalks", "stalking", "scour", "scours", "most often found"], "description");

const FRESHWATER_SETTING_HABITAT_CONTEXT_TEXT_ANCHOR = patternAltAnchor(["native", "native to", "found in", "found among", "inhabit", "inhabits", "inhabiting", "dwell", "dwells", "dwelling", "live", "lives", "living", "call home", "calls home", "make their home", "make their homes", "makes its home", "home", "homes", "spend most of their time in", "centered on", "centered around", "hide", "hides", "lurk", "lurks", "lurking"], "description");

const createCreatureSettingTextNear = (
  alternatives: string[] | string,
  contextAnchor: ReturnType<typeof patternAnchor>,
  window = 8,
) => [
  {
    all: [
      patternAltAnchor(
        Array.isArray(alternatives)
          ? alternatives
          : alternatives.split(",").map((value) => value.trim()).filter((value) => value.length > 0),
        "description",
      ),
      contextAnchor,
    ],
    window,
    scope: "description" as const,
  },
];

const createCreatureSettingTextNearAnchors = (
  anchors: Array<ReturnType<typeof patternAnchor>>,
  contextAnchor: ReturnType<typeof patternAnchor>,
  window = 8,
) => anchors.map((anchor) => ({
    all: [
      anchor,
      contextAnchor,
    ],
    window,
    scope: "description" as const,
  }));

const FRESHWATER_SETTING_TEXT_NEAR = [
  ...createCreatureSettingTextNear(
    ["river", "rivers", "lake", "lakes", "pond", "ponds", "stream", "streams", "spring", "springs", "inland water", "inland waters", "inland pool", "inland pools"],
    FRESHWATER_SETTING_HABITAT_CONTEXT_TEXT_ANCHOR,
  ),
];

const DESERT_SETTING_TEXT_NEAR = [
  ...createCreatureSettingTextNear(
    ["desert", "deserts", "dune", "dunes", "arid", "sandy", "oasis", "oases", "black desert"],
    CREATURE_TERRAIN_HABITAT_CONTEXT_TEXT_ANCHOR,
  ),
  ...createCreatureSettingTextNear(
    ["desert", "deserts", "dune", "dunes", "oasis", "oases", "black desert"],
    CREATURE_SITE_FUNCTION_CONTEXT_TEXT_ANCHOR,
    10,
  ),
];

const FRESHWATER_SETTING_EXACT_TEXT_ANCHORS = [
  patternAnchor("rivers and lakes"),
  patternAnchor("lakes and rivers"),
  patternAnchor("rivers and streams"),
  patternAnchor("streams and rivers"),
  patternAnchor("ponds and lakes"),
  patternAnchor("lakes and ponds"),
  patternAnchor("freshwater inlet"),
  patternAnchor("freshwater inlets"),
  patternAnchor("river tributaries"),
  patternAnchor("tributaries meet"),
  patternAnchor("cool inland waters"),
  patternAnchor("remote lakes"),
  patternAnchor("still inland pools"),
  patternAnchor("geothermal spring"),
];

const FOREST_SETTING_TEXT_NEAR = [
  ...createCreatureSettingTextNear(
    ["forest", "forests", "woodland", "woodlands", "woods", "grove", "groves", "wooded area", "wooded areas", "forest floor", "forest floors", "primeval forest", "primeval forests"],
    CREATURE_TERRAIN_HABITAT_CONTEXT_TEXT_ANCHOR,
  ),
  ...createCreatureSettingTextNear(
    ["forest", "forests", "woodland", "woodlands", "woods", "grove", "groves", "wooded area", "wooded areas", "forest floor", "forest floors", "primeval forest", "primeval forests"],
    CREATURE_SITE_FUNCTION_CONTEXT_TEXT_ANCHOR,
  ),
];

const FOREST_SETTING_EXACT_TEXT_ANCHORS = [
  patternAnchor("forest creature"),
  patternAnchor("forest creatures"),
  patternAnchor("forest denizen"),
  patternAnchor("forest denizens"),
  patternAnchor("deep into the forest"),
  patternAnchor("in a nearby woodland"),
  patternAnchor("the very woods in which they reside"),
  patternAnchor("deep within the woods"),
  patternAnchor("within prehistoric forests"),
];

const CANYON_OR_CAVERNS_TEXT_ANCHORS = [
  patternAnchor("canyons or caverns", "description"),
];

const UNDERGROUND_SETTING_EXACT_TEXT_ANCHORS = [
  patternAnchor("subterranean monster"),
  patternAnchor("subterranean monsters"),
  patternAnchor("subterranean realm"),
  patternAnchor("subterranean realms"),
  patternAnchor("lightless land"),
  patternAnchor("lightless lands"),
  patternAnchor("lightless lair"),
  patternAnchor("lightless lairs"),
  patternAnchor("deep underground"),
  patternAnchor("adventuresome spelunkers"),
  patternAnchor("important subterranean locations"),
  patternAnchor("underground explorers"),
  patternAnchor("subterranean depths"),
];

const GRAVEYARD_SETTING_CORE_TEXT_ANCHORS = [
  patternAnchor("graveyard"),
  patternAnchor("graveyards"),
  patternAnchor("cemetery"),
  patternAnchor("cemeteries"),
  patternAnchor("mausoleum"),
  patternAnchor("mausoleums"),
  patternAnchor("barrow"),
  patternAnchor("barrows"),
  patternAnchor("sepulcher"),
  patternAnchor("sepulchers"),
  patternAnchor("cairn"),
  patternAnchor("cairns"),
  patternAnchor("burial ground"),
  patternAnchor("burial grounds"),
  patternAnchor("crypt"),
  patternAnchor("crypts"),
  patternAnchor("tomb"),
  patternAnchor("tombs"),
  patternAnchor("catacomb"),
  patternAnchor("catacombs"),
  patternAnchor("burial chamber"),
  patternAnchor("burial chambers"),
];

const URBAN_SETTING_EXACT_TEXT_ANCHORS = [
  patternAnchor("upper echelons of society"),
  patternAnchor("city servant"),
  patternAnchor("city servants"),
  patternAnchor("city dwelling"),
  patternAnchor("city-dwelling"),
  patternAnchor("city dwelling gargoyles"),
  patternAnchor("city-dwelling gargoyles"),
  patternAnchor("city of smog"),
  patternAnchor("hide in plain sight in cities"),
  patternAnchor("urban pollution"),
  patternAnchor("lords of sewers"),
  patternAnchor("storm drain"),
  patternAnchor("storm drains"),
  patternAnchor("storm sewer"),
  patternAnchor("storm sewers"),
  patternAnchor("drainage tunnel"),
  patternAnchor("drainage tunnels"),
];

const URBAN_SETTING_GOVERNANCE_TEXT_ANCHORS = [
  patternAnchor("town watch"),
  patternAnchor("city guard"),
  patternAnchor("maintain order and enforce laws"),
  patternAnchor("major urban centers"),
  patternAnchor("political leader of a settlement"),
  patternAnchor("leader of a nation s bureaucracy"),
  patternAnchor("guard important magistrates"),
  patternAnchor("enforce the law"),
  patternAnchor("official spymasters"),
  patternAnchor("loyal city servant"),
  patternAnchor("in rough akitonian cities like seldo"),
];

const SHADOW_PLANE_SETTING_NAME_ANCHORS = [
  patternAnchor("umbral dragon", "name"),
  patternAnchor("umbral archdragon", "name"),
  patternAnchor("d ziriak", "name"),
];

const VOLCANIC_SETTING_EXACT_TEXT_ANCHORS = [
  patternAnchor("volcanoes being a favorite spot"),
  patternAnchor("die to form volcanoes"),
  patternAnchor("dwell near active volcanos"),
  patternAnchor("volcanic glass"),
  patternAnchor("roiling lava"),
  patternAnchor("volcanic homes"),
];

const VOLCANIC_SETTING_TEXT_NEAR = [
  ...createCreatureSettingTextNear(
    "volcanic,volcano,volcanoes,lava,magma,caldera,ash,ashen,cinder,cinders",
    CREATURE_TERRAIN_HABITAT_CONTEXT_TEXT_ANCHOR,
    7,
  ),
];

const WASTELAND_SETTING_TEXT_NEAR = [
  ...createCreatureSettingTextNear(
    "wasteland,wastelands,wastes,barren,blasted,desolate",
    CREATURE_TERRAIN_HABITAT_CONTEXT_TEXT_ANCHOR,
  ),
];

const MOUNTAIN_SETTING_NAME_ANCHORS = [
  patternAnchor("venexus s wyrmling", "name"),
  patternAnchor("venexus s chosen", "name"),
];

const MOUNTAIN_SETTING_TEXT_ANCHORS = [
  patternAnchor("glacial mountaintops", "description"),
  patternAnchor("mountain peaks", "description"),
];

const CIVIC_SUPPORT_TEXT_ANCHORS = [
  patternAnchor("maintain order"),
  patternAnchor("enforce laws"),
  patternAnchor("patrol their assigned areas"),
  patternAnchor("patrols the estate grounds"),
  patternAnchor("patrols the streets"),
  patternAnchor("keeps the peace"),
  patternAnchor("serves the community"),
  patternAnchor("runs the shrine"),
  patternAnchor("tends the shrine"),
  patternAnchor("protect the faithful"),
  patternAnchor("responsible for the livelihood"),
  patternAnchor("shares divine dreams and advice"),
  patternAnchor("safeguard their temples and communities"),
  patternAnchor("surrounding community is taken care of"),
  patternAnchor("for the community"),
  patternAnchor("public works"),
  patternAnchor("civic festivals"),
  patternAnchor("public ceremonies"),
];

const CIVIC_NPC_NAME_ANCHORS = [
  patternAnchor("abbot", "name"),
  patternAnchor("advisor", "name"),
  patternAnchor("apothecary", "name"),
  patternAnchor("barkeep", "name"),
  patternAnchor("captain", "name"),
  patternAnchor("caretaker", "name"),
  patternAnchor("chair", "name"),
  patternAnchor("commoner", "name"),
  patternAnchor("constable", "name"),
  patternAnchor("courtier", "name"),
  patternAnchor("diplomat", "name"),
  patternAnchor("envoy", "name"),
  patternAnchor("gaoler", "name"),
  patternAnchor("guard", "name"),
  patternAnchor("historian", "name"),
  patternAnchor("innkeeper", "name"),
  patternAnchor("jailer", "name"),
  patternAnchor("librarian", "name"),
  patternAnchor("merchant", "name"),
  patternAnchor("noble", "name"),
  patternAnchor("physician", "name"),
  patternAnchor("professor", "name"),
  patternAnchor("prophet", "name"),
  patternAnchor("scribe", "name"),
  patternAnchor("teacher", "name"),
  patternAnchor("translator", "name"),
  patternAnchor("watch officer", "name"),
];

const CIVIC_NPC_ROLE_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("assassin", "name"),
  patternAnchor("bandit", "name"),
  patternAnchor("bladecaller", "name"),
  patternAnchor("commando", "name"),
  patternAnchor("cultist", "name"),
  patternAnchor("cutthroat", "name"),
  patternAnchor("enforcer", "name"),
  patternAnchor("hellknight", "name"),
  patternAnchor("marauder", "name"),
  patternAnchor("mercenary", "name"),
  patternAnchor("murderer", "name"),
  patternAnchor("poisoner", "name"),
  patternAnchor("raider", "name"),
  patternAnchor("reaver", "name"),
  patternAnchor("slayer", "name"),
  patternAnchor("sniper", "name"),
  patternAnchor("warlord", "name"),
];

const COMBATANT_NPC_STRONG_NAME_ANCHORS = [
  patternAnchor("soldier", "name"),
  patternAnchor("bandit", "name"),
  patternAnchor("ruffian", "name"),
  patternAnchor("mercenary", "name"),
  patternAnchor("assassin", "name"),
  patternAnchor("commando", "name"),
  patternAnchor("cutthroat", "name"),
  patternAnchor("hellknight", "name"),
  patternAnchor("lieutenant", "name"),
  patternAnchor("reaver", "name"),
  patternAnchor("sergeant", "name"),
  patternAnchor("sniper", "name"),
  patternAnchor("cultist", "name"),
  patternAnchor("warpriest", "name"),
  patternAnchor("priestess", "name"),
  patternAnchor("poisoner", "name"),
  patternAnchor("sentry", "name"),
];

const COMBATANT_NPC_WEAK_NAME_ANCHORS = [
  patternAnchor("adjutant", "name"),
  patternAnchor("armiger", "name"),
  patternAnchor("brigand", "name"),
  patternAnchor("archer", "name"),
  patternAnchor("bastion", "name"),
  patternAnchor("bladecaller", "name"),
  patternAnchor("duelist", "name"),
  patternAnchor("gladiator", "name"),
  patternAnchor("defender", "name"),
  patternAnchor("enforcer", "name"),
  patternAnchor("constable", "name"),
  patternAnchor("gaoler", "name"),
  patternAnchor("gendarme", "name"),
  patternAnchor("general", "name"),
  patternAnchor("guard", "name"),
  patternAnchor("infantry", "name"),
  patternAnchor("jailer", "name"),
  patternAnchor("marauder", "name"),
  patternAnchor("murderer", "name"),
  patternAnchor("raider", "name"),
  patternAnchor("slayer", "name"),
  patternAnchor("warrior", "name"),
  patternAnchor("disciple", "name"),
  patternAnchor("boss", "name"),
  patternAnchor("infiltrator", "name"),
  patternAnchor("bruiser", "name"),
  patternAnchor("sentinel", "name"),
  patternAnchor("skirmisher", "name"),
  patternAnchor("vanguard", "name"),
  patternAnchor("weapon master", "name"),
  patternAnchor("champion", "name"),
  patternAnchor("warlord", "name"),
  patternAnchor("warden", "name"),
];

const COMBATANT_NPC_CONTEXT_NAME_ANCHORS = [
  patternAnchor("veteran {{gap(2)}} {{alt(armiger,bastion,bladecaller,guard,hellknight,officer,sergeant,soldier,warlord)}}", "name"),
  patternAnchor("{{alt(armiger,bastion,bladecaller,guard,hellknight,officer,sergeant,soldier,warlord)}} {{gap(2)}} veteran", "name"),
];

const ANIMATED_OBJECT_NAME_ANCHORS = [
  patternAnchor("animated tea cart", "name"),
  patternAnchor("animated treasure swarm", "name"),
  patternAnchor("animated doll", "name"),
  patternAnchor("doorwarden", "name"),
  patternAnchor("animated bamboo figurine", "name"),
  patternAnchor("mannequin", "name"),
  patternAnchor("scarecrow", "name"),
];

const ANIMATED_STATUE_NAME_ANCHORS = [
  patternAnchor("statue", "name"),
  patternAnchor("bulwark", "name"),
  patternAnchor("divine warden", "name"),
  patternAnchor("effigy", "name"),
  patternAnchor("idol", "name"),
  patternAnchor("animated colossus", "name"),
];

const POSSESSION_THREAT_TEXT_ANCHORS = [
  patternAnchor("possess"),
  patternAnchor("possesses"),
  patternAnchor("possessed"),
  patternAnchor("possession"),
  patternAnchor("take control of the victim s body"),
  patternAnchor("control the victim from within"),
  patternAnchor("inhabit the body of"),
  patternAnchor("rides within a living host"),
  patternAnchor("body snatcher"),
];

const LIFE_DRAIN_THREAT_TEXT_ANCHORS = [
  patternAnchor("drain blood"),
  patternAnchor("drains blood"),
  patternAnchor("drink blood"),
  patternAnchor("drinks blood"),
  patternAnchor("blood drain"),
  patternAnchor("blood-drain"),
  patternAnchor("drain life"),
  patternAnchor("drains life"),
  patternAnchor("drain vitality"),
  patternAnchor("drains vitality"),
  patternAnchor("drain life force"),
  patternAnchor("drains life force"),
  patternAnchor("life force"),
  patternAnchor("life draining"),
  patternAnchor("life-draining"),
  patternAnchor("drain essence"),
  patternAnchor("drains essence"),
  patternAnchor("siphon life"),
  patternAnchor("siphons life"),
  patternAnchor("steal souls"),
  patternAnchor("steals souls"),
  patternAnchor("siphon souls"),
  patternAnchor("siphons souls"),
  patternAnchor("feed on blood"),
  patternAnchor("feeds on blood"),
  patternAnchor("vampiric"),
];

const SPAWN_CREATOR_TEXT_ANCHORS = [
  patternAnchor("create spawn"),
  patternAnchor("creates spawn"),
  patternAnchor("its spawn"),
  patternAnchor("brood"),
  patternAnchor("brood mother"),
  patternAnchor("broods"),
  patternAnchor("implant eggs"),
  patternAnchor("implants eggs"),
  patternAnchor("lay eggs in"),
  patternAnchor("lays eggs in"),
  patternAnchor("bursts from the host"),
  patternAnchor("offspring"),
  patternAnchor("infest"),
  patternAnchor("infests"),
  patternAnchor("turn victims into"),
  patternAnchor("raise the slain as"),
  patternAnchor("raises the slain as"),
];

const PETRIFICATION_THREAT_TEXT_ANCHORS = [
  patternAnchor("petrify"),
  patternAnchor("petrifies"),
  patternAnchor("petrified"),
  patternAnchor("turn to stone"),
  patternAnchor("turned to stone"),
  patternAnchor("turns creatures to stone"),
  patternAnchor("stone curse"),
];

const REGENERATION_THREAT_TEXT_ANCHORS = [
  patternAnchor("regeneration"),
  patternAnchor("regenerate"),
  patternAnchor("regenerates"),
  patternAnchor("fast healing"),
  patternAnchor("heals each round"),
  patternAnchor("heals itself"),
  patternAnchor("restores hit points each round"),
  patternAnchor("regains hit points each round"),
  patternAnchor("suppresses its regeneration"),
  patternAnchor("can t be killed unless"),
  patternAnchor("can only be destroyed if"),
  patternAnchor("must be destroyed by"),
  patternAnchor("acid or fire"),
];

const AMBUSH_GRABBER_TEXT_ANCHORS = [
  patternAnchor("grabbed"),
  patternAnchor("grabs prey"),
  patternAnchor("grapple"),
  patternAnchor("grapples"),
  patternAnchor("constrict"),
  patternAnchor("constricts"),
  patternAnchor("swallow whole"),
  patternAnchor("swallows prey"),
  patternAnchor("snare prey"),
  patternAnchor("snaring"),
  patternAnchor("snared"),
  patternAnchor("snare"),
  patternAnchor("entangle"),
  patternAnchor("entangles"),
  patternAnchor("entangled"),
  patternAnchor("entangles victims"),
  patternAnchor("ensnare"),
  patternAnchor("ensnares"),
  patternAnchor("webs"),
  patternAnchor("webbed"),
  patternAnchor("sticky webs"),
  patternAnchor("drag prey"),
  patternAnchor("drags prey"),
  patternAnchor("dragged prey"),
  patternAnchor("drag foes"),
  patternAnchor("drags foes"),
  patternAnchor("drag victims"),
  patternAnchor("drags victims"),
  patternAnchor("snatch a creature"),
  patternAnchor("snatches a creature"),
  patternAnchor("snatches prey"),
  patternAnchor("carry off prey"),
  patternAnchor("carries off prey"),
  patternAnchor("pull victims into"),
  patternAnchor("wraps around"),
  patternAnchor("pulls prey"),
  patternAnchor("pulls prey into"),
  patternAnchor("pull prey"),
  patternAnchor("engulf"),
  patternAnchor("engulfs"),
  patternAnchor("ambush predator"),
];

export const CREATURE_DERIVED_TAG_RULES: DerivedTagRule[] = [
  {
    tag: "undead_adjacent",
    category: "creature",
    anyOf: [
      { traitsAny: ["undead", "ghost", "skeleton", "ghoul"] },
      { familiesAny: UNDEAD_GLOSSARY_FAMILIES },
    ],
  },
  {
    tag: "possession_threat",
    category: "creature",
    anyOf: [
      { textAny: POSSESSION_THREAT_TEXT_ANCHORS },
    ],
  },
  {
    tag: "life_drain_threat",
    category: "creature",
    anyOf: [
      { textAny: LIFE_DRAIN_THREAT_TEXT_ANCHORS },
    ],
  },
  {
    tag: "spawn_creator",
    category: "creature",
    anyOf: [
      { textAny: SPAWN_CREATOR_TEXT_ANCHORS },
    ],
  },
  {
    tag: "petrification_threat",
    category: "creature",
    anyOf: [
      { textAny: PETRIFICATION_THREAT_TEXT_ANCHORS },
    ],
  },
  {
    tag: "regeneration_threat",
    category: "creature",
    anyOf: [
      { textAny: REGENERATION_THREAT_TEXT_ANCHORS },
    ],
  },
  {
    tag: "ambush_grabber",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          patternAnchor("constrict"),
          patternAnchor("constricts"),
          patternAnchor("swallow whole"),
          patternAnchor("sticky webs"),
          patternAnchor("snare prey"),
          patternAnchor("drag prey"),
          patternAnchor("drags prey"),
          patternAnchor("drag victims"),
          patternAnchor("drags victims"),
          patternAnchor("snatches prey"),
          patternAnchor("carry off prey"),
          patternAnchor("carries off prey"),
          patternAnchor("pulls prey into"),
          patternAnchor("pull victims into"),
        ],
      },
      { score: 1, textAny: [patternAnchor("grabbed"), patternAnchor("grabs prey"), patternAnchor("webbed"), patternAnchor("ambush predator")] },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(ambush,lying in wait,lurks,lurking)}}"),
              patternAnchor("{{alt(grab,grabs,grapple,grapples,constrict,constricts,swallow,snatch,snatches,drag,drags,carry off,carries off)}}"),
            ],
            window: 7,
            scope: "description",
          },
        ],
      },
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
      { score: 2, textAny: FRESHWATER_SETTING_EXACT_TEXT_ANCHORS },
      { score: 2, textNear: FRESHWATER_SETTING_TEXT_NEAR },
      { score: 1, textAny: FRESHWATER_SETTING_TEXT_ANCHORS },
      { score: 1, textAny: FRESHWATER_SETTING_NAME_ANCHORS },
      { score: 1, traitsAny: ["amphibious"] },
      { score: 1, traitsAny: ["water"] },
    ],
  },
  {
    tag: "aquatic_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["azarketi"] },
    ],
  },
  {
    tag: "aquatic_setting",
    category: "creature",
    threshold: 2,
    noneOf: [
      { textAny: [patternAnchor("inner sea")] },
    ],
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
      { textAny: [patternAnchor("astral")] },
      { textAny: ASTRAL_SETTING_CONTEXT_TEXT_ANCHORS },
    ],
  },
  {
    tag: "ethereal_setting",
    category: "creature",
    anyOf: [
      { textAny: ETHEREAL_SETTING_TEXT_ANCHORS },
    ],
  },
  {
    tag: "plane_of_fire_setting",
    category: "creature",
    anyOf: [
      { traitsAll: ["elemental", "fire"] },
      { traitsAll: ["genie", "fire"] },
    ],
  },
  {
    tag: "plane_of_fire_setting",
    category: "creature",
    anyOf: [
      { textAny: PLANE_OF_FIRE_SETTING_TEXT_ANCHORS },
    ],
  },
  {
    tag: "plane_of_air_setting",
    category: "creature",
    anyOf: [
      { traitsAll: ["elemental", "air"] },
      { traitsAll: ["genie", "air"] },
    ],
  },
  {
    tag: "plane_of_air_setting",
    category: "creature",
    anyOf: [
      { textAny: PLANE_OF_AIR_SETTING_TEXT_ANCHORS },
    ],
  },
  {
    tag: "plane_of_water_setting",
    category: "creature",
    anyOf: [
      { traitsAll: ["elemental", "water"] },
      { traitsAll: ["genie", "water"] },
    ],
  },
  {
    tag: "plane_of_water_setting",
    category: "creature",
    anyOf: [
      { textAny: PLANE_OF_WATER_SETTING_TEXT_ANCHORS },
    ],
  },
  {
    tag: "plane_of_earth_setting",
    category: "creature",
    anyOf: [
      { traitsAll: ["elemental", "earth"] },
      { traitsAll: ["genie", "earth"] },
    ],
  },
  {
    tag: "plane_of_earth_setting",
    category: "creature",
    anyOf: [
      { textAny: PLANE_OF_EARTH_SETTING_TEXT_ANCHORS },
    ],
  },
  {
    tag: "elemental_plane_setting",
    category: "creature",
    requiresTags: ["plane_of_fire_setting"],
  },
  {
    tag: "elemental_plane_setting",
    category: "creature",
    requiresTags: ["plane_of_air_setting"],
  },
  {
    tag: "elemental_plane_setting",
    category: "creature",
    requiresTags: ["plane_of_water_setting"],
  },
  {
    tag: "elemental_plane_setting",
    category: "creature",
    requiresTags: ["plane_of_earth_setting"],
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
    tag: "dreamlands_setting",
    category: "creature",
    noneOf: [
      { textAny: DREAMLANDS_SETTING_BLOCKER_TEXT_ANCHORS },
    ],
    anyOf: [
      { traitsAny: ["dream"] },
    ],
  },
  {
    tag: "dreamlands_setting",
    category: "creature",
    noneOf: [
      { textAny: DREAMLANDS_SETTING_BLOCKER_TEXT_ANCHORS },
    ],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: DREAMLANDS_SETTING_NAME_ANCHORS },
      { score: 2, textAny: DREAMLANDS_SETTING_TEXT_ANCHORS },
      { score: 2, textAny: DREAMLANDS_SETTING_EXTRA_NAME_ANCHORS },
      { score: 2, textAny: DREAMLANDS_SETTING_EXTRA_TEXT_ANCHORS },
    ],
  },
  {
    tag: "boneyard_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["psychopomp"] },
    ],
  },
  {
    tag: "boneyard_setting",
    category: "creature",
    anyOf: [
      { textAny: [patternAnchor("shade (boneyard)", "name")] },
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
      { score: 1, textAny: BONEYARD_SETTING_STEWARDSHIP_TEXT_ANCHORS },
    ],
  },
  {
    tag: "heaven_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["archon"] },
    ],
  },
  {
    tag: "heaven_setting",
    category: "creature",
    anyOf: [
      { textAny: [patternAnchor("shade (heaven)", "name")] },
    ],
  },
  {
    tag: "heaven_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: HEAVEN_SETTING_TEXT_ANCHORS },
      { score: 1, traitsAny: ["archon"] },
    ],
  },
  {
    tag: "nirvana_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["agathion"] },
    ],
  },
  {
    tag: "nirvana_setting",
    category: "creature",
    anyOf: [
      { textAny: [patternAnchor("shade (nirvana)", "name")] },
    ],
  },
  {
    tag: "nirvana_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: NIRVANA_SETTING_TEXT_ANCHORS },
      { score: 1, traitsAny: ["agathion"] },
    ],
  },
  {
    tag: "elysium_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["azata"] },
    ],
  },
  {
    tag: "elysium_setting",
    category: "creature",
    anyOf: [
      { textAny: [patternAnchor("shade (elysium)", "name")] },
    ],
  },
  {
    tag: "elysium_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: ELYSIUM_SETTING_TEXT_ANCHORS },
      { score: 1, traitsAny: ["azata"] },
    ],
  },
  {
    tag: "upper_plane_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["angel"] },
    ],
  },
  {
    tag: "upper_plane_setting",
    category: "creature",
    requiresTags: ["heaven_setting"],
  },
  {
    tag: "upper_plane_setting",
    category: "creature",
    requiresTags: ["nirvana_setting"],
  },
  {
    tag: "upper_plane_setting",
    category: "creature",
    requiresTags: ["elysium_setting"],
  },
  {
    tag: "hell_setting",
    category: "creature",
    threshold: 3,
    noneOf: [
      { textAny: HELL_SETTING_BLOCKER_TEXT_ANCHORS },
    ],
    anyOf: [
      { score: 3, textAny: HELL_SETTING_NAME_ANCHORS },
      { score: 2, textAny: HELL_SETTING_TEXT_ANCHORS },
      { score: 1, traitsAny: ["devil", "fiend"] },
    ],
  },
  {
    tag: "hell_setting",
    category: "creature",
    anyOf: [
      { textAny: [patternAnchor("shade (hell)", "name")] },
    ],
  },
  {
    tag: "abyss_setting",
    category: "creature",
    threshold: 3,
    noneOf: [
      { textAny: ABYSS_SETTING_BLOCKER_TEXT_ANCHORS },
    ],
    anyOf: [
      { score: 2, textAny: ABYSS_SETTING_TEXT_ANCHORS },
      { score: 1, traitsAny: ["demon", "fiend", "qlippoth"] },
    ],
  },
  {
    tag: "abyss_setting",
    category: "creature",
    anyOf: [
      { textAny: [patternAnchor("shade (outer rifts)", "name")] },
    ],
  },
  {
    tag: "abaddon_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["daemon"] },
    ],
  },
  {
    tag: "abaddon_setting",
    category: "creature",
    threshold: 3,
    anyOf: [
      { score: 2, textAny: ABADDON_SETTING_TEXT_ANCHORS },
      { score: 1, traitsAny: ["daemon", "fiend"] },
    ],
  },
  {
    tag: "lower_plane_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["devil", "demon", "daemon", "qlippoth"] },
    ],
  },
  {
    tag: "lower_plane_setting",
    category: "creature",
    requiresTags: ["hell_setting"],
  },
  {
    tag: "lower_plane_setting",
    category: "creature",
    requiresTags: ["abyss_setting"],
  },
  {
    tag: "lower_plane_setting",
    category: "creature",
    requiresTags: ["abaddon_setting"],
  },
  {
    tag: "cosmic_framework_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["monitor"] },
    ],
  },
  {
    tag: "cosmic_framework_setting",
    category: "creature",
    requiresTags: ["axis_setting"],
  },
  {
    tag: "cosmic_framework_setting",
    category: "creature",
    requiresTags: ["boneyard_setting"],
  },
  {
    tag: "cosmic_framework_setting",
    category: "creature",
    requiresTags: ["maelstrom_setting"],
  },
  {
    tag: "axis_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["aeon", "inevitable"] },
    ],
  },
  {
    tag: "axis_setting",
    category: "creature",
    anyOf: [
      { textAny: [patternAnchor("shade (axis)", "name")] },
    ],
  },
  {
    tag: "axis_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AXIS_SETTING_TEXT_ANCHORS },
    ],
  },
  {
    tag: "shadow_plane_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["caligni", "fetchling"] },
    ],
  },
  {
    tag: "shadow_plane_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: SHADOW_PLANE_SETTING_NAME_ANCHORS },
      { score: 2, textAny: SHADOW_PLANE_SETTING_TEXT_ANCHORS },
      { score: 1, traitsAny: ["shadow", "velstrac"] },
    ],
  },
  {
    tag: "maelstrom_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["protean"] },
    ],
  },
  {
    tag: "maelstrom_setting",
    category: "creature",
    anyOf: [
      { textAny: [patternAnchor("shade (maelstrom)", "name")] },
    ],
  },
  {
    tag: "maelstrom_setting",
    category: "creature",
    threshold: 3,
    anyOf: [
      { score: 2, textAny: MAELSTROM_SETTING_TEXT_ANCHORS },
      { score: 1, traitsAny: ["monitor", "protean"] },
    ],
  },
  {
    tag: "island_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textNear: createCreatureSettingTextNear(
          "island,islands,archipelago,archipelagos,atoll,atolls,isle,isles",
          CREATURE_TERRAIN_HABITAT_CONTEXT_TEXT_ANCHOR,
        ),
      },
      {
        score: 2,
        textNear: createCreatureSettingTextNear(
          "island,islands,archipelago,archipelagos,atoll,atolls,isle,isles,eastern isles,remote islands",
          CREATURE_SITE_FUNCTION_CONTEXT_TEXT_ANCHOR,
          10,
        ),
      },
    ],
  },
  {
    tag: "nautical_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: NAUTICAL_SETTING_NAME_ANCHORS },
      { score: 2, textAny: NAUTICAL_SETTING_TEXT_ANCHORS },
      {
        score: 2,
        textAny: [
          patternAnchor("sailor"),
          patternAnchor("sailors"),
          patternAnchor("mariner"),
          patternAnchor("mariners"),
          patternAnchor("pirate"),
          patternAnchor("pirates"),
          patternAnchor("corsair"),
          patternAnchor("corsairs"),
          patternAnchor("dock"),
          patternAnchor("docks"),
          patternAnchor("bilge"),
          patternAnchor("wreck"),
          patternAnchor("crew"),
          patternAnchor("crews"),
          patternAnchor("rigging"),
          patternAnchor("mast"),
          patternAnchor("masts"),
          patternAnchor("shipwreck"),
          patternAnchor("shipwrecks"),
          patternAnchor("shipwreck"),
        ],
      },
      { score: 1, textAny: [patternAnchor("ship"), patternAnchor("ships"), patternAnchor("captain"), patternAnchor("vessel"), patternAnchor("vessels"), patternAnchor("deck"), patternAnchor("decks")] },
      { score: 1, textAny: [patternAnchor("harbor"), patternAnchor("harbors")] },
    ],
  },
  {
    tag: "jungle_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: JUNGLE_SETTING_NAME_ANCHORS },
      { score: 2, textNear: JUNGLE_SETTING_TEXT_NEAR },
      { score: 2, textAny: [patternAnchor("jungles and tropical forests"), patternAnchor("thick jungle foliage")] },
    ],
  },
  {
    tag: "forest_setting",
    category: "creature",
    requiresTags: ["jungle_setting"],
  },
  {
    tag: "forest_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: FOREST_SETTING_EXACT_TEXT_ANCHORS },
      { score: 2, textNear: FOREST_SETTING_TEXT_NEAR },
      { score: 1, textAny: FOREST_SETTING_NAME_ANCHORS },
      { score: 1, textAny: FOREST_SETTING_CORE_TEXT_ANCHORS },
      { score: 1, textAny: FOREST_SETTING_SUPPORT_TEXT_ANCHORS },
      { score: 2, textAny: [patternAnchor("jungles and tropical forests")] },
    ],
  },
  {
    tag: "plains_setting",
    category: "creature",
    anyOf: [
      {
        textAny: [
          patternAnchor("plains"),
          patternAnchor("grassland"),
          patternAnchor("grasslands"),
          patternAnchor("prairie"),
          patternAnchor("prairies"),
          patternAnchor("savanna"),
          patternAnchor("savannah"),
          patternAnchor("steppe"),
          patternAnchor("steppes"),
        ],
      },
    ],
  },
  {
    tag: "canyon_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: CANYON_OR_CAVERNS_TEXT_ANCHORS },
      {
        score: 2,
        textNear: createCreatureSettingTextNear(
          "canyon,canyons,gorge,gorges,mesa,mesas,badlands",
          CREATURE_CANYON_HABITAT_CONTEXT_TEXT_ANCHOR,
        ),
      },
    ],
  },
  {
    tag: "swamp_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, traitsAny: ["boggard"] },
      { score: 1, textAny: SWAMP_SETTING_NAME_ANCHORS },
      { score: 2, textAny: SWAMP_SETTING_TEXT_ANCHORS },
      {
        score: 2,
        textNear: createCreatureSettingTextNear(
          "swamp,swamps,bog,bogs,marsh,marshes,fen,fens,mire,mires,wetland,wetlands,bayou,bayous,mangrove,mangroves,quagmire,quagmires,slough,sloughs",
          CREATURE_TERRAIN_HABITAT_CONTEXT_TEXT_ANCHOR,
        ),
      },
    ],
  },
  {
    tag: "underground_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["drow", "xulgath", "morlock", "dero", "caligni", "duergar", "urdefhan", "munavri", "serpentfolk"] },
    ],
  },
  {
    tag: "underground_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: CANYON_OR_CAVERNS_TEXT_ANCHORS },
      { score: 2, textAny: UNDERGROUND_SETTING_EXACT_TEXT_ANCHORS },
      {
        score: 2,
        textNear: createCreatureSettingTextNear(
          "cave,caves,cavern,caverns,underground,tunnel,tunnels,subterranean,underworld,depths,mine,mines,mineshaft,mineshafts,quarry,quarries,warren,warrens,cave network,cave networks",
          CREATURE_SITE_HABITAT_CONTEXT_TEXT_ANCHOR,
        ),
      },
      {
        score: 2,
        textNear: createCreatureSettingTextNear(
          "darklands,darkland,subterranean,cave network,cave networks",
          CREATURE_TERRAIN_HABITAT_CONTEXT_TEXT_ANCHOR,
        ),
      },
      {
        score: 2,
        textAny: [
          patternAnchor("below ground", "description"),
          patternAnchor("found below ground", "description"),
          patternAnchor("beneath the earth", "description"),
          patternAnchor("under tunnels", "description"),
          patternAnchor("native to the darklands", "description"),
          patternAnchor("upper darklands", "description"),
          patternAnchor("darklands caverns", "description"),
          patternAnchor("sekamina", "description"),
        ],
      },
      {
        score: 2,
        textNear: createCreatureSettingTextNear(
          "cave,caves,cavern,caverns,underground,tunnel,tunnels,sewer,sewers,warren,warrens,below ground,darklands",
          CREATURE_UNDERGROUND_ACTIVITY_CONTEXT_TEXT_ANCHOR,
        ),
      },
      {
        score: 2,
        textAny: [
          patternAnchor("cave", "name"),
          patternAnchor("caves", "name"),
          patternAnchor("cavern", "name"),
          patternAnchor("caverns", "name"),
          patternAnchor("warren", "name"),
          patternAnchor("warrens", "name"),
        ],
      },
    ],
  },
  {
    tag: "underground_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: [patternAnchor("darklands"), patternAnchor("sekamina")] },
      { score: 1, textAny: [patternAnchor("subterranean"), patternAnchor("underground"), patternAnchor("underworld")] },
      { score: 1, textAny: [patternAnchor("cavern"), patternAnchor("caverns"), patternAnchor("lightless")] },
    ],
  },
  {
    tag: "urban_setting",
    category: "creature",
    anyOf: [
      {
        textAny: [
          patternAnchor("hide in plain sight in cities"),
          patternAnchor("city dwelling gargoyles"),
          patternAnchor("city-dwelling gargoyles"),
        ],
      },
    ],
  },
  {
    tag: "urban_setting",
    category: "creature",
    noneOf: [
      { textAny: URBAN_SETTING_BLOCKER_TEXT_ANCHORS },
      { textAny: URBAN_SETTING_SETTLEMENT_LIST_TEXT_ANCHORS, minTextAnyMatches: 2 },
    ],
    anyOf: [
      { score: 2, textAny: URBAN_SETTING_EXACT_TEXT_ANCHORS },
      { score: 2, textAny: URBAN_SETTING_NAME_ANCHORS },
      { score: 2, textAny: URBAN_SETTING_GOVERNANCE_TEXT_ANCHORS },
      { score: 1, familiesAny: ["official"] },
      { score: 1, textAny: [patternAnchor("enforce the law"), patternAnchor("settlement"), patternAnchor("settlements")] },
      {
        score: 2,
        textNear: createCreatureSettingTextNear(
          "city,cities,urban,street,streets,alley,alleys,sewer,sewers,town,towns,gutter,gutters,culvert,culverts,drain,drains,drainage",
          CREATURE_SITE_HABITAT_CONTEXT_TEXT_ANCHOR,
        ),
      },
      {
        score: 2,
        textNear: createCreatureSettingTextNear(
          "city,cities,urban,sewer,sewers,gutter,gutters,culvert,culverts,drain,drains,drainage,metropolis,metropolises,factory,factories,tenement,tenements",
          CREATURE_URBAN_ACTIVITY_CONTEXT_TEXT_ANCHOR,
          10,
        ),
      },
    ],
  },
  {
    tag: "rural_setting",
    category: "creature",
    threshold: 2,
    noneOf: [
      { textNear: RURAL_SETTING_BLOCKER_TEXT_NEAR },
    ],
    anyOf: [
      { score: 2, textAny: RURAL_SETTING_NAME_ANCHORS },
      { score: 2, textAny: RURAL_SETTING_TEXT_ANCHORS },
      {
        score: 2,
        textNear: createCreatureSettingTextNear(
          "village,villages,farmhouse,farmhouses,mill,mills,miller,millers,pasture,pastures,cropland,croplands,countryside",
          RURAL_SETTING_CONTEXT_TEXT_ANCHOR,
        ),
      },
      {
        score: 2,
        textNear: createCreatureSettingTextNear(
          "village,villages",
          RURAL_SETTING_COMMUNITY_CONTEXT_TEXT_ANCHOR,
          12,
        ),
      },
    ],
  },
  {
    tag: "arctic_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: [patternAnchor("arctic"), patternAnchor("snow"), patternAnchor("snows"), patternAnchor("tundra"), patternAnchor("frozen"), patternAnchor("glacier"), patternAnchor("glaciers"), patternAnchor("icebound")] },
      { score: 1, textAny: [patternAnchor("ice"), patternAnchor("icy")] },
    ],
  },
  {
    tag: "desert_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: DESERT_SETTING_EXACT_TEXT_ANCHORS },
      { score: 2, textAny: DESERT_SETTING_STRONG_TEXT_ANCHORS },
      { score: 2, textNear: DESERT_SETTING_TEXT_NEAR },
      { score: 1, textAny: DESERT_SETTING_NAME_ANCHORS },
      { score: 1, textAny: DESERT_SETTING_CORE_TEXT_ANCHORS },
      { score: 1, textAny: DESERT_SETTING_SUPPORT_TEXT_ANCHORS },
    ],
  },
  {
    tag: "wasteland_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textNear: WASTELAND_SETTING_TEXT_NEAR },
      {
        score: 2,
        textAny: [
          patternAnchor("wasteland"),
          patternAnchor("wastelands"),
          patternAnchor("wastes"),
          patternAnchor("barren"),
        ],
      },
      { score: 1, textAny: [patternAnchor("blasted")] },
    ],
  },
  {
    tag: "mountain_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: MOUNTAIN_SETTING_NAME_ANCHORS },
      {
        score: 2,
        textNear: createCreatureSettingTextNear(
          "mountain,mountains,cliff,cliffs,cliffside,cliffsides,peak,peaks,crag,crags,alp,alpine,mountain pass,mountain passes,ridge,ridges,highlands,high altitude,high altitudes,foothills,slope,slopes,escarpment,bluff,bluffs",
          CREATURE_TERRAIN_HABITAT_CONTEXT_TEXT_ANCHOR,
        ),
      },
      {
        score: 2,
        textAny: [
          patternAnchor("misty peaks", "description"),
          patternAnchor("mountain pass", "description"),
          patternAnchor("mountain passes", "description"),
          patternAnchor("mountain dwellers", "description"),
          patternAnchor("mountain dweller", "description"),
          ...MOUNTAIN_SETTING_TEXT_ANCHORS,
        ],
      },
    ],
  },
  {
    tag: "sky_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: SKY_SETTING_NAME_ANCHORS },
      { score: 2, textNear: SKY_SETTING_TEXT_NEAR },
    ],
  },
  {
    tag: "graveyard_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: GRAVEYARD_SETTING_CORE_TEXT_ANCHORS,
      },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(crypt,crypts,tomb,tombs,catacomb,catacombs,burial chamber,burial chambers)}}"),
              patternAnchor("{{alt(guardian,guards,guarding,haunts,dwells,lurks,patrols,watches over,buried dead,buried)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
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
            all: [
              patternAnchor("{{alt(ruin,ruins,derelict,crumbling,fallen,collapsed)}}"),
              patternAnchor("{{alt(hall,halls,temple,city,keep,structure,structures)}}"),
            ],
            window: 4,
            scope: "description",
          },
        ],
      },
      {
        score: 2,
        textAny: [
          patternAnchor("ancient ruins"),
          patternAnchor("ruins of"),
          patternAnchor("fortified ruins"),
          patternAnchor("operating out of fortified ruins"),
          patternAnchor("collapsed temple"),
          patternAnchor("ruined temple"),
          patternAnchor("ruined city"),
          patternAnchor("lost city"),
          patternAnchor("derelict structures"),
          patternAnchor("derelict structure"),
          patternAnchor("abandoned ruin"),
          patternAnchor("abandoned ruins"),
          patternAnchor("forgotten ruins"),
          patternAnchor("sunken city"),
          patternAnchor("overgrown structure"),
          patternAnchor("overgrown structures"),
          patternAnchor("derelict keep"),
          patternAnchor("fallen halls"),
          patternAnchor("shattered halls"),
        ],
      },
      { score: 1, textAny: [patternAnchor("ruins"), patternAnchor("ruin"), patternAnchor("derelict")] },
    ],
  },
  {
    tag: "temple_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textNear: createCreatureSettingTextNear(
          "temple,temples,shrine,shrines,cathedral,cathedrals,monastery,monasteries,chapel,chapels,sanctuary,sanctuaries,abbey,priory,cloister,holy site,holy sites,consecrated hall,ziggurat",
          CREATURE_SITE_HABITAT_CONTEXT_TEXT_ANCHOR,
        ),
      },
      {
        score: 2,
        textNear: createCreatureSettingTextNear(
          "temple,temples,shrine,shrines,cathedral,cathedrals,sanctuary,sanctuaries,holy site,holy sites",
          CREATURE_SITE_FUNCTION_CONTEXT_TEXT_ANCHOR,
          10,
        ),
      },
      { score: 2, textAny: [patternAnchor("place of worship", "description"), patternAnchor("house of worship", "description")] },
    ],
  },
  {
    tag: "fortress_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: FORTRESS_SETTING_NAME_ANCHORS },
      {
        score: 2,
        textNear: createCreatureSettingTextNearAnchors(
          FORTRESS_SETTING_STRUCTURE_TEXT_ANCHORS,
          CREATURE_SITE_HABITAT_CONTEXT_TEXT_ANCHOR,
        ),
      },
      {
        score: 2,
        textNear: createCreatureSettingTextNear(
          "fort,forts,garrison",
          CREATURE_SITE_HABITAT_CONTEXT_TEXT_ANCHOR,
        ),
      },
      {
        score: 2,
        textNear: createCreatureSettingTextNearAnchors(
          FORTRESS_SETTING_SITE_TEXT_ANCHORS,
          CREATURE_SITE_FUNCTION_CONTEXT_TEXT_ANCHOR,
          10,
        ),
      },
      {
        score: 2,
        textNear: createCreatureSettingTextNearAnchors(
          FORTRESS_SETTING_SITE_TEXT_ANCHORS,
          FORTRESS_SETTING_RESIDENCE_CONTEXT_TEXT_ANCHOR,
          6,
        ),
      },
      {
        score: 2,
        textAny: [
          patternAnchor("sky citadel", "description"),
          patternAnchor("fortified ruins", "description"),
          patternAnchor("operating out of fortified ruins", "description"),
        ],
      },
    ],
  },
  {
    tag: "volcanic_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: VOLCANIC_SETTING_EXACT_TEXT_ANCHORS },
      { score: 2, textNear: VOLCANIC_SETTING_TEXT_NEAR },
      { score: 2, textAny: [patternAnchor("lava field", "description"), patternAnchor("lava fields", "description")] },
      { score: 1, textAny: [patternAnchor("ash", "description"), patternAnchor("ashen", "description"), patternAnchor("cinder", "description"), patternAnchor("cinders", "description")] },
    ],
  },
  {
    tag: "profession_npc",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: STRONG_PROFESSION_NAME_ANCHORS },
      { score: 1, textAny: WEAK_PROFESSION_NAME_ANCHORS },
      { score: 1, textAny: CIVIC_SUPPORT_TEXT_ANCHORS },
      { score: 1, traitsAny: ["humanoid", "human"] },
    ],
  },
  {
    tag: "civic_npc",
    category: "creature",
    threshold: 1,
    requiresTags: ["profession_npc"],
    anyOf: [
      { score: 1, textAny: CIVIC_NPC_NAME_ANCHORS },
      { score: 1, textAny: CIVIC_SUPPORT_TEXT_ANCHORS },
    ],
    noneOf: [
      { traitsAny: CIVIC_NPC_BLOCKER_TRAITS },
      { familiesAny: UNDEAD_GLOSSARY_FAMILIES },
      { textAny: CIVIC_NPC_ROLE_BLOCKER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "combatant_npc",
    category: "creature",
    threshold: 2,
    noneOf: [
      { traitsAny: ["construct", "mindless", "animal", "beast", "undead", "ghost", "spirit"] },
      { familiesAny: UNDEAD_GLOSSARY_FAMILIES },
    ],
    anyOf: [
      { score: 2, textAny: COMBATANT_NPC_STRONG_NAME_ANCHORS },
      { score: 1, textAny: COMBATANT_NPC_WEAK_NAME_ANCHORS },
      { score: 1, textAny: COMBATANT_NPC_CONTEXT_NAME_ANCHORS },
      { score: 1, traitsAny: ["humanoid", "human", "troop"] },
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
    tag: "living_artwork",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: LIVING_ARTWORK_NAME_ANCHORS },
      { score: 2, textAny: LIVING_ARTWORK_TEXT_ANCHORS },
      {
        score: 1,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(painting,painted,portrait,mural,graffiti,illustration,drawing,two dimensional,flat surface)}}"),
              patternAnchor("{{alt(comes to life,come to life,given life,animated,animate)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
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
          all: [
            patternAnchor("{{alt(mask,masked,veil,veiled)}}"),
            patternAnchor("{{alt(face,faces,head,features,conceal,conceals,hide,hides)}}"),
          ],
          window: 5,
          scope: "description" as const,
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
          all: [
            patternAnchor("{{alt(faceless,featureless,blank,empty)}}"),
            patternAnchor("{{alt(face,faces,stolen,gone,hidden)}}"),
          ],
          window: 5,
          scope: "description" as const,
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
    allOf: [
      { traitsAny: ["construct"] },
    ],
    threshold: 2,
    noneOf: [
      { traitsAny: BOUND_OBJECT_LIVING_BLOCKER_TRAITS },
      { textAny: BOUND_OBJECT_MIMIC_BLOCKERS },
      { textAny: BOUND_OBJECT_FIGURATIVE_BLOCKERS },
    ],
    anyOf: [
      { score: 2, textAny: ANIMATED_OBJECT_NAME_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(animated,animate,animates,animation,brought to life,brought to unlife,comes to life)}}"),
              patternAnchor("{{alt(object,objects,armor,armors,doll,dolls,cart,carts,broom,brooms,cookware,door,doors,figurine,figurines,mannequin,mannequins,dummy,dummies,scarecrow,scarecrows,silverware,blade,blades,axe,axes,fireplace,fireplaces,vessel,vessels,cauldron,cauldrons,table,tables,chair,chairs,stool,stools,tool,tools)}}"),
            ],
            window: 4,
            scope: "either",
          },
        ],
      },
    ],
  },
  {
    tag: "animated_statue",
    category: "creature",
    allOf: [
      { traitsAny: ["construct"] },
    ],
    threshold: 2,
    noneOf: [
      { textAny: BOUND_OBJECT_MIMIC_BLOCKERS },
      { textAny: BOUND_OBJECT_FIGURATIVE_BLOCKERS },
      { textAny: [patternAnchor("stone animal"), patternAnchor("stone animals"), patternAnchor("stone beast"), patternAnchor("stone beasts"), patternAnchor("stone creature"), patternAnchor("stone creatures")] },
    ],
    anyOf: [
      { score: 2, textAny: ANIMATED_STATUE_NAME_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(statue,statues,effigy,effigies,idol,idols,monument,monuments)}}"),
              patternAnchor("{{alt(animated,animate,animates,animation,guardian,guardians,sentinel,sentinels,watch,watcher,watchers,protect,protects,comes to life,brought to life,stands guard,standing guard)}}"),
            ],
            window: 5,
            scope: "either",
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
