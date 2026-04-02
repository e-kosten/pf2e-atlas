import { DerivedTagCatalogEntry, SearchCategory, SearchSubcategory } from "./types.js";
import { normalizeText, uniqueSorted } from "./utils.js";

type DerivedTagContext = {
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  descriptionText: string | null;
  traits: string[];
  families?: string[];
  references?: DerivedTagReference[];
};

type DerivedTagReference = {
  recordKey: string;
  packName: string;
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  traits: string[];
};

type TextMatchScope = "either" | "name" | "description";
type TextMatchMode = "token" | "phrase";

type TextAnchor = string | {
  value: string;
  mode?: TextMatchMode;
  scope?: TextMatchScope;
};

type DerivedTagMatchClause = {
  score?: number;
  traitsAny?: string[];
  traitsAll?: string[];
  familiesAny?: string[];
  familiesAll?: string[];
  textAny?: TextAnchor[];
  textAll?: TextAnchor[];
  referencesAny?: string[];
  referencesAll?: string[];
};

type DerivedTagRule = {
  tag: string;
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
  threshold?: number;
  requiresTags?: string[];
  anyOf?: DerivedTagMatchClause[];
  allOf?: DerivedTagMatchClause[];
  noneOf?: DerivedTagMatchClause[];
};

type NormalizedTextView = {
  text: string;
  tokenSet: Set<string>;
};

type NormalizedDerivedTagContext = {
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  traits: Set<string>;
  families: Set<string>;
  name: NormalizedTextView;
  description: NormalizedTextView;
  referenceKeys: Set<string>;
};

const tokenAnchor = (value: string, scope: TextMatchScope = "either"): TextAnchor => ({ value, mode: "token", scope });
const phraseAnchor = (value: string, scope: TextMatchScope = "either"): TextAnchor => ({ value, mode: "phrase", scope });

const OFFENSIVE_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("venom"),
  tokenAnchor("bomb"),
  phraseAnchor("injury poison"),
  phraseAnchor("contact poison"),
  phraseAnchor("ingested poison"),
  phraseAnchor("inhaled poison"),
  phraseAnchor("weapon poison"),
  phraseAnchor("afflicts the target"),
];

const GEARISH_SUBCATEGORIES: SearchSubcategory[] = ["gear", "backpack", "kit", "vehicle"];
const DISGUISE_SUBCATEGORIES: SearchSubcategory[] = [...GEARISH_SUBCATEGORIES, "consumable"];
const TRACKING_SUBCATEGORIES: SearchSubcategory[] = [...GEARISH_SUBCATEGORIES, "consumable"];

const STRONG_PROFESSION_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("captain", "name"),
  tokenAnchor("commoner", "name"),
  tokenAnchor("courtier", "name"),
  tokenAnchor("guard", "name"),
  tokenAnchor("scout", "name"),
  tokenAnchor("sailor", "name"),
  tokenAnchor("merchant", "name"),
  tokenAnchor("priest", "name"),
  tokenAnchor("noble", "name"),
  tokenAnchor("prophet", "name"),
  tokenAnchor("advisor", "name"),
  tokenAnchor("acolyte", "name"),
  tokenAnchor("vigilante", "name"),
];

const WEAK_PROFESSION_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("agent", "name"),
  tokenAnchor("apprentice", "name"),
  tokenAnchor("hunter", "name"),
  tokenAnchor("enforcer", "name"),
];

const UNDEAD_GLOSSARY_FAMILIES = [
  "blackfrost-dead",
  "floodslain",
  "ghast",
  "ghost",
  "ghoul",
  "graveknight",
  "lich",
  "ravener",
  "siabrae",
  "vampire",
  "visitant",
];

const SCENE_ADJACENT_BLOCKER_TRAITS = [
  "aberration",
  "animal",
  "construct",
  "dragon",
  "elemental",
  "fey",
  "fiend",
  "fungus",
  "ghost",
  "ghoul",
  "giant",
  "golem",
  "leshy",
  "mindless",
  "monitor",
  "ooze",
  "plant",
  "psychopomp",
  "skeleton",
  "spirit",
  "swarm",
  "troop",
  "undead",
];

/**
 * Derived-tag rules intentionally author linked-record anchors as `pack:name`
 * instead of the full Foundry UUID shape like `spells-srd.Item.Illusory Disguise`.
 * Current PF2E compendium references already resolve on pack plus friendly locator,
 * and the document-type segment is mostly noise for retrieval-oriented rules.
 * If PF2E/Foundry ever starts reusing locators within a pack such that `pack:name`
 * is no longer a stable shorthand, revisit this helper and the rule surface.
 */
function normalizeDerivedTagReference(value: string): string {
  const separatorIndex = value.indexOf(":");
  if (separatorIndex === -1) {
    return normalizeText(value);
  }

  const packName = value.slice(0, separatorIndex);
  const recordName = value.slice(separatorIndex + 1);
  return `${normalizeText(packName)}:${normalizeText(recordName)}`;
}

const referenceAnchor = (packName: string, name: string): string => normalizeDerivedTagReference(`${packName}:${name}`);

const DISGUISE_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("disguise"),
  tokenAnchor("impersonate"),
  phraseAnchor("false identity"),
  tokenAnchor("costume"),
  tokenAnchor("masquerade"),
  phraseAnchor("quick change", "name"),
  phraseAnchor("take on the appearance"),
  phraseAnchor("change your appearance"),
  phraseAnchor("assume role"),
];

