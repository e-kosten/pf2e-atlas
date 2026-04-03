import { DerivedTagCatalogEntry, SearchCategory, SearchSubcategory } from "../types.js";
import {
  deriveRecordTagsFromRules,
  DerivedTagContext,
  DerivedTagRule,
  normalizeDerivedTag,
  normalizeDerivedTagReference,
  TextAnchor,
  TextMatchScope,
  TextProximityConstraint,
} from "./matcher.js";
export { normalizeDerivedTag } from "./matcher.js";

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

const DERIVED_TAG_RULES: DerivedTagRule[] = [
  {
    tag: "offensive",
    category: "equipment",
    subcategories: ["consumable"],
    anyOf: [
      { traitsAny: ["poison", "bomb"] },
      { textAny: OFFENSIVE_TEXT_ANCHORS },
    ],
  },
  {
    tag: "thrown_offense",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["offensive"],
    anyOf: [
      { traitsAny: ["bomb"] },
      { textAny: [tokenAnchor("thrown"), tokenAnchor("hurl"), tokenAnchor("lob"), phraseAnchor("splash weapon"), phraseAnchor("throw the bomb")] },
    ],
  },
  {
    tag: "weapon_applied",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["offensive"],
    anyOf: [
      { textAny: [phraseAnchor("apply to a weapon"), phraseAnchor("coat a weapon"), phraseAnchor("weapon poison"), phraseAnchor("smeared on a weapon"), phraseAnchor("applied to a weapon")] },
    ],
  },
  {
    tag: "ingested_offense",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["offensive"],
    anyOf: [
      { textAny: [phraseAnchor("ingested poison"), phraseAnchor("must be eaten"), phraseAnchor("must be drunk"), phraseAnchor("consumed by the target"), phraseAnchor("when swallowed")] },
    ],
  },
  {
    tag: "contact_offense",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["offensive"],
    anyOf: [
      { textAny: [phraseAnchor("contact poison"), phraseAnchor("through skin contact"), phraseAnchor("through contact"), phraseAnchor("absorbed through the skin")] },
    ],
  },
  {
    tag: "beneficial",
    category: "equipment",
    subcategories: ["consumable"],
    anyOf: [
      { traitsAny: ["elixir", "healing"] },
      {
        textAny: [
          phraseAnchor("status bonus to"),
          phraseAnchor("item bonus to"),
          phraseAnchor("grants you"),
          phraseAnchor("giving you"),
          phraseAnchor("no longer need to breathe"),
          phraseAnchor("do not need air"),
          phraseAnchor("imprecise scent"),
          phraseAnchor("gain a burrow speed"),
          phraseAnchor("gain a climb speed"),
        ],
      },
      {
        textAny: [
          phraseAnchor("trigger you attempt to"),
          phraseAnchor("trigger you become"),
          phraseAnchor("requirements you re"),
          phraseAnchor("in case of kidnapping"),
          phraseAnchor("slippery enough to"),
        ],
        referencesAny: RESTRAINT_ESCAPE_REFERENCE_ANCHORS,
      },
      {
        textAny: [
          tokenAnchor("restorative"),
          tokenAnchor("remedy"),
          tokenAnchor("curative"),
          tokenAnchor("antidote"),
          tokenAnchor("antiplague"),
          tokenAnchor("protective"),
          tokenAnchor("catharsis"),
          tokenAnchor("healing"),
          tokenAnchor("darkvision"),
          phraseAnchor("additional protection"),
          phraseAnchor("absorb damage"),
          phraseAnchor("temporary hit points"),
          phraseAnchor("grants resistance"),
          phraseAnchor("gain resistance"),
          phraseAnchor("protect against poison"),
          phraseAnchor("protect against poisons"),
          phraseAnchor("protect against disease"),
          phraseAnchor("protects you against"),
          phraseAnchor("resistance to"),
          phraseAnchor("gain a bonus"),
          phraseAnchor("bolsters the drinker"),
          phraseAnchor("steady the emotions"),
          phraseAnchor("see in the dark"),
        ],
      },
    ],
    noneOf: [
      { traitsAny: ["poison", "bomb"] },
      { textAny: OFFENSIVE_TEXT_ANCHORS },
    ],
  },
  {
    tag: "healing_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { traitsAny: ["healing"] },
      { textAny: [phraseAnchor("elixir of life"), tokenAnchor("healing"), phraseAnchor("restore hit points"), phraseAnchor("restore hp"), phraseAnchor("regain hit points")] },
    ],
  },
  {
    tag: "anti_poison",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [tokenAnchor("antidote"), phraseAnchor("against poison"), phraseAnchor("against poisons"), phraseAnchor("protect against poison"), phraseAnchor("protects you against poisons"), phraseAnchor("resist poison"), phraseAnchor("ward off poison"), phraseAnchor("persistent poison damage")] },
    ],
  },
  {
    tag: "anti_disease",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [tokenAnchor("antiplague"), phraseAnchor("against disease"), phraseAnchor("against diseases"), phraseAnchor("protect against disease"), phraseAnchor("resist disease"), phraseAnchor("ward off disease")] },
    ],
  },
  {
    tag: "condition_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [tokenAnchor("condition"), tokenAnchor("catharsis"), phraseAnchor("soothe the mind"), phraseAnchor("steady the emotions"), phraseAnchor("calm overwhelming emotions"), phraseAnchor("recover from mental conditions")] },
    ],
  },
  {
    tag: "mental_recovery",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [phraseAnchor("soothe the mind"), phraseAnchor("steady the emotions"), phraseAnchor("calm overwhelming emotions"), phraseAnchor("mental condition"), phraseAnchor("mental conditions"), tokenAnchor("emotion"), tokenAnchor("emotions"), tokenAnchor("frightened"), tokenAnchor("stupefied"), tokenAnchor("confused"), phraseAnchor("mental effect")] },
    ],
  },
  {
    tag: "escape_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [tokenAnchor("escape"), phraseAnchor("slip away"), phraseAnchor("break free"), tokenAnchor("flee"), tokenAnchor("evade"), phraseAnchor("concealing smoke"), phraseAnchor("vanish from sight"), tokenAnchor("misty")] },
      { referencesAny: [referenceAnchor("actionspf2e", "Escape")] },
      { textAny: [tokenAnchor("grabbed"), tokenAnchor("restrained"), tokenAnchor("immobilized"), phraseAnchor("slip free"), phraseAnchor("break the grab")] },
    ],
  },
  {
    tag: "senses_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [tokenAnchor("darkvision"), phraseAnchor("see in the dark"), phraseAnchor("low light vision"), phraseAnchor("keen senses"), phraseAnchor("sharpen your vision"), phraseAnchor("see invisible"), tokenAnchor("scent")] },
    ],
  },
  {
    tag: "energy_resistance",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [phraseAnchor("resistance to fire"), phraseAnchor("resistance to cold"), phraseAnchor("resistance to electricity"), phraseAnchor("resistance to acid"), phraseAnchor("resistance to sonic"), phraseAnchor("resistance to energy"), phraseAnchor("energy resistance"), phraseAnchor("against fire damage"), phraseAnchor("against cold damage"), phraseAnchor("against electricity damage"), phraseAnchor("against acid damage"), phraseAnchor("against sonic damage")] },
    ],
  },
  {
    tag: "buff_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [phraseAnchor("gain a bonus"), phraseAnchor("bonus to"), tokenAnchor("bolster"), tokenAnchor("enhance"), tokenAnchor("empower"), phraseAnchor("heighten your senses"), phraseAnchor("increase your speed"), phraseAnchor("resistance to"), phraseAnchor("grants resistance"), phraseAnchor("temporary hit points"), phraseAnchor("additional protection"), phraseAnchor("absorb damage")] },
    ],
  },
  {
    tag: "self_buff",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [phraseAnchor("you gain"), phraseAnchor("the drinker gains"), phraseAnchor("gain a bonus"), phraseAnchor("you become"), phraseAnchor("you gain resistance"), phraseAnchor("you gain darkvision"), phraseAnchor("protects you against"), phraseAnchor("when you drink"), phraseAnchor("drinking this"), phraseAnchor("spreading the salve on exposed skin"), phraseAnchor("grants you"), phraseAnchor("giving you")] },
    ],
  },
  {
    tag: "ally_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: [phraseAnchor("target gains"), phraseAnchor("an ally gains"), phraseAnchor("creature that drinks gains"), phraseAnchor("the drinker gains")] },
    ],
  },
  {
    tag: "climbing",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("climb"), tokenAnchor("climbing"), tokenAnchor("rappel"), tokenAnchor("rappelling"), tokenAnchor("piton"), tokenAnchor("grappling")] },
    ],
  },
  {
    tag: "mobility",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    threshold: 2,
    anyOf: [
      { score: 2, textAny: [tokenAnchor("mobility"), phraseAnchor("move quickly"), phraseAnchor("increase your speed"), tokenAnchor("rappel"), tokenAnchor("climbing")] },
      {
        score: 2,
        textAny: [
          phraseAnchor("high jump"),
          phraseAnchor("long jump"),
          phraseAnchor("land speed"),
          phraseAnchor("climb speed"),
          phraseAnchor("climb speeds"),
          phraseAnchor("swim speed"),
          phraseAnchor("swim speeds"),
          phraseAnchor("ignore difficult terrain"),
          phraseAnchor("difficult terrain"),
          phraseAnchor("improved grip"),
          phraseAnchor("exceptional traction"),
          tokenAnchor("traction"),
        ],
      },
      { score: 2, referencesAny: MOBILITY_REFERENCE_ANCHORS },
    ],
  },
  {
    tag: "lock_bypass",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("lockpick"), tokenAnchor("lockpicks"), phraseAnchor("pick locks"), phraseAnchor("picking locks"), phraseAnchor("bypass locks"), phraseAnchor("thieves tools"), phraseAnchor("thieves tools"), tokenAnchor("toolkit")] },
    ],
  },
  {
    tag: "concealable",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("concealable"), phraseAnchor("hidden on your person"), phraseAnchor("hidden tools"), phraseAnchor("slim lockpicks")] },
    ],
  },
  {
    tag: "scouting",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("scout"), tokenAnchor("scouting"), tokenAnchor("survey"), tokenAnchor("recon"), phraseAnchor("observe from afar"), tokenAnchor("spyglass")] },
    ],
  },
  {
    tag: "stealth_support",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("stealth"), tokenAnchor("quiet"), tokenAnchor("silent"), phraseAnchor("without drawing attention"), phraseAnchor("avoid notice"), tokenAnchor("infiltration")] },
      { textAny: [tokenAnchor("concealable"), phraseAnchor("hidden on your person")] },
    ],
  },
  {
    tag: "disguise",
    category: "equipment",
    subcategories: DISGUISE_SUBCATEGORIES,
    anyOf: [
      { textAny: DISGUISE_TEXT_ANCHORS },
      { referencesAny: DISGUISE_REFERENCE_ANCHORS },
    ],
  },
  {
    tag: "social_infiltration",
    category: "equipment",
    subcategories: DISGUISE_SUBCATEGORIES,
    requiresTags: ["disguise"],
    anyOf: [
      { textAny: SOCIAL_INFILTRATION_TEXT_ANCHORS },
      { referencesAny: SOCIAL_INFILTRATION_REFERENCE_ANCHORS },
    ],
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
            terms: [tokenAnchor("detect"), tokenAnchor("undead")],
            window: 4,
            scope: "description",
          },
        ],
      },
    ],
    noneOf: [
      { textAny: SPELL_DISGUISE_BLOCKER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "social_infiltration",
    category: "spell",
    requiresTags: ["disguise"],
    anyOf: [
      { textAny: SOCIAL_INFILTRATION_TEXT_ANCHORS },
      { textAny: SPELL_DISGUISE_TEXT_ANCHORS },
    ],
    noneOf: [
      { textAny: SPELL_DISGUISE_BLOCKER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "social_infiltration",
    category: "spell",
    anyOf: [
      { textAny: SPELL_SOCIAL_INFILTRATION_TEXT_ANCHORS },
    ],
  },
  {
    tag: "social_infiltration",
    category: "spell",
    allOf: [
      { textAny: [tokenAnchor("mask", "name"), tokenAnchor("mask", "description")] },
    ],
    anyOf: [
      {
        textNear: [
          {
            terms: [tokenAnchor("deception"), tokenAnchor("lie"), tokenAnchor("feint")],
            window: 6,
            scope: "description",
            minTermsMatched: 2,
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
    noneOf: [
      { textAny: ALARM_STRONG_TEXT_ANCHORS },
    ],
  },
  {
    tag: "message_delivery",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: MESSAGE_DELIVERY_SPELL_NAME_ANCHORS },
      { score: 2, textAny: MESSAGE_DELIVERY_TEXT_ANCHORS },
    ],
    noneOf: [
      { textAny: MESSAGE_DELIVERY_BLOCKER_TEXT_ANCHORS },
    ],
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
        textAll: [
          tokenAnchor("senses"),
          tokenAnchor("through"),
        ],
        textAny: [
          phraseAnchor("through the ear"),
          phraseAnchor("through its eyes"),
          phraseAnchor("through each other s eyes"),
        ],
      },
    ],
    noneOf: [
      { textAny: SPELL_SCOUTING_BLOCKER_TEXT_ANCHORS },
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
      { score: 2, textAny: SPELL_MOBILITY_TEXT_ANCHORS },
    ],
    noneOf: [
      { textAny: SPELL_MOBILITY_BLOCKER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "illumination",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("light"), tokenAnchor("illumination"), tokenAnchor("lantern"), tokenAnchor("torch"), tokenAnchor("glow"), tokenAnchor("illuminate")] },
    ],
  },
  {
    tag: "survival",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("camp"), tokenAnchor("forage"), tokenAnchor("wilderness"), tokenAnchor("survival"), tokenAnchor("shelter"), tokenAnchor("weatherproof")] },
    ],
  },
  {
    tag: "navigation",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      {
        textAny: [
          tokenAnchor("navigate"),
          tokenAnchor("navigation"),
          tokenAnchor("map"),
          tokenAnchor("compass"),
          tokenAnchor("chart"),
          phraseAnchor("sense direction"),
          phraseAnchor("track your heading"),
          phraseAnchor("which direction is north"),
          phraseAnchor("learn which direction you re facing"),
        ],
      },
      { referencesAny: NAVIGATION_REFERENCE_ANCHORS },
    ],
  },
  {
    tag: "navigation",
    category: "equipment",
    subcategories: ["consumable"],
    threshold: 2,
    anyOf: [
      { score: 2, referencesAny: NAVIGATION_REFERENCE_ANCHORS },
      {
        score: 2,
        textAny: [
          phraseAnchor("sense direction"),
          phraseAnchor("functions as if you have a compass"),
          phraseAnchor("learn which direction you re facing"),
          phraseAnchor("which direction is north"),
        ],
      },
    ],
  },
  {
    tag: "tracking",
    category: "equipment",
    subcategories: TRACKING_SUBCATEGORIES,
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          phraseAnchor("sense and follow tracks"),
          phraseAnchor("for later tracking"),
          phraseAnchor("track their movements"),
          phraseAnchor("track a creature by its scent"),
          phraseAnchor("continue to track the same creature"),
          phraseAnchor("flutters toward the affixed one"),
        ],
      },
      {
        score: 2,
        textAll: [
          tokenAnchor("survival"),
          phraseAnchor("sense direction"),
          tokenAnchor("track"),
        ],
      },
      {
        score: 2,
        textAll: [
          tokenAnchor("survival"),
          tokenAnchor("scent"),
        ],
        referencesAny: [referenceAnchor("actionspf2e", "Track")],
      },
      {
        score: 2,
        textAll: [
          tokenAnchor("survival"),
          tokenAnchor("scent"),
          tokenAnchor("track"),
        ],
      },
      { score: 1, textAny: [tokenAnchor("tracker", "name"), tokenAnchor("tracking", "name")] },
    ],
    noneOf: [
      {
        textAny: [
          phraseAnchor("track time"),
          phraseAnchor("track teleportation"),
          phraseAnchor("dc to track"),
          phraseAnchor("attempting to track you"),
          phraseAnchor("circumstance bonus to their check"),
        ],
      },
      {
        textNear: [
          {
            terms: [tokenAnchor("tracking"), tokenAnchor("progress")],
            window: 3,
            scope: "description",
          },
          {
            terms: [tokenAnchor("track"), tokenAnchor("teleportation")],
            window: 2,
            scope: "description",
          },
          {
            terms: [tokenAnchor("track"), tokenAnchor("time")],
            window: 2,
            scope: "description",
          },
        ],
      },
      {
        textAny: [tokenAnchor("trackless", "name")],
      },
    ],
  },
  {
    tag: "anti_tracking",
    category: "equipment",
    subcategories: TRACKING_SUBCATEGORIES,
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          phraseAnchor("dc to track"),
          phraseAnchor("attempting to track you"),
          phraseAnchor("cover any ordinary odors"),
          phraseAnchor("fleeing pursuit"),
        ],
      },
      {
        score: 2,
        textAll: [
          tokenAnchor("stealth"),
          tokenAnchor("hide"),
          tokenAnchor("sneak"),
        ],
        textAny: [
          phraseAnchor("against creatures using primarily smell"),
        ],
      },
      { score: 2, textAny: [tokenAnchor("trackless", "name")] },
    ],
    noneOf: [
      {
        textAny: [
          phraseAnchor("track a creature by its scent"),
          phraseAnchor("track their movements"),
          phraseAnchor("continue to track the same creature"),
          phraseAnchor("sense and follow tracks"),
          phraseAnchor("circumstance bonus to their check"),
        ],
      },
    ],
  },
  {
    tag: "transport",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("transport"), tokenAnchor("wagon"), tokenAnchor("sled"), tokenAnchor("boat"), tokenAnchor("vehicle"), phraseAnchor("carry riders"), phraseAnchor("haul passengers")] },
    ],
  },
  {
    tag: "signaling",
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    threshold: 2,
    anyOf: [
      { score: 1, textAny: SIGNALING_NAME_ANCHORS },
      { score: 2, textAny: SIGNALING_TEXT_ANCHORS },
    ],
  },
  {
    tag: "message_delivery",
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: MESSAGE_DELIVERY_EQUIPMENT_NAME_ANCHORS },
      { score: 2, textAny: MESSAGE_DELIVERY_TEXT_ANCHORS },
    ],
    noneOf: [
      { textAny: MESSAGE_DELIVERY_BLOCKER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "alarm",
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    threshold: 2,
    anyOf: [
      { score: 1, textAny: ALARM_NAME_ANCHORS },
      { score: 2, textAny: ALARM_STRONG_TEXT_ANCHORS },
      { score: 1, textAny: ALARM_TRIGGER_TEXT_ANCHORS },
    ],
  },
  {
    tag: "trap_bypass",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [phraseAnchor("disarm a trap"), phraseAnchor("disable device"), phraseAnchor("trap mechanism"), tokenAnchor("tripwire"), phraseAnchor("bypass a trap")] },
    ],
  },
  {
    tag: "carry_support",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: [tokenAnchor("storage"), tokenAnchor("stow"), tokenAnchor("carry"), tokenAnchor("haul"), tokenAnchor("pouch"), tokenAnchor("backpack"), tokenAnchor("container"), tokenAnchor("pack")] },
    ],
  },
  {
    tag: "countermagic",
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: COUNTERMAGIC_NAME_ANCHORS },
      { score: 2, textAny: COUNTERMAGIC_TEXT_ANCHORS },
      { score: 2, referencesAny: COUNTERMAGIC_REFERENCE_ANCHORS },
      {
        score: 2,
        textAll: [
          tokenAnchor("counteract"),
        ],
        textAny: [
          phraseAnchor("magic effect"),
          phraseAnchor("magical effect"),
          phraseAnchor("magical effects"),
          phraseAnchor("magic item"),
          phraseAnchor("magical darkness"),
          phraseAnchor("triggering spell"),
          phraseAnchor("single spell"),
          phraseAnchor("target spell"),
        ],
      },
    ],
  },
  {
    tag: "magic_protection",
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    threshold: 2,
    anyOf: [
      { score: 2, textAny: [tokenAnchor("antimagic", "name")] },
      { score: 2, textAny: MAGIC_PROTECTION_TEXT_ANCHORS },
    ],
  },
  {
    tag: "restraint_escape",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          phraseAnchor("break free"),
          phraseAnchor("free of a creature grabbing you"),
          phraseAnchor("difficult to hold back"),
          phraseAnchor("whenever you are affected by an effect that lasts until you"),
          phraseAnchor("trigger you become"),
        ],
        referencesAny: RESTRAINT_ESCAPE_REFERENCE_ANCHORS,
      },
      {
        score: 2,
        textAny: [
          phraseAnchor("free someone from manacles"),
          phraseAnchor("free yourself from manacles"),
        ],
      },
      {
        score: 2,
        textAll: [
          phraseAnchor("bonus to athletics checks"),
        ],
        referencesAny: [referenceAnchor("actionspf2e", "Escape")],
      },
    ],
    noneOf: [
      {
        textAny: [
          phraseAnchor("target becomes grabbed"),
          phraseAnchor("target becomes restrained"),
          phraseAnchor("creature becomes grabbed"),
          phraseAnchor("creature becomes restrained"),
          phraseAnchor("restrain the target"),
          phraseAnchor("grabs the target"),
          phraseAnchor("tighten the loop"),
          phraseAnchor("escape dc"),
          phraseAnchor("can continue to grapple to keep your hold on the target"),
          phraseAnchor("gains a bonus to escape"),
          phraseAnchor("escape the net"),
        ],
      },
    ],
  },
  {
    tag: "restraint_capture",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          phraseAnchor("target becomes restrained"),
          phraseAnchor("creature becomes restrained"),
          phraseAnchor("restrain the target"),
        ],
      },
      {
        score: 2,
        textAny: [
          phraseAnchor("tighten the loop"),
          phraseAnchor("without having a free hand"),
          phraseAnchor("up to 10 feet away"),
        ],
        referencesAny: RESTRAINT_CAPTURE_REFERENCE_ANCHORS,
      },
      {
        score: 2,
        textAny: STRONG_RESTRAINT_CAPTURE_NAME_ANCHORS,
      },
      {
        score: 2,
        textAny: WEAK_RESTRAINT_CAPTURE_NAME_ANCHORS,
        referencesAny: RESTRAINT_CAPTURE_REFERENCE_ANCHORS,
      },
      {
        score: 2,
        textAny: [
          phraseAnchor("functions as a typical net"),
          phraseAnchor("grapple with the net"),
        ],
        referencesAny: [referenceAnchor("actionspf2e", "Grapple")],
      },
    ],
    noneOf: [
      {
        textAny: [
          phraseAnchor("break free"),
          phraseAnchor("free someone from manacles"),
          phraseAnchor("free yourself from manacles"),
        ],
      },
    ],
  },
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
    tag: "countermagic",
    category: "spell",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: COUNTERMAGIC_NAME_ANCHORS },
      { score: 2, textAny: COUNTERMAGIC_TEXT_ANCHORS },
      { score: 2, referencesAny: COUNTERMAGIC_REFERENCE_ANCHORS },
      {
        score: 2,
        textAll: [
          tokenAnchor("counteract"),
        ],
        textAny: [
          phraseAnchor("magic effect"),
          phraseAnchor("magical effect"),
          phraseAnchor("magical effects"),
          phraseAnchor("magic item"),
          phraseAnchor("magical darkness"),
          phraseAnchor("triggering spell"),
          phraseAnchor("single spell"),
          phraseAnchor("target spell"),
        ],
      },
    ],
  },
  {
    tag: "undead_threat",
    category: "creature",
    anyOf: [
      { traitsAny: ["undead", "ghost", "spirit", "skeleton", "ghoul"] },
      { familiesAny: UNDEAD_GLOSSARY_FAMILIES },
    ],
  },
  {
    tag: "fey_threat",
    category: "creature",
    anyOf: [
      { traitsAny: ["fey"] },
    ],
  },
  {
    tag: "plant_threat",
    category: "creature",
    anyOf: [
      { traitsAny: ["plant", "fungus", "leshy"] },
    ],
  },
  {
    tag: "freshwater_setting",
    category: "creature",
    threshold: 2,
    noneOf: [
      { textAny: FRESHWATER_SETTING_BLOCKER_TEXT_ANCHORS },
    ],
    anyOf: [
      { score: 2, textAny: FRESHWATER_SETTING_STRONG_TEXT_ANCHORS },
      { score: 2, textNear: FRESHWATER_SETTING_HABITAT_TEXT_NEAR },
      { score: 1, textAny: FRESHWATER_SETTING_NAME_ANCHORS },
      { score: 1, traitsAny: ["water"] },
    ],
  },
  {
    tag: "aquatic_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 3, traitsAny: ["water", "aquatic", "amphibious"] },
      { score: 2, textAny: AQUATIC_SETTING_STRONG_TEXT_ANCHORS },
      { score: 1, textAny: AQUATIC_SETTING_WEAK_TEXT_ANCHORS },
    ],
  },
  {
    tag: "aquatic_setting",
    category: "creature",
    requiresTags: ["freshwater_setting"],
  },
  {
    tag: "coastal_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: COASTAL_SETTING_STRONG_TEXT_ANCHORS },
      { score: 1, textAny: COASTAL_SETTING_NAME_ANCHORS },
      { score: 1, textAny: COASTAL_SETTING_WEAK_TEXT_ANCHORS },
    ],
  },
  {
    tag: "astral_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["astral"] },
    ],
  },
  {
    tag: "astral_setting",
    category: "creature",
    anyOf: [
      { textAny: ASTRAL_SETTING_TEXT_ANCHORS },
    ],
  },
  {
    tag: "astral_setting",
    category: "creature",
    allOf: [
      { textAny: [tokenAnchor("astral")] },
      { textAny: ASTRAL_SETTING_CONTEXT_TEXT_ANCHORS },
    ],
  },
  {
    tag: "first_world_setting",
    category: "creature",
    allOf: [
      { textAny: FIRST_WORLD_SETTING_TEXT_ANCHORS },
    ],
    threshold: 1,
    anyOf: [
      { score: 1, traitsAny: ["fey", "tane"] },
      { score: 1, textAny: FIRST_WORLD_SETTING_CONTEXT_TEXT_ANCHORS },
    ],
  },
  {
    tag: "boneyard_setting",
    category: "creature",
    allOf: [
      { textAny: BONEYARD_SETTING_TEXT_ANCHORS },
    ],
    threshold: 1,
    anyOf: [
      { score: 1, traitsAny: ["psychopomp"] },
      { score: 1, textAny: BONEYARD_SETTING_CONTEXT_TEXT_ANCHORS },
    ],
  },
  {
    tag: "island_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: [tokenAnchor("island"), tokenAnchor("islands"), tokenAnchor("archipelago"), tokenAnchor("archipelagos"), tokenAnchor("atoll"), tokenAnchor("atolls")] },
      { score: 1, textAny: [tokenAnchor("isle"), tokenAnchor("isles")] },
    ],
  },
  {
    tag: "nautical_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          tokenAnchor("sailor"),
          tokenAnchor("sailors"),
          tokenAnchor("mariner"),
          tokenAnchor("mariners"),
          tokenAnchor("pirate"),
          tokenAnchor("pirates"),
          tokenAnchor("corsair"),
          tokenAnchor("corsairs"),
          tokenAnchor("dock"),
          tokenAnchor("docks"),
          tokenAnchor("bilge"),
          tokenAnchor("wreck"),
          tokenAnchor("crew"),
          tokenAnchor("crews"),
          tokenAnchor("rigging"),
          tokenAnchor("mast"),
          tokenAnchor("masts"),
          tokenAnchor("shipwreck"),
          tokenAnchor("shipwrecks"),
          phraseAnchor("shipwreck"),
        ],
      },
      { score: 1, textAny: [tokenAnchor("ship"), tokenAnchor("ships"), tokenAnchor("captain"), tokenAnchor("vessel"), tokenAnchor("vessels"), tokenAnchor("deck"), tokenAnchor("decks")] },
      { score: 1, textAny: [tokenAnchor("harbor"), tokenAnchor("harbors")] },
    ],
  },
  {
    tag: "forest_setting",
    category: "creature",
    anyOf: [
      {
        textAny: [
          tokenAnchor("forest"),
          tokenAnchor("forests"),
          tokenAnchor("woodland"),
          tokenAnchor("woodlands"),
          tokenAnchor("woods"),
          tokenAnchor("grove"),
          tokenAnchor("groves"),
          tokenAnchor("briar"),
          tokenAnchor("jungle"),
          tokenAnchor("jungles"),
          tokenAnchor("rainforest"),
        ],
      },
    ],
  },
  {
    tag: "plains_setting",
    category: "creature",
    anyOf: [
      {
        textAny: [
          tokenAnchor("plains"),
          tokenAnchor("grassland"),
          tokenAnchor("grasslands"),
          tokenAnchor("prairie"),
          tokenAnchor("prairies"),
          tokenAnchor("savanna"),
          tokenAnchor("savannah"),
          tokenAnchor("steppe"),
          tokenAnchor("steppes"),
        ],
      },
    ],
  },
  {
    tag: "canyon_setting",
    category: "creature",
    anyOf: [
      {
        textAny: [
          tokenAnchor("canyon"),
          tokenAnchor("canyons"),
          tokenAnchor("gorge"),
          tokenAnchor("gorges"),
          tokenAnchor("mesa"),
          tokenAnchor("mesas"),
          tokenAnchor("badlands"),
        ],
      },
    ],
  },
  {
    tag: "swamp_setting",
    category: "creature",
    anyOf: [
      { traitsAny: ["boggard"] },
      {
        textAny: [
          tokenAnchor("swamp"),
          tokenAnchor("swamps"),
          tokenAnchor("bog"),
          tokenAnchor("bogs"),
          tokenAnchor("marsh"),
          tokenAnchor("marshes"),
          tokenAnchor("fen"),
          tokenAnchor("fens"),
          tokenAnchor("mire"),
          tokenAnchor("mires"),
        ],
      },
    ],
  },
  {
    tag: "underground_setting",
    category: "creature",
    anyOf: [
      {
        textAny: [
          tokenAnchor("cave"),
          tokenAnchor("caves"),
          tokenAnchor("cavern"),
          tokenAnchor("caverns"),
          tokenAnchor("underground"),
          tokenAnchor("tunnel"),
          tokenAnchor("tunnels"),
          tokenAnchor("subterranean"),
          tokenAnchor("underworld"),
          tokenAnchor("depths"),
          tokenAnchor("crypt"),
          tokenAnchor("crypts"),
        ],
      },
    ],
  },
  {
    tag: "urban_setting",
    category: "creature",
    anyOf: [
      {
        textAny: [
          tokenAnchor("city"),
          tokenAnchor("cities"),
          tokenAnchor("urban"),
          tokenAnchor("street"),
          tokenAnchor("streets"),
          tokenAnchor("alley"),
          tokenAnchor("alleys"),
          tokenAnchor("market"),
          tokenAnchor("markets"),
          tokenAnchor("sewer"),
          tokenAnchor("sewers"),
          tokenAnchor("town"),
          tokenAnchor("towns"),
        ],
      },
    ],
  },
  {
    tag: "arctic_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: [tokenAnchor("arctic"), tokenAnchor("snow"), tokenAnchor("snows"), tokenAnchor("tundra"), tokenAnchor("frozen"), tokenAnchor("glacier"), tokenAnchor("glaciers"), tokenAnchor("icebound")] },
      { score: 1, textAny: [tokenAnchor("ice"), tokenAnchor("icy")] },
    ],
  },
  {
    tag: "desert_setting",
    category: "creature",
    anyOf: [
      { textAny: [tokenAnchor("desert"), tokenAnchor("deserts"), tokenAnchor("dune"), tokenAnchor("dunes"), tokenAnchor("sand"), tokenAnchor("arid")] },
    ],
  },
  {
    tag: "wasteland_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          tokenAnchor("wasteland"),
          tokenAnchor("wastelands"),
          tokenAnchor("wastes"),
          tokenAnchor("barren"),
        ],
      },
      { score: 1, textAny: [tokenAnchor("blasted")] },
    ],
  },
  {
    tag: "mountain_setting",
    category: "creature",
    anyOf: [
      { textAny: [tokenAnchor("mountain"), tokenAnchor("mountains"), tokenAnchor("cliff"), tokenAnchor("cliffs"), tokenAnchor("peak"), tokenAnchor("peaks"), tokenAnchor("crag"), tokenAnchor("crags"), tokenAnchor("alp"), tokenAnchor("alpine"), tokenAnchor("pass"), tokenAnchor("passes")] },
    ],
  },
  {
    tag: "graveyard_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          tokenAnchor("graveyard"),
          tokenAnchor("graveyards"),
          tokenAnchor("cemetery"),
          tokenAnchor("cemeteries"),
          tokenAnchor("mausoleum"),
          tokenAnchor("mausoleums"),
          tokenAnchor("barrow"),
          tokenAnchor("barrows"),
          tokenAnchor("sepulcher"),
          tokenAnchor("sepulchers"),
          tokenAnchor("cairn"),
          tokenAnchor("cairns"),
          phraseAnchor("burial ground"),
          phraseAnchor("burial grounds"),
        ],
      },
      { score: 1, textAny: [tokenAnchor("crypt"), tokenAnchor("crypts")] },
      { score: 1, textAny: [tokenAnchor("tomb"), tokenAnchor("tombs")] },
    ],
  },
  {
    tag: "ruins_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textNear: [
          {
            terms: [
              tokenAnchor("ruin"),
              tokenAnchor("ruins"),
              tokenAnchor("derelict"),
              tokenAnchor("crumbling"),
              tokenAnchor("fallen"),
              tokenAnchor("collapsed"),
              tokenAnchor("hall"),
              tokenAnchor("temple"),
            ],
            window: 3,
            scope: "description",
            minTermsMatched: 2,
          },
        ],
      },
      {
        score: 2,
        textAny: [
          phraseAnchor("ancient ruins"),
          phraseAnchor("ruins of"),
          phraseAnchor("collapsed temple"),
          phraseAnchor("ruined temple"),
          phraseAnchor("ruined city"),
          phraseAnchor("lost city"),
          phraseAnchor("derelict structures"),
          phraseAnchor("derelict structure"),
        ],
      },
      { score: 1, textAny: [tokenAnchor("ruins"), tokenAnchor("ruin"), tokenAnchor("derelict")] },
    ],
  },
  {
    tag: "temple_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          tokenAnchor("temple"),
          tokenAnchor("temples"),
          tokenAnchor("shrine"),
          tokenAnchor("shrines"),
          tokenAnchor("cathedral"),
          tokenAnchor("cathedrals"),
          tokenAnchor("monastery"),
          tokenAnchor("monasteries"),
          tokenAnchor("chapel"),
          tokenAnchor("chapels"),
        ],
      },
      { score: 1, textAny: [phraseAnchor("place of worship"), phraseAnchor("house of worship")] },
    ],
  },
  {
    tag: "fortress_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          tokenAnchor("fortress"),
          tokenAnchor("fortresses"),
          tokenAnchor("castle"),
          tokenAnchor("castles"),
          tokenAnchor("citadel"),
          tokenAnchor("citadels"),
          tokenAnchor("stronghold"),
          tokenAnchor("strongholds"),
        ],
      },
      { score: 2, textAny: [phraseAnchor("sky citadel")] },
      { score: 1, textAny: [tokenAnchor("fort"), tokenAnchor("forts")] },
    ],
  },
  {
    tag: "volcanic_setting",
    category: "creature",
    threshold: 2,
    anyOf: [
      {
        score: 2,
        textAny: [
          tokenAnchor("volcanic"),
          tokenAnchor("volcano"),
          tokenAnchor("volcanoes"),
          tokenAnchor("lava"),
          tokenAnchor("magma"),
          tokenAnchor("caldera"),
        ],
      },
      { score: 1, textAny: [tokenAnchor("ash"), tokenAnchor("ashen"), tokenAnchor("cinder"), tokenAnchor("cinders")] },
    ],
  },
  {
    tag: "profession_npc",
    category: "creature",
    threshold: 2,
    anyOf: [
      { score: 2, textAny: STRONG_PROFESSION_NAME_ANCHORS },
      { score: 1, textAny: WEAK_PROFESSION_NAME_ANCHORS },
      { score: 1, traitsAny: ["humanoid", "human"] },
    ],
  },
  {
    tag: "scene_adjacent",
    category: "creature",
    requiresTags: ["profession_npc"],
    noneOf: [
      { traitsAny: SCENE_ADJACENT_BLOCKER_TRAITS },
      { familiesAny: UNDEAD_GLOSSARY_FAMILIES },
    ],
  },
];

