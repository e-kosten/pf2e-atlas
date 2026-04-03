import { SearchSubcategory } from "../types.js";
import {
  normalizeDerivedTagReference,
  TextAnchor,
  TextMatchScope,
  TextProximityConstraint,
} from "./matcher.js";

const tokenAnchor = (value: string, scope: TextMatchScope = "either"): TextAnchor => ({ value, mode: "token", scope });
const phraseAnchor = (value: string, scope: TextMatchScope = "either"): TextAnchor => ({ value, mode: "phrase", scope });
const templateAnchor = (value: string, scope: TextMatchScope = "either"): TextAnchor => ({ value, mode: "template", scope });

const OFFENSIVE_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("venom"),
  tokenAnchor("bomb"),
  phraseAnchor("injury poison"),
  phraseAnchor("contact poison"),
  phraseAnchor("ingested poison"),
  phraseAnchor("inhaled poison"),
  phraseAnchor("weapon poison"),
  phraseAnchor("afflicts the target"),
];

const GEARISH_SUBCATEGORIES: SearchSubcategory[] = ["gear", "backpack", "kit", "vehicle"];
const DISGUISE_SUBCATEGORIES: SearchSubcategory[] = [...GEARISH_SUBCATEGORIES, "consumable"];
const TRACKING_SUBCATEGORIES: SearchSubcategory[] = [...GEARISH_SUBCATEGORIES, "consumable"];

const STRONG_PROFESSION_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("captain", "name"),
  tokenAnchor("commoner", "name"),
  tokenAnchor("courtier", "name"),
  tokenAnchor("guard", "name"),
  tokenAnchor("scout", "name"),
  tokenAnchor("sailor", "name"),
  tokenAnchor("merchant", "name"),
  tokenAnchor("priest", "name"),
  tokenAnchor("noble", "name"),
  tokenAnchor("prophet", "name"),
  tokenAnchor("advisor", "name"),
  tokenAnchor("acolyte", "name"),
  tokenAnchor("vigilante", "name"),
];

const WEAK_PROFESSION_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("agent", "name"),
  tokenAnchor("apprentice", "name"),
  tokenAnchor("hunter", "name"),
  tokenAnchor("enforcer", "name"),
];

const UNDEAD_GLOSSARY_FAMILIES = [
  "blackfrost-dead",
  "floodslain",
  "ghast",
  "ghost",
  "ghoul",
  "graveknight",
  "lich",
  "ravener",
  "siabrae",
  "vampire",
  "visitant",
];

const SCENE_ADJACENT_BLOCKER_TRAITS = [
  "aberration",
  "animal",
  "construct",
  "dragon",
  "elemental",
  "fey",
  "fiend",
  "fungus",
  "ghost",
  "ghoul",
  "giant",
  "golem",
  "leshy",
  "mindless",
  "monitor",
  "ooze",
  "plant",
  "psychopomp",
  "skeleton",
  "spirit",
  "swarm",
  "troop",
  "undead",
];

const FRESHWATER_SETTING_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("river"),
  tokenAnchor("rivers"),
  tokenAnchor("lake"),
  tokenAnchor("lakes"),
  tokenAnchor("pond"),
  tokenAnchor("ponds"),
  tokenAnchor("stream"),
  tokenAnchor("streams"),
  tokenAnchor("spring"),
  tokenAnchor("springs"),
];

const FRESHWATER_SETTING_STRONG_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("freshwater"),
  phraseAnchor("fresh water"),
  phraseAnchor("body of fresh water"),
  phraseAnchor("bodies of fresh water"),
  phraseAnchor("lake bed"),
  phraseAnchor("lake beds"),
  phraseAnchor("lakeside community"),
  phraseAnchor("lakeside communities"),
];

const FRESHWATER_SETTING_CONTEXT_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("native"),
  tokenAnchor("hide"),
  tokenAnchor("hides"),
  tokenAnchor("inhabit"),
  tokenAnchor("inhabits"),
  tokenAnchor("inhabiting"),
  tokenAnchor("dwell"),
  tokenAnchor("dwells"),
  tokenAnchor("dwelling"),
  tokenAnchor("centered"),
  tokenAnchor("home"),
  tokenAnchor("homes"),
  tokenAnchor("found"),
  tokenAnchor("lurk"),
  tokenAnchor("lurks"),
  tokenAnchor("lurking"),
  tokenAnchor("protect"),
  tokenAnchor("protects"),
  tokenAnchor("ward"),
  tokenAnchor("wards"),
];