const SOCIAL_INFILTRATION_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("false identity"),
  phraseAnchor("pass as"),
  phraseAnchor("blend into society"),
  phraseAnchor("social infiltration"),
  tokenAnchor("impersonate"),
  tokenAnchor("masquerade"),
  phraseAnchor("quick change", "name"),
  tokenAnchor("disguise", "name"),
  phraseAnchor("take on the appearance"),
  phraseAnchor("change your appearance"),
  phraseAnchor("assume role"),
];

const DISGUISE_REFERENCE_ANCHORS = [
  referenceAnchor("actionspf2e", "Impersonate"),
  referenceAnchor("spells-srd", "Illusory Disguise"),
  referenceAnchor("equipment-srd", "Disguise Kit"),
];

const SOCIAL_INFILTRATION_REFERENCE_ANCHORS = [
  referenceAnchor("actionspf2e", "Impersonate"),
  referenceAnchor("spells-srd", "Illusory Disguise"),
];

const RESTRAINT_ESCAPE_REFERENCE_ANCHORS = [
  referenceAnchor("actionspf2e", "Escape"),
  referenceAnchor("conditionitems", "Grabbed"),
  referenceAnchor("conditionitems", "Restrained"),
];

const RESTRAINT_CAPTURE_REFERENCE_ANCHORS = [
  referenceAnchor("actionspf2e", "Grapple"),
  referenceAnchor("conditionitems", "Grabbed"),
  referenceAnchor("conditionitems", "Restrained"),
];

const MOBILITY_REFERENCE_ANCHORS = [
  referenceAnchor("actionspf2e", "Balance"),
  referenceAnchor("actionspf2e", "Climb"),
  referenceAnchor("actionspf2e", "High Jump"),
  referenceAnchor("actionspf2e", "Leap"),
  referenceAnchor("actionspf2e", "Long Jump"),
  referenceAnchor("actionspf2e", "Swim"),
];

const NAVIGATION_REFERENCE_ANCHORS = [
  referenceAnchor("actionspf2e", "Sense Direction"),
];

const TRACKING_REFERENCE_ANCHORS = [
  referenceAnchor("actionspf2e", "Track"),
  referenceAnchor("actionspf2e", "Cover Tracks"),
];

const HAZARD_ALARM_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("alarm"),
  tokenAnchor("alert"),
  tokenAnchor("alerts"),
  phraseAnchor("raise the alarm"),
  phraseAnchor("alert nearby guards"),
  phraseAnchor("sound an alarm"),
  phraseAnchor("warning bell"),
  tokenAnchor("intrusion"),
];

const AFFLICTION_MENTAL_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("confused"),
  tokenAnchor("confusion"),
  tokenAnchor("delirium"),
  tokenAnchor("paranoia"),
  tokenAnchor("frightened"),
  tokenAnchor("hallucination"),
  tokenAnchor("hallucinations"),
  tokenAnchor("whispers"),
  tokenAnchor("panic"),
];

const AFFLICTION_MOBILITY_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("speed is reduced"),
  phraseAnchor("reduces the victim s speed"),
  phraseAnchor("reduces your speed"),
  phraseAnchor("stiffens joints"),
  tokenAnchor("immobilized"),
  tokenAnchor("paralyzed"),
  tokenAnchor("paralysis"),
  tokenAnchor("slowed"),
];

