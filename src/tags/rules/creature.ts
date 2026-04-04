import {
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
  FIRST_WORLD_SETTING_CONTEXT_TEXT_ANCHORS,
  FIRST_WORLD_SETTING_TEXT_ANCHORS,
  FRESHWATER_SETTING_BLOCKER_TEXT_ANCHORS,
  FRESHWATER_SETTING_NAME_ANCHORS,
  FRESHWATER_SETTING_STRONG_TEXT_ANCHORS,
  FRESHWATER_SETTING_TEXT_ANCHORS,
  STRONG_PROFESSION_NAME_ANCHORS,
  UNDEAD_GLOSSARY_FAMILIES,
  WEAK_PROFESSION_NAME_ANCHORS,
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
      patternAnchor("{{alt(performer,performers,entertainer,entertainers,barker,barkers)}}"),
      patternAnchor("{{alt(circus,circuses,carnival,carnivals,traveling show,traveling circus,traveling carnival,jester,jesters)}}"),
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
  patternAnchor("old growth"),
  patternAnchor("bough"),
  patternAnchor("boughs"),
];

const DESERT_SETTING_NAME_ANCHORS = [
  patternAnchor("desert", "name"),
  patternAnchor("dune", "name"),
  patternAnchor("sand", "name"),
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
  patternAnchor("outlying settlement"),
];

const RURAL_SETTING_BLOCKER_TEXT_NEAR = [
  {
    all: [
      patternAnchor("{{alt(village,villages,farm,farms,hamlet,hamlets,countryside)}}"),
      patternAnchor("{{alt(raid,raids,raiding,attack,attacks,plunder,waylay,waylays)}}"),
    ],
    window: 6,
    scope: "description" as const,
  },
];

const URBAN_SETTING_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("market price"),
];

