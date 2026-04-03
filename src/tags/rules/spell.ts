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
