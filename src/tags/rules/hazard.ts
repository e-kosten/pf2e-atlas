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
  patternAnchor,
} from "../shared.js";

const HAZARD_ACID_NAME_ANCHORS = [
  patternAnchor("acid", "name"),
  patternAnchor("acidic", "name"),
  patternAnchor("caustic", "name"),
  patternAnchor("corrosive", "name"),
];

const HAZARD_ACID_TEXT_ANCHORS = [
  patternAnchor("acid"),
  patternAnchor("acidic"),
  patternAnchor("caustic"),
  patternAnchor("corrosive"),
  patternAnchor("corroding"),
  patternAnchor("corrosion"),
  patternAnchor("acid mist"),
  patternAnchor("acid spray"),
  patternAnchor("acid splash"),
  patternAnchor("spray of acid"),
  patternAnchor("pool of acid"),
  patternAnchor("flesh-eating acid"),
  patternAnchor("caustic shower"),
];

const HAZARD_COLD_NAME_ANCHORS = [
  patternAnchor("ice", "name"),
  patternAnchor("iced", "name"),
  patternAnchor("frost", "name"),
  patternAnchor("frozen", "name"),
  patternAnchor("freezing", "name"),
  patternAnchor("glacial", "name"),
  patternAnchor("snow", "name"),
  patternAnchor("hail", "name"),
  patternAnchor("blizzard", "name"),
];

const HAZARD_COLD_TEXT_ANCHORS = [
  patternAnchor("ice"),
  patternAnchor("frost"),
  patternAnchor("frozen"),
  patternAnchor("freezing"),
  patternAnchor("glacial"),
  patternAnchor("snow"),
  patternAnchor("hail"),
  patternAnchor("blizzard"),
  patternAnchor("frigid"),
  patternAnchor("chill"),
  patternAnchor("icy"),
  patternAnchor("thin ice"),
  patternAnchor("freezing floor"),
  patternAnchor("freezing floor tiles"),
  patternAnchor("ice fall"),
  patternAnchor("ice fall trap"),
];

const HAZARD_ELECTRIC_NAME_ANCHORS = [
  patternAnchor("electric", "name"),
  patternAnchor("lightning", "name"),
  patternAnchor("shock", "name"),
  patternAnchor("shocking", "name"),
  patternAnchor("static", "name"),
  patternAnchor("spark", "name"),
  patternAnchor("discharge", "name"),
];

const HAZARD_ELECTRIC_TEXT_ANCHORS = [
  patternAnchor("electric"),
  patternAnchor("electricity"),
  patternAnchor("lightning"),
  patternAnchor("shock"),
  patternAnchor("shocking"),
  patternAnchor("static"),
  patternAnchor("spark"),
  patternAnchor("sparking"),
  patternAnchor("discharge"),
  patternAnchor("current"),
  patternAnchor("arc"),
  patternAnchor("electric shock"),
  patternAnchor("lightning bolt"),
  patternAnchor("shocking rune"),
  patternAnchor("shock glyph"),
  patternAnchor("electricity crackles"),
];

const HAZARD_SOUND_NAME_ANCHORS = [
  patternAnchor("sonic", "name"),
  patternAnchor("sound", "name"),
  patternAnchor("vibration", "name"),
  patternAnchor("resonance", "name"),
  patternAnchor("buzzing", "name"),
  patternAnchor("shrieking", "name"),
  patternAnchor("wail", "name"),
  patternAnchor("harmonic", "name"),
  patternAnchor("symphony", "name"),
];

const HAZARD_SOUND_TEXT_ANCHORS = [
  patternAnchor("sonic"),
  patternAnchor("sound"),
  patternAnchor("soundwave"),
  patternAnchor("soundwaves"),
  patternAnchor("vibration"),
  patternAnchor("vibrations"),
  patternAnchor("resonance"),
  patternAnchor("resonant"),
  patternAnchor("buzzing"),
  patternAnchor("buzz"),
  patternAnchor("shriek"),
  patternAnchor("shrieking"),
  patternAnchor("wail"),
  patternAnchor("wailing"),
  patternAnchor("deafening"),
  patternAnchor("ringing"),
  patternAnchor("drone"),
  patternAnchor("hum"),
  patternAnchor("sonic damage"),
  patternAnchor("sound waves"),
  patternAnchor("deafening noise"),
  patternAnchor("ringing in the ears"),
  patternAnchor("vibrating floor"),
];

const HAZARD_RESPIRATORY_NAME_ANCHORS = [
  patternAnchor("smoke", "name"),
  patternAnchor("breathless", "name"),
  patternAnchor("vapor", "name"),
  patternAnchor("fog", "name"),
];