const URBAN_SETTING_SETTLEMENT_LIST_TEXT_ANCHORS = [
  patternAnchor("village"),
  patternAnchor("villages"),
  patternAnchor("city"),
  patternAnchor("cities"),
  patternAnchor("town"),
  patternAnchor("towns"),
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
  patternAnchor("duelist", "name"),
  patternAnchor("gladiator", "name"),
  patternAnchor("veteran", "name"),
  patternAnchor("defender", "name"),
  patternAnchor("enforcer", "name"),
  patternAnchor("marauder", "name"),
  patternAnchor("raider", "name"),
  patternAnchor("warrior", "name"),
  patternAnchor("disciple", "name"),
  patternAnchor("boss", "name"),
  patternAnchor("infiltrator", "name"),
  patternAnchor("bruiser", "name"),
  patternAnchor("sentinel", "name"),
  patternAnchor("skirmisher", "name"),
  patternAnchor("vanguard", "name"),
  patternAnchor("champion", "name"),
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
      { score: 1, textAny: FRESHWATER_SETTING_TEXT_ANCHORS },
      { score: 1, textAny: FRESHWATER_SETTING_NAME_ANCHORS },
      { score: 1, traitsAny: ["water"] },
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
      { score: 2, textAny: [patternAnchor("island"), patternAnchor("islands"), patternAnchor("archipelago"), patternAnchor("archipelagos"), patternAnchor("atoll"), patternAnchor("atolls")] },
      { score: 1, textAny: [patternAnchor("isle"), patternAnchor("isles")] },
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
    tag: "forest_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: FOREST_SETTING_NAME_ANCHORS },
      { score: 1, textAny: FOREST_SETTING_CORE_TEXT_ANCHORS },
      { score: 1, textAny: FOREST_SETTING_SUPPORT_TEXT_ANCHORS },
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
    anyOf: [
      {
        textAny: [
          patternAnchor("canyon"),
          patternAnchor("canyons"),
          patternAnchor("gorge"),
          patternAnchor("gorges"),
          patternAnchor("mesa"),
          patternAnchor("mesas"),
          patternAnchor("badlands"),
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
          patternAnchor("swamp"),
          patternAnchor("swamps"),
          patternAnchor("bog"),
          patternAnchor("bogs"),
          patternAnchor("marsh"),
          patternAnchor("marshes"),
          patternAnchor("fen"),
          patternAnchor("fens"),
          patternAnchor("mire"),
          patternAnchor("mires"),
          patternAnchor("wetland"),
          patternAnchor("wetlands"),
          patternAnchor("bayou"),
          patternAnchor("bayous"),
          patternAnchor("mangrove"),
          patternAnchor("mangroves"),
          patternAnchor("quagmire"),
          patternAnchor("quagmires"),
          patternAnchor("slough"),
          patternAnchor("sloughs"),
        ],
      },
    ],
  },
  {
    tag: "underground_setting",
    category: "creature",
    anyOf: [
      {
        score: 2,
        textAny: [
          patternAnchor("cave"),
          patternAnchor("caves"),
          patternAnchor("cavern"),
          patternAnchor("caverns"),
          patternAnchor("underground"),
          patternAnchor("tunnel"),
          patternAnchor("tunnels"),
          patternAnchor("subterranean"),
          patternAnchor("underworld"),
          patternAnchor("depths"),
          patternAnchor("crypt"),
          patternAnchor("crypts"),
          patternAnchor("mine"),
          patternAnchor("mines"),
          patternAnchor("mineshaft"),
          patternAnchor("mineshafts"),
          patternAnchor("quarry"),
          patternAnchor("quarries"),
          patternAnchor("warren"),
          patternAnchor("warrens"),
          patternAnchor("beneath the earth"),
          patternAnchor("under tunnels"),
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
      {
        textAny: [
          patternAnchor("city"),
          patternAnchor("cities"),
          patternAnchor("urban"),
          patternAnchor("street"),
          patternAnchor("streets"),
          patternAnchor("alley"),
          patternAnchor("alleys"),
          patternAnchor("sewer"),
          patternAnchor("sewers"),
          patternAnchor("town"),
          patternAnchor("towns"),
        ],
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
    anyOf: [
      { textAny: [patternAnchor("mountain"), patternAnchor("mountains"), patternAnchor("cliff"), patternAnchor("cliffs"), patternAnchor("peak"), patternAnchor("peaks"), patternAnchor("crag"), patternAnchor("crags"), patternAnchor("alp"), patternAnchor("alpine"), patternAnchor("mountain pass"), patternAnchor("mountain passes"), patternAnchor("ridge"), patternAnchor("ridges"), patternAnchor("highlands"), patternAnchor("foothills"), patternAnchor("slope"), patternAnchor("slopes"), patternAnchor("escarpment"), patternAnchor("bluff"), patternAnchor("bluffs")] },
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
        ],
      },
      { score: 1, textAny: [patternAnchor("crypt"), patternAnchor("crypts")] },
      { score: 1, textAny: [patternAnchor("tomb"), patternAnchor("tombs")] },
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
        textAny: [
          patternAnchor("temple"),
          patternAnchor("temples"),
          patternAnchor("shrine"),
          patternAnchor("shrines"),
          patternAnchor("cathedral"),
          patternAnchor("cathedrals"),
          patternAnchor("monastery"),
          patternAnchor("monasteries"),
          patternAnchor("chapel"),
          patternAnchor("chapels"),
          patternAnchor("sanctuary"),
          patternAnchor("abbey"),
          patternAnchor("priory"),
          patternAnchor("cloister"),
          patternAnchor("holy site"),
          patternAnchor("consecrated hall"),
          patternAnchor("ziggurat"),
        ],
      },
      { score: 1, textAny: [patternAnchor("place of worship"), patternAnchor("house of worship")] },
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
          patternAnchor("fortress"),
          patternAnchor("fortresses"),
          patternAnchor("castle"),
          patternAnchor("castles"),
          patternAnchor("citadel"),
          patternAnchor("citadels"),
          patternAnchor("stronghold"),
          patternAnchor("strongholds"),
          patternAnchor("keep"),
          patternAnchor("keeps"),
          patternAnchor("bastion"),
          patternAnchor("bastions"),
          patternAnchor("watchtower"),
          patternAnchor("watchtowers"),
          patternAnchor("rampart"),
          patternAnchor("ramparts"),
          patternAnchor("battlement"),
          patternAnchor("battlements"),
        ],
      },
      { score: 2, textAny: [patternAnchor("sky citadel")] },
      { score: 1, textAny: [patternAnchor("fort"), patternAnchor("forts"), patternAnchor("garrison")] },
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
          patternAnchor("volcanic"),
          patternAnchor("volcano"),
          patternAnchor("volcanoes"),
          patternAnchor("lava"),
          patternAnchor("magma"),
          patternAnchor("caldera"),
        ],
      },
      { score: 1, textAny: [patternAnchor("ash"), patternAnchor("ashen"), patternAnchor("cinder"), patternAnchor("cinders")] },
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
    requiresTags: ["profession_npc"],
    noneOf: [
      { traitsAny: CIVIC_NPC_BLOCKER_TRAITS },
      { familiesAny: UNDEAD_GLOSSARY_FAMILIES },
      { textAny: COMBATANT_NPC_STRONG_NAME_ANCHORS },
      { textAny: COMBATANT_NPC_WEAK_NAME_ANCHORS },
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
