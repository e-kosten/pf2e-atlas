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
} from "../runtime/shared.js";

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
  patternAnchor("counteract an effect of your choice imposing one of these conditions"),
  patternAnchor("counteract an effect applying one of the following conditions"),
  patternAnchor("free the target s limbs from ailments that impede mobility"),
  patternAnchor("drive mental contamination from the target s mind"),
];

const ANTI_POISON_TEXT_ANCHORS = [
  patternAnchor("cure poison"),
  patternAnchor("counteract poison"),
  patternAnchor("neutralize poison"),
  patternAnchor("against poison"),
  patternAnchor("against poisons"),
  patternAnchor("persistent poison damage"),
];

const ANTI_DISEASE_TEXT_ANCHORS = [
  patternAnchor("cure disease"),
  patternAnchor("counteract disease"),
  patternAnchor("counteract a disease"),
  patternAnchor("neutralize disease"),
  patternAnchor("against disease"),
  patternAnchor("against diseases"),
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

const SPELL_STEALTH_SUPPORT_TEXT_ANCHORS = [
  patternAnchor("move quietly"),
  patternAnchor("avoid notice"),
  patternAnchor("without drawing attention"),
  patternAnchor("unnoticed"),
  patternAnchor("remain unnoticed"),
  patternAnchor("stay unnoticed"),
  patternAnchor("muffle the sound"),
  patternAnchor("silent movement"),
  patternAnchor("stealth"),
  patternAnchor("sneak"),
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

const ELEMENTAL_FORM_TEXT_ANCHORS = [patternAnchor("elemental form"), patternAnchor("elemental battle form")];

const SPELL_MENTAL_IMPAIRMENT_TEXT_ANCHORS = [
  patternAnchor("frightened"),
  patternAnchor("stupefied"),
  patternAnchor("confused"),
  patternAnchor("confusion"),
  patternAnchor("fascinated"),
  patternAnchor("mental damage"),
  patternAnchor("terror"),
  patternAnchor("fear"),
  patternAnchor("forbidden thought"),
  patternAnchor("overwhelmed"),
];

const SPELL_MENTAL_IMPAIRMENT_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("bonus to allies"),
  patternAnchor("drive mental contamination from the target s mind"),
  patternAnchor("grant insight"),
  patternAnchor("share knowledge"),
  patternAnchor("encourage companions"),
  patternAnchor("against emotion effects"),
];

const SPELL_SENSORY_IMPAIRMENT_TEXT_ANCHORS = [
  patternAnchor("blind the target"),
  patternAnchor("blinded"),
  patternAnchor("blindness"),
  patternAnchor("deafened"),
  patternAnchor("deafness"),
  patternAnchor("loses hearing"),
  patternAnchor("{{alt(can't, cannot)}} see"),
  patternAnchor("blocking its vision"),
  patternAnchor("blinding"),
];

const SPELL_SENSORY_IMPAIRMENT_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("see invisible"),
  patternAnchor("reveals hidden"),
  patternAnchor("pierces illusions"),
  patternAnchor("unveiled"),
];

const SPELL_SENSES_SUPPORT_TEXT_ANCHORS = [
  patternAnchor("darkvision"),
  patternAnchor("low-light vision"),
  patternAnchor("low light vision"),
  patternAnchor("see in the dark"),
  patternAnchor("see invisible"),
  patternAnchor("pierce illusions and see invisible"),
  patternAnchor("heighten your senses"),
  patternAnchor("heightened senses"),
  patternAnchor("sharpen your vision"),
  patternAnchor("sharpen your senses"),
  patternAnchor("imprecise scent"),
  patternAnchor("scent with a"),
  patternAnchor("tremorsense"),
  patternAnchor("lifesense"),
];

const SPELL_SENSES_SUPPORT_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("floating eye"),
  patternAnchor("scrying sensor"),
  patternAnchor("transmits what it sees"),
  patternAnchor("see what each target sees"),
  patternAnchor("perceive through"),
  patternAnchor("through the magical eye sensor"),
  patternAnchor("through the scouts"),
];

