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
  patternAnchor,
} from "../shared.js";

const HEALING_SUPPORT_NAME_ANCHORS = [
  patternAnchor("healing", "name"),
  patternAnchor("heal", "name"),
  patternAnchor("restoration", "name"),
  patternAnchor("restore", "name"),
];

const HEALING_SUPPORT_TEXT_ANCHORS = [
  patternAnchor("restore hit points"),
  patternAnchor("regain hit points"),
  patternAnchor("fast healing"),
  patternAnchor("heals the target"),
  patternAnchor("heal the target"),
  patternAnchor("recover hit points"),
  patternAnchor("restore the target"),
];

const CONDITION_SUPPORT_TEXT_ANCHORS = [
  patternAnchor("remove a condition"),
  patternAnchor("remove conditions"),
  patternAnchor("counteract an affliction"),
  patternAnchor("counteract the affliction"),
  patternAnchor("delay the affliction"),
  patternAnchor("delay an affliction"),
  patternAnchor("cure disease"),
  patternAnchor("cure poison"),
  patternAnchor("remove curse"),
];

const PROTECTIVE_WARD_NAME_ANCHORS = [
  patternAnchor("sanctuary", "name"),
  patternAnchor("aegis", "name"),
  patternAnchor("boundary", "name"),
  patternAnchor("ward", "name"),
  patternAnchor("shield", "name"),
  patternAnchor("barrier", "name"),
  patternAnchor("protection", "name"),
];

const PROTECTIVE_WARD_TEXT_ANCHORS = [
  patternAnchor("circle of protection"),
  patternAnchor("defended by spirits"),
  patternAnchor("blessed boundary"),
  patternAnchor("warding circle"),
  patternAnchor("warding aura"),
  patternAnchor("protective boundary"),
  patternAnchor("protective ward"),
];

const DEATH_PREVENTION_NAME_ANCHORS = [
  patternAnchor("breath of life"),
  patternAnchor("death ward"),
  patternAnchor("revival", "name"),
  patternAnchor("resurrection", "name"),
];

const DEATH_PREVENTION_TEXT_ANCHORS = [
  patternAnchor("prevent the target from dying"),
  patternAnchor("prevent a creature from dying"),
  patternAnchor("prevent it from dying"),
  patternAnchor("stabilize the target"),
  patternAnchor("stabilize a dying creature"),
  patternAnchor("come back to life"),
  patternAnchor("return to life"),
  patternAnchor("bring the target back to life"),
];

const RESISTANCE_SUPPORT_TEXT_ANCHORS = [
  patternAnchor("gain resistance"),
  patternAnchor("gains resistance"),
  patternAnchor("grants resistance"),
  patternAnchor("resistance to"),
  patternAnchor("immune to"),
  patternAnchor("immunity to"),
  patternAnchor("protects against fire"),
  patternAnchor("protects against cold"),
  patternAnchor("protects against acid"),
  patternAnchor("protects against electricity"),
  patternAnchor("protects against sonic"),
];

const TRANSFORMATION_NAME_ANCHORS = [
  patternAnchor("metamorphosis", "name"),
  patternAnchor("polymorph", "name"),
  patternAnchor("transformation", "name"),
  patternAnchor("avatar", "name"),
  patternAnchor("incarnate", "name"),
];

const TRANSFORMATION_TEXT_ANCHORS = [
  patternAnchor("assume the form"),
  patternAnchor("take the form"),
  patternAnchor("take on the form"),
  patternAnchor("change shape"),
  patternAnchor("change into"),
  patternAnchor("transform into"),
  patternAnchor("transform your appearance"),
  patternAnchor("reshape your body"),
  patternAnchor("reshape the target s body"),
  patternAnchor("take on a new form"),
  patternAnchor("body becomes"),
  patternAnchor("body is transformed"),
];

const TRANSFORMATION_SUMMON_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("summon"),
  patternAnchor("summons"),
  patternAnchor("summoned"),
  patternAnchor("conjure"),
  patternAnchor("conjures"),
  patternAnchor("call forth"),
  patternAnchor("bring forth"),
  patternAnchor("call into being"),
];

const TRANSFORMATION_DISGUISE_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("disguise"),
  patternAnchor("disguises"),
  patternAnchor("disguised"),
  patternAnchor("impersonate"),
  patternAnchor("impersonates"),
  patternAnchor("masquerade"),
  patternAnchor("masquerades"),
  patternAnchor("false identity"),
  patternAnchor("take on the appearance"),
  patternAnchor("change your appearance"),
  patternAnchor("appearance becomes bland and nondescript"),
  patternAnchor("pass as someone else"),
];

const TRANSFORMATION_SIZE_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("increase your size"),
  patternAnchor("decrease your size"),
  patternAnchor("increase the target s size"),
  patternAnchor("decrease the target s size"),
  patternAnchor("grow larger"),
  patternAnchor("grow smaller"),
  patternAnchor("become larger"),
  patternAnchor("become smaller"),
  patternAnchor("only changes your size"),
  patternAnchor("size changes"),
  patternAnchor("enlarge"),
  patternAnchor("shrink"),
  patternAnchor("reduce"),
];