export const DERIVED_TAG_CATALOG: DerivedTagCatalogEntry[] = [
  {
    category: "equipment",
    subcategories: ["consumable"],
    family: "function",
    description: "Beneficial consumable outcome and recovery tags.",
    tags: [
      { value: "beneficial", description: "Broad support-oriented consumable with non-hostile intent." },
      { value: "healing_support", description: "Restores hit points or provides direct healing." },
      { value: "anti_poison", description: "Helps resist, prevent, or recover from poison." },
      { value: "anti_disease", description: "Helps resist, prevent, or recover from disease." },
      { value: "condition_support", description: "Helps clear or mitigate harmful conditions." },
      { value: "mental_recovery", description: "Helps stabilize emotions or recover from mental conditions." },
      { value: "escape_support", description: "Helps flee, slip away, or break free." },
      { value: "senses_support", description: "Improves vision or other senses." },
      { value: "energy_resistance", description: "Grants resistance against one or more energy types." },
      { value: "buff_support", description: "Provides a general beneficial enhancement or bonus." },
    ],
  },
  {
    category: "equipment",
    subcategories: ["consumable"],
    family: "polarity",
    description: "Offense/support polarity and delivery-style consumable tags.",
    tags: [
      { value: "offensive", description: "Hostile consumable primarily meant to harm or debilitate a target." },
      { value: "self_buff", description: "Support consumable primarily applied to the user." },
      { value: "ally_support", description: "Support consumable that can directly benefit another creature." },
      { value: "weapon_applied", description: "Offensive consumable applied to a weapon before use." },
      { value: "thrown_offense", description: "Offensive consumable delivered by throwing it." },
      { value: "ingested_offense", description: "Offensive consumable delivered when swallowed or consumed." },
      { value: "contact_offense", description: "Offensive consumable delivered through touch or skin contact." },
    ],
  },
  {
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    family: "purpose",
    description: "Utility and logistics gear-purpose tags.",
    tags: [
      { value: "climbing", description: "Helps climb, rappel, or navigate vertical obstacles." },
      { value: "lock_bypass", description: "Helps open locks or bypass secured entry points." },
      { value: "concealable", description: "Easy to hide on the person or carry discreetly." },
      { value: "scouting", description: "Helps observe, survey, or reconnoiter an area." },
      { value: "mobility", description: "Improves movement or traversal flexibility." },
      { value: "stealth_support", description: "Helps move quietly or avoid notice." },
      { value: "illumination", description: "Produces or improves light in dark environments." },
      { value: "survival", description: "Supports wilderness travel, shelter, or long-term field use." },
      { value: "navigation", description: "Helps track direction, route, or position." },
      { value: "tracking", description: "Helps follow trails, mark a target, or relocate something later." },
      { value: "anti_tracking", description: "Helps hide your trail, mask scent, or make pursuit harder." },
      { value: "transport", description: "Helps move creatures or cargo from place to place." },
      { value: "trap_bypass", description: "Helps disarm, disable, or get past traps." },
      { value: "carry_support", description: "Helps stow, carry, or organize equipment." },
      { value: "restraint_escape", description: "Helps break free from grabs, restraints, or similar immobilizing holds." },
      { value: "restraint_capture", description: "Helps capture, bind, or keep a target restrained." },
    ],
  },
  {
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    family: "communication",
    description: "Coordination, signaling, and message-relay equipment.",
    tags: [
      { value: "signaling", description: "Helps draw attention, mark a location, or coordinate allies." },
      { value: "message_delivery", description: "Sends, stores, or relays actual content across time or distance." },
    ],
  },
  {
    category: "equipment",
    subcategories: DISGUISE_SUBCATEGORIES,
    family: "infiltration",
    description: "Appearance-changing and social-passing equipment across gear and consumables.",
    tags: [
      { value: "disguise", description: "Helps alter appearance or impersonate another identity." },
      { value: "social_infiltration", description: "Helps blend into a group or pass under social scrutiny." },
    ],
  },
  {
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    family: "magic_interference",
    description: "Equipment that disrupts hostile magic or protects against it.",
    tags: [
      { value: "countermagic", description: "Counteracts, dispels, suppresses, or shuts down magic." },
      { value: "magic_protection", description: "Protects the user or target against hostile magical effects." },
    ],
  },
  {
    category: "spell",
    family: "infiltration",
    description: "Appearance-changing and social-passing spells.",
    tags: [
      { value: "disguise", description: "Helps alter appearance or impersonate another identity." },
      { value: "social_infiltration", description: "Helps blend into a group or pass under social scrutiny." },
    ],
  },
  {
    category: "spell",
    family: "communication",
    description: "Spells for signaling, telepathy, and message exchange.",
    tags: [
      { value: "signaling", description: "Helps draw attention, mark a location, or coordinate allies." },
      { value: "message_delivery", description: "Sends, stores, or relays actual content across time or distance." },
    ],
  },
  {
    category: "spell",
    family: "reconnaissance",
    description: "Remote-observation and scouting spells.",
    tags: [
      { value: "scouting", description: "Helps observe at a distance, extend senses, or locate a target." },
    ],
  },
  {
    category: "spell",
    family: "wayfinding",
    description: "Spells that guide direction, route-finding, or destination travel.",
    tags: [
      { value: "navigation", description: "Helps orient, guide a route, or identify a destination's direction." },
    ],
  },
  {
    category: "spell",
    family: "traversal",
    description: "Spells that improve movement modes, speed, or practical traversal.",
    tags: [
      { value: "mobility", description: "Helps move faster, gain movement modes, or traverse terrain more effectively." },
    ],
  },
  {
    category: "spell",
    family: "magic_interference",
    description: "Spells that disrupt, dispel, or suppress magic.",
    tags: [
      { value: "countermagic", description: "Counteracts, dispels, suppresses, or shuts down magic." },
    ],
  },
  {
    category: "spell",
    family: "security",
    description: "Area-warning and intrusion-alert spells.",
    tags: [
      { value: "alarm", description: "Alerts you or others when a watched area, threshold, or ward is crossed." },
    ],
  },
  {
    category: "equipment",
    subcategories: [...GEARISH_SUBCATEGORIES, "consumable"],
    family: "security",
    description: "Intrusion-warning gear and consumables.",
    tags: [
      { value: "alarm", description: "Alerts you or others when a watched area, threshold, or device is triggered." },
    ],
  },
  {
    category: "hazard",
    family: "function",
    description: "Hazard practical-function tags for alerts and restraint effects.",
    tags: [
      { value: "alarm", description: "Alerts guardians, onlookers, or nearby creatures to an intrusion." },
      { value: "restraint_capture", description: "Hazard that binds, restrains, or holds intruders in place." },
      { value: "barrier_lockdown", description: "Hazard that seals, closes, or blocks passage to trap or delay intruders." },
    ],
  },
  {
    category: "hazard",
    family: "impact",
    description: "Hazard impact tags for mental destabilization and movement-limiting effects.",
    tags: [
      { value: "mental_impairment", description: "Impairs judgment, emotions, or perception through fear, confusion, or similar effects." },
      { value: "mobility_impairment", description: "Paralyzes, immobilizes, or otherwise heavily hampers movement." },
    ],
  },
  {
    category: "hazard",
    family: "environmental_danger",
    description: "Hazards defined by recurring elemental or toxic environmental threats.",
    tags: [
      { value: "fire_hazard", description: "Hazard centered on open fire, flames, burning spread, or explosive ignition." },
      { value: "poison_hazard", description: "Hazard centered on poison gas, toxic delivery, or other poisonous exposure." },
    ],
  },
  {
    category: "hazard",
    family: "forced_position",
    description: "Hazards that drop, collapse, or forcibly reposition creatures.",
    tags: [
      { value: "pitfall", description: "Hazard built around a concealed pit, drop, or similar vertical fall trap." },
      { value: "collapse_hazard", description: "Hazard built around collapsing structures, cave-ins, rockfalls, or crumbling ground." },
      { value: "forced_movement", description: "Hazard that pushes, pulls, drags, submerges, or otherwise forcibly repositions creatures." },
    ],
  },
  {
    category: "affliction",
    family: "impact",
    description: "Affliction impact tags for practical downstream consequences.",
    tags: [
      { value: "mental_impairment", description: "Impairs judgment, emotions, or perception through confusion, fear, or delirium." },
      { value: "mobility_impairment", description: "Reduces speed, stiffens movement, or leaves the victim immobilized." },
      { value: "physical_debilitation", description: "Weakens the body through exhaustion, sickness, drained vitality, or similar bodily degradation." },
    ],
  },
  {
    category: "creature",
    family: "setting",
    description: "Creature environment and setting tags.",
    tags: [
      { value: "aquatic_setting", description: "Strongly associated with open water, underwater spaces, or aquatic environments." },
      { value: "freshwater_setting", description: "Strongly associated with rivers, lakes, ponds, streams, springs, or other inland waters." },
      { value: "coastal_setting", description: "Strongly associated with coasts, shores, reefs, or littoral edges." },
      { value: "astral_setting", description: "Strongly associated with Astral Plane scenes, silver-void travel, or stable portal routes." },
      { value: "first_world_setting", description: "Strongly associated with the First World, fey realms, or thin-boundary crossings into that plane." },
      { value: "boneyard_setting", description: "Strongly associated with the Boneyard, psychopomp duties, or soul-processing afterlife scenes." },
      { value: "island_setting", description: "Strongly associated with islands, archipelagos, or isolated isles." },
      { value: "nautical_setting", description: "Strongly associated with ships, sailors, wrecks, or harbors." },
      { value: "forest_setting", description: "Strongly associated with forests, jungles, groves, or briar-choked wilds." },
      { value: "plains_setting", description: "Strongly associated with open plains, grasslands, prairies, or savannas." },
      { value: "canyon_setting", description: "Strongly associated with canyons, gorges, mesas, or badlands." },
      { value: "swamp_setting", description: "Strongly associated with bogs, marshes, fens, or mires." },
      { value: "underground_setting", description: "Strongly associated with caves, tunnels, crypts, or subterranean spaces." },
      { value: "urban_setting", description: "Strongly associated with cities, streets, alleys, markets, or sewers." },
      { value: "arctic_setting", description: "Strongly associated with snow, ice, tundra, or frozen reaches." },
      { value: "desert_setting", description: "Strongly associated with dunes, sand, or arid wastes." },
      { value: "wasteland_setting", description: "Strongly associated with barren wastes, blasted wastelands, or desolate badlands." },
      { value: "mountain_setting", description: "Strongly associated with cliffs, peaks, passes, or rocky heights." },
      { value: "graveyard_setting", description: "Strongly associated with cemeteries, tombs, barrows, or burial grounds." },
      { value: "ruins_setting", description: "Strongly associated with ancient ruins or derelict structures." },
      { value: "temple_setting", description: "Strongly associated with temples, shrines, monasteries, or other sacred sites." },
      { value: "fortress_setting", description: "Strongly associated with castles, fortresses, citadels, or strongholds." },
      { value: "volcanic_setting", description: "Strongly associated with volcanoes, calderas, lava, or magma." },
    ],
  },
  {
    category: "creature",
    family: "scene_fit",
    description: "Creature practical-fit tags for distinguishing scene-adjacent NPCs from primary threats.",
    tags: [
      { value: "profession_npc", description: "Role-defined NPC such as a captain, guard, merchant, or commoner." },
      { value: "scene_adjacent", description: "Fits the scene or social fabric, but is usually not the primary monster answer." },
      { value: "undead_threat", description: "Threat signal derived from undead-like native traits." },
      { value: "fey_threat", description: "Threat signal derived from fey native traits." },
      { value: "plant_threat", description: "Threat signal derived from plant-like native traits." },
    ],
  },
];

export function deriveRecordTags(input: DerivedTagContext): string[] {
  return deriveRecordTagsFromRules(DERIVED_TAG_RULES, input);
}