const FRESHWATER_SETTING_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("pond", "name"),
  tokenAnchor("spring", "name"),
];

const FRESHWATER_SETTING_BLOCKER_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("river kingdom"),
  phraseAnchor("river kingdoms"),
  phraseAnchor("river of souls"),
];

const FRESHWATER_SETTING_HABITAT_TEXT_NEAR: TextProximityConstraint[] = [
  {
    terms: [tokenAnchor("river"), tokenAnchor("rivers"), ...FRESHWATER_SETTING_CONTEXT_TEXT_ANCHORS],
    window: 6,
    scope: "description",
    minTermsMatched: 2,
  },
  {
    terms: [tokenAnchor("lake"), tokenAnchor("lakes"), ...FRESHWATER_SETTING_CONTEXT_TEXT_ANCHORS],
    window: 6,
    scope: "description",
    minTermsMatched: 2,
  },
  {
    terms: [tokenAnchor("pond"), tokenAnchor("ponds"), ...FRESHWATER_SETTING_CONTEXT_TEXT_ANCHORS],
    window: 6,
    scope: "description",
    minTermsMatched: 2,
  },
  {
    terms: [tokenAnchor("stream"), tokenAnchor("streams"), ...FRESHWATER_SETTING_CONTEXT_TEXT_ANCHORS],
    window: 6,
    scope: "description",
    minTermsMatched: 2,
  },
  {
    terms: [tokenAnchor("spring"), tokenAnchor("springs"), ...FRESHWATER_SETTING_CONTEXT_TEXT_ANCHORS],
    window: 6,
    scope: "description",
    minTermsMatched: 2,
  },
];

const AQUATIC_SETTING_STRONG_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("aquatic"),
  tokenAnchor("underwater"),
  tokenAnchor("ocean"),
  tokenAnchor("oceans"),
  tokenAnchor("sea"),
  tokenAnchor("seas"),
  tokenAnchor("pelagic"),
  tokenAnchor("fjord"),
  tokenAnchor("fjords"),
  tokenAnchor("waterways"),
  phraseAnchor("water dwelling"),
  phraseAnchor("breathe water"),
];

const AQUATIC_SETTING_WEAK_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("water"),
  tokenAnchor("wave"),
  tokenAnchor("waves"),
  tokenAnchor("surf"),
  tokenAnchor("tide"),
  tokenAnchor("tides"),
  tokenAnchor("tidal"),
  tokenAnchor("swim"),
  tokenAnchor("swims"),
  tokenAnchor("swimming"),
  tokenAnchor("flooded"),
];

const COASTAL_SETTING_STRONG_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("coast"),
  tokenAnchor("coasts"),
  tokenAnchor("coastal"),
  tokenAnchor("marine"),
  tokenAnchor("reef"),
  tokenAnchor("reefs"),
  tokenAnchor("littoral"),
  tokenAnchor("estuary"),
  tokenAnchor("estuaries"),
  tokenAnchor("estuarine"),
  phraseAnchor("coastal region"),
  phraseAnchor("coastal regions"),
  phraseAnchor("coral reef"),
  phraseAnchor("coral reefs"),
  phraseAnchor("marine environment"),
  phraseAnchor("marine environments"),
  phraseAnchor("ocean facing cliffs"),
  phraseAnchor("along any shore"),
];

const COASTAL_SETTING_WEAK_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("shore"),
  tokenAnchor("shores"),
  tokenAnchor("shoreline"),
  tokenAnchor("beach"),
  tokenAnchor("beaches"),
  tokenAnchor("harbor"),
  tokenAnchor("harbors"),
  tokenAnchor("brackish"),
  tokenAnchor("coral"),
  tokenAnchor("ocean"),
  tokenAnchor("oceans"),
  tokenAnchor("sea"),
  tokenAnchor("seas"),
  tokenAnchor("tidal"),
  tokenAnchor("tidepool"),
  tokenAnchor("tidepools"),
];

