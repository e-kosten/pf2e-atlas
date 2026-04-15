import { SearchSubcategory } from "../types.js";
import {
  normalizeDerivedTagReference,
  TextAnchor,
  TextMatchScope,
} from "./matcher.js";

const patternAnchor = (value: string, scope: TextMatchScope = "either"): TextAnchor => ({ value, scope });

const OFFENSIVE_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("venom"),
  patternAnchor("bomb"),
  patternAnchor("injury poison"),
  patternAnchor("contact poison"),
  patternAnchor("ingested poison"),
  patternAnchor("inhaled poison"),
  patternAnchor("weapon poison"),
  patternAnchor("afflicts the target"),
];

const GEARISH_SUBCATEGORIES: SearchSubcategory[] = ["gear", "backpack", "kit", "vehicle"];
const DISGUISE_SUBCATEGORIES: SearchSubcategory[] = [...GEARISH_SUBCATEGORIES, "consumable"];
const TRACKING_SUBCATEGORIES: SearchSubcategory[] = [...GEARISH_SUBCATEGORIES, "consumable"];

const STRONG_PROFESSION_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("abbot", "name"),
  patternAnchor("apothecary", "name"),
  patternAnchor("barkeep", "name"),
  patternAnchor("battlepriest", "name"),
  patternAnchor("captain", "name"),
  patternAnchor("commoner", "name"),
  patternAnchor("courtier", "name"),
  patternAnchor("diplomat", "name"),
  patternAnchor("forgepriest", "name"),
  patternAnchor("guard", "name"),
  patternAnchor("jailer", "name"),
  patternAnchor("librarian", "name"),
  patternAnchor("scout", "name"),
  patternAnchor("scribe", "name"),
  patternAnchor("sailor", "name"),
  patternAnchor("merchant", "name"),
  patternAnchor("physician", "name"),
  patternAnchor("porter", "name"),
  patternAnchor("professor", "name"),
  patternAnchor("priest", "name"),
  patternAnchor("noble", "name"),
  patternAnchor("prophet", "name"),
  patternAnchor("advisor", "name"),
  patternAnchor("acolyte", "name"),
  patternAnchor("teacher", "name"),
  patternAnchor("technician", "name"),
  patternAnchor("translator", "name"),
  patternAnchor("vigilante", "name"),
  patternAnchor("watch officer", "name"),
  patternAnchor("watchmage", "name"),
  patternAnchor("bodyguard", "name"),
  patternAnchor("gaoler", "name"),
];

const WEAK_PROFESSION_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("acrobat", "name"),
  patternAnchor("agent", "name"),
  patternAnchor("apprentice", "name"),
  patternAnchor("artisan", "name"),
  patternAnchor("chemist", "name"),
  patternAnchor("drummer", "name"),
  patternAnchor("engineer", "name"),
  patternAnchor("hunter", "name"),
  patternAnchor("lutenist", "name"),
  patternAnchor("musician", "name"),
  patternAnchor("enforcer", "name"),
  patternAnchor("officer", "name"),
  patternAnchor("scholar", "name"),
  patternAnchor("singer", "name"),
  patternAnchor("steward", "name"),
  patternAnchor("warden", "name"),
  patternAnchor("caretaker", "name"),
  patternAnchor("logger", "name"),
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

const CIVIC_NPC_BLOCKER_TRAITS = [
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
  patternAnchor("river"),
  patternAnchor("rivers"),
  patternAnchor("lake"),
  patternAnchor("lakes"),
  patternAnchor("pond"),
  patternAnchor("ponds"),
  patternAnchor("stream"),
  patternAnchor("streams"),
  patternAnchor("spring"),
  patternAnchor("springs"),
];

const FRESHWATER_SETTING_STRONG_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("freshwater"),
  patternAnchor("fresh water"),
  patternAnchor("body of fresh water"),
  patternAnchor("bodies of fresh water"),
  patternAnchor("lake bed"),
  patternAnchor("lake beds"),
  patternAnchor("lakeside community"),
  patternAnchor("lakeside communities"),
];