const HAZARD_RESPIRATORY_TEXT_ANCHORS = [
  patternAnchor("smoke-filled"),
  patternAnchor("smoke filled"),
  patternAnchor("noxious vapors"),
  patternAnchor("poison gas"),
  patternAnchor("steals the breath"),
  patternAnchor("can t breathe"),
  patternAnchor("cannot breathe"),
  patternAnchor("difficult to see and breathe"),
  patternAnchor("breathless"),
  patternAnchor("choking smoke"),
  patternAnchor("inhaled"),
];

const HAZARD_WATER_NAME_ANCHORS = [
  patternAnchor("flood", "name"),
  patternAnchor("geyser", "name"),
  patternAnchor("tsunami", "name"),
  patternAnchor("wave", "name"),
];

const HAZARD_WATER_TEXT_ANCHORS = [
  patternAnchor("flash flood"),
  patternAnchor("flood"),
  patternAnchor("flooding"),
  patternAnchor("geyser"),
  patternAnchor("geysers"),
  patternAnchor("megatsunami"),
  patternAnchor("rushing water"),
  patternAnchor("torrent"),
  patternAnchor("surge"),
  patternAnchor("surging water"),
  patternAnchor("wave"),
  patternAnchor("submerged"),
];

const HAZARD_SPAWNED_ATTACKERS_NAME_ANCHORS = [
  patternAnchor("summoning rune", "name"),
  patternAnchor("shadow guards", "name"),
  patternAnchor("spectral archers", "name"),
  patternAnchor("clone mirrors", "name"),
  patternAnchor("darkside mirror", "name"),
];

const HAZARD_SPAWNED_ATTACKERS_TEXT_ANCHORS = [
  patternAnchor("animate into"),
  patternAnchor("animates and clambers out of the painting"),
  patternAnchor("become a real creature"),
  patternAnchor("becomes a real creature"),
  patternAnchor("clockwork humanoid figures"),
  patternAnchor("move throughout the area"),
  patternAnchor("summons"),
  patternAnchor("calls forth"),
  patternAnchor("conjures"),
  patternAnchor("spawns"),
  patternAnchor("peel themselves from the floor and attack"),
  patternAnchor("replaced by its victim"),
  patternAnchor("masked guardians"),
];

const HAZARD_NAVIGATION_DISRUPTION_TEXT_ANCHORS = [
  patternAnchor("confound creatures into circling the room"),
  patternAnchor("attempted to exit"),
  patternAnchor("disorienting illusions"),
  patternAnchor("maze of mirrors"),
  patternAnchor("shadow maze"),
  patternAnchor("endless realities"),
  patternAnchor("walls shift"),
  patternAnchor("ever shifting maze"),
  patternAnchor("walls vanish"),
  patternAnchor("circling the room"),
];

const HAZARD_OVERHEAD_STRIKE_NAME_ANCHORS = [
  patternAnchor("falling chandelier", "name"),
  patternAnchor("falling debris", "name"),
  patternAnchor("falling crates", "name"),
  patternAnchor("pendulum blades", "name"),
  patternAnchor("rockfall ceiling", "name"),
  patternAnchor("deadfall", "name"),
  patternAnchor("falling ceiling", "name"),
];

const HAZARD_OVERHEAD_STRIKE_TEXT_ANCHORS = [
  patternAnchor("ceiling"),
  patternAnchor("chandelier"),
  patternAnchor("crashes down"),
  patternAnchor("dropped on"),
  patternAnchor("from above"),
  patternAnchor("held up by"),
  patternAnchor("pendulum"),
  patternAnchor("slats in the ceiling open"),
  patternAnchor("bundle of boulders"),
];

const HAZARD_PROJECTILE_EMITTER_NAME_ANCHORS = [
  patternAnchor("ballista trap", "name"),
  patternAnchor("dart barrage", "name"),
  patternAnchor("dart launcher", "name"),
  patternAnchor("flame projector", "name"),
  patternAnchor("hail of darts", "name"),
  patternAnchor("spear launcher", "name"),
  patternAnchor("spike thrower", "name"),
  patternAnchor("turret", "name"),
];

const HAZARD_PROJECTILE_EMITTER_TEXT_ANCHORS = [
  patternAnchor("ballista"),
  patternAnchor("bolt launcher"),
  patternAnchor("ceiling mounted darts"),
  patternAnchor("dart launcher"),
  patternAnchor("dart loaded pipes"),
  patternAnchor("expel darts"),
  patternAnchor("fires darts"),
  patternAnchor("fixed weapon"),
  patternAnchor("flame jet"),
  patternAnchor("flame projector"),
  patternAnchor("launches bolts"),
  patternAnchor("launches spears"),
  patternAnchor("loaded with a wooden spear"),
  patternAnchor("mounted cannon"),
  patternAnchor("nozzle"),
  patternAnchor("pepper the chamber"),
  patternAnchor("projector"),
  patternAnchor("rake across the room"),
  patternAnchor("spring loaded tubes"),
  patternAnchor("sprays flames"),
  patternAnchor("turret"),
];