const SPELL_ILLUMINATION_TEXT_ANCHORS = [
  patternAnchor("bright light"),
  patternAnchor("shed light"),
  patternAnchor("sheds light"),
  patternAnchor("shed bright light"),
  patternAnchor("sheds bright light"),
  patternAnchor("emit light"),
  patternAnchor("emits light"),
  patternAnchor("illuminate the area"),
  patternAnchor("illuminates the area"),
  patternAnchor("create light"),
  patternAnchor("creates light"),
  patternAnchor("produce light"),
  patternAnchor("produces light"),
  patternAnchor("light for the campsite"),
];

const SPELL_FORCED_MOVEMENT_TEXT_ANCHORS = [
  patternAnchor("forced movement"),
  patternAnchor("drag each target directly toward you"),
  patternAnchor("knocks it back"),
  patternAnchor("knocked back"),
  patternAnchor("launching nearby creatures into the sky"),
  patternAnchor("pulls creatures toward the center"),
  patternAnchor("pushes creatures away"),
  patternAnchor("pushes the target"),
  patternAnchor("pulls the target"),
  patternAnchor("move the target"),
  patternAnchor("sweeps away"),
];

const SPELL_RESTRAINT_CAPTURE_TEXT_ANCHORS = [
  patternAnchor("restrained"),
  patternAnchor("immobilized"),
  patternAnchor("sticky web"),
  patternAnchor("can't escape"),
  patternAnchor("cannot escape"),
  patternAnchor("trap the target"),
  patternAnchor("trapping it inside"),
  patternAnchor("entangle"),
  patternAnchor("grabbed"),
  patternAnchor("immobile illusory walls"),
];

const SPELL_RESTRAINT_CAPTURE_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("life pact"),
  patternAnchor("guardian s aegis"),
  patternAnchor("alarm"),
  patternAnchor("rune trap"),
];

const FEAR_PRESSURE_NAME_ANCHORS = [
  patternAnchor("fear", "name"),
  patternAnchor("fright", "name"),
  patternAnchor("terror", "name"),
  patternAnchor("panic", "name"),
  patternAnchor("dread", "name"),
  patternAnchor("horror", "name"),
];

const FEAR_PRESSURE_TEXT_ANCHORS = [
  patternAnchor("frightened"),
  patternAnchor("fear"),
  patternAnchor("fearful"),
  patternAnchor("terror"),
  patternAnchor("terrified"),
  patternAnchor("panic"),
  patternAnchor("panicked"),
  patternAnchor("cower"),
  patternAnchor("cowers"),
  patternAnchor("flee"),
  patternAnchor("flees"),
  patternAnchor("dread"),
  patternAnchor("repelled by fear"),
  patternAnchor("aura of fear"),
  patternAnchor("strike terror"),
];

const FEAR_PRESSURE_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("against fear"),
  patternAnchor("against emotion effects"),
  patternAnchor("protects against fear"),
  patternAnchor("bonus against fear"),
  patternAnchor("immune to fear"),
  patternAnchor("remove frightened"),
  patternAnchor("reduce frightened"),
  patternAnchor("counteract an effect applying"),
];

const CONCEALMENT_NAME_ANCHORS = [
  patternAnchor("conceal", "name"),
  patternAnchor("concealed", "name"),
  patternAnchor("invisible", "name"),
  patternAnchor("blur", "name"),
  patternAnchor("veil", "name"),
  patternAnchor("hidden", "name"),
];

