import {
  AFFLICTION_MENTAL_TEXT_ANCHORS,
  AFFLICTION_MOBILITY_TEXT_ANCHORS,
  AFFLICTION_PHYSICAL_DEBILITATION_TEXT_ANCHORS,
  DerivedTagRule,
  patternAnchor,
} from "../shared.js";

const AFFLICTION_HEALING_SUPPRESSION_TEXT_ANCHORS = [
  patternAnchor("{{alt(can't, cannot)}} {{alt(heal damage, be healed)}}"),
  patternAnchor("{{alt(regain, regains)}} {{opt(only )}}half as many hit points from {{alt(healing, healing effects, all healing, all healing effects)}}"),
  patternAnchor("gains no benefit from healing"),
  patternAnchor("{{alt(can't, cannot)}} be reduced below stage {{number}}, nor the damage from it healed"),
  patternAnchor("{{alt(can't, cannot)}} heal {{opt(the )}}damage it takes"),
];

const AFFLICTION_COGNITIVE_IMPAIRMENT_TEXT_ANCHORS = [
  patternAnchor("clouds the mind"),
  patternAnchor("dulls the mind"),
  patternAnchor("haze {{alt(a, the)}} victim's mind"),
  patternAnchor("mind-robbing"),
  patternAnchor("decision-making skills"),
  patternAnchor("mental capacity"),
  patternAnchor("mental faculties"),
  patternAnchor("memory-altering"),
  patternAnchor("befuddle a target"),
  patternAnchor("addles the mind"),
  patternAnchor("cognitive impairment"),
];

const AFFLICTION_SENSORY_IMPAIRMENT_TEXT_ANCHORS = [
  patternAnchor("shuts down the senses"),
  patternAnchor("shuts down the imbiber's senses"),
  patternAnchor("loss of senses"),
  patternAnchor("lose sight"),
  patternAnchor("lose hearing"),
  patternAnchor("{{alt(can't, cannot)}} see"),
  patternAnchor("sightless"),
  patternAnchor("blind and deaf"),
  patternAnchor("blinded"),
  patternAnchor("deafened"),
  patternAnchor("blind"),
];

const AFFLICTION_SEDATION_TEXT_ANCHORS = [
  patternAnchor("{{alt(fall, falls)}} unconscious"),
  patternAnchor("deep unconsciousness"),
  patternAnchor("can't attempt perception checks to wake up"),
  patternAnchor("no perception check to wake up"),
  patternAnchor("sleep normally"),
  patternAnchor("can't wake up"),
  patternAnchor("wake up by any means"),
  patternAnchor("deep sleep"),
  patternAnchor("slumber"),
  patternAnchor("soporific"),
  patternAnchor("asleep"),
  patternAnchor("unconscious"),
];

export const AFFLICTION_DERIVED_TAG_RULES: DerivedTagRule[] = [
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
    tag: "physical_debilitation",
    category: "affliction",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AFFLICTION_PHYSICAL_DEBILITATION_TEXT_ANCHORS },
    ],
  },
  {
    tag: "healing_suppression",
    category: "affliction",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AFFLICTION_HEALING_SUPPRESSION_TEXT_ANCHORS },
    ],
  },
  {
    tag: "cognitive_impairment",
    category: "affliction",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AFFLICTION_COGNITIVE_IMPAIRMENT_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            terms: [
              patternAnchor("mind"),
              patternAnchor("mind's"),
              patternAnchor("mental"),
              patternAnchor("thinking"),
              patternAnchor("thought"),
              patternAnchor("thoughts"),
              patternAnchor("decision"),
              patternAnchor("decision-making"),
              patternAnchor("faculties"),
              patternAnchor("capacity"),
              patternAnchor("memory"),
              patternAnchor("memories"),
              patternAnchor("stupefied"),
            ],
            window: 5,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
    ],
  },
  {
    tag: "sensory_impairment",
    category: "affliction",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AFFLICTION_SENSORY_IMPAIRMENT_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            terms: [
              patternAnchor("blind"),
              patternAnchor("blinded"),
              patternAnchor("deaf"),
              patternAnchor("deafened"),
              patternAnchor("sight"),
              patternAnchor("sights"),
              patternAnchor("vision"),
              patternAnchor("perception"),
              patternAnchor("senses"),
            ],
            window: 5,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
    ],
  },
  {
    tag: "sedation",
    category: "affliction",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AFFLICTION_SEDATION_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            terms: [
              patternAnchor("sleep"),
              patternAnchor("sleeping"),
              patternAnchor("slumber"),
              patternAnchor("unconscious"),
              patternAnchor("wake"),
              patternAnchor("waking"),
            ],
            window: 6,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
    ],
    noneOf: [
      { textAny: [patternAnchor("dreamtime tea", "name")] },
    ],
  },
];