const HAZARD_ILLUSION_ASSAULT_TEXT_ANCHORS = [
  patternAnchor("assaults trespassers with terrible illusions"),
  patternAnchor("distort a viewer s reflection"),
  patternAnchor("painfully reshaping their body"),
  patternAnchor("reflection in the mirror subtly twists and distorts"),
  patternAnchor("reflected images"),
  patternAnchor("reflects the viewer s deepest desires"),
  patternAnchor("twists and distorts"),
];

const HAUNT_LIFE_DRAIN_TEXT_ANCHORS = [
  patternAnchor("draining their life force away"),
  patternAnchor("draining life force"),
  patternAnchor("life-draining light"),
  patternAnchor("soul-draining light"),
  patternAnchor("drain life force"),
  patternAnchor("drains life force"),
  patternAnchor("drain vitality"),
  patternAnchor("drains vitality"),
  patternAnchor("siphon souls"),
  patternAnchor("siphons souls"),
  patternAnchor("siphoning souls"),
  patternAnchor("inhales blood"),
  patternAnchor("inhales blood from living creatures"),
];

const HAUNT_PHANTOM_ASSAILANT_TEXT_ANCHORS = [
  patternAnchor("spectral assailant"),
  patternAnchor("ghostly attackers"),
  patternAnchor("ghostly orcs"),
  patternAnchor("phantom hounds"),
  patternAnchor("cannibalistic spirits swarm"),
  patternAnchor("spectral undead converge"),
  patternAnchor("host of angry spirits"),
  patternAnchor("angry spirits"),
  patternAnchor("ghostly pair"),
];

const HAUNT_LURE_COMPULSION_TEXT_ANCHORS = [
  patternAnchor("beckons anyone"),
  patternAnchor("compels all who hear it to dance"),
  patternAnchor("forces them to join the battle"),
  patternAnchor("pull someone down into the water to join them"),
  patternAnchor("join her in the pool"),
  patternAnchor("join the battle"),
  patternAnchor("must dance"),
];

const HAUNT_BATTLEFIELD_DISRUPTION_TEXT_ANCHORS = [
  patternAnchor("battle cries"),
  patternAnchor("knock the tables about"),
  patternAnchor("send dishes flying"),
  patternAnchor("pull fleeing creatures into the room"),
  patternAnchor("all allies have vanished"),
  patternAnchor("murderous orcs"),
  patternAnchor("revisiting old wounds"),
  patternAnchor("shifts from a harmless patch of terrain into something truly deadly"),
  patternAnchor("phantasmal crimson worm"),
  patternAnchor("mindscape draws from memories of pain and agony"),
];