const CONCEALMENT_TEXT_ANCHORS = [
  patternAnchor("shroud the target"),
  patternAnchor("concealed"),
  patternAnchor("undetected"),
  patternAnchor("invisible"),
  patternAnchor("hidden"),
  patternAnchor("becomes concealed"),
  patternAnchor("gains concealment"),
  patternAnchor("greater concealment"),
  patternAnchor("concealed and undetected"),
  patternAnchor("becomes invisible"),
  patternAnchor("becomes hidden"),
  patternAnchor("becomes undetected"),
  patternAnchor("hidden from sight"),
  patternAnchor("hard to see"),
  patternAnchor("difficult to detect"),
  patternAnchor("blurred"),
  patternAnchor("blur"),
  patternAnchor("shrouded in shadow"),
  patternAnchor("shroud of mist"),
];

const CONCEALMENT_BLOCKER_TEXT_ANCHORS = [
  patternAnchor("blocks line of sight"),
  patternAnchor("obscures vision"),
  patternAnchor("obscures sight"),
  patternAnchor("total darkness"),
  patternAnchor("magical darkness"),
  patternAnchor("dense fog"),
  patternAnchor("thick fog"),
  patternAnchor("bank of fog"),
  patternAnchor("cloud of smoke"),
  patternAnchor("obscuring mist"),
  patternAnchor("obscuring cloud"),
];

const LINE_OF_SIGHT_CONTROL_NAME_ANCHORS = [
  patternAnchor("darkness", "name"),
  patternAnchor("fog", "name"),
  patternAnchor("mist", "name"),
  patternAnchor("smoke", "name"),
  patternAnchor("screen", "name"),
  patternAnchor("shroud", "name"),
  patternAnchor("wall", "name"),
];

const LINE_OF_SIGHT_CONTROL_TEXT_ANCHORS = [
  patternAnchor("blocks line of sight"),
  patternAnchor("line of sight"),
  patternAnchor("obscures vision"),
  patternAnchor("obscures sight"),
  patternAnchor("blocks vision"),
  patternAnchor("can't see through"),
  patternAnchor("cannot see through"),
  patternAnchor("total darkness"),
  patternAnchor("magical darkness"),
  patternAnchor("dense fog"),
  patternAnchor("thick fog"),
  patternAnchor("bank of fog"),
  patternAnchor("cloud of smoke"),
  patternAnchor("obscuring mist"),
  patternAnchor("obscuring cloud"),
];

const BATTLEFIELD_DISRUPTION_NAME_ANCHORS = [
  patternAnchor("wall", "name"),
  patternAnchor("entangle", "name"),
  patternAnchor("grease", "name"),
  patternAnchor("terrain", "name"),
  patternAnchor("barrier", "name"),
  patternAnchor("maze", "name"),
  patternAnchor("snare", "name"),
];

const BATTLEFIELD_DISRUPTION_TEXT_ANCHORS = [
  patternAnchor("cage"),
  patternAnchor("difficult terrain"),
  patternAnchor("blocks passage"),
  patternAnchor("block passage"),
  patternAnchor("hinders movement"),
  patternAnchor("slows movement"),
  patternAnchor("hindering"),
  patternAnchor("obstacle"),
  patternAnchor("obstacles"),
  patternAnchor("entangle"),
  patternAnchor("web"),
  patternAnchor("sticky web"),
  patternAnchor("immobile illusory walls"),
  patternAnchor("lock creatures inside"),
  patternAnchor("false prison"),
  patternAnchor("surround the target"),
  patternAnchor("trap it inside"),
  patternAnchor("trapping it inside"),
  patternAnchor("grease"),
  patternAnchor("spike growth"),
  patternAnchor("creates a barrier"),
  patternAnchor("impassable"),
  patternAnchor("wall of force"),
  patternAnchor("wall of stone"),
];

const TEMPORARY_HP_SUPPORT_TEXT_ANCHORS = [
  patternAnchor("gain temporary hit points"),
  patternAnchor("gains temporary hit points"),
  patternAnchor("temporary hit points"),
  patternAnchor("temporary hp"),
  patternAnchor("buffer of temporary hit points"),
];

