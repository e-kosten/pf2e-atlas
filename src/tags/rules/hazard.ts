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

const HAZARD_ACID_NAME_ANCHORS = [
  tokenAnchor("acid", "name"),
  tokenAnchor("acidic", "name"),
  tokenAnchor("caustic", "name"),
  tokenAnchor("corrosive", "name"),
];

const HAZARD_ACID_TEXT_ANCHORS = [
  tokenAnchor("acid"),
  tokenAnchor("acidic"),
  tokenAnchor("caustic"),
  tokenAnchor("corrosive"),
  tokenAnchor("corroding"),
  tokenAnchor("corrosion"),
  phraseAnchor("acid mist"),
  phraseAnchor("acid spray"),
  phraseAnchor("acid splash"),
  phraseAnchor("spray of acid"),
  phraseAnchor("pool of acid"),
  phraseAnchor("flesh-eating acid"),
  phraseAnchor("caustic shower"),
];

const HAZARD_COLD_NAME_ANCHORS = [
  tokenAnchor("ice", "name"),
  tokenAnchor("iced", "name"),
  tokenAnchor("frost", "name"),
  tokenAnchor("frozen", "name"),
  tokenAnchor("freezing", "name"),
  tokenAnchor("glacial", "name"),
  tokenAnchor("snow", "name"),
  tokenAnchor("hail", "name"),
  tokenAnchor("blizzard", "name"),
];

const HAZARD_COLD_TEXT_ANCHORS = [
  tokenAnchor("ice"),
  tokenAnchor("frost"),
  tokenAnchor("frozen"),
  tokenAnchor("freezing"),
  tokenAnchor("glacial"),
  tokenAnchor("snow"),
  tokenAnchor("hail"),
  tokenAnchor("blizzard"),
  tokenAnchor("frigid"),
  tokenAnchor("chill"),
  tokenAnchor("icy"),
  phraseAnchor("thin ice"),
  phraseAnchor("freezing floor"),
  phraseAnchor("freezing floor tiles"),
  phraseAnchor("ice fall"),
  phraseAnchor("ice fall trap"),
];

const HAZARD_ELECTRIC_NAME_ANCHORS = [
  tokenAnchor("electric", "name"),
  tokenAnchor("lightning", "name"),
  tokenAnchor("shock", "name"),
  tokenAnchor("shocking", "name"),
  tokenAnchor("static", "name"),
  tokenAnchor("spark", "name"),
  tokenAnchor("discharge", "name"),
];

const HAZARD_ELECTRIC_TEXT_ANCHORS = [
  tokenAnchor("electric"),
  tokenAnchor("electricity"),
  tokenAnchor("lightning"),
  tokenAnchor("shock"),
  tokenAnchor("shocking"),
  tokenAnchor("static"),
  tokenAnchor("spark"),
  tokenAnchor("sparking"),
  tokenAnchor("discharge"),
  tokenAnchor("current"),
  tokenAnchor("arc"),
  phraseAnchor("electric shock"),
  phraseAnchor("lightning bolt"),
  phraseAnchor("shocking rune"),
  phraseAnchor("shock glyph"),
  phraseAnchor("electricity crackles"),
];

const HAZARD_SOUND_NAME_ANCHORS = [
  tokenAnchor("sonic", "name"),
  tokenAnchor("sound", "name"),
  tokenAnchor("vibration", "name"),
  tokenAnchor("resonance", "name"),
  tokenAnchor("buzzing", "name"),
  tokenAnchor("shrieking", "name"),
  tokenAnchor("wail", "name"),
  tokenAnchor("harmonic", "name"),
  tokenAnchor("symphony", "name"),
];

const HAZARD_SOUND_TEXT_ANCHORS = [
  tokenAnchor("sonic"),
  tokenAnchor("sound"),
  tokenAnchor("soundwave"),
  tokenAnchor("soundwaves"),
  tokenAnchor("vibration"),
  tokenAnchor("vibrations"),
  tokenAnchor("resonance"),
  tokenAnchor("resonant"),
  tokenAnchor("buzzing"),
  tokenAnchor("buzz"),
  tokenAnchor("shriek"),
  tokenAnchor("shrieking"),
  tokenAnchor("wail"),
  tokenAnchor("wailing"),
  tokenAnchor("deafening"),
  tokenAnchor("ringing"),
  tokenAnchor("drone"),
  tokenAnchor("hum"),
  phraseAnchor("sonic damage"),
  phraseAnchor("sound waves"),
  phraseAnchor("deafening noise"),
  phraseAnchor("ringing in the ears"),
  phraseAnchor("vibrating floor"),
];

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
    tag: "acid_hazard",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 1,
        textAny: HAZARD_ACID_NAME_ANCHORS,
      },
      {
        score: 2,
        textAny: HAZARD_ACID_TEXT_ANCHORS,
      },
      {
        score: 2,
        textNear: [
          {
            terms: [
              tokenAnchor("acid"),
              tokenAnchor("acidic"),
              tokenAnchor("caustic"),
              tokenAnchor("corrosive"),
              tokenAnchor("corroding"),
            ],
            window: 5,
            scope: "description",
            minTermsMatched: 1,
          },
          {
            terms: [
              tokenAnchor("spray"),
              tokenAnchor("mist"),
              tokenAnchor("cloud"),
              tokenAnchor("pool"),
              tokenAnchor("shower"),
              tokenAnchor("splash"),
              tokenAnchor("runoff"),
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
    tag: "cold_hazard",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 1,
        textAny: HAZARD_COLD_NAME_ANCHORS,
      },
      {
        score: 2,
        textAny: HAZARD_COLD_TEXT_ANCHORS,
      },
      {
        score: 2,
        textNear: [
          {
            terms: [
              tokenAnchor("ice"),
              tokenAnchor("frost"),
              tokenAnchor("frozen"),
              tokenAnchor("freezing"),
              tokenAnchor("glacial"),
              tokenAnchor("snow"),
              tokenAnchor("hail"),
              tokenAnchor("blizzard"),
              tokenAnchor("frigid"),
              tokenAnchor("chill"),
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
    tag: "electric_hazard",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 1,
        textAny: HAZARD_ELECTRIC_NAME_ANCHORS,
      },
      {
        score: 2,
        textAny: HAZARD_ELECTRIC_TEXT_ANCHORS,
      },
      {
        score: 2,
        textNear: [
          {
            terms: [
              tokenAnchor("electric"),
              tokenAnchor("electricity"),
              tokenAnchor("lightning"),
              tokenAnchor("shock"),
              tokenAnchor("shocking"),
              tokenAnchor("static"),
              tokenAnchor("spark"),
              tokenAnchor("discharge"),
              tokenAnchor("current"),
              tokenAnchor("arc"),
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
    tag: "sound_hazard",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 1,
        textAny: HAZARD_SOUND_NAME_ANCHORS,
      },
      {
        score: 2,
        textAny: HAZARD_SOUND_TEXT_ANCHORS,
      },
      {
        score: 2,
        textNear: [
          {
            terms: [
              tokenAnchor("sonic"),
              tokenAnchor("sound"),
              tokenAnchor("soundwave"),
              tokenAnchor("vibration"),
              tokenAnchor("resonance"),
              tokenAnchor("buzzing"),
              tokenAnchor("shriek"),
              tokenAnchor("wail"),
              tokenAnchor("deafening"),
              tokenAnchor("ringing"),
            ],
            window: 6,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
    ],
    noneOf: [
      { textAny: HAZARD_ALARM_TEXT_ANCHORS },
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
