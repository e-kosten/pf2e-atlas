import {
  AFFLICTION_MENTAL_TEXT_ANCHORS,
  AFFLICTION_MOBILITY_TEXT_ANCHORS,
  AFFLICTION_PHYSICAL_DEBILITATION_TEXT_ANCHORS,
  DerivedTagRule,
  patternAnchor,
} from "../runtime/shared.js";

const AFFLICTION_HEALING_SUPPRESSION_TEXT_ANCHORS = [
  patternAnchor("{{alt(can't, cannot)}} {{alt(heal damage, be healed)}}"),
  patternAnchor(
    "{{alt(regain, regains)}} {{opt(only )}}half as many hit points from {{alt(healing, healing effects, all healing, all healing effects)}}",
  ),
  patternAnchor("gains no benefit from healing"),
  patternAnchor("{{alt(can't, cannot)}} be reduced below stage {{number}}, nor the damage from it healed"),
  patternAnchor("{{alt(can't, cannot)}} heal {{opt(the )}}damage it takes"),
];

const AFFLICTION_COGNITIVE_IMPAIRMENT_TEXT_ANCHORS = [
  patternAnchor("clouds the mind"),
  patternAnchor("dulls the mind"),
  patternAnchor("haze {{alt(a, the)}} victim's mind"),
  patternAnchor("fractured thoughts"),
  patternAnchor("hallucination"),
  patternAnchor("hallucinations"),
  patternAnchor("mind-robbing"),
  patternAnchor("decision-making skills"),
  patternAnchor("mental capacity"),
  patternAnchor("mental faculties"),
  patternAnchor("amnesia"),
  patternAnchor("memory loss"),
  patternAnchor("memory-altering"),
  patternAnchor("can't remember"),
  patternAnchor("cannot remember"),
  patternAnchor("forgets"),
  patternAnchor("forgets who"),
  patternAnchor("befuddle a target"),
  patternAnchor("addles the mind"),
  patternAnchor("cognitive impairment"),
  patternAnchor("stupefied"),
];

const AFFLICTION_SENSORY_IMPAIRMENT_TEXT_ANCHORS = [
  patternAnchor("shuts down the senses"),
  patternAnchor("shuts down the imbiber's senses"),
  patternAnchor("loss of senses"),
  patternAnchor("blinding"),
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
  patternAnchor("falls asleep"),
  patternAnchor("drowsy"),
  patternAnchor("deep unconsciousness"),
  patternAnchor("can't attempt perception checks to wake up"),
  patternAnchor("no perception check to wake up"),
  patternAnchor("sleep normally"),
  patternAnchor("can't wake up"),
  patternAnchor("can't be awakened"),
  patternAnchor("cannot be awakened"),
  patternAnchor("wake up by any means"),
  patternAnchor("deep sleep"),
  patternAnchor("sleeping sickness"),
  patternAnchor("slumber"),
  patternAnchor("soporific"),
  patternAnchor("asleep"),
  patternAnchor("unconscious"),
];

const AFFLICTION_SEDATION_EXTENDED_TEXT_ANCHORS = [
  patternAnchor("trance"),
  patternAnchor("trancelike"),
  patternAnchor("somnolent"),
  patternAnchor("somnolence"),
  patternAnchor("deep sleep"),
  patternAnchor("deep sleep state"),
  patternAnchor("slumbering"),
  patternAnchor("cannot wake"),
  patternAnchor("can't wake"),
  patternAnchor("cannot be awakened"),
  patternAnchor("can't be awakened"),
  patternAnchor("unconsciousness"),
];

const AFFLICTION_ROT_DECAY_NAME_ANCHORS = [
  patternAnchor("rot", "name"),
  patternAnchor("mummy rot", "name"),
  patternAnchor("necrotic", "name"),
  patternAnchor("blight", "name"),
  patternAnchor("slough", "name"),
];

const AFFLICTION_ROT_DECAY_TEXT_ANCHORS = [
  patternAnchor("rot"),
  patternAnchor("rotting"),
  patternAnchor("decay"),
  patternAnchor("decaying"),
  patternAnchor("putrefy"),
  patternAnchor("putrefies"),
  patternAnchor("necrotic"),
  patternAnchor("mummif"),
  patternAnchor("blight"),
  patternAnchor("slough"),
  patternAnchor("sloughing"),
];