const AFFLICTION_CLEANUP_NAME_ANCHORS = [
  patternAnchor("cleanse", "name"),
  patternAnchor("purify", "name"),
  patternAnchor("purge", "name"),
  patternAnchor("antidote", "name"),
  patternAnchor("cure", "name"),
  patternAnchor("neutralize", "name"),
];

const AFFLICTION_CLEANUP_TEXT_ANCHORS = [
  patternAnchor("counteract an affliction"),
  patternAnchor("counteract the affliction"),
  patternAnchor("remove a curse"),
  patternAnchor("remove curse"),
  patternAnchor("cure disease"),
  patternAnchor("cure poison"),
  patternAnchor("remove disease"),
  patternAnchor("remove poison"),
  patternAnchor("neutralize poison"),
  patternAnchor("neutralize disease"),
  patternAnchor("purge poison"),
  patternAnchor("purge disease"),
  patternAnchor("wash away the affliction"),
  patternAnchor("counteract poison"),
  patternAnchor("counteract disease"),
];

const ANTI_FEAR_TEXT_ANCHORS = [
  patternAnchor("against fear"),
  patternAnchor("against fear effects"),
  patternAnchor("protects against fear"),
  patternAnchor("bonus against fear"),
  patternAnchor("immune to fear"),
  patternAnchor("remove frightened"),
  patternAnchor("reduce frightened"),
  patternAnchor("frightened condition"),
  patternAnchor("the frightened condition"),
  patternAnchor("steady courage"),
];

const ANTI_CONFUSION_TEXT_ANCHORS = [
  patternAnchor("against confusion"),
  patternAnchor("confused condition"),
  patternAnchor("the confused condition"),
  patternAnchor("remove confusion"),
  patternAnchor("remove the confused condition"),
  patternAnchor("counteract confusion"),
  patternAnchor("steady a disordered mind"),
  patternAnchor("clear confusion from the target s mind"),
];

const ANTI_PARALYSIS_TEXT_ANCHORS = [
  patternAnchor("against paralysis"),
  patternAnchor("free the target from paralysis"),
  patternAnchor("remove paralysis"),
  patternAnchor("remove the paralyzed condition"),
  patternAnchor("end paralysis"),
  patternAnchor("restore movement to paralyzed limbs"),
];

const ANTI_PETRIFICATION_TEXT_ANCHORS = [
  patternAnchor("against petrification"),
  patternAnchor("restore flesh from stone"),
  patternAnchor("restore a petrified creature"),
  patternAnchor("reverse petrification"),
  patternAnchor("counteract petrification"),
  patternAnchor("counteracting petrification"),
  patternAnchor("remove the petrified condition"),
];

const ANTI_BLEED_TEXT_ANCHORS = [
  patternAnchor("stop bleeding"),
  patternAnchor("staunch bleeding"),
  patternAnchor("staunch persistent bleed damage"),
  patternAnchor("end persistent bleed damage"),
  patternAnchor("close bleeding wounds"),
];

const CURSE_REMOVAL_TEXT_ANCHORS = [
  patternAnchor("remove curse"),
  patternAnchor("remove a curse"),
  patternAnchor("lift a curse"),
  patternAnchor("break a curse"),
  patternAnchor("counteract a curse"),
  patternAnchor("end a curse"),
];

const ESCAPE_SUPPORT_NAME_ANCHORS = [
  patternAnchor("escape", "name"),
  patternAnchor("freedom", "name"),
  patternAnchor("blink", "name"),
  patternAnchor("vanish", "name"),
];

const ESCAPE_SUPPORT_TEXT_ANCHORS = [
  patternAnchor("break free"),
  patternAnchor("slip away"),
  patternAnchor("flee"),
  patternAnchor("escape"),
  patternAnchor("free yourself"),
  patternAnchor("freedom of movement"),
  patternAnchor("teleport away"),
  patternAnchor("move away from danger"),
  patternAnchor("extricate"),
  patternAnchor("evade pursuit"),
];