const DERIVED_TAG_RULES: DerivedTagRule[] = [
  {
    tag: "offensive",
    category: "equipment",
    subcategories: ["consumable"],
    anyOf: [
      { traitsAny: ["poison", "bomb"] },
      { textAny: OFFENSIVE_TEXT_ANCHORS },
    ],
  },
  {
    tag: "thrown_offense",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["offensive"],
    anyOf: [
      { traitsAny: ["bomb"] },
      { textAny: [tokenAnchor("thrown"), tokenAnchor("hurl"), tokenAnchor("lob"), phraseAnchor("splash weapon"), phraseAnchor("throw the bomb")] },
    ],
  },
  {
    tag: "weapon_applied",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["offensive"],
    anyOf: [
      { textAny: [phraseAnchor("apply to a weapon"), phraseAnchor("coat a weapon"), phraseAnchor("weapon poison"), phraseAnchor("smeared on a weapon"), phraseAnchor("applied to a weapon")] },
    ],
  },
  {
    tag: "ingested_offense",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["offensive"],
    anyOf: [
      { textAny: [phraseAnchor("ingested poison"), phraseAnchor("must be eaten"), phraseAnchor("must be drunk"), phraseAnchor("consumed by the target"), phraseAnchor("when swallowed")] },
    ],
  },
  {
    tag: "contact_offense",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["offensive"],
    anyOf: [
      { textAny: [phraseAnchor("contact poison"), phraseAnchor("through skin contact"), phraseAnchor("through contact"), phraseAnchor("absorbed through the skin")] },
    ],
  },
  {
    tag: "beneficial",
    category: "equipment",
    subcategories: ["consumable"],
    anyOf: [
      { traitsAny: ["elixir", "healing"] },
      {
        textAny: [
          phraseAnchor("status bonus to"),
          phraseAnchor("item bonus to"),
          phraseAnchor("grants you"),
          phraseAnchor("giving you"),
          phraseAnchor("no longer need to breathe"),
          phraseAnchor("do not need air"),
          phraseAnchor("imprecise scent"),
          phraseAnchor("gain a burrow speed"),
          phraseAnchor("gain a climb speed"),
        ],
      },
      {
        textAny: [
          phraseAnchor("trigger you attempt to"),
          phraseAnchor("trigger you become"),
          phraseAnchor("requirements you re"),
          phraseAnchor("in case of kidnapping"),
          phraseAnchor("slippery enough to"),
        ],
        referencesAny: RESTRAINT_ESCAPE_REFERENCE_ANCHORS,
      },
      {
        textAny: [
          tokenAnchor("restorative"),
          tokenAnchor("remedy"),
          tokenAnchor("curative"),
          tokenAnchor("antidote"),
          tokenAnchor("antiplague"),
          tokenAnchor("protective"),
          tokenAnchor("catharsis"),
          tokenAnchor("healing"),
          tokenAnchor("darkvision"),
          phraseAnchor("additional protection"),
          phraseAnchor("absorb damage"),
          phraseAnchor("temporary hit points"),
          phraseAnchor("grants resistance"),
          phraseAnchor("gain resistance"),
          phraseAnchor("protect against poison"),
          phraseAnchor("protect against poisons"),
          phraseAnchor("protect against disease"),
          phraseAnchor("protects you against"),
          phraseAnchor("resistance to"),
          phraseAnchor("gain a bonus"),
          phraseAnchor("bolsters the drinker"),
          phraseAnchor("steady the emotions"),
          phraseAnchor("see in the dark"),
        ],
      },
    ],
    noneOf: [
      { traitsAny: ["poison", "bomb"] },
      { textAny: OFFENSIVE_TEXT_ANCHORS },
    ],
  },
  {
    tag: "healing_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { traitsAny: ["healing"] },
      { textAny: [phraseAnchor("elixir of life"), tokenAnchor("healing"), phraseAnchor("restore hit points"), phraseAnchor("restore hp"), phraseAnchor("regain hit points")] },
    ],
  },
  {
    tag: "anti_poison",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [tokenAnchor("antidote"), phraseAnchor("against poison"), phraseAnchor("against poisons"), phraseAnchor("protect against poison"), phraseAnchor("protects you against poisons"), phraseAnchor("resist poison"), phraseAnchor("ward off poison"), phraseAnchor("persistent poison damage")] },
    ],
  },
  {
    tag: "anti_disease",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [tokenAnchor("antiplague"), phraseAnchor("against disease"), phraseAnchor("against diseases"), phraseAnchor("protect against disease"), phraseAnchor("resist disease"), phraseAnchor("ward off disease")] },
    ],
  },
  {
    tag: "condition_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [tokenAnchor("condition"), tokenAnchor("catharsis"), phraseAnchor("soothe the mind"), phraseAnchor("steady the emotions"), phraseAnchor("calm overwhelming emotions"), phraseAnchor("recover from mental conditions")] },
    ],
  },
  {
    tag: "mental_recovery",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [phraseAnchor("soothe the mind"), phraseAnchor("steady the emotions"), phraseAnchor("calm overwhelming emotions"), phraseAnchor("mental condition"), phraseAnchor("mental conditions"), tokenAnchor("emotion"), tokenAnchor("emotions"), tokenAnchor("frightened"), tokenAnchor("stupefied"), tokenAnchor("confused"), phraseAnchor("mental effect")] },
    ],
  },
  {
    tag: "escape_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [tokenAnchor("escape"), phraseAnchor("slip away"), phraseAnchor("break free"), tokenAnchor("flee"), tokenAnchor("evade"), phraseAnchor("concealing smoke"), phraseAnchor("vanish from sight"), tokenAnchor("misty")] },
      { referencesAny: [referenceAnchor("actionspf2e", "Escape")] },
      { textAny: [tokenAnchor("grabbed"), tokenAnchor("restrained"), tokenAnchor("immobilized"), phraseAnchor("slip free"), phraseAnchor("break the grab")] },
    ],
  },
  {
    tag: "senses_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [tokenAnchor("darkvision"), phraseAnchor("see in the dark"), phraseAnchor("low light vision"), phraseAnchor("keen senses"), phraseAnchor("sharpen your vision"), phraseAnchor("see invisible"), tokenAnchor("scent")] },
    ],
  },
  {
    tag: "energy_resistance",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [phraseAnchor("resistance to fire"), phraseAnchor("resistance to cold"), phraseAnchor("resistance to electricity"), phraseAnchor("resistance to acid"), phraseAnchor("resistance to sonic"), phraseAnchor("resistance to energy"), phraseAnchor("energy resistance"), phraseAnchor("against fire damage"), phraseAnchor("against cold damage"), phraseAnchor("against electricity damage"), phraseAnchor("against acid damage"), phraseAnchor("against sonic damage")] },
    ],
  },
  {
    tag: "buff_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [phraseAnchor("gain a bonus"), phraseAnchor("bonus to"), tokenAnchor("bolster"), tokenAnchor("enhance"), tokenAnchor("empower"), phraseAnchor("heighten your senses"), phraseAnchor("increase your speed"), phraseAnchor("resistance to"), phraseAnchor("grants resistance"), phraseAnchor("temporary hit points"), phraseAnchor("additional protection"), phraseAnchor("absorb damage")] },
    ],
  },
  {
    tag: "self_buff",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [phraseAnchor("you gain"), phraseAnchor("the drinker gains"), phraseAnchor("gain a bonus"), phraseAnchor("you become"), phraseAnchor("you gain resistance"), phraseAnchor("you gain darkvision"), phraseAnchor("protects you against"), phraseAnchor("when you drink"), phraseAnchor("drinking this"), phraseAnchor("spreading the salve on exposed skin"), phraseAnchor("grants you"), phraseAnchor("giving you")] },
    ],
  },
  {
    tag: "ally_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [phraseAnchor("target gains"), phraseAnchor("an ally gains"), phraseAnchor("creature that drinks gains"), phraseAnchor("the drinker gains")] },
    ],
  },
  {
    tag: "climbing",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("climb"), tokenAnchor("climbing"), tokenAnchor("rappel"), tokenAnchor("rappelling"), tokenAnchor("piton"), tokenAnchor("grappling")] },
    ],
  },
  {
    tag: "mobility",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    threshold: 2,
    anyOf: [
      { score: 2, textAny: [tokenAnchor("mobility"), phraseAnchor("move quickly"), phraseAnchor("increase your speed"), tokenAnchor("rappel"), tokenAnchor("climbing")] },
      {
        score: 2,
        textAny: [
          phraseAnchor("high jump"),
          phraseAnchor("long jump"),
          phraseAnchor("land speed"),
          phraseAnchor("climb speed"),
          phraseAnchor("climb speeds"),
          phraseAnchor("swim speed"),
          phraseAnchor("swim speeds"),
          phraseAnchor("ignore difficult terrain"),
          phraseAnchor("difficult terrain"),
          phraseAnchor("improved grip"),
          phraseAnchor("exceptional traction"),
          tokenAnchor("traction"),
        ],
      },
      { score: 2, referencesAny: MOBILITY_REFERENCE_ANCHORS },
    ],
  },
  {
    tag: "lock_bypass",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("lockpick"), tokenAnchor("lockpicks"), phraseAnchor("pick locks"), phraseAnchor("picking locks"), phraseAnchor("bypass locks"), phraseAnchor("thieves tools"), phraseAnchor("thieves tools"), tokenAnchor("toolkit")] },
    ],
  },
  {
    tag: "concealable",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("concealable"), phraseAnchor("hidden on your person"), phraseAnchor("hidden tools"), phraseAnchor("slim lockpicks")] },
    ],
  },
  {
    tag: "scouting",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("scout"), tokenAnchor("scouting"), tokenAnchor("survey"), tokenAnchor("recon"), phraseAnchor("observe from afar"), tokenAnchor("spyglass")] },
    ],
  },
  {
    tag: "stealth_support",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("stealth"), tokenAnchor("quiet"), tokenAnchor("silent"), phraseAnchor("without drawing attention"), phraseAnchor("avoid notice"), tokenAnchor("infiltration")] },
      { textAny: [tokenAnchor("concealable"), phraseAnchor("hidden on your person")] },
    ],
  },
  {
    tag: "disguise",
    category: "equipment",
    subcategories: DISGUISE_SUBCATEGORIES,
    anyOf: [
      { textAny: DISGUISE_TEXT_ANCHORS },
      { referencesAny: DISGUISE_REFERENCE_ANCHORS },
    ],
  },
  {
    tag: "social_infiltration",
    category: "equipment",
    subcategories: DISGUISE_SUBCATEGORIES,
    requiresTags: ["disguise"],
    anyOf: [
      { textAny: SOCIAL_INFILTRATION_TEXT_ANCHORS },
      { referencesAny: SOCIAL_INFILTRATION_REFERENCE_ANCHORS },
    ],
  },
  {
    tag: "disguise",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: DISGUISE_TEXT_ANCHORS },
      { score: 1, traitsAny: ["illusion"] },
    ],
  },
  {
    tag: "social_infiltration",
    category: "spell",
    requiresTags: ["disguise"],
    anyOf: [
      { textAny: SOCIAL_INFILTRATION_TEXT_ANCHORS },
      { traitsAny: ["illusion"] },
    ],
  },
  {
    tag: "illumination",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("light"), tokenAnchor("illumination"), tokenAnchor("lantern"), tokenAnchor("torch"), tokenAnchor("glow"), tokenAnchor("illuminate")] },
    ],
  },
  {
    tag: "survival",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("camp"), tokenAnchor("forage"), tokenAnchor("wilderness"), tokenAnchor("survival"), tokenAnchor("shelter"), tokenAnchor("weatherproof")] },
    ],
  },
  {
    tag: "navigation",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      {
        textAny: [
          tokenAnchor("navigate"),
          tokenAnchor("navigation"),
          tokenAnchor("map"),
          tokenAnchor("compass"),
          tokenAnchor("chart"),
          phraseAnchor("sense direction"),
          phraseAnchor("track your heading"),
          phraseAnchor("which direction is north"),
          phraseAnchor("learn which direction you re facing"),
        ],
      },
      { referencesAny: NAVIGATION_REFERENCE_ANCHORS },
    ],
  },
  {
    tag: "navigation",
    category: "equipment",
    subcategories: ["consumable"],
    threshold: 2,
    anyOf: [
      { score: 2, referencesAny: NAVIGATION_REFERENCE_ANCHORS },
      {
        score: 2,
        textAny: [
          phraseAnchor("sense direction"),
          phraseAnchor("functions as if you have a compass"),
          phraseAnchor("learn which direction you re facing"),
          phraseAnchor("which direction is north"),
        ],
      },
    ],
  },
  {
    tag: "tracking",
    category: "equipment",
    subcategories: TRACKING_SUBCATEGORIES,
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          phraseAnchor("sense and follow tracks"),
          phraseAnchor("for later tracking"),
          phraseAnchor("track their movements"),
          phraseAnchor("track a creature by its scent"),
          phraseAnchor("continue to track the same creature"),
          phraseAnchor("flutters toward the affixed one"),
        ],
      },
      {
        score: 2,
        textAll: [
          tokenAnchor("survival"),
          phraseAnchor("sense direction"),
          tokenAnchor("track"),
        ],
      },
      {
        score: 2,
        textAll: [
          tokenAnchor("survival"),
          tokenAnchor("scent"),
        ],
        referencesAny: [referenceAnchor("actionspf2e", "Track")],
      },
      {
        score: 2,
        textAll: [
          tokenAnchor("survival"),
          tokenAnchor("scent"),
          tokenAnchor("track"),
        ],
      },
      { score: 1, textAny: [tokenAnchor("tracker", "name"), tokenAnchor("tracking", "name")] },
    ],
    noneOf: [
      {
        textAny: [
          phraseAnchor("track time"),
          phraseAnchor("tracking a gunslinger s progress"),
          phraseAnchor("track teleportation"),
          phraseAnchor("dc to track"),
          phraseAnchor("attempting to track you"),
          phraseAnchor("circumstance bonus to their check"),
        ],
      },
      {
        textAny: [tokenAnchor("trackless", "name")],
      },
    ],
  },
  {
    tag: "anti_tracking",
    category: "equipment",
    subcategories: TRACKING_SUBCATEGORIES,
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          phraseAnchor("dc to track"),
          phraseAnchor("attempting to track you"),
          phraseAnchor("cover any ordinary odors"),
          phraseAnchor("fleeing pursuit"),
        ],
      },
      {
        score: 2,
        textAll: [
          tokenAnchor("stealth"),
          tokenAnchor("hide"),
          tokenAnchor("sneak"),
        ],
        textAny: [
          phraseAnchor("against creatures using primarily smell"),
        ],
      },
      { score: 2, textAny: [tokenAnchor("trackless", "name")] },
    ],
    noneOf: [
      {
        textAny: [
          phraseAnchor("track a creature by its scent"),
          phraseAnchor("track their movements"),
          phraseAnchor("continue to track the same creature"),
          phraseAnchor("sense and follow tracks"),
          phraseAnchor("circumstance bonus to their check"),
        ],
      },
    ],
  },
  {
    tag: "transport",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("transport"), tokenAnchor("wagon"), tokenAnchor("sled"), tokenAnchor("boat"), tokenAnchor("vehicle"), phraseAnchor("carry riders"), phraseAnchor("haul passengers")] },
    ],
  },
  {
    tag: "trap_bypass",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [phraseAnchor("disarm a trap"), phraseAnchor("disable device"), phraseAnchor("trap mechanism"), tokenAnchor("tripwire"), phraseAnchor("bypass a trap")] },
    ],
  },
  {
    tag: "carry_support",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("storage"), tokenAnchor("stow"), tokenAnchor("carry"), tokenAnchor("haul"), tokenAnchor("pouch"), tokenAnchor("backpack"), tokenAnchor("container"), tokenAnchor("pack")] },
    ],
  },
  {
    tag: "restraint_escape",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          phraseAnchor("break free"),
          phraseAnchor("free of a creature grabbing you"),
          phraseAnchor("difficult to hold back"),
          phraseAnchor("whenever you are affected by an effect that lasts until you"),
          phraseAnchor("trigger you become"),
        ],
        referencesAny: RESTRAINT_ESCAPE_REFERENCE_ANCHORS,
      },
      {
        score: 2,
        textAny: [
          phraseAnchor("free someone from manacles"),
          phraseAnchor("free yourself from manacles"),
        ],
      },
      {
        score: 2,
        textAll: [
          tokenAnchor("athletics"),
          tokenAnchor("escape"),
        ],
        referencesAny: [referenceAnchor("actionspf2e", "Escape")],
      },
    ],
    noneOf: [
      {
        textAny: [
          phraseAnchor("target becomes grabbed"),
          phraseAnchor("target becomes restrained"),
          phraseAnchor("creature becomes grabbed"),
          phraseAnchor("creature becomes restrained"),
          phraseAnchor("restrain the target"),
          phraseAnchor("grabs the target"),
          phraseAnchor("tighten the loop"),
        ],
      },
    ],
  },
  {
    tag: "restraint_capture",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          phraseAnchor("cinched down on a captive"),
          phraseAnchor("at your mercy"),
          phraseAnchor("legs bound"),
        ],
      },
      {
        score: 2,
        textAny: [
          phraseAnchor("target becomes restrained"),
          phraseAnchor("creature becomes restrained"),
          phraseAnchor("restrain the target"),
        ],
      },
      {
        score: 2,
        textAny: [
          phraseAnchor("tighten the loop"),
          phraseAnchor("without having a free hand"),
          phraseAnchor("up to 10 feet away"),
        ],
        referencesAny: RESTRAINT_CAPTURE_REFERENCE_ANCHORS,
      },
    ],
    noneOf: [
      {
        textAny: [
          phraseAnchor("break free"),
          phraseAnchor("free someone from manacles"),
          phraseAnchor("free yourself from manacles"),
        ],
      },
    ],
  },
  {
    tag: "alarm",
    category: "hazard",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: HAZARD_ALARM_TEXT_ANCHORS },
      { score: 1, textAny: [tokenAnchor("glyph"), tokenAnchor("ward"), tokenAnchor("threshold")] },
    ],
  },
  {
    tag: "restraint_capture",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          phraseAnchor("bind intruders"),
          phraseAnchor("holds intruders in place"),
          phraseAnchor("hold creatures in place"),
          phraseAnchor("lashes out with force bands"),
          phraseAnchor("until they escape"),
        ],
        referencesAny: RESTRAINT_CAPTURE_REFERENCE_ANCHORS,
      },
      {
        score: 2,
        textAny: [
          phraseAnchor("creature becomes restrained"),
          phraseAnchor("target becomes restrained"),
        ],
      },
    ],
  },
  {
    tag: "mental_impairment",
    category: "affliction",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AFFLICTION_MENTAL_TEXT_ANCHORS },
      { score: 1, traitsAny: ["mental", "emotion", "fear"] },
    ],
  },
  {
    tag: "mobility_impairment",
    category: "affliction",
    anyOf: [
      { textAny: AFFLICTION_MOBILITY_TEXT_ANCHORS },
    ],
  },
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
    tag: "aquatic_context",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 3, traitsAny: ["water", "aquatic", "amphibious"] },
      { score: 2, textAny: [tokenAnchor("aquatic"), tokenAnchor("ocean"), tokenAnchor("river"), tokenAnchor("coast"), tokenAnchor("coasts")] },
      { score: 1, textAny: [tokenAnchor("sea"), tokenAnchor("harbor"), tokenAnchor("water")] },
    ],
  },
  {
    tag: "nautical",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: [tokenAnchor("sailor"), tokenAnchor("mariner"), tokenAnchor("dock"), tokenAnchor("docks"), tokenAnchor("bilge"), tokenAnchor("wreck"), tokenAnchor("crew"), tokenAnchor("shipwreck"), tokenAnchor("shipwrecks"), phraseAnchor("shipwreck")] },
      { score: 1, textAny: [tokenAnchor("ship"), tokenAnchor("captain")] },
      { score: 1, textAny: [tokenAnchor("harbor")] },
    ],
  },
  {
    tag: "forest",
    category: "creature",
    anyOf: [
      { textAny: [tokenAnchor("forest"), tokenAnchor("woodland"), tokenAnchor("grove"), tokenAnchor("briar")] },
    ],
  },
  {
    tag: "swamp",
    category: "creature",
    anyOf: [
      { textAny: [tokenAnchor("swamp"), tokenAnchor("bog"), tokenAnchor("marsh"), tokenAnchor("fen"), tokenAnchor("mire")] },
    ],
  },
  {
    tag: "underground",
    category: "creature",
    anyOf: [
      { textAny: [tokenAnchor("cave"), tokenAnchor("cavern"), tokenAnchor("underground"), tokenAnchor("tunnel"), tokenAnchor("subterranean"), tokenAnchor("underworld"), tokenAnchor("depths"), tokenAnchor("crypt"), tokenAnchor("crypts")] },
    ],
  },
  {
    tag: "urban",
    category: "creature",
    anyOf: [
      { textAny: [tokenAnchor("city"), tokenAnchor("urban"), tokenAnchor("street"), tokenAnchor("alley"), tokenAnchor("market"), tokenAnchor("sewer"), tokenAnchor("town")] },
    ],
  },
  {
    tag: "arctic",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: [tokenAnchor("arctic"), tokenAnchor("snow"), tokenAnchor("tundra"), tokenAnchor("frozen"), tokenAnchor("glacier")] },
      { score: 1, textAny: [tokenAnchor("ice")] },
    ],
  },
  {
    tag: "desert",
    category: "creature",
    anyOf: [
      { textAny: [tokenAnchor("desert"), tokenAnchor("dune"), tokenAnchor("sand"), tokenAnchor("arid"), tokenAnchor("wastes")] },
    ],
  },
  {
    tag: "mountain",
    category: "creature",
    anyOf: [
      { textAny: [tokenAnchor("mountain"), tokenAnchor("cliff"), tokenAnchor("peak"), tokenAnchor("crag"), tokenAnchor("alp")] },
    ],
  },
  {
    tag: "graveyard",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: [tokenAnchor("graveyard"), tokenAnchor("cemetery")] },
      { score: 1, textAny: [tokenAnchor("crypt"), tokenAnchor("crypts")] },
      { score: 1, textAny: [tokenAnchor("tomb"), tokenAnchor("tombs")] },
    ],
  },
  {
    tag: "ruins",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: [phraseAnchor("crumbling hall"), phraseAnchor("fallen temple"), phraseAnchor("ancient hall")] },
      { score: 1, textAny: [tokenAnchor("ruins"), tokenAnchor("ruin"), tokenAnchor("derelict")] },
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