const AFFLICTION_ROT_DECAY_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("mind-rotting", "name"),
  patternAnchor("mind rotting", "name"),
];

const AFFLICTION_INFESTATION_NAME_ANCHORS = [
  patternAnchor("infestation", "name"),
  patternAnchor("larva", "name"),
  patternAnchor("larvae", "name"),
  patternAnchor("eggs", "name"),
];

const AFFLICTION_INFESTATION_TEXT_ANCHORS = [
  patternAnchor("parasite"),
  patternAnchor("parasites"),
  patternAnchor("larva"),
  patternAnchor("larvae"),
  patternAnchor("eggs"),
  patternAnchor("spores"),
  patternAnchor("infestation"),
  patternAnchor("hatch"),
  patternAnchor("hatches"),
  patternAnchor("hatching"),
  patternAnchor("burrow"),
  patternAnchor("burrows"),
  patternAnchor("colonize"),
  patternAnchor("colonizes"),
  patternAnchor("inside the host"),
  patternAnchor("inside the body"),
];

const AFFLICTION_EPIDEMIC_PESTILENCE_NAME_ANCHORS = [
  patternAnchor("plague", "name"),
  patternAnchor("fever", "name"),
  patternAnchor("pox", "name"),
  patternAnchor("sickness", "name"),
  patternAnchor("pestilence", "name"),
  patternAnchor("contagion", "name"),
];

const AFFLICTION_EPIDEMIC_PESTILENCE_TEXT_ANCHORS = [
  patternAnchor("contagious"),
  patternAnchor("infectious"),
  patternAnchor("transmit"),
  patternAnchor("spread through entire communities"),
  patternAnchor("sweep through entire communities"),
  patternAnchor("widespread illness"),
];

const AFFLICTION_COMPULSION_TEXT_ANCHORS = [
  patternAnchor("speak only the truth"),
  patternAnchor("can't speak deliberate lies"),
  patternAnchor("cannot speak deliberate lies"),
  patternAnchor("can use no actions but"),
  patternAnchor("controlled by"),
  patternAnchor("compelled to"),
  patternAnchor("forbidden cravings"),
  patternAnchor("must obey"),
  patternAnchor("must eat"),
  patternAnchor("must spend each action"),
  patternAnchor("twisted loyalties"),
  patternAnchor("treat allies as enemies"),
  patternAnchor("driven to"),
];

const AFFLICTION_RESPIRATORY_IMPAIRMENT_NAME_ANCHORS = [
  patternAnchor("apoxia", "name"),
  patternAnchor("fill lungs", "name"),
  patternAnchor("flood breath", "name"),
];

const AFFLICTION_RESPIRATORY_IMPAIRMENT_TEXT_ANCHORS = [
  patternAnchor("{{alt(can't, cannot)}} breathe"),
  patternAnchor("unable to breathe"),
  patternAnchor("suffocate"),
  patternAnchor("suffocates"),
  patternAnchor("fill lungs"),
  patternAnchor("flood breath"),
  patternAnchor("lungs fill"),
  patternAnchor("lungs filled"),
  patternAnchor("inhale water"),
  patternAnchor("water in its lungs"),
  patternAnchor("water in their lungs"),
];

const AFFLICTION_TRANSFORMATIVE_CORRUPTION_TEXT_ANCHORS = [
  patternAnchor("slowly turn to solid crystal"),
  patternAnchor("slowly turns to solid crystal"),
  patternAnchor("slowly turns into crystal"),
  patternAnchor("turns into crystal"),
  patternAnchor("turn into crystal"),
  patternAnchor("soft tissues harden"),
  patternAnchor("turn the victim s lungs into a soft jelly"),
  patternAnchor("turns the victim s lungs into a soft jelly"),
  patternAnchor("turns into a plant"),
  patternAnchor("turns into plant matter"),
  patternAnchor("turns into fungus"),
  patternAnchor("turns into fungal matter"),
  patternAnchor("body changes"),
  patternAnchor("mutate"),
  patternAnchor("mutates"),
  patternAnchor("transformation"),
  patternAnchor("creeping sprout"),
  patternAnchor("lamashtu s bloom"),
  patternAnchor("crystalline"),
  patternAnchor("bloom"),
  patternAnchor("sprout"),
  patternAnchor("sprouts"),
  patternAnchor("root"),
  patternAnchor("roots"),
  patternAnchor("bark"),
  patternAnchor("fungal"),
];