const FRESHWATER_SETTING_CONTEXT_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("native"),
  patternAnchor("hide"),
  patternAnchor("hides"),
  patternAnchor("inhabit"),
  patternAnchor("inhabits"),
  patternAnchor("inhabiting"),
  patternAnchor("dwell"),
  patternAnchor("dwells"),
  patternAnchor("dwelling"),
  patternAnchor("centered"),
  patternAnchor("home"),
  patternAnchor("homes"),
  patternAnchor("found"),
  patternAnchor("lurk"),
  patternAnchor("lurks"),
  patternAnchor("lurking"),
  patternAnchor("protect"),
  patternAnchor("protects"),
  patternAnchor("ward"),
  patternAnchor("wards"),
];

const FRESHWATER_SETTING_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("pond", "name"),
  patternAnchor("spring", "name"),
];

const FRESHWATER_SETTING_BLOCKER_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("inner sea"),
  patternAnchor("river kingdom"),
  patternAnchor("river kingdoms"),
  patternAnchor("river of souls"),
];

const AQUATIC_SETTING_STRONG_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("aquatic"),
  patternAnchor("underwater"),
  patternAnchor("ocean"),
  patternAnchor("oceans"),
  patternAnchor("sea"),
  patternAnchor("seas"),
  patternAnchor("pelagic"),
  patternAnchor("fjord"),
  patternAnchor("fjords"),
  patternAnchor("waterways"),
  patternAnchor("water dwelling"),
  patternAnchor("breathe water"),
];

const AQUATIC_SETTING_WEAK_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("water"),
  patternAnchor("wave"),
  patternAnchor("waves"),
  patternAnchor("surf"),
  patternAnchor("tide"),
  patternAnchor("tides"),
  patternAnchor("tidal"),
  patternAnchor("swim"),
  patternAnchor("swims"),
  patternAnchor("swimming"),
  patternAnchor("flooded"),
];

const COASTAL_SETTING_STRONG_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("coast"),
  patternAnchor("coasts"),
  patternAnchor("coastal"),
  patternAnchor("marine"),
  patternAnchor("reef"),
  patternAnchor("reefs"),
  patternAnchor("littoral"),
  patternAnchor("estuary"),
  patternAnchor("estuaries"),
  patternAnchor("estuarine"),
  patternAnchor("coastal region"),
  patternAnchor("coastal regions"),
  patternAnchor("coral reef"),
  patternAnchor("coral reefs"),
  patternAnchor("marine environment"),
  patternAnchor("marine environments"),
  patternAnchor("ocean facing cliffs"),
  patternAnchor("along any shore"),
];

const COASTAL_SETTING_WEAK_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("shore"),
  patternAnchor("shores"),
  patternAnchor("shoreline"),
  patternAnchor("coastline"),
  patternAnchor("coastlines"),
  patternAnchor("beach"),
  patternAnchor("beaches"),
  patternAnchor("harbor"),
  patternAnchor("harbors"),
  patternAnchor("brackish"),
  patternAnchor("coral"),
  patternAnchor("ocean"),
  patternAnchor("oceans"),
  patternAnchor("sea"),
  patternAnchor("seas"),
  patternAnchor("tidal"),
  patternAnchor("tidepool"),
  patternAnchor("tidepools"),
];

const COASTAL_SETTING_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("coastal", "name"),
  patternAnchor("coral", "name"),
  patternAnchor("reef", "name"),
  patternAnchor("sea", "name"),
  patternAnchor("tidepool", "name"),
  patternAnchor("tidewater", "name"),
];

const ASTRAL_SETTING_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("astral plane"),
  patternAnchor("silver sea"),
  patternAnchor("silver void"),
  patternAnchor("silvery gray void"),
  patternAnchor("stable portal"),
  patternAnchor("stable portals"),
];

const ASTRAL_SETTING_ACTIVITY_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("ambush"),
  patternAnchor("ambushes"),
  patternAnchor("guardian"),
  patternAnchor("guardians"),
  patternAnchor("guard"),
  patternAnchor("guards"),
  patternAnchor("home"),
  patternAnchor("hunt"),
  patternAnchor("hunts"),
  patternAnchor("hunting"),
  patternAnchor("patrol"),
  patternAnchor("patrols"),
  patternAnchor("protect"),
  patternAnchor("protects"),
  patternAnchor("travel"),
  patternAnchor("travels"),
  patternAnchor("traveling"),
];