const TRANSFORMATION_OBJECT_ANIMATION_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("animate object"),
  patternAnchor("animate objects"),
  patternAnchor("animated object"),
  patternAnchor("animated objects"),
  patternAnchor("turn them into animated objects"),
  patternAnchor("turn it into an animated object"),
  patternAnchor("objects are animated"),
  patternAnchor("objects come to life"),
  patternAnchor("animate the object"),
  patternAnchor("animate the objects"),
];

const TRANSFORMATION_BLOCKER_TEXT_ANCHORS = [
  ...TRANSFORMATION_SUMMON_BLOCKER_TEXT_ANCHORS,
  ...TRANSFORMATION_DISGUISE_BLOCKER_TEXT_ANCHORS,
  ...TRANSFORMATION_SIZE_BLOCKER_TEXT_ANCHORS,
  ...TRANSFORMATION_OBJECT_ANIMATION_BLOCKER_TEXT_ANCHORS,
];

const BATTLE_FORM_TEXT_ANCHORS = [
  patternAnchor("battle form"),
  patternAnchor("combat form"),
  patternAnchor("gain the following statistics"),
  patternAnchor("gain the following statistics and abilities"),
  patternAnchor("you gain the following statistics"),
  patternAnchor("you gain the following statistics and abilities"),
];

const ANIMAL_FORM_TEXT_ANCHORS = [
  patternAnchor("animal form"),
  patternAnchor("beast form"),
  patternAnchor("pest form"),
  patternAnchor("dinosaur form"),
  patternAnchor("animal battle form"),
  patternAnchor("beast battle form"),
  patternAnchor("pest battle form"),
  patternAnchor("dinosaur battle form"),
];

const ELEMENTAL_FORM_TEXT_ANCHORS = [
  patternAnchor("elemental form"),
  patternAnchor("elemental battle form"),
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
            terms: [patternAnchor("detect"), patternAnchor("undead")],
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
      { textAny: [patternAnchor("mask", "name"), patternAnchor("mask", "description")] },
    ],
    anyOf: [
      {
        textNear: [
          {
            terms: [patternAnchor("deception"), patternAnchor("lie"), patternAnchor("feint")],
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
          patternAnchor("senses"),
          patternAnchor("through"),
        ],
        textAny: [
          patternAnchor("through the ear"),
          patternAnchor("through its eyes"),
          patternAnchor("through each other s eyes"),
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
    tag: "transformation",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, traitsAny: ["polymorph", "morph", "transmutation"] },
      { score: 1, textAny: TRANSFORMATION_NAME_ANCHORS },
      { score: 2, textAny: TRANSFORMATION_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            terms: [patternAnchor("form"), patternAnchor("shape"), patternAnchor("body"), patternAnchor("transform"), patternAnchor("change")],
            window: 6,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
    ],
    noneOf: [
      { textAny: TRANSFORMATION_BLOCKER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "battle_form",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, traitsAny: ["polymorph", "morph"] },
      { score: 2, textAny: BATTLE_FORM_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            terms: [patternAnchor("battle"), patternAnchor("form"), patternAnchor("statistics"), patternAnchor("abilities")],
            window: 6,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
    ],
    noneOf: [
      { textAny: TRANSFORMATION_BLOCKER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "animal_form",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, traitsAny: ["polymorph", "morph"] },
      { score: 2, textAny: ANIMAL_FORM_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            terms: [patternAnchor("animal"), patternAnchor("beast"), patternAnchor("pest"), patternAnchor("dinosaur"), patternAnchor("form")],
            window: 6,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
    ],
    noneOf: [
      { textAny: TRANSFORMATION_BLOCKER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "elemental_form",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, traitsAny: ["polymorph", "morph"] },
      { score: 2, textAny: ELEMENTAL_FORM_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            terms: [patternAnchor("elemental"), patternAnchor("form"), patternAnchor("battle"), patternAnchor("statistics")],
            window: 6,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
    ],
    noneOf: [
      { textAny: TRANSFORMATION_BLOCKER_TEXT_ANCHORS },
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
              patternAnchor("protect"),
              patternAnchor("shield"),
              patternAnchor("ward"),
              patternAnchor("barrier"),
              patternAnchor("creature"),
              patternAnchor("target"),
              patternAnchor("ally"),
              patternAnchor("allies"),
              patternAnchor("area"),
              patternAnchor("self"),
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
              patternAnchor("stabilize"),
              patternAnchor("dying"),
              patternAnchor("die"),
              patternAnchor("death"),
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
              patternAnchor("return"),
              patternAnchor("life"),
              patternAnchor("revive"),
              patternAnchor("resurrect"),
              patternAnchor("death"),
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
      { score: 1, textAny: [patternAnchor("resistance"), patternAnchor("immune"), patternAnchor("immunity")] },
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
];