const COASTAL_SETTING_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("coastal", "name"),
  tokenAnchor("coral", "name"),
  tokenAnchor("reef", "name"),
  tokenAnchor("sea", "name"),
  tokenAnchor("tidepool", "name"),
  tokenAnchor("tidewater", "name"),
];

const ASTRAL_SETTING_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("astral plane"),
  phraseAnchor("silver sea"),
  phraseAnchor("silver void"),
  phraseAnchor("silvery gray void"),
  phraseAnchor("stable portal"),
  phraseAnchor("stable portals"),
];

const ASTRAL_SETTING_ACTIVITY_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("ambush"),
  tokenAnchor("ambushes"),
  tokenAnchor("guardian"),
  tokenAnchor("guardians"),
  tokenAnchor("guard"),
  tokenAnchor("guards"),
  tokenAnchor("home"),
  tokenAnchor("hunt"),
  tokenAnchor("hunts"),
  tokenAnchor("hunting"),
  tokenAnchor("patrol"),
  tokenAnchor("patrols"),
  tokenAnchor("protect"),
  tokenAnchor("protects"),
  tokenAnchor("travel"),
  tokenAnchor("travels"),
  tokenAnchor("traveling"),
];

const ASTRAL_SETTING_CONTEXT_TEXT_ANCHORS: TextAnchor[] = [
  ...ASTRAL_SETTING_ACTIVITY_TEXT_ANCHORS,
  tokenAnchor("demiplane"),
  tokenAnchor("demiplanes"),
  tokenAnchor("portal"),
  tokenAnchor("portals"),
  phraseAnchor("stable portal"),
  phraseAnchor("stable portals"),
];

const FIRST_WORLD_SETTING_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("first world"),
  phraseAnchor("realm of the fey"),
  phraseAnchor("fey realm"),
];

const FIRST_WORLD_SETTING_CONTEXT_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("boundary"),
  tokenAnchor("boundaries"),
  tokenAnchor("court"),
  tokenAnchor("courts"),
  tokenAnchor("eldest"),
  tokenAnchor("home"),
  tokenAnchor("homes"),
  tokenAnchor("native"),
  tokenAnchor("natives"),
  tokenAnchor("origin"),
  tokenAnchor("origins"),
  tokenAnchor("portal"),
  tokenAnchor("portals"),
  tokenAnchor("spawn"),
  tokenAnchor("spawned"),
  phraseAnchor("worn thin"),
];

const BONEYARD_SETTING_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("boneyard"),
];

const BONEYARD_SETTING_CONTEXT_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("guard"),
  tokenAnchor("guards"),
  tokenAnchor("passing"),
  tokenAnchor("psychopomp"),
  tokenAnchor("repair"),
  tokenAnchor("repairs"),
  tokenAnchor("sentinel"),
  tokenAnchor("sentinels"),
  phraseAnchor("guard dogs"),
  phraseAnchor("repair damaged souls"),
  phraseAnchor("ease a spirit s passing"),
];

const referenceAnchor = (packName: string, name: string): string => normalizeDerivedTagReference(`${packName}:${name}`);

const DISGUISE_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("disguise"),
  tokenAnchor("impersonate"),
  phraseAnchor("false identity"),
  tokenAnchor("costume"),
  tokenAnchor("masquerade"),
  phraseAnchor("quick change", "name"),
  phraseAnchor("take on the appearance"),
  phraseAnchor("change your appearance"),
  phraseAnchor("assume role"),
];

const SOCIAL_INFILTRATION_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("false identity"),
  phraseAnchor("pass as"),
  phraseAnchor("blend into society"),
  phraseAnchor("social infiltration"),
  tokenAnchor("impersonate"),
  tokenAnchor("masquerade"),
  phraseAnchor("quick change", "name"),
  tokenAnchor("disguise", "name"),
  phraseAnchor("take on the appearance"),
  phraseAnchor("change your appearance"),
  phraseAnchor("assume role"),
];

