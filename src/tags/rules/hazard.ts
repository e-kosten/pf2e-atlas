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

const HAZARD_BARRIER_LOCKDOWN_NAME_ANCHORS = [
  patternAnchor("crushing gate", "name"),
  patternAnchor("slamming door", "name"),
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
  patternAnchor("conjuring"),
  patternAnchor("spawns"),
  patternAnchor("into being"),
  patternAnchor("ersatz ghost"),
  patternAnchor("animate dreams"),
  patternAnchor("disgorge"),
  patternAnchor("disgorges"),
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
  patternAnchor("phantom soldiers"),
  patternAnchor("phantom hounds"),
  patternAnchor("cannibalistic spirits swarm"),
  patternAnchor("spectral undead converge"),
  patternAnchor("host of angry spirits"),
  patternAnchor("angry spirits"),
  patternAnchor("ghostly pair"),
  patternAnchor("spirits of long-dead soldiers appear in the mist and attack intruders"),
  patternAnchor("ghostly soldiers wielding crossbows manifest"),
  patternAnchor("rain bolts down upon intruders"),
  patternAnchor("rise and resume their final battle"),
  patternAnchor("rise and begin a deadly brawl"),
  patternAnchor("ghostly kobolds rise from the rubble"),
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
              patternAnchor("{{alt(rune,glyph,sigil,ward,seal,symbol,inscription)}}"),
              patternAnchor("{{alt(trigger,triggered,activate,activated,touch,touches,step,steps,enter,enters,approach,approaches,open,opened,disturb,disturbed,cross,crosses)}}"),
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
              patternAnchor("{{alt(pressure,plate,plates,floor panel,stone tile)}}"),
              patternAnchor("{{alt(step,steps,stepped,trigger,triggered,depresses,pressed)}}"),
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
              patternAnchor("{{alt(wire,cord,line,tripwire)}}"),
              patternAnchor("{{alt(trip,trips,snag,snags,cross,crosses,trigger,triggered)}}"),
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
              patternAnchor("{{alt(door,doors,gate,gates,threshold,thresholds,doorway,entrance,portcullis,latch)}}"),
              patternAnchor("{{alt(lock,locks,locked,seal,seals,sealed,close,closes,closed,shut,bar,bars,drop,drops,block,blocks)}}"),
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
          patternAnchor("controlled by a lever"),
          patternAnchor("triggered manually"),
          patternAnchor("bypass button"),
          patternAnchor("slides the vault into view"),
        ],
      },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(button,buttons,lever,levers,console,consoles,panel,panels,switch,switches,dial,dials,crank,cranks,wheel,wheels,tumbler,tumblers)}}"),
              patternAnchor("{{alt(press,presses,push,pushes,pull,pulls,flip,flips,turn,turns,operate,operates,activate,activates,control,controls,controlled,set,slide,slides,deactivate,deactivates)}}"),
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
              patternAnchor("{{alt(portal,portals,rift,rifts,breach,breaches,tear,tears,fissure,fissures,void,opening,openings)}}"),
              patternAnchor("{{alt(open,opens,opening,opened,unstable,seam,seams,reality,planar)}}"),
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
              patternAnchor("{{alt(summon,summons,call,calls,conjure,conjures,conjuring,spawn,spawns,replaced,peel,peel themselves,disgorge,disgorges)}}"),
              patternAnchor("{{alt(guards,guardians,archers,devil,demons,duplicates,dreams,ghost,shadows,attack,attacker,attackers,creature,creatures,being)}}"),
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
              patternAnchor("{{alt(ghostly,spectral,phantom,spirit,spirits,undead,apparition,wraith)}}"),
              patternAnchor("{{alt(attack,attacks,attacker,attackers,assailant,assailants,swarm,swarms,chase,rush,slaughter,devour,converge,crush,bite,brawl,battle,soldiers,archers,bolts)}}"),
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
              patternAnchor("{{alt(restrained,immobilized,grabbed,stuck,web,webs,tar,cage,ensnare,ensnared)}}"),
              patternAnchor("{{alt(creature,target,escape,held fast,tears free)}}"),
            ],
            window: 6,
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
        textAny: HAZARD_BARRIER_LOCKDOWN_NAME_ANCHORS,
      },
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
          patternAnchor("snap shut and trap the triggering creature inside"),
          patternAnchor("doors to fall forward from their gate"),
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
          {
            all: [patternAnchor("{{alt(door,doors,gate,gates)}}"), patternAnchor("{{alt(fall forward,falls forward)}}")],
            window: 6,
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
              patternAnchor("{{alt(acid,acidic,caustic,corrosive,corroding)}}"),
              patternAnchor("{{alt(spray,mist,cloud,pool,shower,splash,runoff)}}"),
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
              patternAnchor("{{alt(ice,frost,frozen,freezing,glacial,snow,hail,blizzard,frigid,chill)}}"),
              patternAnchor("{{alt(floor,water,wind,air,sheet,storm)}}"),
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
              patternAnchor("{{alt(fire,flame,flames,burns,burning,ignite,ignites)}}"),
              patternAnchor("{{alt(explodes,spread,spreads,burst,blast)}}"),
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
              patternAnchor("{{alt(electric,electricity,lightning,static,spark,discharge,current,arc)}}"),
              patternAnchor("{{alt(shock,shocking,surge,crackles,crackle)}}"),
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
              patternAnchor("{{alt(poison,poisonous,venom,toxic)}}"),
              patternAnchor("{{alt(gas,smoke,dart,darts,needle,spine,vent,vents,nozzle,nozzles,cloud)}}"),
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
              patternAnchor("{{alt(sonic,sound,soundwave,vibration,resonance,buzzing,shriek,wail,ringing)}}"),
              patternAnchor("{{alt(deafening,burst,blast,scream,shrieking)}}"),
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
              patternAnchor("{{alt(smoke,vapor,vapors,fog,gas,air,breath)}}"),
              patternAnchor("{{alt(choking,suffocate,breathless,inhaled,breathe,difficult)}}"),
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
              patternAnchor("{{alt(water,flood,geyser,wave,torrent,surge)}}"),
              patternAnchor("{{alt(erupts,erupts upward,fills,drenches,chamber,floor)}}"),
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
              patternAnchor("{{alt(ceiling,floor,bridge,stairs,supports,tunnel,cavern,structure,pillar)}}"),
              patternAnchor("{{alt(collapse,collapses,crumble,crumbles,fall,falls)}}"),
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
              patternAnchor("{{alt(sucks,pulls,pushes,drags,sweeps,submerge,trample,trampling)}}"),
              patternAnchor("{{alt(creature,creatures,toward,away,into)}}"),
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
              patternAnchor("{{alt(maze,mirror,mirrors,walls,portal,hallway)}}"),
              patternAnchor("{{alt(confound,disorient,circling,shift,shifting,vanish,exit)}}"),
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
              patternAnchor("{{alt(illusion,illusions,mirror,mirrors,reflection,reflections,phantasm)}}"),
              patternAnchor("{{alt(assault,distort,distorts,malice,desires,reshape,reshaping)}}"),
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
              patternAnchor("{{alt(turret,launcher,projector,emitter,nozzle,ballista,cannon)}}"),
              patternAnchor("{{alt(fires,launches,sprays,shoots,emits,bolt,dart,flame,spear,spike,projectile)}}"),
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