export const DERIVED_TAG_CATALOG: DerivedTagCatalogEntry[] = [
  {
    category: "equipment",
    subcategories: ["consumable"],
    family: "function",
    description: "Beneficial consumable outcome and recovery tags.",
    tags: [
      { value: "beneficial", description: "Broad support-oriented consumable with non-hostile intent." },
      { value: "healing_support", description: "Restores hit points or provides direct healing." },
      { value: "anti_poison", description: "Helps resist, prevent, or recover from poison." },
      { value: "anti_disease", description: "Helps resist, prevent, or recover from disease." },
      { value: "condition_support", description: "Helps clear or mitigate harmful conditions." },
      { value: "mental_recovery", description: "Helps stabilize emotions or recover from mental conditions." },
      { value: "escape_support", description: "Helps flee, slip away, or break free." },
      { value: "senses_support", description: "Improves vision or other senses." },
      { value: "energy_resistance", description: "Grants resistance against one or more energy types." },
      { value: "buff_support", description: "Provides a general beneficial enhancement or bonus." },
    ],
  },
  {
    category: "equipment",
    subcategories: ["consumable"],
    family: "polarity",
    description: "Offense/support polarity and delivery-style consumable tags.",
    tags: [
      { value: "offensive", description: "Hostile consumable primarily meant to harm or debilitate a target." },
      { value: "self_buff", description: "Support consumable primarily applied to the user." },
      { value: "ally_support", description: "Support consumable that can directly benefit another creature." },
      { value: "weapon_applied", description: "Offensive consumable applied to a weapon before use." },
      { value: "thrown_offense", description: "Offensive consumable delivered by throwing it." },
      { value: "ingested_offense", description: "Offensive consumable delivered when swallowed or consumed." },
      { value: "contact_offense", description: "Offensive consumable delivered through touch or skin contact." },
    ],
  },
  {
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    family: "purpose",
    description: "Utility and logistics gear-purpose tags.",
    tags: [
      { value: "climbing", description: "Helps climb, rappel, or navigate vertical obstacles." },
      { value: "lock_bypass", description: "Helps open locks or bypass secured entry points." },
      { value: "concealable", description: "Easy to hide on the person or carry discreetly." },
      { value: "scouting", description: "Helps observe, survey, or reconnoiter an area." },
      { value: "mobility", description: "Improves movement or traversal flexibility." },
      { value: "stealth_support", description: "Helps move quietly or avoid notice." },
      { value: "illumination", description: "Produces or improves light in dark environments." },
      { value: "survival", description: "Supports wilderness travel, shelter, or long-term field use." },
      { value: "navigation", description: "Helps track direction, route, or position." },
      { value: "tracking", description: "Helps follow trails, mark a target, or relocate something later." },
      { value: "anti_tracking", description: "Helps hide your trail, mask scent, or make pursuit harder." },
      { value: "transport", description: "Helps move creatures or cargo from place to place." },
      { value: "trap_bypass", description: "Helps disarm, disable, or get past traps." },
      { value: "carry_support", description: "Helps stow, carry, or organize equipment." },
      { value: "restraint_escape", description: "Helps break free from grabs, restraints, or similar immobilizing holds." },
      { value: "restraint_capture", description: "Helps capture, bind, or keep a target restrained." },
    ],
  },
  {
    category: "equipment",
    subcategories: DISGUISE_SUBCATEGORIES,
    family: "infiltration",
    description: "Appearance-changing and social-passing equipment across gear and consumables.",
    tags: [
      { value: "disguise", description: "Helps alter appearance or impersonate another identity." },
      { value: "social_infiltration", description: "Helps blend into a group or pass under social scrutiny." },
    ],
  },
  {
    category: "spell",
    family: "infiltration",
    description: "Appearance-changing and social-passing spells.",
    tags: [
      { value: "disguise", description: "Helps alter appearance or impersonate another identity." },
      { value: "social_infiltration", description: "Helps blend into a group or pass under social scrutiny." },
    ],
  },
  {
    category: "hazard",
    family: "function",
    description: "Hazard practical-function tags for alerts and restraint effects.",
    tags: [
      { value: "alarm", description: "Alerts guardians, onlookers, or nearby creatures to an intrusion." },
      { value: "restraint_capture", description: "Hazard that binds, restrains, or holds intruders in place." },
    ],
  },
  {
    category: "affliction",
    family: "impact",
    description: "Affliction impact tags for practical downstream consequences.",
    tags: [
      { value: "mental_impairment", description: "Impairs judgment, emotions, or perception through confusion, fear, or delirium." },
      { value: "mobility_impairment", description: "Reduces speed, stiffens movement, or leaves the victim immobilized." },
    ],
  },
  {
    category: "creature",
    family: "context",
    description: "Creature environment and scene-context tags.",
    tags: [
      { value: "nautical", description: "Strongly associated with ships, sailors, wrecks, or harbors." },
      { value: "aquatic_context", description: "Strongly associated with water or aquatic environments." },
      { value: "forest", description: "Strongly associated with forests, groves, or briar-choked wilds." },
      { value: "swamp", description: "Strongly associated with bogs, marshes, or mires." },
      { value: "underground", description: "Strongly associated with caves, tunnels, crypts, or subterranean spaces." },
      { value: "urban", description: "Strongly associated with cities, streets, alleys, or sewers." },
      { value: "arctic", description: "Strongly associated with snow, ice, tundra, or frozen coasts." },
      { value: "desert", description: "Strongly associated with dunes, sand, or arid wastes." },
      { value: "mountain", description: "Strongly associated with cliffs, peaks, or rocky heights." },
      { value: "graveyard", description: "Strongly associated with cemeteries, tombs, or burial grounds." },
      { value: "ruins", description: "Strongly associated with ancient ruins or derelict structures." },
    ],
  },
  {
    category: "creature",
    family: "scene_fit",
    description: "Creature practical-fit tags for distinguishing scene-adjacent NPCs from primary threats.",
    tags: [
      { value: "profession_npc", description: "Role-defined NPC such as a captain, guard, merchant, or commoner." },
      { value: "scene_adjacent", description: "Fits the scene or social fabric, but is usually not the primary monster answer." },
      { value: "undead_threat", description: "Threat signal derived from undead-like native traits." },
      { value: "fey_threat", description: "Threat signal derived from fey native traits." },
      { value: "plant_threat", description: "Threat signal derived from plant-like native traits." },
    ],
  },
];