const DISGUISE_REFERENCE_ANCHORS = [
  referenceAnchor("actionspf2e", "Impersonate"),
  referenceAnchor("spells-srd", "Illusory Disguise"),
  referenceAnchor("equipment-srd", "Disguise Kit"),
];

const SOCIAL_INFILTRATION_REFERENCE_ANCHORS = [
  referenceAnchor("actionspf2e", "Impersonate"),
  referenceAnchor("spells-srd", "Illusory Disguise"),
];

const SPELL_DISGUISE_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("disguise", "name"),
  tokenAnchor("visage", "name"),
];

const SPELL_DISGUISE_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("appear as another creature"),
  phraseAnchor("appears as another creature"),
  phraseAnchor("appearance becomes bland and nondescript"),
  phraseAnchor("change your appearance"),
  phraseAnchor("transform your appearance"),
  phraseAnchor("alter a minor detail of your appearance"),
  phraseAnchor("trade appearances"),
  phraseAnchor("look and sound like the target"),
  phraseAnchor("disguises you"),
  phraseAnchor("disguises the target"),
  phraseAnchor("mask the target s features"),
  phraseAnchor("pass the target off as"),
];

const SPELL_DISGUISE_BLOCKER_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("see things as they are"),
  phraseAnchor("see through it"),
  phraseAnchor("see the creature s true form"),
  phraseAnchor("counteract check against each illusion"),
  phraseAnchor("unveiled by attempts to magically cloak the truth"),
];

const SPELL_SOCIAL_INFILTRATION_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("go incognito"),
  phraseAnchor("appearance becomes bland and nondescript"),
  phraseAnchor("suitable for a particular occasion"),
  phraseAnchor("pass as someone else"),
  phraseAnchor("blend into"),
];

const RESTRAINT_ESCAPE_REFERENCE_ANCHORS = [
  referenceAnchor("actionspf2e", "Escape"),
  referenceAnchor("conditionitems", "Grabbed"),
  referenceAnchor("conditionitems", "Restrained"),
];

const RESTRAINT_CAPTURE_REFERENCE_ANCHORS = [
  referenceAnchor("actionspf2e", "Grapple"),
  referenceAnchor("conditionitems", "Grabbed"),
  referenceAnchor("conditionitems", "Restrained"),
];

const STRONG_RESTRAINT_CAPTURE_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("handcuffs", "name"),
  tokenAnchor("manacles", "name"),
];

const WEAK_RESTRAINT_CAPTURE_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("lasso", "name"),
  tokenAnchor("net", "name"),
];

const MOBILITY_REFERENCE_ANCHORS = [
  referenceAnchor("actionspf2e", "Balance"),
  referenceAnchor("actionspf2e", "Climb"),
  referenceAnchor("actionspf2e", "High Jump"),
  referenceAnchor("actionspf2e", "Leap"),
  referenceAnchor("actionspf2e", "Long Jump"),
  referenceAnchor("actionspf2e", "Swim"),
];

const NAVIGATION_REFERENCE_ANCHORS = [
  referenceAnchor("actionspf2e", "Sense Direction"),
];

const TRACKING_REFERENCE_ANCHORS = [
  referenceAnchor("actionspf2e", "Track"),
  referenceAnchor("actionspf2e", "Cover Tracks"),
];

const HAZARD_ALARM_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("alarm"),
  tokenAnchor("alert"),
  tokenAnchor("alerts"),
  phraseAnchor("raise the alarm"),
  phraseAnchor("alert nearby guards"),
  phraseAnchor("sound an alarm"),
  phraseAnchor("warning bell"),
  tokenAnchor("intrusion"),
];

const HAZARD_FIRE_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("fire", "name"),
  tokenAnchor("fires", "name"),
  tokenAnchor("flame", "name"),
  tokenAnchor("burning", "name"),
  tokenAnchor("inferno", "name"),
  phraseAnchor("aflame", "name"),
];