export const HAZARD_DERIVED_TAG_RULES: DerivedTagRule[] = [
  {
    tag: "ward_trigger",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 1,
        textAny: [
          patternAnchor("rune"),
          patternAnchor("glyph"),
          patternAnchor("sigil"),
          patternAnchor("ward"),
          patternAnchor("seal"),
          patternAnchor("symbol"),
          patternAnchor("inscription"),
        ],
      },
      {
        score: 2,
        textAny: [
          patternAnchor("when triggered"),
          patternAnchor("when activated"),
          patternAnchor("when touched"),
          patternAnchor("when opened"),
          patternAnchor("when approached"),
          patternAnchor("when a creature crosses"),
          patternAnchor("crosses the threshold"),
          patternAnchor("crosses the ward"),
          patternAnchor("hidden rune"),
          patternAnchor("invisible rune"),
          patternAnchor("etched rune"),
          patternAnchor("upon being disturbed"),
          patternAnchor("set off the rune"),
          patternAnchor("set off the glyph"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("rune"),
              patternAnchor("glyph"),
              patternAnchor("sigil"),
              patternAnchor("ward"),
              patternAnchor("seal"),
              patternAnchor("symbol"),
              patternAnchor("inscription"),
            ],
            window: 6,
            scope: "description",
          },
          {
            all: [
              patternAnchor("trigger"),
              patternAnchor("triggered"),
              patternAnchor("activate"),
              patternAnchor("activated"),
              patternAnchor("touch"),
              patternAnchor("touches"),
              patternAnchor("step"),
              patternAnchor("steps"),
              patternAnchor("enter"),
              patternAnchor("enters"),
              patternAnchor("approach"),
              patternAnchor("approaches"),
              patternAnchor("open"),
              patternAnchor("opened"),
              patternAnchor("disturb"),
              patternAnchor("disturbed"),
              patternAnchor("cross"),
              patternAnchor("crosses"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [
      { textAny: [patternAnchor("protective ward"), patternAnchor("warding bell"), patternAnchor("warded against")] },
    ],
  },
  {
    tag: "pressure_trigger",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          patternAnchor("pressure plate"),
          patternAnchor("pressure plates"),
          patternAnchor("pressure-sensitive"),
          patternAnchor("pressure sensitive"),
          patternAnchor("stepping on the plate"),
          patternAnchor("when stepped on"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("pressure"),
              patternAnchor("plate"),
              patternAnchor("plates"),
              patternAnchor("floor panel"),
              patternAnchor("stone tile"),
            ],
            window: 6,
            scope: "description",
          },
          {
            all: [
              patternAnchor("step"),
              patternAnchor("steps"),
              patternAnchor("stepped"),
              patternAnchor("trigger"),
              patternAnchor("triggered"),
              patternAnchor("depresses"),
              patternAnchor("pressed"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "tripwire_trigger",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          patternAnchor("tripwire"),
          patternAnchor("trip wire"),
          patternAnchor("tension wire"),
          patternAnchor("taut wire"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("wire"),
              patternAnchor("cord"),
              patternAnchor("line"),
              patternAnchor("tripwire"),
            ],
            window: 5,
            scope: "description",
          },
          {
            all: [
              patternAnchor("trip"),
              patternAnchor("trips"),
              patternAnchor("snag"),
              patternAnchor("snags"),
              patternAnchor("cross"),
              patternAnchor("crosses"),
              patternAnchor("trigger"),
              patternAnchor("triggered"),
            ],
            window: 5,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "threshold_lockdown",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 1,
        textAny: [
          patternAnchor("door"),
          patternAnchor("doors"),
          patternAnchor("gate"),
          patternAnchor("gates"),
          patternAnchor("threshold"),
          patternAnchor("thresholds"),
          patternAnchor("doorway"),
          patternAnchor("doorways"),
          patternAnchor("entrance"),
          patternAnchor("entrances"),
          patternAnchor("portcullis"),
          patternAnchor("latch"),
          patternAnchor("lock"),
          patternAnchor("locks"),
        ],
      },
      {
        score: 2,
        textAny: [
          patternAnchor("slams shut"),
          patternAnchor("locks shut"),
          patternAnchor("seals the door"),
          patternAnchor("seals the doorway"),
          patternAnchor("seals the entrance"),
          patternAnchor("blocks the doorway"),
          patternAnchor("blocks the entrance"),
          patternAnchor("bars the passage"),
          patternAnchor("bars the door"),
          patternAnchor("drops a portcullis"),
          patternAnchor("pushes the iron doors closed"),
          patternAnchor("trap the triggering creature inside"),
          patternAnchor("traps the triggering creature inside"),
          patternAnchor("entry and exit seal"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("door"),
              patternAnchor("doors"),
              patternAnchor("gate"),
              patternAnchor("gates"),
              patternAnchor("threshold"),
              patternAnchor("thresholds"),
              patternAnchor("doorway"),
              patternAnchor("entrance"),
              patternAnchor("portcullis"),
              patternAnchor("latch"),
            ],
            window: 5,
            scope: "description",
          },
          {
            all: [
              patternAnchor("lock"),
              patternAnchor("locks"),
              patternAnchor("locked"),
              patternAnchor("seal"),
              patternAnchor("seals"),
              patternAnchor("sealed"),
              patternAnchor("close"),
              patternAnchor("closes"),
              patternAnchor("closed"),
              patternAnchor("shut"),
              patternAnchor("bar"),
              patternAnchor("bars"),
              patternAnchor("drop"),
              patternAnchor("drops"),
              patternAnchor("block"),
              patternAnchor("blocks"),
            ],
            window: 5,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [
      { textAny: [patternAnchor("cave in"), patternAnchor("collapses the tunnel"), patternAnchor("collapse the passage")] },
    ],
  },
  {
    tag: "control_interface",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 1,
        textAny: [
          patternAnchor("button"),
          patternAnchor("buttons"),
          patternAnchor("lever"),
          patternAnchor("levers"),
          patternAnchor("console"),
          patternAnchor("consoles"),
          patternAnchor("panel"),
          patternAnchor("panels"),
          patternAnchor("switch"),
          patternAnchor("switches"),
          patternAnchor("dial"),
          patternAnchor("dials"),
          patternAnchor("crank"),
          patternAnchor("cranks"),
          patternAnchor("wheel"),
          patternAnchor("wheels"),
          patternAnchor("tumbler"),
          patternAnchor("tumblers"),
        ],
      },
      {
        score: 2,
        textAny: [
          patternAnchor("press a button"),
          patternAnchor("press the button"),
          patternAnchor("push the button"),
          patternAnchor("pull a lever"),
          patternAnchor("pull the lever"),
          patternAnchor("flip the switch"),
          patternAnchor("operate the console"),
          patternAnchor("control panel"),
          patternAnchor("command console"),
          patternAnchor("turn the crank"),
          patternAnchor("set the tumbler"),
          patternAnchor("button mash"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(button,buttons,lever,levers,console,consoles,panel,panels,switch,switches,dial,dials,crank,cranks,wheel,wheels,tumbler,tumblers)}}"),
              patternAnchor("{{alt(press,presses,push,pushes,pull,pulls,flip,flips,turn,turns,operate,operates,activate,activates,control,controls,set)}}"),
            ],
            window: 5,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "planar_breach",
    category: "hazard",
    threshold: 2,
    anyOf: [
      {
        score: 1,
        textAny: [
          patternAnchor("portal"),
          patternAnchor("portals"),
          patternAnchor("rift"),
          patternAnchor("rifts"),
          patternAnchor("breach"),
          patternAnchor("breaches"),
          patternAnchor("tear"),
          patternAnchor("tears"),
          patternAnchor("fissure"),
          patternAnchor("fissures"),
          patternAnchor("opening"),
          patternAnchor("openings"),
          patternAnchor("void"),
          patternAnchor("aperture"),
        ],
      },
      {
        score: 2,
        textAny: [
          patternAnchor("planar breach"),
          patternAnchor("open a portal"),
          patternAnchor("opens a portal"),
          patternAnchor("open a rift"),
          patternAnchor("opens a rift"),
          patternAnchor("tear in reality"),
          patternAnchor("tears in reality"),
          patternAnchor("tear in the fabric of reality"),
          patternAnchor("tear the fabric of reality"),
          patternAnchor("reality tears open"),
          patternAnchor("unstable rift"),
          patternAnchor("unstable portal"),
          patternAnchor("hole in reality"),
          patternAnchor("breach in reality"),
          patternAnchor("gate to the"),
          patternAnchor("portal network"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("portal"),
              patternAnchor("portals"),
              patternAnchor("rift"),
              patternAnchor("rifts"),
              patternAnchor("breach"),
              patternAnchor("breaches"),
              patternAnchor("tear"),
              patternAnchor("tears"),
              patternAnchor("fissure"),
              patternAnchor("fissures"),
              patternAnchor("void"),
              patternAnchor("opening"),
              patternAnchor("openings"),
            ],
            window: 6,
            scope: "description",
          },
          {
            all: [
              patternAnchor("open"),
              patternAnchor("opens"),
              patternAnchor("opening"),
              patternAnchor("opened"),
              patternAnchor("unstable"),
              patternAnchor("tear"),
              patternAnchor("tears"),
              patternAnchor("breach"),
              patternAnchor("breaches"),
              patternAnchor("seam"),
              patternAnchor("seams"),
              patternAnchor("reality"),
              patternAnchor("planar"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [
      { textAny: [patternAnchor("teleportation circle"), patternAnchor("safe portal"), patternAnchor("portal travel"), patternAnchor("protective ward"), patternAnchor("ward against portals")] },
    ],
  },
  {
    tag: "alarm",
    category: "hazard",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: HAZARD_ALARM_TEXT_ANCHORS },
      { score: 1, textAny: [patternAnchor("glyph"), patternAnchor("ward"), patternAnchor("threshold")] },
    ],
  },
  {
    tag: "spawned_attackers",
    category: "hazard",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: HAZARD_SPAWNED_ATTACKERS_NAME_ANCHORS },
      { score: 2, textAny: HAZARD_SPAWNED_ATTACKERS_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("summon"),
              patternAnchor("summons"),
              patternAnchor("call"),
              patternAnchor("calls"),
              patternAnchor("conjure"),
              patternAnchor("spawns"),
              patternAnchor("replaced"),
              patternAnchor("peel"),
              patternAnchor("peel themselves"),
              patternAnchor("guards"),
              patternAnchor("guardians"),
              patternAnchor("archers"),
              patternAnchor("devil"),
              patternAnchor("duplicates"),
              patternAnchor("shadows"),
              patternAnchor("attack"),
              patternAnchor("attacker"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "life_drain_hazard",
    category: "hazard",
    subcategories: ["haunt"],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: HAUNT_LIFE_DRAIN_TEXT_ANCHORS },
    ],
  },
  {
    tag: "phantom_assailants",
    category: "hazard",
    subcategories: ["haunt"],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: HAUNT_PHANTOM_ASSAILANT_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("ghostly"),
              patternAnchor("spectral"),
              patternAnchor("phantom"),
              patternAnchor("spirit"),
              patternAnchor("spirits"),
              patternAnchor("undead"),
              patternAnchor("apparition"),
              patternAnchor("wraith"),
            ],
            window: 5,
            scope: "description",
          },
          {
            all: [
              patternAnchor("attack"),
              patternAnchor("attacks"),
              patternAnchor("attacker"),
              patternAnchor("attackers"),
              patternAnchor("assailant"),
              patternAnchor("assailants"),
              patternAnchor("swarm"),
              patternAnchor("swarms"),
              patternAnchor("chase"),
              patternAnchor("rush"),
              patternAnchor("slaughter"),
              patternAnchor("devour"),
              patternAnchor("converge"),
              patternAnchor("crush"),
              patternAnchor("bite"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "lure_compulsion",
    category: "hazard",
    subcategories: ["haunt"],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: HAUNT_LURE_COMPULSION_TEXT_ANCHORS },
    ],
  },
  {
    tag: "battlefield_disruption",
    category: "hazard",
    subcategories: ["haunt"],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: HAUNT_BATTLEFIELD_DISRUPTION_TEXT_ANCHORS },
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
          patternAnchor("bind intruders"),
          patternAnchor("holds intruders in place"),
          patternAnchor("hold creatures in place"),
          patternAnchor("lashes out with force bands"),
          patternAnchor("until they escape"),
        ],
        referencesAny: RESTRAINT_CAPTURE_REFERENCE_ANCHORS,
      },
      {
        score: 2,
        textAny: [
          patternAnchor("creature becomes restrained"),
          patternAnchor("target becomes restrained"),
          patternAnchor("immobilized"),
          patternAnchor("held fast"),
          patternAnchor("ensnared"),
          patternAnchor("entangled"),
          patternAnchor("stuck in place"),
          patternAnchor("stuck fast"),
          patternAnchor("pins the creature"),
          patternAnchor("cage"),
          patternAnchor("webs"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("restrained"),
              patternAnchor("immobilized"),
              patternAnchor("grabbed"),
              patternAnchor("stuck"),
              patternAnchor("web"),
              patternAnchor("webs"),
              patternAnchor("tar"),
              patternAnchor("cage"),
              patternAnchor("ensnare"),
              patternAnchor("ensnared"),
            ],
            window: 5,
            scope: "description",
          },
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
          patternAnchor("slams shut"),
          patternAnchor("slam down into place"),
          patternAnchor("sealing the entrance"),
          patternAnchor("entry and exit seal with a force barrier"),
          patternAnchor("magically seals the door"),
          patternAnchor("block progress through this area"),
          patternAnchor("filling the passage with stone"),
          patternAnchor("trap the triggering creature inside"),
          patternAnchor("imprisons intruders"),
          patternAnchor("pushes two iron doors closed"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            all: [patternAnchor("portcullis"), patternAnchor("drops")],
            window: 4,
            scope: "description",
          },
          {
            all: [patternAnchor("portcullis"), patternAnchor("slam")],
            window: 6,
            scope: "description",
          },
          {
            all: [patternAnchor("door"), patternAnchor("locks")],
            window: 3,
            scope: "description",
          },
          {
            all: [patternAnchor("door"), patternAnchor("seal")],
            window: 4,
            scope: "description",
          },
          {
            all: [patternAnchor("gate"), patternAnchor("shut")],
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
            all: [
              patternAnchor("acid"),
              patternAnchor("acidic"),
              patternAnchor("caustic"),
              patternAnchor("corrosive"),
              patternAnchor("corroding"),
            ],
            window: 5,
            scope: "description",
          },
          {
            all: [
              patternAnchor("spray"),
              patternAnchor("mist"),
              patternAnchor("cloud"),
              patternAnchor("pool"),
              patternAnchor("shower"),
              patternAnchor("splash"),
              patternAnchor("runoff"),
            ],
            window: 5,
            scope: "description",
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
            all: [
              patternAnchor("ice"),
              patternAnchor("frost"),
              patternAnchor("frozen"),
              patternAnchor("freezing"),
              patternAnchor("glacial"),
              patternAnchor("snow"),
              patternAnchor("hail"),
              patternAnchor("blizzard"),
              patternAnchor("frigid"),
              patternAnchor("chill"),
            ],
            window: 6,
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
            all: [
              patternAnchor("fire"),
              patternAnchor("flame"),
              patternAnchor("flames"),
              patternAnchor("burns"),
              patternAnchor("burning"),
              patternAnchor("ignite"),
              patternAnchor("ignites"),
              patternAnchor("explodes"),
              patternAnchor("spread"),
              patternAnchor("spreads"),
            ],
            window: 6,
            scope: "description",
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
            all: [
              patternAnchor("electric"),
              patternAnchor("electricity"),
              patternAnchor("lightning"),
              patternAnchor("shock"),
              patternAnchor("shocking"),
              patternAnchor("static"),
              patternAnchor("spark"),
              patternAnchor("discharge"),
              patternAnchor("current"),
              patternAnchor("arc"),
            ],
            window: 6,
            scope: "description",
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
            all: [
              patternAnchor("poison"),
              patternAnchor("poisonous"),
              patternAnchor("venom"),
              patternAnchor("toxic"),
              patternAnchor("gas"),
              patternAnchor("smoke"),
              patternAnchor("dart"),
              patternAnchor("darts"),
              patternAnchor("needle"),
              patternAnchor("spine"),
              patternAnchor("vent"),
              patternAnchor("vents"),
              patternAnchor("nozzle"),
              patternAnchor("nozzles"),
              patternAnchor("cloud"),
            ],
            window: 6,
            scope: "description",
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
            all: [
              patternAnchor("sonic"),
              patternAnchor("sound"),
              patternAnchor("soundwave"),
              patternAnchor("vibration"),
              patternAnchor("resonance"),
              patternAnchor("buzzing"),
              patternAnchor("shriek"),
              patternAnchor("wail"),
              patternAnchor("deafening"),
              patternAnchor("ringing"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [
      { textAny: HAZARD_ALARM_TEXT_ANCHORS },
    ],
  },
  {
    tag: "respiratory_hazard",
    category: "hazard",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: HAZARD_RESPIRATORY_NAME_ANCHORS },
      { score: 2, textAny: HAZARD_RESPIRATORY_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("smoke"),
              patternAnchor("vapor"),
              patternAnchor("vapors"),
              patternAnchor("fog"),
              patternAnchor("gas"),
              patternAnchor("air"),
              patternAnchor("breath"),
            ],
            window: 6,
            scope: "description",
          },
          {
            all: [
              patternAnchor("choking"),
              patternAnchor("suffocate"),
              patternAnchor("breathless"),
              patternAnchor("inhaled"),
              patternAnchor("breathe"),
              patternAnchor("difficult"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "water_hazard",
    category: "hazard",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: HAZARD_WATER_NAME_ANCHORS },
      { score: 2, textAny: HAZARD_WATER_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("water"),
              patternAnchor("flood"),
              patternAnchor("geyser"),
              patternAnchor("wave"),
              patternAnchor("torrent"),
              patternAnchor("surge"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [
      { textAny: [patternAnchor("blood"), patternAnchor("tar"), patternAnchor("lava"), patternAnchor("magma")] },
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
          patternAnchor("trapdoor covers a pit"),
          patternAnchor("covers a pit"),
          patternAnchor("pit filled with spikes"),
          patternAnchor("falls into the pit"),
          patternAnchor("drops into the pit"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(pit,trapdoor)}}"),
              patternAnchor("{{alt(deep,spikes,water,fall,falls,drop,drops,covers,conceals)}}"),
            ],
            window: 6,
            scope: "description",
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
          patternAnchor("ceiling collapses"),
          patternAnchor("triggers a cave in"),
          patternAnchor("structure to collapse"),
          patternAnchor("collapse into rubble"),
          patternAnchor("bridge itself groans and shakes then crumbles"),
          patternAnchor("collapse inward"),
          patternAnchor("floor collapses"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("ceiling"),
              patternAnchor("floor"),
              patternAnchor("bridge"),
              patternAnchor("stairs"),
              patternAnchor("supports"),
              patternAnchor("tunnel"),
              patternAnchor("cavern"),
              patternAnchor("structure"),
              patternAnchor("pillar"),
              patternAnchor("collapse"),
              patternAnchor("collapses"),
              patternAnchor("crumble"),
              patternAnchor("crumbles"),
              patternAnchor("fall"),
              patternAnchor("falls"),
            ],
            window: 5,
            scope: "description",
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
            all: [
              patternAnchor("sucks"),
              patternAnchor("pulls"),
              patternAnchor("pushes"),
              patternAnchor("drags"),
              patternAnchor("sweeps"),
              patternAnchor("submerge"),
              patternAnchor("trample"),
              patternAnchor("trampling"),
              patternAnchor("creatures"),
              patternAnchor("toward"),
              patternAnchor("away"),
              patternAnchor("into"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "navigation_disruption",
    category: "hazard",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: HAZARD_NAVIGATION_DISRUPTION_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("maze"),
              patternAnchor("mirror"),
              patternAnchor("mirrors"),
              patternAnchor("walls"),
              patternAnchor("portal"),
              patternAnchor("hallway"),
              patternAnchor("confound"),
              patternAnchor("disorient"),
              patternAnchor("circling"),
              patternAnchor("shift"),
              patternAnchor("shifting"),
              patternAnchor("vanish"),
              patternAnchor("exit"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [
      { textAny: HAZARD_SPAWNED_ATTACKERS_TEXT_ANCHORS },
    ],
  },
  {
    tag: "illusion_assault",
    category: "hazard",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: HAZARD_ILLUSION_ASSAULT_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("illusion"),
              patternAnchor("illusions"),
              patternAnchor("mirror"),
              patternAnchor("mirrors"),
              patternAnchor("reflection"),
              patternAnchor("reflections"),
              patternAnchor("phantasm"),
            ],
            window: 6,
            scope: "description",
          },
          {
            all: [
              patternAnchor("assault"),
              patternAnchor("distort"),
              patternAnchor("distorts"),
              patternAnchor("malice"),
              patternAnchor("desires"),
              patternAnchor("reshape"),
              patternAnchor("reshaping"),
            ],
            window: 8,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [
      { textAny: HAZARD_SPAWNED_ATTACKERS_TEXT_ANCHORS },
    ],
  },
  {
    tag: "overhead_strike",
    category: "hazard",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: HAZARD_OVERHEAD_STRIKE_NAME_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(ceiling,overhead,above,beam,chandelier,rope pulley,pendulum)}}"),
              patternAnchor("{{alt(fall,falls,falling,drops,crashes down,dropped,rocks,debris,crates)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
      { score: 1, textAny: HAZARD_OVERHEAD_STRIKE_TEXT_ANCHORS },
    ],
    noneOf: [
      { textAny: [patternAnchor("floor collapses"), patternAnchor("drop a creature"), patternAnchor("drops a creature"), patternAnchor("pit")] },
    ],
  },
  {
    tag: "projectile_emitter",
    category: "hazard",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: HAZARD_PROJECTILE_EMITTER_NAME_ANCHORS },
      { score: 2, textAny: HAZARD_PROJECTILE_EMITTER_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("turret"),
              patternAnchor("launcher"),
              patternAnchor("projector"),
              patternAnchor("emitter"),
              patternAnchor("nozzle"),
              patternAnchor("ballista"),
              patternAnchor("cannon"),
            ],
            window: 6,
            scope: "description",
          },
          {
            all: [
              patternAnchor("fires"),
              patternAnchor("launches"),
              patternAnchor("sprays"),
              patternAnchor("shoots"),
              patternAnchor("emits"),
              patternAnchor("bolt"),
              patternAnchor("dart"),
              patternAnchor("flame"),
              patternAnchor("spear"),
              patternAnchor("spike"),
              patternAnchor("projectile"),
            ],
            window: 6,
            scope: "description",
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
          patternAnchor("damages the mind"),
          patternAnchor("filling the heads"),
          patternAnchor("fear"),
          patternAnchor("frightened"),
          patternAnchor("flood the minds"),
          patternAnchor("paranoia"),
          patternAnchor("confused"),
          patternAnchor("confusion"),
          patternAnchor("hallucination"),
          patternAnchor("hallucinations"),
          patternAnchor("maddening visions"),
          patternAnchor("mental trauma"),
          patternAnchor("psychic scream"),
          patternAnchor("disorients"),
        ],
      },
      {
        score: 1,
        textAny: [
          patternAnchor("psychic"),
          patternAnchor("mental"),
          patternAnchor("mind"),
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
          patternAnchor("paralyzed"),
          patternAnchor("immobilized"),
          patternAnchor("restrained"),
          patternAnchor("grabbed"),
          patternAnchor("holds the creature in place"),
          patternAnchor("holds intruders in place"),
          patternAnchor("hold creatures in place"),
          patternAnchor("attempting to restrain nearby creatures"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            all: [patternAnchor("hamper"), patternAnchor("creatures")],
            window: 3,
            scope: "description",
          },
        ],
      },
      {
        score: 1,
        textAny: [
          patternAnchor("slowed"),
        ],
      },
    ],
  },
];
