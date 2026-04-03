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
  patternAnchor,
  referenceAnchor,
} from "../shared.js";
import type { SearchSubcategory } from "../../types.js";

const AMMO_SUBCATEGORIES: SearchSubcategory[] = ["ammo"];
const ARMOR_SUBCATEGORIES: SearchSubcategory[] = ["armor"];

const AMMO_ILLUMINATION_TEXT_ANCHORS = [
  patternAnchor("shining", "name"),
  patternAnchor("beacon", "name"),
  patternAnchor("gives off a faint glow"),
  patternAnchor("shed bright light"),
  patternAnchor("shed light"),
  patternAnchor("spews sparks"),
];

const AMMO_SIGNALING_TEXT_ANCHORS = [
  patternAnchor("beacon", "name"),
  patternAnchor("spews sparks"),
  patternAnchor("mark a location"),
  patternAnchor("draw attention"),
];

const AMMO_MOBILITY_TEXT_ANCHORS = [
  patternAnchor("dimension", "name"),
  patternAnchor("starshot", "name"),
  patternAnchor("transposition", "name"),
  patternAnchor("burrowing", "name"),
  patternAnchor("climbing", "name"),
  patternAnchor("teleport to a location near where the ammunition hits"),
  patternAnchor("teleport to an unoccupied space adjacent"),
  patternAnchor("enlarges into a 50-foot-long rope"),
  patternAnchor("twine unwinds"),
  patternAnchor("burrows through up to"),
];

const AMMO_RESTRAINT_CAPTURE_TEXT_ANCHORS = [
  patternAnchor("antler", "name"),
  patternAnchor("bola", "name"),
  patternAnchor("garrote", "name"),
  patternAnchor("tendril", "name"),
  patternAnchor("pin it down"),
  patternAnchor("become stuck to the surface"),
  patternAnchor("wraps around one of the target's appendages"),
  patternAnchor("wraps around the target's throat"),
  patternAnchor("encase the target"),
  patternAnchor("until the target Escapes"),
  patternAnchor("restrain their foes"),
];

const AMMO_CREATURE_BANE_TEXT_ANCHORS = [
  patternAnchor("bane", "name"),
  patternAnchor("tailored to a particular type of creature"),
  patternAnchor("monster hunters favor bane ammunition"),
  patternAnchor("trait matching the selected type"),
];

const AMMO_ELEMENTAL_PAYLOAD_TEXT_ANCHORS = [
  patternAnchor("elemental ammunition"),
  patternAnchor("reservoir of alchemical reagents"),
  patternAnchor("gains a trait matching the damage type"),
  patternAnchor("each damage type requires a different formula"),
];

const AMMO_EXPLOSIVE_PAYLOAD_TEXT_ANCHORS = [
  patternAnchor("explosive", "name"),
  patternAnchor("meteor", "name"),
  patternAnchor("missile explodes"),
  patternAnchor("explodes into a small swarm of meteors"),
  patternAnchor("explodes in a burst"),
];

const ARMOR_MOBILITY_TEXT_ANCHORS = [
  patternAnchor("fly speed"),
  patternAnchor("fly speeds"),
  patternAnchor("gain a fly speed"),
  patternAnchor("gain a climb speed"),
  patternAnchor("gain a burrow speed"),
  patternAnchor("gain a swim speed"),
  patternAnchor("glide safely"),
  patternAnchor("glide to earth"),
  patternAnchor("hover above the ground"),
  patternAnchor("move more freely while underwater"),
  patternAnchor("swim speed"),
  patternAnchor("swim speeds"),
  patternAnchor("burrow"),
  patternAnchor("climb"),
  patternAnchor("float"),
  patternAnchor("glide"),
  patternAnchor("hover"),
  patternAnchor("fly"),
];