const HAZARD_FIRE_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("catch fire and explode"),
  phraseAnchor("catches fire and explode"),
  phraseAnchor("rains fire from the sky"),
  phraseAnchor("sheets of fire"),
  phraseAnchor("beams of fire"),
  phraseAnchor("burns and threatens to spread"),
  phraseAnchor("a fire engulfs"),
  phraseAnchor("spreads on each of its turns"),
];

const HAZARD_POISON_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("poison", "name"),
  tokenAnchor("venom", "name"),
  tokenAnchor("envenomed", "name"),
  tokenAnchor("toxic", "name"),
];

const HAZARD_POISON_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("poison gas"),
  phraseAnchor("pumping poison gas"),
  phraseAnchor("releases a poison gas"),
  phraseAnchor("poisonous smoke"),
  phraseAnchor("pressurized poison"),
  phraseAnchor("poisoned spine"),
  phraseAnchor("needle delivers a magical poison"),
  phraseAnchor("toxic darts"),
  phraseAnchor("acidic poison"),
];

const HAZARD_PITFALL_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("pit", "name"),
  tokenAnchor("pitfall", "name"),
  tokenAnchor("sinkhole", "name"),
];

const HAZARD_COLLAPSE_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("collapse", "name"),
  tokenAnchor("collapsing", "name"),
  tokenAnchor("deadfall", "name"),
  tokenAnchor("rockfall", "name"),
  phraseAnchor("cave in", "name"),
];

const HAZARD_FORCED_MOVEMENT_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("sucks creatures in the area toward"),
  phraseAnchor("raging wind sucks creatures"),
  phraseAnchor("attempts to submerge creatures"),
  phraseAnchor("trample each other"),
  phraseAnchor("pulls creatures toward"),
  phraseAnchor("pushes creatures"),
  phraseAnchor("sweeps creatures away"),
];

const ALARM_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("alarm", "name"),
  tokenAnchor("warning", "name"),
  tokenAnchor("sentry", "name"),
];

const ALARM_STRONG_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("audible alarm"),
  phraseAnchor("mental alarm"),
  phraseAnchor("mental alert"),
  phraseAnchor("alarm system"),
  phraseAnchor("raise the alarm"),
  phraseAnchor("raising the alarm"),
  phraseAnchor("alerting nearby guards"),
  phraseAnchor("keeps watch over an area"),
  phraseAnchor("watch over an area"),
  phraseAnchor("without speaking the password"),
  phraseAnchor("without giving the password"),
  phraseAnchor("the snare makes a noise"),
  phraseAnchor("emits an ear piercing wail"),
];

const ALARM_TRIGGER_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("enters the spell s area"),
  phraseAnchor("enters the area"),
  phraseAnchor("enters the square"),
  phraseAnchor("detect creatures moving in its area"),
  phraseAnchor("trip wire"),
  phraseAnchor("pressure plate"),
  tokenAnchor("password"),
];

const SIGNALING_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("signal", "name"),
  tokenAnchor("signaling", "name"),
  tokenAnchor("beacon", "name"),
  tokenAnchor("whistle", "name"),
];

const SIGNALING_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("request rescue"),
  phraseAnchor("coordinate assaults"),
  phraseAnchor("signal directions"),
  phraseAnchor("heard clearly"),
  phraseAnchor("heard up to"),
  phraseAnchor("seen from miles away"),
  phraseAnchor("target becomes more visible"),
  phraseAnchor("spews sparks"),
  phraseAnchor("emblazon a message across the sky"),
];

const MESSAGE_DELIVERY_EQUIPMENT_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("message", "name"),
  tokenAnchor("mail", "name"),
  tokenAnchor("mailbox", "name"),
  tokenAnchor("communication", "name"),
];

const MESSAGE_DELIVERY_SPELL_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("message", "name"),
  tokenAnchor("mailbox", "name"),
  tokenAnchor("telepathy", "name"),
  phraseAnchor("telepathic bond", "name"),
  phraseAnchor("dream council", "name"),
  tokenAnchor("mindlink", "name"),
];

const MESSAGE_DELIVERY_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("send a message"),
  phraseAnchor("record a message"),
  phraseAnchor("write a message"),
  phraseAnchor("message bearer"),
  phraseAnchor("messages can be coded"),
  phraseAnchor("brief response"),
  phraseAnchor("communicate telepathically"),
  phraseAnchor("message is one way"),
  phraseAnchor("shared dream"),
  phraseAnchor("communicate with one another"),
  phraseAnchor("message up to"),
];