const AQUATIC_SUPPORT_TEXT_ANCHORS = [
  patternAnchor("breathe underwater"),
  patternAnchor("water breathing"),
  patternAnchor("swim speed"),
  patternAnchor("underwater"),
  patternAnchor("walk on water"),
  patternAnchor("water walk"),
  patternAnchor("surface of water"),
  patternAnchor("air bubble"),
];

const SUSTENANCE_NAME_ANCHORS = [
  patternAnchor("food", "name"),
  patternAnchor("water", "name"),
  patternAnchor("ration", "name"),
  patternAnchor("feast", "name"),
  patternAnchor("cornucopia", "name"),
  patternAnchor("pantry", "name"),
  patternAnchor("sustenance", "name"),
  patternAnchor("nourish", "name"),
];

const SUSTENANCE_TEXT_ANCHORS = [
  patternAnchor("create food"),
  patternAnchor("create water"),
  patternAnchor("fresh food"),
  patternAnchor("fresh water"),
  patternAnchor("day's worth of food"),
  patternAnchor("day's worth of water"),
  patternAnchor("nourishes"),
  patternAnchor("feed"),
  patternAnchor("sate hunger"),
  patternAnchor("rations"),
];

const FIELD_SHELTER_NAME_ANCHORS = [
  patternAnchor("shelter", "name"),
  patternAnchor("cabin", "name"),
  patternAnchor("hut", "name"),
  patternAnchor("tent", "name"),
  patternAnchor("camp", "name"),
  patternAnchor("refuge", "name"),
  patternAnchor("bivouac", "name"),
  patternAnchor("lodge", "name"),
];

const FIELD_SHELTER_TEXT_ANCHORS = [
  patternAnchor("safe shelter"),
  patternAnchor("temporary shelter"),
  patternAnchor("protect from weather"),
  patternAnchor("shelter from the elements"),
  patternAnchor("resting place"),
  patternAnchor("small cabin"),
  patternAnchor("tiny hut"),
  patternAnchor("cozy cabin"),
  patternAnchor("weather"),
  patternAnchor("elements"),
];

const QUICKENED_SUPPORT_NAME_ANCHORS = [
  patternAnchor("quicken", "name"),
  patternAnchor("quickened", "name"),
  patternAnchor("haste", "name"),
  patternAnchor("allegro", "name"),
  patternAnchor("celerity", "name"),
];

const QUICKENED_SUPPORT_TEXT_ANCHORS = [
  patternAnchor("becomes quickened"),
  patternAnchor("quickened"),
  patternAnchor("gain an extra action"),
  patternAnchor("extra action"),
  patternAnchor("additional action"),
  patternAnchor("your ally is quickened"),
  patternAnchor("haste"),
  patternAnchor("speed up"),
];