const ASTRAL_SETTING_CONTEXT_TEXT_ANCHORS: TextAnchor[] = [
  ...ASTRAL_SETTING_ACTIVITY_TEXT_ANCHORS,
  patternAnchor("demiplane"),
  patternAnchor("demiplanes"),
  patternAnchor("portal"),
  patternAnchor("portals"),
  patternAnchor("stable portal"),
  patternAnchor("stable portals"),
];

const FIRST_WORLD_SETTING_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("first world"),
  patternAnchor("realm of the fey"),
  patternAnchor("fey realm"),
];

const DREAMLANDS_SETTING_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("dreamlands"),
  patternAnchor("in the dreamlands"),
  patternAnchor("from the dreamlands"),
  patternAnchor("of the dreamlands"),
];

const DREAMLANDS_SETTING_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("dreamlands", "name"),
  patternAnchor("nightgaunt", "name"),
  patternAnchor("shantak", "name"),
  patternAnchor("dramofir", "name"),
  patternAnchor("somnalu", "name"),
];

const FIRST_WORLD_SETTING_CONTEXT_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("boundary"),
  patternAnchor("boundaries"),
  patternAnchor("court"),
  patternAnchor("courts"),
  patternAnchor("eldest"),
  patternAnchor("home"),
  patternAnchor("homes"),
  patternAnchor("native"),
  patternAnchor("natives"),
  patternAnchor("origin"),
  patternAnchor("origins"),
  patternAnchor("portal"),
  patternAnchor("portals"),
  patternAnchor("spawn"),
  patternAnchor("spawned"),
  patternAnchor("worn thin"),
];

const BONEYARD_SETTING_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("boneyard"),
];

const BONEYARD_SETTING_CONTEXT_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("guard"),
  patternAnchor("guards"),
  patternAnchor("passing"),
  patternAnchor("psychopomp"),
  patternAnchor("repair"),
  patternAnchor("repairs"),
  patternAnchor("sentinel"),
  patternAnchor("sentinels"),
  patternAnchor("guard dogs"),
  patternAnchor("repair damaged souls"),
  patternAnchor("ease a spirit s passing"),
];

const HEAVEN_SETTING_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("connection to heaven"),
  patternAnchor("direct connection to heaven"),
  patternAnchor("defend heaven"),
  patternAnchor("defends heaven"),
  patternAnchor("heaven s armies"),
  patternAnchor("of heaven"),
];

const NIRVANA_SETTING_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("in nirvana"),
  patternAnchor("nirvana s blessing"),
  patternAnchor("nirvana s attendants"),
  patternAnchor("halls of nirvana"),
  patternAnchor("serve as nirvana s attendants"),
];

const ELYSIUM_SETTING_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("of elysium"),
  patternAnchor("in elysium"),
  patternAnchor("lords of elysium"),
  patternAnchor("champions of the freedom"),
];

const HELL_SETTING_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("hell s armies"),
  patternAnchor("from hell"),
  patternAnchor("of hell"),
  patternAnchor("in hell"),
  patternAnchor("first layer of hell"),
];

const ABYSS_SETTING_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("the abyss"),
  patternAnchor("of the abyss"),
  patternAnchor("from the abyss"),
  patternAnchor("in the abyss"),
  patternAnchor("abyssal realm"),
];

const SHADOW_PLANE_SETTING_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("shadow plane"),
  patternAnchor("plane of shadow"),
];

const MAELSTROM_SETTING_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("the maelstrom"),
  patternAnchor("of the maelstrom"),
];

const referenceAnchor = (packName: string, name: string): string => normalizeDerivedTagReference(`${packName}:${name}`);

const DISGUISE_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("disguise"),
  patternAnchor("impersonate"),
  patternAnchor("false identity"),
  patternAnchor("costume"),
  patternAnchor("masquerade"),
  patternAnchor("quick change", "name"),
  patternAnchor("take on the appearance"),
  patternAnchor("change your appearance"),
  patternAnchor("assume role"),
];