const MESSAGE_DELIVERY_BLOCKER_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("suggestion spell"),
  phraseAnchor("harder for them to communicate"),
  phraseAnchor("can communicate only by singing"),
  phraseAnchor("no special method of communication"),
];

const COUNTERMAGIC_NAME_ANCHORS: TextAnchor[] = [
  tokenAnchor("antimagic", "name"),
  tokenAnchor("countering", "name"),
  tokenAnchor("nullification", "name"),
  tokenAnchor("dispelling", "name"),
  tokenAnchor("dispel", "name"),
  tokenAnchor("counterspell", "name"),
];

const COUNTERMAGIC_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("spell s effect doesn t occur"),
  phraseAnchor("all magic is suppressed"),
  phraseAnchor("antimagic field"),
  phraseAnchor("counteract magical effects"),
  phraseAnchor("counteract the triggering spell"),
  phraseAnchor("counteract a single spell"),
  phraseAnchor("counteract the target spell"),
];

const COUNTERMAGIC_REFERENCE_ANCHORS = [
  referenceAnchor("spells-srd", "Dispel Magic"),
  referenceAnchor("spells-srd", "Antimagic Field"),
  referenceAnchor("spells-srd", "Dispelling Globe"),
];

const MAGIC_PROTECTION_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("saving throws against magical effects"),
  phraseAnchor("bonus to saving throws against magical effects"),
  phraseAnchor("becomes immune to all spells"),
  phraseAnchor("immune to all spells"),
  phraseAnchor("effects of magic items"),
  phraseAnchor("effects with the magical trait"),
  phraseAnchor("spell targets you"),
  phraseAnchor("spells that target you"),
];

const SPELL_SCOUTING_NAME_ANCHORS: TextAnchor[] = [
  phraseAnchor("clairaudience", "name"),
  phraseAnchor("clairvoyance", "name"),
  phraseAnchor("familiar s face", "name"),
  phraseAnchor("know location", "name"),
  phraseAnchor("painted scout", "name"),
  phraseAnchor("prying survey", "name"),
  phraseAnchor("proliferating eyes", "name"),
  phraseAnchor("rune of observation", "name"),
  phraseAnchor("scouting eye", "name"),
  phraseAnchor("scrying", "name"),
  phraseAnchor("scrying ripples", "name"),
  phraseAnchor("unrelenting observation", "name"),
  phraseAnchor("web of eyes", "name"),
];

const SPELL_SCOUTING_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("can see hear and smell through"),
  phraseAnchor("create a sensor"),
  phraseAnchor("floating ear"),
  phraseAnchor("floating eye"),
  phraseAnchor("hear through the ear"),
  phraseAnchor("know the location"),
  phraseAnchor("learn its approximate distance and direction"),
  phraseAnchor("looks where that target looks"),
  phraseAnchor("magical eye sensor"),
  phraseAnchor("scrying sensor"),
  phraseAnchor("see hear and smell through"),
  phraseAnchor("see in all directions from that point"),
  phraseAnchor("see through its eyes"),
  phraseAnchor("transmits what it sees"),
  phraseAnchor("transmitting rough impressions"),
  phraseAnchor("you magically spy on"),
];

const SPELL_SCOUTING_BLOCKER_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("preventing it from being used for magical observation"),
  phraseAnchor("fools any attempts to scry"),
  phraseAnchor("hide a creature from magic that would spy on it"),
  phraseAnchor("make the target difficult to detect via magic"),
  phraseAnchor("counteract all detection revelation and scrying effects"),
];

const SPELL_NAVIGATION_NAME_ANCHORS: TextAnchor[] = [
  phraseAnchor("guiding star", "name"),
  phraseAnchor("know the way", "name"),
  phraseAnchor("wanderer s guide", "name"),
];