const ARMOR_CARRY_SUPPORT_TEXT_ANCHORS = [
  patternAnchor("carry 2 more bulk"),
  patternAnchor("carry more bulk"),
  patternAnchor("find tool"),
  patternAnchor("spacious pouches"),
  patternAnchor("store items"),
  patternAnchor("tool storage"),
  patternAnchor("up to 4 more bulk"),
  patternAnchor("bulk"),
  patternAnchor("carry"),
  patternAnchor("encumbered"),
  patternAnchor("pocket"),
  patternAnchor("pockets"),
  patternAnchor("pouch"),
  patternAnchor("pouches"),
  patternAnchor("storage"),
  patternAnchor("stow"),
];

const ARMOR_STEALTH_SUPPORT_TEXT_ANCHORS = [
  patternAnchor("avoid notice"),
  patternAnchor("hidden on your person"),
  patternAnchor("loses the noisy trait"),
  patternAnchor("noisy trait"),
  patternAnchor("surreptitiously"),
  patternAnchor("turn partially invisible"),
  patternAnchor("wear it surreptitiously"),
  patternAnchor("exceptionally quiet"),
  patternAnchor("hidden"),
  patternAnchor("invisible"),
  patternAnchor("quiet"),
];

const ARMOR_SURVIVAL_TEXT_ANCHORS = [
  patternAnchor("air supply"),
  patternAnchor("allows you to breathe underwater"),
  patternAnchor("breathe underwater"),
  patternAnchor("breathable air"),
  patternAnchor("can breathe underwater"),
  patternAnchor("drowning and suffocation"),
  patternAnchor("filter exterior air"),
  patternAnchor("protects you from drowning"),
  patternAnchor("protected from extreme cold"),
  patternAnchor("protected from severe heat"),
  patternAnchor("protected from extreme heat"),
  patternAnchor("sealed environment"),
  patternAnchor("sealed suit"),
  patternAnchor("waterproof fabric"),
  patternAnchor("weatherproof"),
  patternAnchor("move more freely while underwater"),
  patternAnchor("swim speed"),
  patternAnchor("waterproof"),
  patternAnchor("weatherproof"),
  patternAnchor("sealed"),
  patternAnchor("underwater"),
];

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
      { textAny: [patternAnchor("thrown"), patternAnchor("hurl"), patternAnchor("lob"), patternAnchor("splash weapon"), patternAnchor("throw the bomb")] },
    ],
  },
  {
    tag: "weapon_applied",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["offensive"],
    anyOf: [
      { textAny: [patternAnchor("apply to a weapon"), patternAnchor("coat a weapon"), patternAnchor("weapon poison"), patternAnchor("smeared on a weapon"), patternAnchor("applied to a weapon")] },
    ],
  },
  {
    tag: "ingested_offense",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["offensive"],
    anyOf: [
      { textAny: [patternAnchor("ingested poison"), patternAnchor("must be eaten"), patternAnchor("must be drunk"), patternAnchor("consumed by the target"), patternAnchor("when swallowed")] },
    ],
  },
  {
    tag: "contact_offense",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["offensive"],
    anyOf: [
      { textAny: [patternAnchor("contact poison"), patternAnchor("through skin contact"), patternAnchor("through contact"), patternAnchor("absorbed through the skin")] },
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
          patternAnchor("status bonus to"),
          patternAnchor("item bonus to"),
          patternAnchor("grants you"),
          patternAnchor("giving you"),
          patternAnchor("no longer need to breathe"),
          patternAnchor("do not need air"),
          patternAnchor("imprecise scent"),
          patternAnchor("gain a burrow speed"),
          patternAnchor("gain a climb speed"),
        ],
      },
      {
        textAny: [
          patternAnchor("trigger you attempt to"),
          patternAnchor("trigger you become"),
          patternAnchor("requirements you re"),
          patternAnchor("in case of kidnapping"),
          patternAnchor("slippery enough to"),
        ],
        referencesAny: RESTRAINT_ESCAPE_REFERENCE_ANCHORS,
      },
      {
        textAny: [
          patternAnchor("restorative"),
          patternAnchor("remedy"),
          patternAnchor("curative"),
          patternAnchor("antidote"),
          patternAnchor("antiplague"),
          patternAnchor("protective"),
          patternAnchor("catharsis"),
          patternAnchor("healing"),
          patternAnchor("darkvision"),
          patternAnchor("additional protection"),
          patternAnchor("absorb damage"),
          patternAnchor("temporary hit points"),
          patternAnchor("grants resistance"),
          patternAnchor("gain resistance"),
          patternAnchor("protect against poison"),
          patternAnchor("protect against poisons"),
          patternAnchor("protect against disease"),
          patternAnchor("protects you against"),
          patternAnchor("resistance to"),
          patternAnchor("gain a bonus"),
          patternAnchor("bolsters the drinker"),
          patternAnchor("steady the emotions"),
          patternAnchor("see in the dark"),
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
      { textAny: [patternAnchor("elixir of life"), patternAnchor("healing"), patternAnchor("restore hit points"), patternAnchor("restore hp"), patternAnchor("regain hit points")] },
    ],
  },
  {
    tag: "anti_poison",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [patternAnchor("antidote"), patternAnchor("against poison"), patternAnchor("against poisons"), patternAnchor("protect against poison"), patternAnchor("protects you against poisons"), patternAnchor("resist poison"), patternAnchor("ward off poison"), patternAnchor("persistent poison damage")] },
    ],
  },
  {
    tag: "anti_disease",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [patternAnchor("antiplague"), patternAnchor("against disease"), patternAnchor("against diseases"), patternAnchor("protect against disease"), patternAnchor("resist disease"), patternAnchor("ward off disease")] },
    ],
  },
  {
    tag: "condition_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [patternAnchor("condition"), patternAnchor("catharsis"), patternAnchor("soothe the mind"), patternAnchor("steady the emotions"), patternAnchor("calm overwhelming emotions"), patternAnchor("recover from mental conditions")] },
    ],
  },
  {
    tag: "mental_recovery",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [patternAnchor("soothe the mind"), patternAnchor("steady the emotions"), patternAnchor("calm overwhelming emotions"), patternAnchor("mental condition"), patternAnchor("mental conditions"), patternAnchor("emotion"), patternAnchor("emotions"), patternAnchor("frightened"), patternAnchor("stupefied"), patternAnchor("confused"), patternAnchor("mental effect")] },
    ],
  },
  {
    tag: "escape_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [patternAnchor("escape"), patternAnchor("slip away"), patternAnchor("break free"), patternAnchor("flee"), patternAnchor("evade"), patternAnchor("concealing smoke"), patternAnchor("vanish from sight"), patternAnchor("misty")] },
      { referencesAny: [referenceAnchor("actionspf2e", "Escape")] },
      { textAny: [patternAnchor("grabbed"), patternAnchor("restrained"), patternAnchor("immobilized"), patternAnchor("slip free"), patternAnchor("break the grab")] },
    ],
  },
  {
    tag: "senses_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [patternAnchor("darkvision"), patternAnchor("see in the dark"), patternAnchor("low light vision"), patternAnchor("keen senses"), patternAnchor("sharpen your vision"), patternAnchor("see invisible"), patternAnchor("scent")] },
    ],
  },
  {
    tag: "energy_resistance",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [patternAnchor("resistance to fire"), patternAnchor("resistance to cold"), patternAnchor("resistance to electricity"), patternAnchor("resistance to acid"), patternAnchor("resistance to sonic"), patternAnchor("resistance to energy"), patternAnchor("energy resistance"), patternAnchor("against fire damage"), patternAnchor("against cold damage"), patternAnchor("against electricity damage"), patternAnchor("against acid damage"), patternAnchor("against sonic damage")] },
    ],
  },
  {
    tag: "buff_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [patternAnchor("gain a bonus"), patternAnchor("bonus to"), patternAnchor("bolster"), patternAnchor("enhance"), patternAnchor("empower"), patternAnchor("heighten your senses"), patternAnchor("increase your speed"), patternAnchor("resistance to"), patternAnchor("grants resistance"), patternAnchor("temporary hit points"), patternAnchor("additional protection"), patternAnchor("absorb damage")] },
    ],
  },
  {
    tag: "self_buff",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [patternAnchor("you gain"), patternAnchor("the drinker gains"), patternAnchor("gain a bonus"), patternAnchor("you become"), patternAnchor("you gain resistance"), patternAnchor("you gain darkvision"), patternAnchor("protects you against"), patternAnchor("when you drink"), patternAnchor("drinking this"), patternAnchor("spreading the salve on exposed skin"), patternAnchor("grants you"), patternAnchor("giving you")] },
    ],
  },
  {
    tag: "ally_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [patternAnchor("target gains"), patternAnchor("an ally gains"), patternAnchor("creature that drinks gains"), patternAnchor("the drinker gains")] },
    ],
  },
  {
    tag: "climbing",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [patternAnchor("climb"), patternAnchor("climbing"), patternAnchor("rappel"), patternAnchor("rappelling"), patternAnchor("piton"), patternAnchor("grappling")] },
    ],
  },
  {
    tag: "mobility",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    threshold: 2,
    anyOf: [
      { score: 2, textAny: [patternAnchor("mobility"), patternAnchor("move quickly"), patternAnchor("increase your speed"), patternAnchor("rappel"), patternAnchor("climbing")] },
      {
        score: 2,
        textAny: [
          patternAnchor("high jump"),
          patternAnchor("long jump"),
          patternAnchor("land speed"),
          patternAnchor("climb speed"),
          patternAnchor("climb speeds"),
          patternAnchor("swim speed"),
          patternAnchor("swim speeds"),
          patternAnchor("ignore difficult terrain"),
          patternAnchor("difficult terrain"),
          patternAnchor("improved grip"),
          patternAnchor("exceptional traction"),
          patternAnchor("traction"),
        ],
      },
      { score: 2, referencesAny: MOBILITY_REFERENCE_ANCHORS },
    ],
  },
  {
    tag: "mobility",
    category: "equipment",
    subcategories: [...AMMO_SUBCATEGORIES],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AMMO_MOBILITY_TEXT_ANCHORS },
    ],
  },
  {
    tag: "mobility",
    category: "equipment",
    subcategories: ARMOR_SUBCATEGORIES,
    threshold: 2,
    anyOf: [
      { score: 2, textAny: ARMOR_MOBILITY_TEXT_ANCHORS },
    ],
  },
  {
    tag: "lock_bypass",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [patternAnchor("lockpick"), patternAnchor("lockpicks"), patternAnchor("pick locks"), patternAnchor("picking locks"), patternAnchor("bypass locks"), patternAnchor("thieves tools"), patternAnchor("thieves tools"), patternAnchor("toolkit")] },
    ],
  },
  {
    tag: "concealable",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [patternAnchor("concealable"), patternAnchor("hidden on your person"), patternAnchor("hidden tools"), patternAnchor("slim lockpicks")] },
    ],
  },
  {
    tag: "scouting",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [patternAnchor("scout"), patternAnchor("scouting"), patternAnchor("survey"), patternAnchor("recon"), patternAnchor("observe from afar"), patternAnchor("spyglass")] },
    ],
  },
  {
    tag: "stealth_support",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [patternAnchor("stealth"), patternAnchor("quiet"), patternAnchor("silent"), patternAnchor("without drawing attention"), patternAnchor("avoid notice"), patternAnchor("infiltration")] },
      { textAny: [patternAnchor("concealable"), patternAnchor("hidden on your person")] },
    ],
  },
  {
    tag: "stealth_support",
    category: "equipment",
    subcategories: ARMOR_SUBCATEGORIES,
    threshold: 2,
    anyOf: [
      { score: 2, textAny: ARMOR_STEALTH_SUPPORT_TEXT_ANCHORS },
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
      { textAny: [patternAnchor("light"), patternAnchor("illumination"), patternAnchor("lantern"), patternAnchor("torch"), patternAnchor("glow"), patternAnchor("illuminate")] },
    ],
  },
  {
    tag: "illumination",
    category: "equipment",
    subcategories: [...AMMO_SUBCATEGORIES],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AMMO_ILLUMINATION_TEXT_ANCHORS },
    ],
  },
  {
    tag: "survival",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [patternAnchor("camp"), patternAnchor("forage"), patternAnchor("wilderness"), patternAnchor("survival"), patternAnchor("shelter"), patternAnchor("weatherproof")] },
    ],
  },
  {
    tag: "survival",
    category: "equipment",
    subcategories: ARMOR_SUBCATEGORIES,
    threshold: 2,
    anyOf: [
      { score: 2, textAny: ARMOR_SURVIVAL_TEXT_ANCHORS },
    ],
  },
  {
    tag: "navigation",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      {
        textAny: [
          patternAnchor("navigate"),
          patternAnchor("navigation"),
          patternAnchor("map"),
          patternAnchor("compass"),
          patternAnchor("chart"),
          patternAnchor("sense direction"),
          patternAnchor("track your heading"),
          patternAnchor("which direction is north"),
          patternAnchor("learn which direction you re facing"),
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
          patternAnchor("sense direction"),
          patternAnchor("functions as if you have a compass"),
          patternAnchor("learn which direction you re facing"),
          patternAnchor("which direction is north"),
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
          patternAnchor("sense and follow tracks"),
          patternAnchor("for later tracking"),
          patternAnchor("track their movements"),
          patternAnchor("track a creature by its scent"),
          patternAnchor("continue to track the same creature"),
          patternAnchor("flutters toward the affixed one"),
        ],
      },
      {
        score: 2,
        textAll: [
          patternAnchor("survival"),
          patternAnchor("sense direction"),
          patternAnchor("track"),
        ],
      },
      {
        score: 2,
        textAll: [
          patternAnchor("survival"),
          patternAnchor("scent"),
        ],
        referencesAny: [referenceAnchor("actionspf2e", "Track")],
      },
      {
        score: 2,
        textAll: [
          patternAnchor("survival"),
          patternAnchor("scent"),
          patternAnchor("track"),
        ],
      },
      { score: 1, textAny: [patternAnchor("tracker", "name"), patternAnchor("tracking", "name")] },
    ],
    noneOf: [
      {
        textAny: [
          patternAnchor("track time"),
          patternAnchor("track teleportation"),
          patternAnchor("dc to track"),
          patternAnchor("attempting to track you"),
          patternAnchor("circumstance bonus to their check"),
        ],
      },
      {
        textNear: [
          {
            terms: [patternAnchor("tracking"), patternAnchor("progress")],
            window: 3,
            scope: "description",
          },
          {
            terms: [patternAnchor("track"), patternAnchor("teleportation")],
            window: 2,
            scope: "description",
          },
          {
            terms: [patternAnchor("track"), patternAnchor("time")],
            window: 2,
            scope: "description",
          },
        ],
      },
      {
        textAny: [patternAnchor("trackless", "name")],
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
          patternAnchor("dc to track"),
          patternAnchor("attempting to track you"),
          patternAnchor("cover any ordinary odors"),
          patternAnchor("fleeing pursuit"),
        ],
      },
      {
        score: 2,
        textAll: [
          patternAnchor("stealth"),
          patternAnchor("hide"),
          patternAnchor("sneak"),
        ],
        textAny: [
          patternAnchor("against creatures using primarily smell"),
        ],
      },
      { score: 2, textAny: [patternAnchor("trackless", "name")] },
    ],
    noneOf: [
      {
        textAny: [
          patternAnchor("track a creature by its scent"),
          patternAnchor("track their movements"),
          patternAnchor("continue to track the same creature"),
          patternAnchor("sense and follow tracks"),
          patternAnchor("circumstance bonus to their check"),
        ],
      },
    ],
  },
  {
    tag: "transport",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [patternAnchor("transport"), patternAnchor("wagon"), patternAnchor("sled"), patternAnchor("boat"), patternAnchor("vehicle"), patternAnchor("carry riders"), patternAnchor("haul passengers")] },
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
    tag: "signaling",
    category: "equipment",
    subcategories: [...AMMO_SUBCATEGORIES],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AMMO_SIGNALING_TEXT_ANCHORS },
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
      { textAny: [patternAnchor("disarm a trap"), patternAnchor("disable device"), patternAnchor("trap mechanism"), patternAnchor("tripwire"), patternAnchor("bypass a trap")] },
    ],
  },
  {
    tag: "carry_support",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [patternAnchor("storage"), patternAnchor("stow"), patternAnchor("carry"), patternAnchor("haul"), patternAnchor("pouch"), patternAnchor("backpack"), patternAnchor("container"), patternAnchor("pack")] },
    ],
  },
  {
    tag: "carry_support",
    category: "equipment",
    subcategories: ARMOR_SUBCATEGORIES,
    threshold: 2,
    anyOf: [
      { score: 2, textAny: ARMOR_CARRY_SUPPORT_TEXT_ANCHORS },
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
          patternAnchor("counteract"),
        ],
        textAny: [
          patternAnchor("magic effect"),
          patternAnchor("magical effect"),
          patternAnchor("magical effects"),
          patternAnchor("magic item"),
          patternAnchor("magical darkness"),
          patternAnchor("triggering spell"),
          patternAnchor("single spell"),
          patternAnchor("target spell"),
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
      { score: 2, textAny: [patternAnchor("antimagic", "name")] },
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
          patternAnchor("break free"),
          patternAnchor("free of a creature grabbing you"),
          patternAnchor("difficult to hold back"),
          patternAnchor("whenever you are affected by an effect that lasts until you"),
          patternAnchor("trigger you become"),
        ],
        referencesAny: RESTRAINT_ESCAPE_REFERENCE_ANCHORS,
      },
      {
        score: 2,
        textAny: [
          patternAnchor("free someone from manacles"),
          patternAnchor("free yourself from manacles"),
        ],
      },
      {
        score: 2,
        textAll: [
          patternAnchor("bonus to athletics checks"),
        ],
        referencesAny: [referenceAnchor("actionspf2e", "Escape")],
      },
    ],
    noneOf: [
      {
        textAny: [
          patternAnchor("target becomes grabbed"),
          patternAnchor("target becomes restrained"),
          patternAnchor("creature becomes grabbed"),
          patternAnchor("creature becomes restrained"),
          patternAnchor("restrain the target"),
          patternAnchor("grabs the target"),
          patternAnchor("tighten the loop"),
          patternAnchor("escape dc"),
          patternAnchor("can continue to grapple to keep your hold on the target"),
          patternAnchor("gains a bonus to escape"),
          patternAnchor("escape the net"),
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
          patternAnchor("target becomes restrained"),
          patternAnchor("creature becomes restrained"),
          patternAnchor("restrain the target"),
        ],
      },
      {
        score: 2,
        textAny: [
          patternAnchor("tighten the loop"),
          patternAnchor("without having a free hand"),
          patternAnchor("up to 10 feet away"),
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
          patternAnchor("functions as a typical net"),
          patternAnchor("grapple with the net"),
        ],
        referencesAny: [referenceAnchor("actionspf2e", "Grapple")],
      },
    ],
    noneOf: [
      {
        textAny: [
          patternAnchor("break free"),
          patternAnchor("free someone from manacles"),
          patternAnchor("free yourself from manacles"),
        ],
      },
    ],
  },
  {
    tag: "restraint_capture",
    category: "equipment",
    subcategories: [...AMMO_SUBCATEGORIES],
    threshold: 1,
    anyOf: [
      { score: 1, textAny: AMMO_RESTRAINT_CAPTURE_TEXT_ANCHORS },
    ],
  },
  {
    tag: "creature_bane",
    category: "equipment",
    subcategories: [...AMMO_SUBCATEGORIES],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AMMO_CREATURE_BANE_TEXT_ANCHORS },
    ],
  },
  {
    tag: "elemental_payload",
    category: "equipment",
    subcategories: [...AMMO_SUBCATEGORIES],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AMMO_ELEMENTAL_PAYLOAD_TEXT_ANCHORS },
    ],
  },
  {
    tag: "explosive_payload",
    category: "equipment",
    subcategories: [...AMMO_SUBCATEGORIES],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AMMO_EXPLOSIVE_PAYLOAD_TEXT_ANCHORS },
    ],
  },
];