const SOCIAL_INFILTRATION_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("false identity"),
  patternAnchor("pass as"),
  patternAnchor("blend into society"),
  patternAnchor("social infiltration"),
  patternAnchor("impersonate"),
  patternAnchor("masquerade"),
  patternAnchor("quick change", "name"),
  patternAnchor("disguise", "name"),
  patternAnchor("take on the appearance"),
  patternAnchor("change your appearance"),
  patternAnchor("assume role"),
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
  patternAnchor("disguise", "name"),
  patternAnchor("visage", "name"),
];

const SPELL_DISGUISE_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("appear as another creature"),
  patternAnchor("appears as another creature"),
  patternAnchor("appearance becomes bland and nondescript"),
  patternAnchor("change your appearance"),
  patternAnchor("transform your appearance"),
  patternAnchor("alter a minor detail of your appearance"),
  patternAnchor("trade appearances"),
  patternAnchor("look and sound like the target"),
  patternAnchor("disguises you"),
  patternAnchor("disguises the target"),
  patternAnchor("mask the target s features"),
  patternAnchor("pass the target off as"),
];

const SPELL_DISGUISE_BLOCKER_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("see things as they are"),
  patternAnchor("see through it"),
  patternAnchor("see the creature s true form"),
  patternAnchor("counteract check against each illusion"),
  patternAnchor("unveiled by attempts to magically cloak the truth"),
];

const SPELL_SOCIAL_INFILTRATION_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("go incognito"),
  patternAnchor("appearance becomes bland and nondescript"),
  patternAnchor("suitable for a particular occasion"),
  patternAnchor("pass as someone else"),
  patternAnchor("blend into"),
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
  patternAnchor("handcuffs", "name"),
  patternAnchor("manacles", "name"),
];

const WEAK_RESTRAINT_CAPTURE_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("lasso", "name"),
  patternAnchor("net", "name"),
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
  patternAnchor("alarm"),
  patternAnchor("alert"),
  patternAnchor("alerts"),
  patternAnchor("raise the alarm"),
  patternAnchor("alert nearby guards"),
  patternAnchor("sound an alarm"),
  patternAnchor("warning bell"),
  patternAnchor("intrusion"),
];

const HAZARD_FIRE_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("fire", "name"),
  patternAnchor("fires", "name"),
  patternAnchor("flame", "name"),
  patternAnchor("burning", "name"),
  patternAnchor("inferno", "name"),
  patternAnchor("aflame", "name"),
];

const HAZARD_FIRE_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("catch fire and explode"),
  patternAnchor("catches fire and explode"),
  patternAnchor("rains fire from the sky"),
  patternAnchor("sheets of fire"),
  patternAnchor("beams of fire"),
  patternAnchor("burns and threatens to spread"),
  patternAnchor("a fire engulfs"),
  patternAnchor("spreads on each of its turns"),
];

const HAZARD_POISON_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("poison", "name"),
  patternAnchor("venom", "name"),
  patternAnchor("envenomed", "name"),
  patternAnchor("toxic", "name"),
];

const HAZARD_POISON_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("poison gas"),
  patternAnchor("pumping poison gas"),
  patternAnchor("releases a poison gas"),
  patternAnchor("poisonous smoke"),
  patternAnchor("pressurized poison"),
  patternAnchor("poisoned spine"),
  patternAnchor("needle delivers a magical poison"),
  patternAnchor("toxic darts"),
  patternAnchor("acidic poison"),
];

const HAZARD_PITFALL_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("pit", "name"),
  patternAnchor("pitfall", "name"),
  patternAnchor("sinkhole", "name"),
];

const HAZARD_COLLAPSE_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("collapse", "name"),
  patternAnchor("collapsing", "name"),
  patternAnchor("deadfall", "name"),
  patternAnchor("rockfall", "name"),
  patternAnchor("cave in", "name"),
];

const HAZARD_FORCED_MOVEMENT_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("sucks creatures in the area toward"),
  patternAnchor("raging wind sucks creatures"),
  patternAnchor("attempts to submerge creatures"),
  patternAnchor("trample each other"),
  patternAnchor("pulls creatures toward"),
  patternAnchor("pushes creatures"),
  patternAnchor("sweeps creatures away"),
  patternAnchor("push it back"),
  patternAnchor("hurling them into"),
  patternAnchor("dragging them underwater"),
  patternAnchor("drag them underground"),
  patternAnchor("pull them into the machinery"),
  patternAnchor("pull someone down into the water"),
  patternAnchor("knocking people into the pit"),
  patternAnchor("slide around the room"),
];

