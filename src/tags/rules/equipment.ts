import {
  ALARM_NAME_ANCHORS,
  ALARM_STRONG_TEXT_ANCHORS,
  ALARM_TRIGGER_TEXT_ANCHORS,
  COUNTERMAGIC_NAME_ANCHORS,
  COUNTERMAGIC_REFERENCE_ANCHORS,
  COUNTERMAGIC_TEXT_ANCHORS,
  DISGUISE_REFERENCE_ANCHORS,
  DISGUISE_SUBCATEGORIES,
  DISGUISE_TEXT_ANCHORS,
  DerivedTagRule,
  GEARISH_SUBCATEGORIES,
  MAGIC_PROTECTION_TEXT_ANCHORS,
  MESSAGE_DELIVERY_BLOCKER_TEXT_ANCHORS,
  MESSAGE_DELIVERY_EQUIPMENT_NAME_ANCHORS,
  MESSAGE_DELIVERY_TEXT_ANCHORS,
  MOBILITY_REFERENCE_ANCHORS,
  NAVIGATION_REFERENCE_ANCHORS,
  OFFENSIVE_TEXT_ANCHORS,
  RESTRAINT_CAPTURE_REFERENCE_ANCHORS,
  RESTRAINT_ESCAPE_REFERENCE_ANCHORS,
  SIGNALING_NAME_ANCHORS,
  SIGNALING_TEXT_ANCHORS,
  SOCIAL_INFILTRATION_REFERENCE_ANCHORS,
  SOCIAL_INFILTRATION_TEXT_ANCHORS,
  STRONG_RESTRAINT_CAPTURE_NAME_ANCHORS,
  TRACKING_SUBCATEGORIES,
  WEAK_RESTRAINT_CAPTURE_NAME_ANCHORS,
  phraseAnchor,
  referenceAnchor,
  tokenAnchor,
} from "../shared.js";

export const EQUIPMENT_DERIVED_TAG_RULES: DerivedTagRule[] = [
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
          phraseAnchor("track teleportation"),
          phraseAnchor("dc to track"),
          phraseAnchor("attempting to track you"),
          phraseAnchor("circumstance bonus to their check"),
        ],
      },
      {
        textNear: [
          {
            terms: [tokenAnchor("tracking"), tokenAnchor("progress")],
            window: 3,
            scope: "description",
          },
          {
            terms: [tokenAnchor("track"), tokenAnchor("teleportation")],
            window: 2,
            scope: "description",
          },
          {
            terms: [tokenAnchor("track"), tokenAnchor("time")],
            window: 2,
            scope: "description",
          },
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
    tag: "signaling",
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    threshold: 2,
    anyOf: [
      { score: 1, textAny: SIGNALING_NAME_ANCHORS },
      { score: 2, textAny: SIGNALING_TEXT_ANCHORS },
    ],
  },
  {
    tag: "message_delivery",
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: MESSAGE_DELIVERY_EQUIPMENT_NAME_ANCHORS },
      { score: 2, textAny: MESSAGE_DELIVERY_TEXT_ANCHORS },
    ],
    noneOf: [
      { textAny: MESSAGE_DELIVERY_BLOCKER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "alarm",
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    threshold: 2,
    anyOf: [
      { score: 1, textAny: ALARM_NAME_ANCHORS },
      { score: 2, textAny: ALARM_STRONG_TEXT_ANCHORS },
      { score: 1, textAny: ALARM_TRIGGER_TEXT_ANCHORS },
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
    tag: "countermagic",
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: COUNTERMAGIC_NAME_ANCHORS },
      { score: 2, textAny: COUNTERMAGIC_TEXT_ANCHORS },
      { score: 2, referencesAny: COUNTERMAGIC_REFERENCE_ANCHORS },
      {
        score: 2,
        textAll: [
          tokenAnchor("counteract"),
        ],
        textAny: [
          phraseAnchor("magic effect"),
          phraseAnchor("magical effect"),
          phraseAnchor("magical effects"),
          phraseAnchor("magic item"),
          phraseAnchor("magical darkness"),
          phraseAnchor("triggering spell"),
          phraseAnchor("single spell"),
          phraseAnchor("target spell"),
        ],
      },
    ],
  },
  {
    tag: "magic_protection",
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: [tokenAnchor("antimagic", "name")] },
      { score: 2, textAny: MAGIC_PROTECTION_TEXT_ANCHORS },
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
          phraseAnchor("bonus to athletics checks"),
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
          phraseAnchor("escape dc"),
          phraseAnchor("can continue to grapple to keep your hold on the target"),
          phraseAnchor("gains a bonus to escape"),
          phraseAnchor("escape the net"),
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
      {
        score: 2,
        textAny: STRONG_RESTRAINT_CAPTURE_NAME_ANCHORS,
      },
      {
        score: 2,
        textAny: WEAK_RESTRAINT_CAPTURE_NAME_ANCHORS,
        referencesAny: RESTRAINT_CAPTURE_REFERENCE_ANCHORS,
      },
      {
        score: 2,
        textAny: [
          phraseAnchor("functions as a typical net"),
          phraseAnchor("grapple with the net"),
        ],
        referencesAny: [referenceAnchor("actionspf2e", "Grapple")],
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
];