const SPELL_NAVIGATION_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("guide your route"),
  phraseAnchor("guided to the location"),
  phraseAnchor("inspired route"),
  phraseAnchor("know which direction is north"),
  phraseAnchor("learn what direction it lies"),
  phraseAnchor("mental nudge toward your chosen location"),
  phraseAnchor("know its relative direction from you"),
  phraseAnchor("find the correct gate to your desired location"),
];

const SPELL_MOBILITY_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("gain a fly speed"),
  phraseAnchor("giving you a fly speed"),
  phraseAnchor("gain a climb speed"),
  phraseAnchor("gain a swim speed"),
  phraseAnchor("gain a burrow speed"),
  phraseAnchor("climb speed equal to your speed"),
  phraseAnchor("climb speed equal to"),
  phraseAnchor("swim speed equal to your speed"),
  phraseAnchor("swim speed equal to"),
  phraseAnchor("fly speed equal to your speed"),
  phraseAnchor("fly speed equal to"),
  phraseAnchor("ignore difficult terrain"),
  templateAnchor("jump {n} feet"),
  templateAnchor("you jump {n} feet"),
  templateAnchor("gain a +{n}-foot status bonus to your speed"),
  templateAnchor("you gain a +{n}-foot status bonus to your speed"),
  phraseAnchor("teleport you to an unoccupied space"),
  phraseAnchor("transported with you"),
  phraseAnchor("reduce the movement penalty from difficult terrain"),
  phraseAnchor("hinder a creature or slow its movement"),
  phraseAnchor("circumstance penalty to speed"),
];

const SPELL_MOBILITY_BLOCKER_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("battle form"),
];

const AFFLICTION_MENTAL_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("confused"),
  tokenAnchor("confusion"),
  tokenAnchor("delirium"),
  tokenAnchor("paranoia"),
  tokenAnchor("frightened"),
  tokenAnchor("hallucination"),
  tokenAnchor("hallucinations"),
  tokenAnchor("whispers"),
  tokenAnchor("panic"),
];

const AFFLICTION_MOBILITY_TEXT_ANCHORS: TextAnchor[] = [
  phraseAnchor("speed is reduced"),
  phraseAnchor("reduces the victim s speed"),
  phraseAnchor("reduces your speed"),
  phraseAnchor("stiffens joints"),
  phraseAnchor("interferes with a victim s movement"),
  phraseAnchor("takes one or more steps in a random direction"),
  phraseAnchor("movement is forced"),
  tokenAnchor("immobilized"),
  tokenAnchor("paralyzed"),
  tokenAnchor("paralysis"),
  tokenAnchor("slowed"),
];

const AFFLICTION_PHYSICAL_DEBILITATION_TEXT_ANCHORS: TextAnchor[] = [
  tokenAnchor("drained"),
  tokenAnchor("enfeebled"),
  tokenAnchor("clumsy"),
  tokenAnchor("sickened"),
  tokenAnchor("fatigued"),
  phraseAnchor("drain health and strength"),
  phraseAnchor("drains health and strength"),
];

export type { DerivedTagContext, DerivedTagRule, TextAnchor, TextMatchScope, TextProximityConstraint } from "./matcher.js";
export { normalizeDerivedTag } from "./matcher.js";