const ALARM_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("alarm", "name"),
  patternAnchor("warning", "name"),
  patternAnchor("sentry", "name"),
];

const ALARM_STRONG_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("audible alarm"),
  patternAnchor("mental alarm"),
  patternAnchor("mental alert"),
  patternAnchor("alarm system"),
  patternAnchor("raise the alarm"),
  patternAnchor("raising the alarm"),
  patternAnchor("alerting nearby guards"),
  patternAnchor("keeps watch over an area"),
  patternAnchor("watch over an area"),
  patternAnchor("without speaking the password"),
  patternAnchor("without giving the password"),
  patternAnchor("the snare makes a noise"),
  patternAnchor("emits an ear piercing wail"),
];

const ALARM_TRIGGER_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("enters the spell s area"),
  patternAnchor("enters the area"),
  patternAnchor("enters the square"),
  patternAnchor("detect creatures moving in its area"),
  patternAnchor("trip wire"),
  patternAnchor("pressure plate"),
  patternAnchor("password"),
];

const SIGNALING_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("signal", "name"),
  patternAnchor("signaling", "name"),
  patternAnchor("beacon", "name"),
  patternAnchor("whistle", "name"),
];

const SIGNALING_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("request rescue"),
  patternAnchor("coordinate assaults"),
  patternAnchor("signal directions"),
  patternAnchor("heard clearly"),
  patternAnchor("heard up to"),
  patternAnchor("seen from miles away"),
  patternAnchor("target becomes more visible"),
  patternAnchor("spews sparks"),
  patternAnchor("emblazon a message across the sky"),
];

const MESSAGE_DELIVERY_EQUIPMENT_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("message", "name"),
  patternAnchor("mail", "name"),
  patternAnchor("mailbox", "name"),
  patternAnchor("communication", "name"),
];

const MESSAGE_DELIVERY_SPELL_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("message", "name"),
  patternAnchor("mailbox", "name"),
  patternAnchor("telepathy", "name"),
  patternAnchor("telepathic bond", "name"),
  patternAnchor("dream council", "name"),
  patternAnchor("mindlink", "name"),
];

const MESSAGE_DELIVERY_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("attach a small object or note"),
  patternAnchor("carry a message"),
  patternAnchor("delivers its whisper quiet message"),
  patternAnchor("send a message"),
  patternAnchor("send the creature a mental message"),
  patternAnchor("record a message"),
  patternAnchor("respond immediately with its own message"),
  patternAnchor("write a message"),
  patternAnchor("message bearer"),
  patternAnchor("messages can be coded"),
  patternAnchor("brief response"),
  patternAnchor("communicate telepathically"),
  patternAnchor("mental message"),
  patternAnchor("message is one way"),
  patternAnchor("shared dream"),
  patternAnchor("communicate with one another"),
  patternAnchor("message up to"),
  patternAnchor("whisper a secret message"),
];

const MESSAGE_DELIVERY_BLOCKER_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("suggestion spell"),
  patternAnchor("harder for them to communicate"),
  patternAnchor("can communicate only by singing"),
  patternAnchor("no special method of communication"),
];

const COUNTERMAGIC_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("antimagic", "name"),
  patternAnchor("countering", "name"),
  patternAnchor("nullification", "name"),
  patternAnchor("dispelling", "name"),
  patternAnchor("dispel", "name"),
  patternAnchor("counterspell", "name"),
];

const COUNTERMAGIC_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("spell s effect doesn t occur"),
  patternAnchor("all magic is suppressed"),
  patternAnchor("antimagic field"),
  patternAnchor("counteract magical effects"),
  patternAnchor("counteract the triggering spell"),
  patternAnchor("counteract a single spell"),
  patternAnchor("counteract the target spell"),
];

const COUNTERMAGIC_REFERENCE_ANCHORS = [
  referenceAnchor("spells-srd", "Dispel Magic"),
  referenceAnchor("spells-srd", "Antimagic Field"),
  referenceAnchor("spells-srd", "Dispelling Globe"),
];

