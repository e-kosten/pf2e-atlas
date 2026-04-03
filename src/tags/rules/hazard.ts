import {
  DerivedTagRule,
  HAZARD_ALARM_TEXT_ANCHORS,
  HAZARD_COLLAPSE_NAME_ANCHORS,
  HAZARD_FIRE_NAME_ANCHORS,
  HAZARD_FIRE_TEXT_ANCHORS,
  HAZARD_FORCED_MOVEMENT_TEXT_ANCHORS,
  HAZARD_PITFALL_NAME_ANCHORS,
  HAZARD_POISON_NAME_ANCHORS,
  HAZARD_POISON_TEXT_ANCHORS,
  RESTRAINT_CAPTURE_REFERENCE_ANCHORS,
  phraseAnchor,
  tokenAnchor,
} from "../shared.js";

export const HAZARD_DERIVED_TAG_RULES: DerivedTagRule[] = [
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
    tag: "barrier_lockdown",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          phraseAnchor("slams shut"),
          phraseAnchor("slam down into place"),
          phraseAnchor("sealing the entrance"),
          phraseAnchor("entry and exit seal with a force barrier"),
          phraseAnchor("magically seals the door"),
          phraseAnchor("block progress through this area"),
          phraseAnchor("filling the passage with stone"),
          phraseAnchor("trap the triggering creature inside"),
          phraseAnchor("imprisons intruders"),
          phraseAnchor("pushes two iron doors closed"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            terms: [tokenAnchor("portcullis"), tokenAnchor("drops")],
            window: 4,
            scope: "description",
          },
          {
            terms: [tokenAnchor("portcullis"), tokenAnchor("slam")],
            window: 6,
            scope: "description",
          },
          {
            terms: [tokenAnchor("door"), tokenAnchor("locks")],
            window: 3,
            scope: "description",
          },
          {
            terms: [tokenAnchor("door"), tokenAnchor("seal")],
            window: 4,
            scope: "description",
          },
          {
            terms: [tokenAnchor("gate"), tokenAnchor("shut")],
            window: 4,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "fire_hazard",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 1,
        textAny: HAZARD_FIRE_NAME_ANCHORS,
      },
      {
        score: 2,
        textAny: HAZARD_FIRE_TEXT_ANCHORS,
      },
      {
        score: 2,
        textNear: [
          {
            terms: [
              tokenAnchor("fire"),
              tokenAnchor("flame"),
              tokenAnchor("flames"),
              tokenAnchor("burns"),
              tokenAnchor("burning"),
              tokenAnchor("ignite"),
              tokenAnchor("ignites"),
              tokenAnchor("explodes"),
              tokenAnchor("spread"),
              tokenAnchor("spreads"),
            ],
            window: 6,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
    ],
  },
  {
    tag: "poison_hazard",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 1,
        textAny: HAZARD_POISON_NAME_ANCHORS,
      },
      {
        score: 2,
        textAny: HAZARD_POISON_TEXT_ANCHORS,
      },
      {
        score: 2,
        textNear: [
          {
            terms: [
              tokenAnchor("poison"),
              tokenAnchor("poisonous"),
              tokenAnchor("venom"),
              tokenAnchor("toxic"),
              tokenAnchor("gas"),
              tokenAnchor("smoke"),
              tokenAnchor("dart"),
              tokenAnchor("darts"),
              tokenAnchor("needle"),
              tokenAnchor("spine"),
              tokenAnchor("vent"),
              tokenAnchor("vents"),
              tokenAnchor("nozzle"),
              tokenAnchor("nozzles"),
              tokenAnchor("cloud"),
            ],
            window: 6,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
    ],
  },
  {
    tag: "pitfall",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 1,
        textAny: HAZARD_PITFALL_NAME_ANCHORS,
      },
      {
        score: 2,
        textAny: [
          phraseAnchor("trapdoor covers a pit"),
          phraseAnchor("covers a pit"),
          phraseAnchor("pit filled with spikes"),
          phraseAnchor("falls into the pit"),
          phraseAnchor("drops into the pit"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            terms: [
              tokenAnchor("pit"),
              tokenAnchor("trapdoor"),
              tokenAnchor("deep"),
              tokenAnchor("spikes"),
              tokenAnchor("water"),
              tokenAnchor("fall"),
              tokenAnchor("falls"),
              tokenAnchor("drop"),
              tokenAnchor("drops"),
              tokenAnchor("covers"),
              tokenAnchor("conceals"),
            ],
            window: 6,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
    ],
  },
  {
    tag: "collapse_hazard",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 1,
        textAny: HAZARD_COLLAPSE_NAME_ANCHORS,
      },
      {
        score: 2,
        textAny: [
          phraseAnchor("ceiling collapses"),
          phraseAnchor("triggers a cave in"),
          phraseAnchor("structure to collapse"),
          phraseAnchor("collapse into rubble"),
          phraseAnchor("bridge itself groans and shakes then crumbles"),
          phraseAnchor("collapse inward"),
          phraseAnchor("floor collapses"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            terms: [
              tokenAnchor("ceiling"),
              tokenAnchor("floor"),
              tokenAnchor("bridge"),
              tokenAnchor("stairs"),
              tokenAnchor("supports"),
              tokenAnchor("tunnel"),
              tokenAnchor("cavern"),
              tokenAnchor("structure"),
              tokenAnchor("pillar"),
              tokenAnchor("collapse"),
              tokenAnchor("collapses"),
              tokenAnchor("crumble"),
              tokenAnchor("crumbles"),
              tokenAnchor("fall"),
              tokenAnchor("falls"),
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
    tag: "forced_movement",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: HAZARD_FORCED_MOVEMENT_TEXT_ANCHORS,
      },
      {
        score: 2,
        textNear: [
          {
            terms: [
              tokenAnchor("sucks"),
              tokenAnchor("pulls"),
              tokenAnchor("pushes"),
              tokenAnchor("drags"),
              tokenAnchor("sweeps"),
              tokenAnchor("submerge"),
              tokenAnchor("trample"),
              tokenAnchor("trampling"),
              tokenAnchor("creatures"),
              tokenAnchor("toward"),
              tokenAnchor("away"),
              tokenAnchor("into"),
            ],
            window: 6,
            scope: "description",
            minTermsMatched: 3,
          },
        ],
      },
    ],
  },
  {
    tag: "mental_impairment",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          phraseAnchor("damages the mind"),
          phraseAnchor("filling the heads"),
          tokenAnchor("fear"),
          tokenAnchor("frightened"),
          phraseAnchor("flood the minds"),
          tokenAnchor("paranoia"),
          tokenAnchor("confused"),
          tokenAnchor("confusion"),
          tokenAnchor("hallucination"),
          tokenAnchor("hallucinations"),
          phraseAnchor("maddening visions"),
          phraseAnchor("mental trauma"),
          phraseAnchor("psychic scream"),
          tokenAnchor("disorients"),
        ],
      },
      {
        score: 1,
        textAny: [
          tokenAnchor("psychic"),
          tokenAnchor("mental"),
          tokenAnchor("mind"),
        ],
      },
    ],
  },
  {
    tag: "mobility_impairment",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          tokenAnchor("paralyzed"),
          tokenAnchor("immobilized"),
          tokenAnchor("restrained"),
          tokenAnchor("grabbed"),
          phraseAnchor("holds the creature in place"),
          phraseAnchor("holds intruders in place"),
          phraseAnchor("hold creatures in place"),
          phraseAnchor("attempting to restrain nearby creatures"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            terms: [tokenAnchor("hamper"), tokenAnchor("creatures")],
            window: 3,
            scope: "description",
          },
        ],
      },
      {
        score: 1,
        textAny: [
          tokenAnchor("slowed"),
        ],
      },
    ],
  },
];