function buildTextView(value: string): NormalizedTextView {
  const text = normalizeText(value);
  return {
    text,
    tokenSet: new Set(text.length > 0 ? text.split(" ") : []),
  };
}

function normalizeAnchor(anchor: TextAnchor): { value: string; mode: TextMatchMode; scope: TextMatchScope } | null {
  const raw = typeof anchor === "string" ? { value: anchor } : anchor;
  const value = normalizeText(raw.value);
  if (!value) {
    return null;
  }

  return {
    value,
    mode: raw.mode ?? (value.includes(" ") ? "phrase" : "token"),
    scope: raw.scope ?? "either",
  };
}

function containsPhrase(text: string, phrase: string): boolean {
  return ` ${text} `.includes(` ${phrase} `);
}

function matchesTextAnchor(context: NormalizedDerivedTagContext, anchor: TextAnchor): boolean {
  const normalized = normalizeAnchor(anchor);
  if (!normalized) {
    return false;
  }

  const views = normalized.scope === "name"
    ? [context.name]
    : normalized.scope === "description"
      ? [context.description]
      : [context.name, context.description];

  return views.some((view) => {
    if (normalized.mode === "token") {
      return view.tokenSet.has(normalized.value);
    }

    return containsPhrase(view.text, normalized.value);
  });
}

