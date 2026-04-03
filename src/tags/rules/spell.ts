import {
  ALARM_NAME_ANCHORS,
  ALARM_STRONG_TEXT_ANCHORS,
  ALARM_TRIGGER_TEXT_ANCHORS,
  COUNTERMAGIC_NAME_ANCHORS,
  COUNTERMAGIC_REFERENCE_ANCHORS,
  COUNTERMAGIC_TEXT_ANCHORS,
  DerivedTagRule,
  MESSAGE_DELIVERY_BLOCKER_TEXT_ANCHORS,
  MESSAGE_DELIVERY_SPELL_NAME_ANCHORS,
  MESSAGE_DELIVERY_TEXT_ANCHORS,
  SIGNALING_NAME_ANCHORS,
  SIGNALING_TEXT_ANCHORS,
  SOCIAL_INFILTRATION_TEXT_ANCHORS,
  SPELL_DISGUISE_BLOCKER_TEXT_ANCHORS,
  SPELL_DISGUISE_NAME_ANCHORS,
  SPELL_DISGUISE_TEXT_ANCHORS,
  SPELL_MOBILITY_BLOCKER_TEXT_ANCHORS,
  SPELL_MOBILITY_TEXT_ANCHORS,
  SPELL_NAVIGATION_NAME_ANCHORS,
  SPELL_NAVIGATION_TEXT_ANCHORS,
  SPELL_SCOUTING_BLOCKER_TEXT_ANCHORS,
  SPELL_SCOUTING_NAME_ANCHORS,
  SPELL_SCOUTING_TEXT_ANCHORS,
  SPELL_SOCIAL_INFILTRATION_TEXT_ANCHORS,
  phraseAnchor,
  tokenAnchor,
} from "../shared.js";

const HEALING_SUPPORT_NAME_ANCHORS = [
  tokenAnchor("healing", "name"),
  tokenAnchor("heal", "name"),
  tokenAnchor("restoration", "name"),
  tokenAnchor("restore", "name"),
];

const HEALING_SUPPORT_TEXT_ANCHORS = [
  phraseAnchor("restore hit points"),
  phraseAnchor("regain hit points"),
  phraseAnchor("fast healing"),
  phraseAnchor("heals the target"),
  phraseAnchor("heal the target"),
  phraseAnchor("recover hit points"),
  phraseAnchor("restore the target"),
];

const CONDITION_SUPPORT_TEXT_ANCHORS = [
  phraseAnchor("remove a condition"),
  phraseAnchor("remove conditions"),
  phraseAnchor("counteract an affliction"),
  phraseAnchor("counteract the affliction"),
  phraseAnchor("delay the affliction"),
  phraseAnchor("delay an affliction"),
  phraseAnchor("cure disease"),
  phraseAnchor("cure poison"),
  phraseAnchor("remove curse"),
];

const PROTECTIVE_WARD_NAME_ANCHORS = [
  tokenAnchor("sanctuary", "name"),
  tokenAnchor("aegis", "name"),
  tokenAnchor("boundary", "name"),
  tokenAnchor("ward", "name"),
  tokenAnchor("shield", "name"),
  tokenAnchor("barrier", "name"),
  tokenAnchor("protection", "name"),
];

const PROTECTIVE_WARD_TEXT_ANCHORS = [
  phraseAnchor("circle of protection"),
  phraseAnchor("defended by spirits"),
  phraseAnchor("blessed boundary"),
  phraseAnchor("warding circle"),
  phraseAnchor("warding aura"),
  phraseAnchor("protective boundary"),
  phraseAnchor("protective ward"),
];

const DEATH_PREVENTION_NAME_ANCHORS = [
  phraseAnchor("breath of life"),
  phraseAnchor("death ward"),
  tokenAnchor("revival", "name"),
  tokenAnchor("resurrection", "name"),
];

const DEATH_PREVENTION_TEXT_ANCHORS = [
  phraseAnchor("prevent the target from dying"),
  phraseAnchor("prevent a creature from dying"),
  phraseAnchor("prevent it from dying"),
  phraseAnchor("stabilize the target"),
  phraseAnchor("stabilize a dying creature"),
  phraseAnchor("come back to life"),
  phraseAnchor("return to life"),
  phraseAnchor("bring the target back to life"),
];

const RESISTANCE_SUPPORT_TEXT_ANCHORS = [
  phraseAnchor("gain resistance"),
  phraseAnchor("gains resistance"),
  phraseAnchor("grants resistance"),
  phraseAnchor("resistance to"),
  phraseAnchor("immune to"),
  phraseAnchor("immunity to"),
  phraseAnchor("protects against fire"),
  phraseAnchor("protects against cold"),
  phraseAnchor("protects against acid"),
  phraseAnchor("protects against electricity"),
  phraseAnchor("protects against sonic"),
];

