import {
  AFFLICTION_MENTAL_TEXT_ANCHORS,
  AFFLICTION_MOBILITY_TEXT_ANCHORS,
  AFFLICTION_PHYSICAL_DEBILITATION_TEXT_ANCHORS,
  DerivedTagRule,
  phraseAnchor,
  tokenAnchor,
} from "../shared.js";

const AFFLICTION_HEALING_SUPPRESSION_TEXT_ANCHORS = [
  phraseAnchor("can't heal damage"),
  phraseAnchor("cannot heal damage"),
  phraseAnchor("can't be healed"),
  phraseAnchor("cannot be healed"),
  phraseAnchor("regains half as many hit points from all healing"),
  phraseAnchor("regains only half as many hit points from healing effects"),
  phraseAnchor("regain only half as many hit points from healing effects"),
  phraseAnchor("gains no benefit from healing"),
  phraseAnchor("can't be reduced below stage 1, nor the damage from it healed"),
  phraseAnchor("can't heal damage it takes"),
  phraseAnchor("can't heal the damage it takes"),
];

const AFFLICTION_COGNITIVE_IMPAIRMENT_TEXT_ANCHORS = [
  phraseAnchor("clouds the mind"),
  phraseAnchor("dulls the mind"),
  phraseAnchor("haze a victim's mind"),
  phraseAnchor("haze the victim's mind"),
  phraseAnchor("mind-robbing"),
  phraseAnchor("decision-making skills"),
  phraseAnchor("mental capacity"),
  phraseAnchor("mental faculties"),
  phraseAnchor("memory-altering"),
  phraseAnchor("befuddle a target"),
  phraseAnchor("addles the mind"),
  phraseAnchor("cognitive impairment"),
];

const AFFLICTION_SENSORY_IMPAIRMENT_TEXT_ANCHORS = [
  phraseAnchor("shuts down the senses"),
  phraseAnchor("shuts down the imbiber's senses"),
  phraseAnchor("loss of senses"),
  phraseAnchor("lose sight"),
  phraseAnchor("lose hearing"),
  phraseAnchor("can't see"),
  phraseAnchor("cannot see"),
  phraseAnchor("sightless"),
  phraseAnchor("blind and deaf"),
  tokenAnchor("blinded"),
  tokenAnchor("deafened"),
  tokenAnchor("blind"),
];

const AFFLICTION_SEDATION_TEXT_ANCHORS = [
  phraseAnchor("fall unconscious"),
  phraseAnchor("falls unconscious"),
  phraseAnchor("deep unconsciousness"),
  phraseAnchor("can't attempt perception checks to wake up"),
  phraseAnchor("no perception check to wake up"),
  phraseAnchor("sleep normally"),
  phraseAnchor("can't wake up"),
  phraseAnchor("wake up by any means"),
  phraseAnchor("deep sleep"),
  tokenAnchor("slumber"),
  tokenAnchor("soporific"),
  tokenAnchor("asleep"),
  tokenAnchor("unconscious"),
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
              tokenAnchor("mind"),
              tokenAnchor("mind's"),
              tokenAnchor("mental"),
              tokenAnchor("thinking"),
              tokenAnchor("thought"),
              tokenAnchor("thoughts"),
              tokenAnchor("decision"),
              tokenAnchor("decision-making"),
              tokenAnchor("faculties"),
              tokenAnchor("capacity"),
              tokenAnchor("memory"),
              tokenAnchor("memories"),
              tokenAnchor("stupefied"),
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
              tokenAnchor("blind"),
              tokenAnchor("blinded"),
              tokenAnchor("deaf"),
              tokenAnchor("deafened"),
              tokenAnchor("sight"),
              tokenAnchor("sights"),
              tokenAnchor("vision"),
              tokenAnchor("perception"),
              tokenAnchor("senses"),
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
              tokenAnchor("sleep"),
              tokenAnchor("sleeping"),
              tokenAnchor("slumber"),
              tokenAnchor("unconscious"),
              tokenAnchor("wake"),
              tokenAnchor("waking"),
            ],
            window: 6,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
    ],
    noneOf: [
      { textAny: [phraseAnchor("dreamtime tea", "name")] },
    ],
  },
];