function matchesClause(context: NormalizedDerivedTagContext, clause: DerivedTagMatchClause): boolean {
  if (clause.traitsAny && !clause.traitsAny.some((trait) => context.traits.has(normalizeText(trait)))) {
    return false;
  }
  if (clause.traitsAll && !clause.traitsAll.every((trait) => context.traits.has(normalizeText(trait)))) {
    return false;
  }
  if (clause.familiesAny && !clause.familiesAny.some((family) => context.families.has(normalizeText(family)))) {
    return false;
  }
  if (clause.familiesAll && !clause.familiesAll.every((family) => context.families.has(normalizeText(family)))) {
    return false;
  }
  if (clause.textAny && !clause.textAny.some((anchor) => matchesTextAnchor(context, anchor))) {
    return false;
  }
  if (clause.textAll && !clause.textAll.every((anchor) => matchesTextAnchor(context, anchor))) {
    return false;
  }
  if (clause.referencesAny && !clause.referencesAny.some((reference) => context.referenceKeys.has(normalizeDerivedTagReference(reference)))) {
    return false;
  }
  if (clause.referencesAll && !clause.referencesAll.every((reference) => context.referenceKeys.has(normalizeDerivedTagReference(reference)))) {
    return false;
  }
  return true;
}

function scoreClause(context: NormalizedDerivedTagContext, clause: DerivedTagMatchClause): number {
  if (!matchesClause(context, clause)) {
    return 0;
  }

  return clause.score ?? 1;
}