export const SPELL_DERIVED_TAG_RULES: DerivedTagRule[] = [
  {
    tag: "disguise",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: SPELL_DISGUISE_NAME_ANCHORS },
      { score: 2, textAny: SPELL_DISGUISE_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            terms: [tokenAnchor("detect"), tokenAnchor("undead")],
            window: 4,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [
      { textAny: SPELL_DISGUISE_BLOCKER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "social_infiltration",
    category: "spell",
    requiresTags: ["disguise"],
    anyOf: [
      { textAny: SOCIAL_INFILTRATION_TEXT_ANCHORS },
      { textAny: SPELL_DISGUISE_TEXT_ANCHORS },
    ],
    noneOf: [
      { textAny: SPELL_DISGUISE_BLOCKER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "social_infiltration",
    category: "spell",
    anyOf: [
      { textAny: SPELL_SOCIAL_INFILTRATION_TEXT_ANCHORS },
    ],
  },
  {
    tag: "social_infiltration",
    category: "spell",
    allOf: [
      { textAny: [tokenAnchor("mask", "name"), tokenAnchor("mask", "description")] },
    ],
    anyOf: [
      {
        textNear: [
          {
            terms: [tokenAnchor("deception"), tokenAnchor("lie"), tokenAnchor("feint")],
            window: 6,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
    ],
  },
  {
    tag: "alarm",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: ALARM_NAME_ANCHORS },
      { score: 2, textAny: ALARM_STRONG_TEXT_ANCHORS },
      { score: 1, textAny: ALARM_TRIGGER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "signaling",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: SIGNALING_NAME_ANCHORS },
      { score: 2, textAny: SIGNALING_TEXT_ANCHORS },
    ],
    noneOf: [
      { textAny: ALARM_STRONG_TEXT_ANCHORS },
    ],
  },
  {
    tag: "message_delivery",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: MESSAGE_DELIVERY_SPELL_NAME_ANCHORS },
      { score: 2, textAny: MESSAGE_DELIVERY_TEXT_ANCHORS },
    ],
    noneOf: [
      { textAny: MESSAGE_DELIVERY_BLOCKER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "scouting",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, traitsAny: ["scrying"] },
      { score: 2, textAny: SPELL_SCOUTING_NAME_ANCHORS },
      { score: 2, textAny: SPELL_SCOUTING_TEXT_ANCHORS },
      {
        score: 2,
        textAll: [
          tokenAnchor("senses"),
          tokenAnchor("through"),
        ],
        textAny: [
          phraseAnchor("through the ear"),
          phraseAnchor("through its eyes"),
          phraseAnchor("through each other s eyes"),
        ],
      },
    ],
    noneOf: [
      { textAny: SPELL_SCOUTING_BLOCKER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "navigation",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: SPELL_NAVIGATION_NAME_ANCHORS },
      { score: 2, textAny: SPELL_NAVIGATION_TEXT_ANCHORS },
    ],
  },
  {
    tag: "mobility",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: SPELL_MOBILITY_TEXT_ANCHORS },
    ],
    noneOf: [
      { textAny: SPELL_MOBILITY_BLOCKER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "healing_support",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: HEALING_SUPPORT_NAME_ANCHORS },
      { score: 2, textAny: HEALING_SUPPORT_TEXT_ANCHORS },
      { score: 1, traitsAny: ["healing"] },
    ],
  },
  {
    tag: "condition_support",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: CONDITION_SUPPORT_TEXT_ANCHORS },
    ],
  },
  {
    tag: "protective_ward",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: PROTECTIVE_WARD_NAME_ANCHORS },
      { score: 2, textAny: PROTECTIVE_WARD_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            terms: [
              tokenAnchor("protect"),
              tokenAnchor("shield"),
              tokenAnchor("ward"),
              tokenAnchor("barrier"),
              tokenAnchor("creature"),
              tokenAnchor("target"),
              tokenAnchor("ally"),
              tokenAnchor("allies"),
              tokenAnchor("area"),
              tokenAnchor("self"),
            ],
            window: 6,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
    ],
    noneOf: [
      { textAny: [...ALARM_STRONG_TEXT_ANCHORS, ...ALARM_TRIGGER_TEXT_ANCHORS] },
    ],
  },
  {
    tag: "death_prevention",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: DEATH_PREVENTION_NAME_ANCHORS },
      { score: 2, textAny: DEATH_PREVENTION_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            terms: [
              tokenAnchor("stabilize"),
              tokenAnchor("dying"),
              tokenAnchor("die"),
              tokenAnchor("death"),
            ],
            window: 6,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
      {
        score: 2,
        textNear: [
          {
            terms: [
              tokenAnchor("return"),
              tokenAnchor("life"),
              tokenAnchor("revive"),
              tokenAnchor("resurrect"),
              tokenAnchor("death"),
            ],
            window: 8,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
    ],
  },
  {
    tag: "resistance_support",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: RESISTANCE_SUPPORT_TEXT_ANCHORS },
      { score: 1, textAny: [tokenAnchor("resistance"), tokenAnchor("immune"), tokenAnchor("immunity")] },
    ],
  },
  {
    tag: "countermagic",
    category: "spell",
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
];
