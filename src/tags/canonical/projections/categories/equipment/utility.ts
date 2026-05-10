import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const equipmentUtilityProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.UTILITY_ACCESS_BYPASS, {
    barrier_bypass: {
      description:
        "Helps get through barred windows, grates, force screens, or other blocked passage without relying on brute-force breaching.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    lock_bypass: {
      description: "Helps open locks or bypass secured entry points.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    mechanism_manipulation: {
      description:
        "Helps operate levers, latches, panels, pressure surfaces, or similar scene mechanisms from a safer or more advantageous position.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    trap_bypass: {
      description: "Helps disarm, disable, or get past traps.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.UTILITY_ANTI_MAGIC, {
    countermagic: {
      concept: "active_magic_counteraction",
      label: "countermagic",
      description: "Counteracts, dispels, suppresses, or shuts down magic.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      appliesWhen: [
        "The item's main value is actively cancelling, suppressing, or interfering with hostile or ongoing magic.",
        "A user would retrieve it as an anti-magic tool rather than a general protective charm.",
      ],
      doesNotApplyWhen: [
        "The item only protects the wearer from magical harm without disrupting the spell itself.",
        "The item focuses on blocking surveillance or hiding information rather than broader anti-magic interference.",
      ],
      adjacentTags: ["magic_protection", "scrying_protection"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    magic_protection: {
      description: "Protects the user or target against hostile magical effects.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      appliesWhen: [
        "The item's value comes from warding the bearer against curses, spells, hostile magical conditions, or magical damage.",
        "Protection matters more than actually counteracting or suppressing the incoming magic.",
      ],
      doesNotApplyWhen: [
        "The item mainly shuts down active magic rather than defending a wearer or target.",
        "The stronger fit is scrying_protection because surveillance denial is the specific retrieval hook.",
      ],
      adjacentTags: ["countermagic", "hazard_shielding"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.UTILITY_BREACHING, {
    barrier_breaking: {
      description: "Designed to tear through walls, barricades, ice, webs, or other physical obstructions.",
      subcategories: ["gear", "kit", "consumable", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    breaching: {
isComposite: true,
      description:
        "Broad force-entry umbrella for equipment used to break doors, barriers, fortifications, or route-blocking structures.",
      subcategories: ["gear", "kit", "consumable", "weapon"],
      adjacentTags: ["door_breaching", "barrier_breaking", "demolition"],
      compositeOfAnyTags: ["door_breaching", "barrier_breaking", "excavation", "siege_support", "demolition"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    demolition: {
      description: "Designed for blasting, collapsing, or otherwise violently dismantling structures and obstacles.",
      subcategories: ["gear", "kit", "consumable", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    door_breaching: {
      description:
        "Helps force doors, shutters, gates, or similar entry points open by strength, impact, or destructive entry.",
      subcategories: ["gear", "kit", "consumable", "weapon"],
      appliesWhen: [
        "The item's retrieval value is getting through doors, shutters, gates, or secured entry points.",
        "It solves access by force rather than by keys, stealth, or lock tools.",
      ],
      doesNotApplyWhen: [
        "The item is for larger demolition or siegework rather than point-of-entry breach.",
        "The item bypasses access quietly through locks or trickery instead of force.",
      ],
      adjacentTags: ["lock_bypass", "barrier_breaking"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    excavation: {
      description:
        "Helps dig, cut through earth or stone, or otherwise open a route by excavation or practical earth-moving work.",
      subcategories: ["gear", "kit", "consumable", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    siege_support: {
      description: "Supports attacking gates, fortifications, vehicles, or other larger hardened targets.",
      subcategories: ["gear", "kit", "consumable", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.UTILITY_CARRY_LOGISTICS, {
    carry_support: {
      description: "Helps stow, carry, or organize equipment.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.UTILITY_COMMUNICATION, {
    communication: {
isComposite: true,
      description:
        "Broad communication umbrella for equipment used to signal allies, relay messages, bridge language barriers, or coordinate silently.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      adjacentTags: ["signaling", "telepathic_communication", "message_delivery"],
      compositeOfAnyTags: ["signaling", "telepathic_communication", "message_delivery", "translation_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    message_delivery: {
      description: "Sends, stores, or relays actual content across time or distance.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    signaling: {
      description: "Helps draw attention, mark a location, or coordinate allies.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    telepathic_communication: {
      description:
        "Enables silent mind-to-mind coordination, psychic speech, or communication that bypasses normal hearing.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      appliesWhen: [
        "The item's retrieval value comes from silent psychic coordination, mind-to-mind speech, or communication that bypasses ordinary sound.",
        "It is naturally sought when stealth, silence, distance, or noise would make spoken coordination unreliable.",
      ],
      doesNotApplyWhen: [
        "The item only boosts ordinary signaling, writing, or message relay without true mind-to-mind communication.",
        "The item is mainly about surveillance or recording rather than live coordination.",
      ],
      adjacentTags: ["signaling", "message_delivery"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    translation_support: {
      description:
        "Bridges language barriers through translation, deciphering, script interpretation, or speech-understanding aids.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      appliesWhen: [
        "The item's retrieval value comes from understanding foreign languages, translating speech, or decoding otherwise unreadable text or symbols.",
        "It is naturally sought when communication fails because of language barriers rather than distance or secrecy.",
      ],
      doesNotApplyWhen: [
        "The item only stores, relays, or broadcasts messages without solving comprehension.",
        "The item only provides psychic communication between already understood participants.",
      ],
      adjacentTags: ["telepathic_communication", "message_delivery"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.UTILITY_CRAFTING_SUPPORT, {
    alchemical_crafting: {
      description: "Supports alchemical preparation, formula work, reagent handling, or crafting-related field setup.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    forgery_support: {
      description: "Supports document falsification, seal imitation, signature copying, or bureaucratic deception.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      appliesWhen: [
        "The item's retrieval value comes from imitating documents, seals, credentials, or official paperwork.",
        "It supports passing administrative scrutiny rather than just changing clothing or physical appearance.",
      ],
      doesNotApplyWhen: [
        "The item only changes appearance or supports social disguise without document work.",
        "The item is a normal writing or archival tool with no deception-facing use.",
      ],
      adjacentTags: ["disguise", "writing_recordkeeping"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    medical_support: {
      description: "Supports first aid, diagnosis, treatment, or ongoing medical care outside direct magical healing.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    repair_support: {
      description: "Supports item repair, patchwork, upkeep, or restoring damaged gear and structures.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    ritual_support: {
      description:
        "Supports ritual casting, ceremonial setup, circles, offerings, or other extended magical preparation.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      appliesWhen: [
        "The item meaningfully supports ceremonial, circle-based, offering-based, or extended-casting magic work.",
        "A user would retrieve it for magic preparation rather than ordinary crafting or adventuring gear.",
      ],
      doesNotApplyWhen: [
        "The item is only generally magical without helping ritual process or setup.",
        "The item is mainly a focus of worship or symbolism rather than ritual procedure.",
      ],
      adjacentTags: ["alchemical_crafting", "magic_protection"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    writing_recordkeeping: {
      description: "Supports note-taking, mapmaking, copying text, archival work, or durable information storage.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.UTILITY_EXPEDITION, {
    aquatic_support: {
      description: "Helps with swimming, underwater breathing, flotation, water-surface travel, or watercraft use.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "shield", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    camp_setup: {
      description: "Supports campsite creation, resting infrastructure, shelter setup, or extended overland staging.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "shield", "weapon"],
      appliesWhen: [
        "The item is naturally retrieved for making camp, resting outdoors, or supporting expedition downtime.",
        "Its value is in field habitation rather than only carrying gear or feeding travelers.",
      ],
      doesNotApplyWhen: [
        "The item only provides sustenance, mobility, or transport without campsite infrastructure.",
        "The item is merely general survival gear with no setup or shelter-facing role.",
      ],
      adjacentTags: ["sustenance", "carry_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    environmental_adaptation: {
      description:
        "Helps travelers endure extreme weather, thin air, smoke, pressure, or other dangerous environmental exposure.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "shield", "weapon"],
      appliesWhen: [
        "The item's retrieval value comes from surviving punishing climate, altitude, breathing hazards, immersion pressure, or similar expedition environments.",
        "It is naturally sought as environmental survival gear rather than a general defense item or campsite tool.",
      ],
      doesNotApplyWhen: [
        "The item only protects against one incoming attack or hazard burst without broader travel-survival use.",
        "The item mainly creates camp infrastructure, carries provisions, or improves aquatic movement instead of adapting the user to the environment.",
      ],
      adjacentTags: ["aquatic_support", "camp_setup", "hazard_shielding"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    expedition: {
isComposite: true,
      description:
        "Broad expedition umbrella for travel gear, camp support, sustainment, mounts, aquatic operations, and hostile-environment endurance.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "shield", "weapon"],
      adjacentTags: ["survival", "sustenance", "environmental_adaptation"],
      compositeOfAnyTags: [
        "survival",
        "mounted_support",
        "sustenance",
        "aquatic_support",
        "environmental_adaptation",
        "camp_setup",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    mounted_support: {
      description: "Supports mounted combat, rider control, saddle use, or mount-specific loadouts.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "shield", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    survival: {
      description: "Supports wilderness travel, shelter, or long-term field use.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "shield", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    sustenance: {
      description: "Provides food, feed, water, or other practical nourishment for travel and survival.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "shield", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.UTILITY_INFILTRATION, {
    concealable: {
      description: "Easy to hide on the person or carry discreetly.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    concealment: {
      description: "Helps obscure a creature, item, or area from sight or make it harder to perceive.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    disguise: {
      description: "Helps alter appearance or impersonate another identity.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    infiltration: {
isComposite: true,
      description:
        "Broad infiltration umbrella for quiet-entry, discreet-carry, disguise, and covert-passing equipment.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      adjacentTags: ["stealth_support", "disguise", "social_infiltration"],
      compositeOfAnyTags: ["stealth_support", "concealable", "disguise", "social_infiltration", "concealment"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    social_infiltration: {
      description: "Helps blend into a group or pass under social scrutiny.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    stealth_support: {
      description:
        "Helps move quietly, avoid notice, muffle noise, or otherwise support covert entry and low-profile movement.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.UTILITY_MOVEMENT_TRAVERSAL, {
    climbing: {
      description: "Helps climb, rappel, or navigate vertical obstacles.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    mobility: {
      description: "Improves movement or traversal flexibility.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    movement_traversal: {
isComposite: true,
      description:
        "Broad movement-and-travel umbrella for equipment that solves climbing, routefinding, repositioning, or transport problems.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      adjacentTags: ["climbing", "navigation", "transport"],
      compositeOfAnyTags: ["climbing", "mobility", "navigation", "transport"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    navigation: {
      description: "Helps track direction, route, or position.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    transport: {
      description: "Helps move creatures or cargo from place to place.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.UTILITY_RECONNAISSANCE, {
    anti_tracking: {
      description: "Helps hide your trail, mask scent, or make pursuit harder.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    illumination: {
      description: "Produces or improves light in dark environments.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    reconnaissance: {
isComposite: true,
      description:
        "Broad recon umbrella for equipment used to scout, illuminate, record evidence, track targets, or frustrate pursuit.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      adjacentTags: ["scouting", "tracking", "anti_tracking"],
      compositeOfAnyTags: ["scouting", "illumination", "surveillance_recording", "tracking", "anti_tracking"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    scouting: {
      description: "Helps observe, survey, or reconnoiter an area.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    surveillance_recording: {
      description: "Captures, stores, or replays images, sound, or other evidence for later review.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      appliesWhen: [
        "The item's retrieval value comes from preserving sights, sounds, or observations for later replay, proof, or analysis.",
        "It is naturally sought as evidence capture, remote monitoring, or watch-post support rather than live conversation gear.",
      ],
      doesNotApplyWhen: [
        "The item only sends a live message or helps coordinate allies without retaining evidence.",
        "The item protects against observation instead of performing it.",
      ],
      adjacentTags: ["scouting", "tamper_evidence"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    tracking: {
      description: "Helps follow trails, mark a target, or relocate something later.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.UTILITY_RESOLUTION, {
    contamination_cleanup: {
      description:
        "Helps neutralize tainted residue, clean corrupted surfaces, purify contaminated supplies, or scrub a dangerous site back to safety.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      adjacentTags: ["quarantine_containment", "source_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    curse_removal: {
      concept: "curse_remediation",
      description:
        "Helps remove, break, or counteract curses as a direct answer path rather than only easing symptoms.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      adjacentTags: ["sanctification", "source_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    quarantine_containment: {
      concept: "outbreak_containment",
      description:
        "Helps isolate victims, secure contaminated areas, or impose practical containment procedures that stop spread while treatment proceeds.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      adjacentTags: ["contamination_cleanup", "source_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    resolution: {
      concept: "problem_resolution",
isComposite: true,
      description:
        "Broad resolution umbrella for equipment used to break curses, sanctify places, contain spread, clean contamination, or solve a problem at its source.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      adjacentTags: [
        "curse_removal",
        "ritual_appeasement",
        "source_revelation",
        "contamination_cleanup",
        "source_cleanup",
      ],
      compositeOfAnyTags: [
        "curse_removal",
        "sanctification",
        "ritual_appeasement",
        "source_revelation",
        "quarantine_containment",
        "contamination_cleanup",
        "source_cleanup",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    ritual_appeasement: {
      description:
        "Supports offerings, restitution, funerary observance, or appeasement ceremonies used to satisfy a spirit, haunt, curse, or sacred demand without directly expelling it.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      appliesWhen: [
        "The item's retrieval value comes from helping perform offerings, appeasement rites, restitution rituals, or ceremonial observance meant to settle a supernatural grievance.",
        "It is naturally sought for placation or ritual satisfaction rather than direct cleansing, banishment, or ordinary worship.",
      ],
      doesNotApplyWhen: [
        "The item only supports broad ritual process with no real appeasement, offering, or restitution-facing role.",
        "The stronger fit is sanctification or ritual_support because the item purifies generally or supports any rite rather than a placation answer path.",
      ],
      adjacentTags: ["ritual_support", "sanctification"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    sanctification: {
      concept: "sacred_taint_sanctification",
      label: "sanctification",
      description:
        "Supports hallowing, consecration, spiritual purification, or cleansing rites applied to a creature, object, or site.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      appliesWhen: [
        "The item's retrieval value comes from consecrating, hallowing, purifying, or spiritually cleansing a target or place.",
        "It is naturally sought as part of sacred-site cleanup, anti-haunt work, or ritual purification rather than general divine symbolism.",
      ],
      doesNotApplyWhen: [
        "The item is only religious, ceremonial, or devotional without materially helping purification or consecration.",
        "The stronger fit is ritual_support because the item supports a broad rite rather than sanctification in particular.",
      ],
      adjacentTags: ["ritual_support", "ritual_appeasement", "curse_removal"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    source_cleanup: {
      description:
        "Helps find, remove, neutralize, or safely dispose of the cursed object, infected material, corrupted remains, or other source driving the problem.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      adjacentTags: ["source_revelation", "contamination_cleanup", "ritual_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    source_revelation: {
      concept: "source_discovery",
      description:
        "Helps identify the cursed anchor, contaminated material, infected origin, hidden carrier, or other source driving the problem before cleanup begins.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      appliesWhen: [
        "The item's retrieval value comes from finding or confirming the hidden source of a curse, contamination, outbreak, or spiritually tainted problem.",
        "It is naturally sought for tracing the origin or anchor rather than directly cleansing or disposing of it.",
      ],
      doesNotApplyWhen: [
        "The item only helps perform a cleanup, disposal, or purification step after the source is already known.",
        "The stronger fit is medical_support, tracking, or ritual_support because the item supports a broader process without specifically revealing the source.",
      ],
      adjacentTags: ["source_cleanup", "contamination_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.UTILITY_RESTRAINT, {
    restraint_capture: {
      description: "Helps capture, bind, or keep a target restrained.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    restraint_escape: {
      description: "Helps break free from grabs, restraints, or similar immobilizing holds.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.UTILITY_SECURITY, {
    alarm: {
      description: "Alerts you or others when a watched area, threshold, or device is triggered.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    scrying_protection: {
      description:
        "Blocks magical observation, remote viewing, or information leakage through divination-like effects.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      appliesWhen: [
        "The item's main value is preventing magical spying, remote observation, or divination-led tracking.",
        "A user would retrieve it to keep plans, rooms, or identities hidden from magical surveillance.",
      ],
      doesNotApplyWhen: [
        "The item only improves ordinary stealth or concealment without anti-divination protection.",
        "The item counters magic generally but is not particularly about observation or information leakage.",
      ],
      adjacentTags: ["alarm", "concealment"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    security: {
isComposite: true,
      description:
        "Broad security umbrella for gear that warns about intrusion, blocks magical spying, or reveals after-the-fact interference.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      adjacentTags: ["alarm", "scrying_protection", "tamper_evidence"],
      compositeOfAnyTags: ["alarm", "scrying_protection", "tamper_evidence"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    tamper_evidence: {
      description: "Makes intrusion, opening, theft, or interference easier to notice after the fact.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      appliesWhen: [
        "The item is naturally retrieved to reveal whether a lock, crate, seal, letter, cache, or room has been disturbed.",
        "Evidence of interference matters more than immediate warning, direct defense, or anti-scrying.",
      ],
      doesNotApplyWhen: [
        "The item mainly alerts in real time when someone crosses a boundary.",
        "The item hides or protects a target without preserving signs of intrusion.",
      ],
      adjacentTags: ["alarm", "surveillance_recording"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"equipment">[];