function matchesRule(
  context: NormalizedDerivedTagContext,
  tags: Set<string>,
  rule: DerivedTagRule,
): boolean {
  if (context.category !== rule.category) {
    return false;
  }
  if (rule.subcategories && (!context.subcategory || !rule.subcategories.includes(context.subcategory))) {
    return false;
  }
  if (rule.requiresTags && !rule.requiresTags.every((tag) => tags.has(tag))) {
    return false;
  }
  if (rule.allOf && !rule.allOf.every((clause) => matchesClause(context, clause))) {
    return false;
  }
  if (rule.noneOf && rule.noneOf.some((clause) => matchesClause(context, clause))) {
    return false;
  }

  if (rule.anyOf) {
    const totalScore = rule.anyOf.reduce((sum, clause) => sum + scoreClause(context, clause), 0);
    const threshold = rule.threshold ?? 1;
    if (totalScore < threshold) {
      return false;
    }
  }

  return true;
}

export function normalizeDerivedTag(value: string): string {
  return normalizeText(value).replace(/\s+/g, "_");
}

export function deriveRecordTags(input: DerivedTagContext): string[] {
  const context: NormalizedDerivedTagContext = {
    category: input.category,
    subcategory: input.subcategory,
    traits: new Set(input.traits.map((trait) => normalizeText(trait)).filter(Boolean)),
    families: new Set((input.families ?? []).map((family) => normalizeText(family)).filter(Boolean)),
    name: buildTextView(input.name),
    description: buildTextView(input.descriptionText ?? ""),
    referenceKeys: new Set((input.references ?? []).map((reference) => normalizeDerivedTagReference(`${reference.packName}:${reference.name}`))),
  };
  const tags = new Set<string>();

  for (const rule of DERIVED_TAG_RULES) {
    if (matchesRule(context, tags, rule)) {
      tags.add(rule.tag);
    }
  }

  return uniqueSorted([...tags]);
}