export const SPELL_DERIVED_TAG_RULES: DerivedTagRule[] = [
  {
    tag: "stealth_support",
    category: "spell",
    threshold: 2,
    anyOf: [{ score: 2, textAny: SPELL_STEALTH_SUPPORT_TEXT_ANCHORS }],
  },
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
            all: [patternAnchor("detect"), patternAnchor("undead")],
            window: 4,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: SPELL_DISGUISE_BLOCKER_TEXT_ANCHORS }],
  },
  {
    tag: "social_infiltration",
    category: "spell",
    requiresTags: ["disguise"],
    anyOf: [{ textAny: SOCIAL_INFILTRATION_TEXT_ANCHORS }, { textAny: SPELL_DISGUISE_TEXT_ANCHORS }],
    noneOf: [{ textAny: SPELL_DISGUISE_BLOCKER_TEXT_ANCHORS }],
  },
  {
    tag: "social_infiltration",
    category: "spell",
    anyOf: [{ textAny: SPELL_SOCIAL_INFILTRATION_TEXT_ANCHORS }],
  },
  {
    tag: "social_infiltration",
    category: "spell",
    allOf: [{ textAny: [patternAnchor("mask", "name"), patternAnchor("mask", "description")] }],
    anyOf: [
      {
        textNear: [
          {
            all: [patternAnchor("mask"), patternAnchor("{{alt(deception,lie,feint)}}")],
            window: 12,
            scope: "description",
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
    noneOf: [{ textAny: ALARM_STRONG_TEXT_ANCHORS }],
  },
  {
    tag: "message_delivery",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: MESSAGE_DELIVERY_SPELL_NAME_ANCHORS },
      { score: 2, textAny: MESSAGE_DELIVERY_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [patternAnchor("message"), patternAnchor("{{alt(mental,whisper,note,respond,response)}}")],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: MESSAGE_DELIVERY_BLOCKER_TEXT_ANCHORS }],
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
        textAll: [patternAnchor("senses"), patternAnchor("through")],
        textAny: [
          patternAnchor("through the ear"),
          patternAnchor("through its eyes"),
          patternAnchor("through each other s eyes"),
        ],
      },
    ],
    noneOf: [{ textAny: SPELL_SCOUTING_BLOCKER_TEXT_ANCHORS }],
  },
  {
    tag: "senses_support",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: SPELL_SENSES_SUPPORT_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(target,you,ally,creature)}}"),
              patternAnchor("{{alt(darkvision,see invisible,scent,tremorsense,lifesense)}}"),
            ],
            window: 8,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: SPELL_SENSES_SUPPORT_BLOCKER_TEXT_ANCHORS }],
  },
  {
    tag: "illumination",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: SPELL_ILLUMINATION_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(light,glow,glowing,illuminate,illuminates)}}"),
              patternAnchor("{{alt(area,darkness,camp,campsite,radius)}}"),
            ],
            window: 8,
            scope: "description",
          },
        ],
      },
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
      { score: 1, traitsAny: ["teleportation"] },
      { score: 2, textAny: SPELL_MOBILITY_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(teleport,teleports,transport,transports,swap)}}"),
              patternAnchor("{{alt(instantly,target,creature,you)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: SPELL_MOBILITY_BLOCKER_TEXT_ANCHORS }],
  },
  {
    tag: "mental_impairment",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, traitsAny: ["mental", "emotion", "fear"] },
      { score: 2, textAny: SPELL_MENTAL_IMPAIRMENT_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(mind,mental,will)}}"),
              patternAnchor("{{alt(terror,fear,confusion,stupefied)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: SPELL_MENTAL_IMPAIRMENT_BLOCKER_TEXT_ANCHORS }],
  },
  {
    tag: "sensory_impairment",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: SPELL_SENSORY_IMPAIRMENT_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(blind,blinded,deaf,deafened)}}"),
              patternAnchor("{{alt(vision,hearing,eyes,ears)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: SPELL_SENSORY_IMPAIRMENT_BLOCKER_TEXT_ANCHORS }],
  },
  {
    tag: "forced_movement",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: SPELL_FORCED_MOVEMENT_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(push,pushes,pull,pulls,drag,drags,knock,knocks,launch,launches,sweep,sweeps)}}"),
              patternAnchor("{{alt(target,creature,foe)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: SPELL_MOBILITY_BLOCKER_TEXT_ANCHORS }],
  },
  {
    tag: "restraint_capture",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: SPELL_RESTRAINT_CAPTURE_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(restrained,immobilized,grabbed,entangle,web,sticky,prison,trap)}}"),
              patternAnchor("{{alt(escape,target,creature)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: SPELL_RESTRAINT_CAPTURE_BLOCKER_TEXT_ANCHORS }],
  },
  {
    tag: "fear_pressure",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, traitsAny: ["fear", "emotion"] },
      { score: 2, textAny: FEAR_PRESSURE_NAME_ANCHORS },
      { score: 2, textAny: FEAR_PRESSURE_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(fear,frightened,terror,panic,dread)}}"),
              patternAnchor("{{alt(flee,cower,target,creature)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: FEAR_PRESSURE_BLOCKER_TEXT_ANCHORS }],
  },
  {
    tag: "concealment",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, traitsAny: ["illusion", "shadow"] },
      { score: 1, textAny: CONCEALMENT_NAME_ANCHORS },
      {
        score: 1,
        textAny: [
          patternAnchor("veil"),
          patternAnchor("conceal"),
          patternAnchor("concealed"),
          patternAnchor("undetected"),
          patternAnchor("invisible"),
          patternAnchor("hidden"),
        ],
      },
      { score: 2, textAny: CONCEALMENT_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(target,creature,self,ally)}}"),
              patternAnchor("{{alt(hidden,concealed,undetected,invisible)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: CONCEALMENT_BLOCKER_TEXT_ANCHORS }],
  },
  {
    tag: "line_of_sight_control",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: LINE_OF_SIGHT_CONTROL_NAME_ANCHORS },
      { score: 2, textAny: LINE_OF_SIGHT_CONTROL_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(line of sight,vision,see)}}"),
              patternAnchor("{{alt(obscure,darkness,fog,smoke,mist)}}"),
            ],
            window: 7,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "battlefield_disruption",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: BATTLEFIELD_DISRUPTION_NAME_ANCHORS },
      { score: 2, textAny: BATTLEFIELD_DISRUPTION_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor(
                "{{alt(difficult terrain,wall,barrier,cage,obstacle,obstacles,entangle,web,grease,passage,prison)}}",
              ),
              patternAnchor("{{alt(movement,hindering,blocks passage,slow pursuit)}}"),
            ],
            window: 7,
            scope: "description",
          },
        ],
      },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(wall,barrier,cage,prison,cell)}}"),
              patternAnchor("{{alt(blocks,block,trap,traps,contain,contains,imprison,imprisons)}}"),
            ],
            window: 7,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "affliction_cleanup",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: AFFLICTION_CLEANUP_NAME_ANCHORS },
      { score: 2, textAny: AFFLICTION_CLEANUP_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(remove,cure,counteract,neutralize,purge)}}"),
              patternAnchor("{{alt(affliction,poison,disease,curse)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [
      {
        textAny: [
          patternAnchor("without affecting"),
          patternAnchor("without effect on"),
          patternAnchor("does not affect"),
        ],
      },
    ],
  },
  {
    tag: "escape_support",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: ESCAPE_SUPPORT_NAME_ANCHORS },
      { score: 2, textAny: ESCAPE_SUPPORT_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(escape,flee,break,free,slip,freedom)}}"),
              patternAnchor("{{alt(away,grab,restraint,movement,teleport)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: SPELL_MOBILITY_BLOCKER_TEXT_ANCHORS }],
  },
  {
    tag: "aquatic_support",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, traitsAny: ["water"] },
      { score: 2, textAny: AQUATIC_SUPPORT_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(water,underwater,swim,surface)}}"),
              patternAnchor("{{alt(breathe,breathing,walk)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "sustenance",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: SUSTENANCE_NAME_ANCHORS },
      { score: 2, textAny: SUSTENANCE_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(food,water,ration,rations)}}"),
              patternAnchor("{{alt(nourish,feed,sate,hunger)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "field_shelter",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: FIELD_SHELTER_NAME_ANCHORS },
      { score: 2, textAny: FIELD_SHELTER_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(shelter,sheltered,cabin,hut,tent,refuge)}}"),
              patternAnchor("{{alt(camp,weather,elements,rest)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "quickened_support",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 1, textAny: QUICKENED_SUPPORT_NAME_ANCHORS },
      { score: 2, textAny: QUICKENED_SUPPORT_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(quickened,haste)}}"),
              patternAnchor("{{alt(extra action,additional action,ally,target)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
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
            all: [patternAnchor("{{alt(form,shape,body)}}"), patternAnchor("{{alt(transform,change)}}")],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: TRANSFORMATION_BLOCKER_TEXT_ANCHORS }],
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
            all: [patternAnchor("battle"), patternAnchor("form")],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: TRANSFORMATION_BLOCKER_TEXT_ANCHORS }],
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
            all: [patternAnchor("{{alt(animal,beast,pest,dinosaur)}}"), patternAnchor("form")],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: TRANSFORMATION_BLOCKER_TEXT_ANCHORS }],
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
            all: [patternAnchor("elemental"), patternAnchor("form")],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: TRANSFORMATION_BLOCKER_TEXT_ANCHORS }],
  },
  {
    tag: "temporary_hp_support",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: TEMPORARY_HP_SUPPORT_TEXT_ANCHORS },
      {
        score: 2,
        textAny: [
          patternAnchor("{{alt(gain,gains,grant,grants,granted)}} {{gap(4)}} temporary hit points"),
          patternAnchor("{{alt(gain,gains,grant,grants,granted)}} {{gap(4)}} a buffer of temporary hit points"),
        ],
      },
    ],
    noneOf: [{ textAny: BATTLE_FORM_TEXT_ANCHORS }],
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
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(counteract,delay,free,drive)}}"),
              patternAnchor("{{alt(condition,conditions,affliction,mobility,mind,mental)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "anti_fear",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: ANTI_FEAR_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(counteract,remove,reduce,protect)}}"),
              patternAnchor("{{alt(frightened,fear)}}"),
            ],
            window: 8,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "anti_confusion",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: ANTI_CONFUSION_TEXT_ANCHORS },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(counteract,remove,clear,steady)}}"),
              patternAnchor("{{alt(confused,confusion)}}"),
            ],
            window: 8,
            scope: "description",
          },
        ],
      },
    ],
  },
  {
    tag: "anti_poison",
    category: "spell",
    threshold: 2,
    anyOf: [{ score: 2, textAny: ANTI_POISON_TEXT_ANCHORS }],
  },
  {
    tag: "anti_disease",
    category: "spell",
    threshold: 2,
    anyOf: [{ score: 2, textAny: ANTI_DISEASE_TEXT_ANCHORS }],
  },
  {
    tag: "anti_paralysis",
    category: "spell",
    threshold: 2,
    anyOf: [{ score: 2, textAny: ANTI_PARALYSIS_TEXT_ANCHORS }],
  },
  {
    tag: "anti_petrification",
    category: "spell",
    threshold: 2,
    anyOf: [{ score: 2, textAny: ANTI_PETRIFICATION_TEXT_ANCHORS }],
  },
  {
    tag: "anti_bleed",
    category: "spell",
    threshold: 2,
    anyOf: [{ score: 2, textAny: ANTI_BLEED_TEXT_ANCHORS }],
  },
  {
    tag: "curse_removal",
    category: "spell",
    threshold: 2,
    anyOf: [{ score: 2, textAny: CURSE_REMOVAL_TEXT_ANCHORS }],
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
            all: [
              patternAnchor("{{alt(protect,shield,ward,barrier)}}"),
              patternAnchor("{{alt(creature,target,ally,allies,area,self)}}"),
            ],
            window: 6,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [{ textAny: [...ALARM_STRONG_TEXT_ANCHORS, ...ALARM_TRIGGER_TEXT_ANCHORS] }],
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
            all: [patternAnchor("stabilize"), patternAnchor("{{alt(dying,die,death)}}")],
            window: 6,
            scope: "description",
          },
        ],
      },
      {
        score: 2,
        textNear: [
          {
            all: [
              patternAnchor("{{alt(return,returns,revive,resurrect,bring back,brings back)}}"),
              patternAnchor("{{alt(life,death)}}"),
            ],
            window: 8,
            scope: "description",
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
        textAll: [patternAnchor("counteract")],
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