const AFFLICTION_TRANSFORMATIVE_CORRUPTION_NAME_ANCHORS = [
  patternAnchor("bloom", "name"),
  patternAnchor("crystal", "name"),
  patternAnchor("crystalline", "name"),
  patternAnchor("corruption", "name"),
  patternAnchor("fungal", "name"),
  patternAnchor("fungus", "name"),
];

const AFFLICTION_VOID_SOUL_CORRUPTION_NAME_ANCHORS = [
  patternAnchor("void s embrace", "name"),
  patternAnchor("void death", "name"),
  patternAnchor("reaper", "name"),
  patternAnchor("lifeblight", "name"),
];

const AFFLICTION_VOID_SOUL_CORRUPTION_TEXT_ANCHORS = [
  patternAnchor("body and soul"),
  patternAnchor("life force"),
  patternAnchor("leeches life force"),
  patternAnchor("erodes the connection between body and soul"),
  patternAnchor("assuming the former has already died"),
  patternAnchor("final breath"),
];

const AFFLICTION_NIGHTMARE_TORMENT_NAME_ANCHORS = [
  patternAnchor("nightmare", "name"),
  patternAnchor("torment", "name"),
];

const AFFLICTION_PHYSICAL_DEBILITATION_EXTENDED_TEXT_ANCHORS = [
  patternAnchor("clumsy"),
  patternAnchor("wasting"),
  patternAnchor("wastes away"),
  patternAnchor("wither"),
  patternAnchor("withers"),
  patternAnchor("withering"),
  patternAnchor("exhaustion"),
  patternAnchor("exhausted"),
  patternAnchor("blood loss"),
  patternAnchor("bleeding"),
  patternAnchor("hemorrhage"),
  patternAnchor("hemorrhaging"),
  patternAnchor("drains vitality"),
  patternAnchor("draining vitality"),
  patternAnchor("shock and exhaustion"),
  patternAnchor("weakness"),
  patternAnchor("weakened"),
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
    anyOf: [{ textAny: AFFLICTION_MOBILITY_TEXT_ANCHORS }],
  },
  {
    tag: "physical_debilitation",
    category: "affliction",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AFFLICTION_PHYSICAL_DEBILITATION_TEXT_ANCHORS },
      { score: 2, textAny: AFFLICTION_PHYSICAL_DEBILITATION_EXTENDED_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("stage"),
              patternAnchor("{{alt(drained,enfeebled,fatigued,sickened,exhausted,wasting,bleeding,hemorrhage)}}"),
            ],
            window: 4,
            scope: "description",
          },
          {
            all: [
              patternAnchor("{{alt(blood,vitality,body)}}"),
              patternAnchor("{{alt(weakness,weakened,wasting,bleeding,hemorrhage,drained)}}"),
            ],
            window: 5,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "healing_suppression",
    category: "affliction",
    threshold: 2,
    anyOf: [{ score: 2, textAny: AFFLICTION_HEALING_SUPPRESSION_TEXT_ANCHORS }],
  },
  {
    tag: "cognitive_impairment",
    category: "affliction",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: [patternAnchor("amnesia", "name"), patternAnchor("wisdom", "name")] },
      { score: 2, textAny: AFFLICTION_COGNITIVE_IMPAIRMENT_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor(
                "{{alt(mind,mind's,mental,thinking,thought,thoughts,decision,decision-making,faculties,capacity,memory,memories)}}",
              ),
              patternAnchor("{{alt(stupefied,dulls,clouds,inhibits,fractured,mud)}}"),
            ],
            window: 6,
            scope: "description",
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
            all: [
              patternAnchor("{{alt(blind,blinded,deaf,deafened)}}"),
              patternAnchor("{{alt(sight,sights,vision,eyes,perception,senses,hearing,ears)}}"),
            ],
            window: 5,
            scope: "description",
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
      { score: 2, textAny: AFFLICTION_SEDATION_EXTENDED_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(sleep,sleeping,slumber,asleep)}}"),
              patternAnchor("{{alt(unconscious,wake,waking,drowsy,drowsiness,trancelike)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "respiratory_impairment",
    category: "affliction",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AFFLICTION_RESPIRATORY_IMPAIRMENT_NAME_ANCHORS },
      { score: 2, textAny: AFFLICTION_RESPIRATORY_IMPAIRMENT_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(breathe,breathing,breath,breathless,lungs,air)}}"),
              patternAnchor("{{alt(unable,suffocate,suffocates,apoxia,can't breathe,cannot breathe)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "rot_decay",
    category: "affliction",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AFFLICTION_ROT_DECAY_NAME_ANCHORS },
      { score: 2, textAny: AFFLICTION_ROT_DECAY_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(rot,rotting,decay,decaying,blight,necrotic)}}"),
              patternAnchor("{{alt(flesh,wounds,body,curse,disease)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: AFFLICTION_ROT_DECAY_BLOCKER_TEXT_ANCHORS }],
  },
  {
    tag: "infestation_implant",
    category: "affliction",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AFFLICTION_INFESTATION_NAME_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(spores,egg,eggs,larva,larvae,parasite,parasites,infestation)}}"),
              patternAnchor("{{alt(hatch,hatches,hatching,burrow,burrows,colonize,colonizes,host,body,inside)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
      { score: 2, textAny: AFFLICTION_INFESTATION_TEXT_ANCHORS },
    ],
  },
  {
    tag: "epidemic_pestilence",
    category: "affliction",
    subcategories: ["disease"],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AFFLICTION_EPIDEMIC_PESTILENCE_NAME_ANCHORS },
      { score: 2, textAny: AFFLICTION_EPIDEMIC_PESTILENCE_TEXT_ANCHORS },
    ],
  },
  {
    tag: "compulsion",
    category: "affliction",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AFFLICTION_COMPULSION_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(must,forced,compelled,controlled,commanded,driven)}}"),
              patternAnchor(
                "{{alt(behavior,action,actions,obey,truth,lies,move,attack,eat,allies,loyalty,cravings,betray,join)}}",
              ),
            ],
            window: 5,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "transformative_corruption",
    category: "affliction",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AFFLICTION_TRANSFORMATIVE_CORRUPTION_TEXT_ANCHORS },
      { score: 2, textAny: AFFLICTION_TRANSFORMATIVE_CORRUPTION_NAME_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(turn,turns,transforms,transformation,mutates)}}"),
              patternAnchor("{{alt(body,flesh,skin,lungs,tissues)}}"),
              patternAnchor(
                "{{alt(crystal,crystalline,petrified,sprout,bloom,plant,fungus,fungal,root,roots,bark,jelly)}}",
              ),
            ],
            window: 8,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: [patternAnchor("moral corruption"), patternAnchor("spiritual corruption")] }],
  },
  {
    tag: "void_soul_corruption",
    category: "affliction",
    threshold: 3,
    anyOf: [
      { score: 2, textAny: AFFLICTION_VOID_SOUL_CORRUPTION_NAME_ANCHORS },
      { score: 2, textAny: AFFLICTION_VOID_SOUL_CORRUPTION_TEXT_ANCHORS },
      { score: 1, traitsAny: ["void", "death"] },
    ],
  },
  {
    tag: "nightmare_torment",
    category: "affliction",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: AFFLICTION_NIGHTMARE_TORMENT_NAME_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(dream,dreams,nightmare,nightmares)}}"),
              patternAnchor(
                "{{alt(terrifying,visions,torment,torments,haunting,can't be awakened,cannot be awakened)}}",
              ),
            ],
            window: 8,
            scope: "description",
          },
        ],
      },
    ],
  },
];