export {
  tokenAnchor,
  phraseAnchor,
  templateAnchor,
  referenceAnchor,
  OFFENSIVE_TEXT_ANCHORS,
  GEARISH_SUBCATEGORIES,
  DISGUISE_SUBCATEGORIES,
  TRACKING_SUBCATEGORIES,
  STRONG_PROFESSION_NAME_ANCHORS,
  WEAK_PROFESSION_NAME_ANCHORS,
  UNDEAD_GLOSSARY_FAMILIES,
  SCENE_ADJACENT_BLOCKER_TRAITS,
  FRESHWATER_SETTING_TEXT_ANCHORS,
  FRESHWATER_SETTING_STRONG_TEXT_ANCHORS,
  FRESHWATER_SETTING_CONTEXT_TEXT_ANCHORS,
  FRESHWATER_SETTING_NAME_ANCHORS,
  FRESHWATER_SETTING_BLOCKER_TEXT_ANCHORS,
  FRESHWATER_SETTING_HABITAT_TEXT_NEAR,
  AQUATIC_SETTING_STRONG_TEXT_ANCHORS,
  AQUATIC_SETTING_WEAK_TEXT_ANCHORS,
  COASTAL_SETTING_STRONG_TEXT_ANCHORS,
  COASTAL_SETTING_WEAK_TEXT_ANCHORS,
  COASTAL_SETTING_NAME_ANCHORS,
  ASTRAL_SETTING_TEXT_ANCHORS,
  ASTRAL_SETTING_ACTIVITY_TEXT_ANCHORS,
  ASTRAL_SETTING_CONTEXT_TEXT_ANCHORS,
  FIRST_WORLD_SETTING_TEXT_ANCHORS,
  FIRST_WORLD_SETTING_CONTEXT_TEXT_ANCHORS,
  BONEYARD_SETTING_TEXT_ANCHORS,
  BONEYARD_SETTING_CONTEXT_TEXT_ANCHORS,
  DISGUISE_TEXT_ANCHORS,
  SOCIAL_INFILTRATION_TEXT_ANCHORS,
  DISGUISE_REFERENCE_ANCHORS,
  SOCIAL_INFILTRATION_REFERENCE_ANCHORS,
  SPELL_DISGUISE_NAME_ANCHORS,
  SPELL_DISGUISE_TEXT_ANCHORS,
  SPELL_DISGUISE_BLOCKER_TEXT_ANCHORS,
  SPELL_SOCIAL_INFILTRATION_TEXT_ANCHORS,
  RESTRAINT_ESCAPE_REFERENCE_ANCHORS,
  RESTRAINT_CAPTURE_REFERENCE_ANCHORS,
  STRONG_RESTRAINT_CAPTURE_NAME_ANCHORS,
  WEAK_RESTRAINT_CAPTURE_NAME_ANCHORS,
  MOBILITY_REFERENCE_ANCHORS,
  NAVIGATION_REFERENCE_ANCHORS,
  TRACKING_REFERENCE_ANCHORS,
  HAZARD_ALARM_TEXT_ANCHORS,
  HAZARD_FIRE_NAME_ANCHORS,
  HAZARD_FIRE_TEXT_ANCHORS,
  HAZARD_POISON_NAME_ANCHORS,
  HAZARD_POISON_TEXT_ANCHORS,
  HAZARD_PITFALL_NAME_ANCHORS,
  HAZARD_COLLAPSE_NAME_ANCHORS,
  HAZARD_FORCED_MOVEMENT_TEXT_ANCHORS,
  ALARM_NAME_ANCHORS,
  ALARM_STRONG_TEXT_ANCHORS,
  ALARM_TRIGGER_TEXT_ANCHORS,
  SIGNALING_NAME_ANCHORS,
  SIGNALING_TEXT_ANCHORS,
  MESSAGE_DELIVERY_EQUIPMENT_NAME_ANCHORS,
  MESSAGE_DELIVERY_SPELL_NAME_ANCHORS,
  MESSAGE_DELIVERY_TEXT_ANCHORS,
  MESSAGE_DELIVERY_BLOCKER_TEXT_ANCHORS,
  COUNTERMAGIC_NAME_ANCHORS,
  COUNTERMAGIC_TEXT_ANCHORS,
  COUNTERMAGIC_REFERENCE_ANCHORS,
  MAGIC_PROTECTION_TEXT_ANCHORS,
  SPELL_SCOUTING_NAME_ANCHORS,
  SPELL_SCOUTING_TEXT_ANCHORS,
  SPELL_SCOUTING_BLOCKER_TEXT_ANCHORS,
  SPELL_NAVIGATION_NAME_ANCHORS,
  SPELL_NAVIGATION_TEXT_ANCHORS,
  SPELL_MOBILITY_TEXT_ANCHORS,
  SPELL_MOBILITY_BLOCKER_TEXT_ANCHORS,
  AFFLICTION_MENTAL_TEXT_ANCHORS,
  AFFLICTION_MOBILITY_TEXT_ANCHORS,
  AFFLICTION_PHYSICAL_DEBILITATION_TEXT_ANCHORS,
};