const MAGIC_PROTECTION_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("saving throws against magical effects"),
  patternAnchor("bonus to saving throws against magical effects"),
  patternAnchor("becomes immune to all spells"),
  patternAnchor("immune to all spells"),
  patternAnchor("effects of magic items"),
  patternAnchor("effects with the magical trait"),
  patternAnchor("spell targets you"),
  patternAnchor("spells that target you"),
];

const SPELL_SCOUTING_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("clairaudience", "name"),
  patternAnchor("clairvoyance", "name"),
  patternAnchor("familiar s face", "name"),
  patternAnchor("know location", "name"),
  patternAnchor("painted scout", "name"),
  patternAnchor("prying survey", "name"),
  patternAnchor("proliferating eyes", "name"),
  patternAnchor("rune of observation", "name"),
  patternAnchor("scouting eye", "name"),
  patternAnchor("scrying", "name"),
  patternAnchor("scrying ripples", "name"),
  patternAnchor("unrelenting observation", "name"),
  patternAnchor("web of eyes", "name"),
];

const SPELL_SCOUTING_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("can see hear and smell through"),
  patternAnchor("create a sensor"),
  patternAnchor("floating ear"),
  patternAnchor("floating eye"),
  patternAnchor("hear through the ear"),
  patternAnchor("know the location"),
  patternAnchor("learn its approximate distance and direction"),
  patternAnchor("looks where that target looks"),
  patternAnchor("magical eye sensor"),
  patternAnchor("scrying sensor"),
  patternAnchor("see hear and smell through"),
  patternAnchor("see in all directions from that point"),
  patternAnchor("see through its eyes"),
  patternAnchor("transmits what it sees"),
  patternAnchor("transmitting rough impressions"),
  patternAnchor("you magically spy on"),
];

const SPELL_SCOUTING_BLOCKER_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("preventing it from being used for magical observation"),
  patternAnchor("fools any attempts to scry"),
  patternAnchor("hide a creature from magic that would spy on it"),
  patternAnchor("make the target difficult to detect via magic"),
  patternAnchor("counteract all detection revelation and scrying effects"),
];

const SPELL_NAVIGATION_NAME_ANCHORS: TextAnchor[] = [
  patternAnchor("guiding star", "name"),
  patternAnchor("know the way", "name"),
  patternAnchor("wanderer s guide", "name"),
];

const SPELL_NAVIGATION_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("guide your route"),
  patternAnchor("guided to the location"),
  patternAnchor("inspired route"),
  patternAnchor("know which direction is north"),
  patternAnchor("learn what direction it lies"),
  patternAnchor("mental nudge toward your chosen location"),
  patternAnchor("know its relative direction from you"),
  patternAnchor("find the correct gate to your desired location"),
];

const SPELL_MOBILITY_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("gain a fly speed"),
  patternAnchor("giving you a fly speed"),
  patternAnchor("gain a climb speed"),
  patternAnchor("gain a swim speed"),
  patternAnchor("gain a burrow speed"),
  patternAnchor("climb speed equal to your speed"),
  patternAnchor("climb speed equal to"),
  patternAnchor("swim speed equal to your speed"),
  patternAnchor("swim speed equal to"),
  patternAnchor("fly speed equal to your speed"),
  patternAnchor("fly speed equal to"),
  patternAnchor("ignore difficult terrain"),
  patternAnchor("jump {{number}} feet"),
  patternAnchor("you jump {{number}} feet"),
  patternAnchor("gain a +{{number}}-foot status bonus to your speed"),
  patternAnchor("you gain a +{{number}}-foot status bonus to your speed"),
  patternAnchor("teleport you to an unoccupied space"),
  patternAnchor("teleport to an unoccupied space"),
  patternAnchor("teleport to a space"),
  patternAnchor("teleport up to"),
  patternAnchor("targets are instantly transported"),
  patternAnchor("transport yourself"),
  patternAnchor("swap places"),
  patternAnchor("instantly teleport"),
  patternAnchor("walk on the surface of water"),
  patternAnchor("move across liquid surfaces"),
  patternAnchor("cover roughly as much ground as"),
  patternAnchor("transported with you"),
  patternAnchor("reduce the movement penalty from difficult terrain"),
  patternAnchor("hinder a creature or slow its movement"),
  patternAnchor("circumstance penalty to speed"),
];

