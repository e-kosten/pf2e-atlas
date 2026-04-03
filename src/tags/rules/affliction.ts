import {
  AFFLICTION_MENTAL_TEXT_ANCHORS,
  AFFLICTION_MOBILITY_TEXT_ANCHORS,
  AFFLICTION_PHYSICAL_DEBILITATION_TEXT_ANCHORS,
  DerivedTagRule,
} from "../shared.js";

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
];