const SPELL_MOBILITY_BLOCKER_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("battle form"),
  patternAnchor("target object"),
  patternAnchor("one object"),
  patternAnchor("swap an item"),
  patternAnchor("teleport the snare"),
  patternAnchor("teleportation circle"),
  patternAnchor("prison"),
  patternAnchor("maze"),
  patternAnchor("trap them there"),
  patternAnchor("trap the target"),
];

const AFFLICTION_MENTAL_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("confused"),
  patternAnchor("confusion"),
  patternAnchor("delirium"),
  patternAnchor("paranoia"),
  patternAnchor("frightened"),
  patternAnchor("hallucination"),
  patternAnchor("hallucinations"),
  patternAnchor("whispers"),
  patternAnchor("panic"),
];

const AFFLICTION_MOBILITY_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("speed is reduced"),
  patternAnchor("reduces the victim s speed"),
  patternAnchor("reduces your speed"),
  patternAnchor("stiffens joints"),
  patternAnchor("interferes with a victim s movement"),
  patternAnchor("takes one or more steps in a random direction"),
  patternAnchor("movement is forced"),
  patternAnchor("immobilized"),
  patternAnchor("paralyzed"),
  patternAnchor("paralysis"),
  patternAnchor("slowed"),
];

const AFFLICTION_PHYSICAL_DEBILITATION_TEXT_ANCHORS: TextAnchor[] = [
  patternAnchor("drained"),
  patternAnchor("enfeebled"),
  patternAnchor("clumsy"),
  patternAnchor("sickened"),
  patternAnchor("fatigued"),
  patternAnchor("drain health and strength"),
  patternAnchor("drains health and strength"),
];

export type { DerivedTagContext, DerivedTagRule, TextAnchor, TextMatchScope, TextNearConstraint } from "./matcher.js";
export { normalizeDerivedTag } from "./matcher.js";

export {
  patternAnchor,
  referenceAnchor,
  OFFENSIVE_TEXT_ANCHORS,
  GEARISH_SUBCATEGORIES,
  DISGUISE_SUBCATEGORIES,
  TRACKING_SUBCATEGORIES,
  STRONG_PROFESSION_NAME_ANCHORS,
  WEAK_PROFESSION_NAME_ANCHORS,
  UNDEAD_GLOSSARY_FAMILIES,
  CIVIC_NPC_BLOCKER_TRAITS,
  FRESHWATER_SETTING_TEXT_ANCHORS,
  FRESHWATER_SETTING_STRONG_TEXT_ANCHORS,
  FRESHWATER_SETTING_CONTEXT_TEXT_ANCHORS,
  FRESHWATER_SETTING_NAME_ANCHORS,
  FRESHWATER_SETTING_BLOCKER_TEXT_ANCHORS,
  AQUATIC_SETTING_STRONG_TEXT_ANCHORS,
  AQUATIC_SETTING_WEAK_TEXT_ANCHORS,
  COASTAL_SETTING_STRONG_TEXT_ANCHORS,
  COASTAL_SETTING_WEAK_TEXT_ANCHORS,
  COASTAL_SETTING_NAME_ANCHORS,
  ASTRAL_SETTING_TEXT_ANCHORS,
  ASTRAL_SETTING_ACTIVITY_TEXT_ANCHORS,
  ASTRAL_SETTING_CONTEXT_TEXT_ANCHORS,
  FIRST_WORLD_SETTING_TEXT_ANCHORS,
  DREAMLANDS_SETTING_TEXT_ANCHORS,
  DREAMLANDS_SETTING_NAME_ANCHORS,
  FIRST_WORLD_SETTING_CONTEXT_TEXT_ANCHORS,
  BONEYARD_SETTING_TEXT_ANCHORS,
  BONEYARD_SETTING_CONTEXT_TEXT_ANCHORS,
  HEAVEN_SETTING_TEXT_ANCHORS,
  NIRVANA_SETTING_TEXT_ANCHORS,
  ELYSIUM_SETTING_TEXT_ANCHORS,
  HELL_SETTING_TEXT_ANCHORS,
  ABYSS_SETTING_TEXT_ANCHORS,
  SHADOW_PLANE_SETTING_TEXT_ANCHORS,
  MAELSTROM_SETTING_TEXT_ANCHORS,
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
