import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const spellUtilityProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.UTILITY_ACCESS_BYPASS, {
    barrier_bypass: {
      description:
        "Gets a creature through a blocked threshold, wall, seal, force barrier, or magical ward that otherwise prevents passage.",
      appliesWhen: [
        "The spell is naturally retrieved to pass through, nullify, or ignore a blocking wall, sealed threshold, force barrier, or magical ward.",
        "Crossing the obstruction matters more than simply traveling farther or counteracting magic in the abstract.",
      ],
      doesNotApplyWhen: [
        "The spell only unlocks a door or manipulates a mechanism without really solving a barrier or ward.",
        "The spell's value is ordinary travel or relocation rather than penetrating a blocked passage.",
      ],
      adjacentTags: ["lock_bypass", "countermagic"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    lock_bypass: {
      description:
        "Opens locks, sealed containers, secured doors, or similar closed access points through magic rather than physical lockpicking.",
      appliesWhen: [
        "The spell is naturally retrieved to unlock, unseal, or open a secured entry point, door, chest, manacle, or similar closure.",
        "Accessing something closed matters more than broad movement, damage, or generic anti-magic.",
      ],
      doesNotApplyWhen: [
        "The spell mainly destroys the obstacle, bypasses the whole wall, or teleports past the problem without interacting with the locked access point.",
        "The spell only manipulates unattended objects generally and opening secured access is not a real retrieval hook.",
      ],
      adjacentTags: ["trap_bypass", "barrier_bypass"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    mechanism_manipulation: {
      description:
        "Precisely triggers, moves, holds, or operates levers, buttons, switches, pressure plates, locks, or similar scene mechanisms.",
      appliesWhen: [
        "The spell is naturally retrieved to operate a lever, button, latch, control panel, pressure surface, or similar mechanism from a safe or unusual position.",
        "The mechanism interaction itself matters more than broad telekinesis, damage, or ordinary object movement.",
      ],
      doesNotApplyWhen: [
        "The spell only moves creatures or loose objects without a real access-, control-, or mechanism-facing use case.",
        "The spell bypasses the obstacle by teleporting or destroying it instead of operating the mechanism.",
      ],
      adjacentTags: ["lock_bypass", "trap_bypass"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    trap_bypass: {
      description:
        "Disarms, suppresses, safely triggers, or helps bypass a trap, warded threshold, or similar trapped access problem.",
      appliesWhen: [
        "The spell is naturally retrieved to disable, neutralize, or get past a trap or trapped access point without simply enduring the hazard.",
        "Trap-solving matters more than generic revelation, scouting, or damage prevention.",
      ],
      doesNotApplyWhen: [
        "The spell only reveals that a trap exists without helping bypass or disable it.",
        "The spell mainly counters open combat hazards or battlefield effects rather than access-facing traps.",
      ],
      adjacentTags: ["lock_bypass", "mechanism_manipulation"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.UTILITY_COMMUNICATION, {
    communication: {
isComposite: true,
      description:
        "Broad communication umbrella for spells used to signal allies, relay messages, bridge language barriers, or coordinate silently.",
      adjacentTags: ["signaling", "telepathic_communication", "message_delivery", "translation_support"],
      compositeOfAnyTags: ["signaling", "telepathic_communication", "message_delivery", "translation_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    message_delivery: {
      description: "Sends, stores, or relays actual content across time or distance.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    signaling: {
      description: "Helps draw attention, mark a location, or coordinate allies.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    telepathic_communication: {
      description:
        "Creates direct mind-to-mind communication, silent tactical coordination, or psychic speech between creatures.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    translation_support: {
      description:
        "Bridges spoken or written language barriers through translation, comprehension, deciphering, or magically shared understanding.",
      appliesWhen: [
        "The spell is naturally retrieved to understand, translate, or make oneself understood across otherwise incompatible languages or scripts.",
        "Language access matters more than merely sending a message or speaking silently.",
      ],
      doesNotApplyWhen: [
        "The spell only transmits content farther or more privately without solving a language barrier.",
        "The spell reveals truth, thoughts, or memories without actually translating speech or writing.",
      ],
      adjacentTags: ["telepathic_communication", "message_delivery"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.UTILITY_CONSULTATION, {
    consultation: {
isComposite: true,
      description:
        "Broad consultation umbrella for spells used to seek cosmic answers, diagnose mysteries, or gain non-sensory divinatory guidance.",
      adjacentTags: ["lore_consultation", "problem_diagnosis", "omen_guidance"],
      compositeOfAnyTags: ["lore_consultation", "problem_diagnosis", "omen_guidance"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    lore_consultation: {
      description:
        "Provides interpretive insight, shared knowledge, or focused understanding about a subject, clue, history, or magical situation.",
      adjacentTags: ["truth_reveal", "problem_diagnosis"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    omen_guidance: {
      description:
        "Asks for omens, directional guidance, or advisory insight about the best course of action, likely outcome, or strategic choice.",
      adjacentTags: ["lore_consultation", "wayfinding"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    problem_diagnosis: {
      description:
        "Helps determine what hidden magical, spiritual, cursed, or otherwise obscure problem is actually affecting a target, site, or situation.",
      adjacentTags: ["curse_revelation", "magic_detection"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.UTILITY_EXPEDITION, {
    aquatic_support: {
      description: "Helps with swimming, underwater breathing, water-surface travel, or other aquatic movement.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    environmental_adaptation: {
      description:
        "Helps creatures endure hostile climates, thin air, smoke, pressure, vacuum, or other expedition-grade environmental extremes.",
      appliesWhen: [
        "The spell is naturally retrieved to survive extreme heat, cold, altitude, smoke, pressure, or other punishing environmental conditions during travel or exploration.",
        "Environmental endurance matters more than only resisting one attack form or creating a place to rest.",
      ],
      doesNotApplyWhen: [
        "The spell mainly grants combat resistance, a protective ward, or aquatic mobility without broader expedition-survival value.",
        "The spell only creates shelter or sustenance rather than adapting creatures to the surrounding environment.",
      ],
      adjacentTags: ["aquatic_support", "field_shelter", "resistance_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    expedition: {
isComposite: true,
      description:
        "Broad expedition umbrella for spells used for routefinding, travel-ready movement, shelter, sustainment, aquatic operations, and hostile-environment survival.",
      adjacentTags: ["navigation", "field_shelter", "environmental_adaptation"],
      compositeOfAnyTags: [
        "navigation",
        "flight",
        "aquatic_support",
        "sustenance",
        "field_shelter",
        "environmental_adaptation",
        "wayfinding",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    field_shelter: {
      description: "Creates shelter, refuge, or a protected resting place in the field.",
      appliesWhen: [
        "The spell is naturally retrieved to create a campsite refuge, safe resting place, or expedition shelter in hostile territory.",
        "Its value is prolonged field habitation or protected rest rather than momentary combat defense.",
      ],
      doesNotApplyWhen: [
        "The spell only creates a brief combat ward, cover effect, or instant defensive barrier.",
        "The spell merely transports creatures away instead of establishing a place to rest.",
      ],
      adjacentTags: ["protective_ward", "planar_travel"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    flight: {
      description: "Grants flying movement, sustained aerial travel, or practical airborne maneuvering.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    mobility: {
      description: "Helps move faster, gain movement modes, or traverse terrain more effectively.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    navigation: {
      description: "Helps orient, guide a route, or identify a destination's direction.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    sustenance: {
      description: "Provides food, water, rations, or practical nourishment for travel and survival.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.UTILITY_INFILTRATION, {
    disguise: {
      description: "Helps alter appearance or impersonate another identity.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    infiltration: {
isComposite: true,
      description: "Broad infiltration umbrella for quiet-entry, disguise, and covert social-passing spells.",
      adjacentTags: ["stealth_support", "disguise", "social_infiltration"],
      compositeOfAnyTags: ["stealth_support", "disguise", "social_infiltration"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    social_infiltration: {
      description: "Helps blend into a group or pass under social scrutiny.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    stealth_support: {
      description:
        "Helps move quietly, avoid notice, suppress noisy presence, or otherwise support covert entry and low-profile movement.",
      appliesWhen: [
        "The spell is naturally retrieved to help a creature move quietly, avoid notice, pass unseen, or keep a covert approach from drawing attention.",
        "The retrieval hook is quiet entry or low-profile movement rather than only broad battlefield obscurity.",
      ],
      doesNotApplyWhen: [
        "The spell only creates a combat concealment effect or visual obstruction without really supporting a covert approach.",
        "The spell changes appearance or social presentation without materially helping the target move unnoticed.",
      ],
      adjacentTags: ["concealment", "silencing"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.UTILITY_RECONNAISSANCE, {
    reconnaissance: {
isComposite: true,
      description:
        "Broad scouting umbrella for spells that gather remote information, extend senses, or track a target from afar.",
      adjacentTags: ["scouting", "tracking", "scouting_summons"],
      compositeOfAnyTags: ["scouting", "tracking", "scouting_summons"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    scouting: {
      description: "Helps observe at a distance, extend senses, or locate a target.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    tracking: {
      description: "Locates a specific creature, object, or destination, or follows a supernatural trail toward it.",
      appliesWhen: [
        "The spell is naturally retrieved to find a named target, trace a quarry, or point the caster toward a specific creature, object, or place.",
        "Target location matters more than broad sensory surveillance or general route guidance.",
      ],
      doesNotApplyWhen: [
        "The spell mainly reveals an area, extends senses, or scouts without locking onto a specific target.",
        "The spell only helps orient a journey or choose a route once the destination is already known.",
      ],
      adjacentTags: ["scouting", "navigation"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.UTILITY_RESOLUTION, {
    contamination_cleanup: {
      description:
        "Cleanses tainted residue, neutralizes corrupted ground, removes lingering pollution, or purifies a contaminated space.",
      adjacentTags: ["quarantine_containment", "source_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    curse_removal: {
      concept: "curse_remediation",
      description:
        "Breaks, removes, or counteracts curses as a direct answer path rather than only suppressing symptoms.",
      adjacentTags: ["exorcism", "sanctification"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    exorcism: {
      concept: "hostile_presence_expulsion",
      label: "exorcism",
      description:
        "Banishes, expels, or spiritually drives out a hostile spirit, possession, haunt, or invading supernatural presence.",
      adjacentTags: ["curse_removal", "sanctification"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    quarantine_containment: {
      concept: "outbreak_containment",
      description:
        "Helps isolate victims, secure a dangerous area, or impose protective boundaries that stop spread while the problem is being solved.",
      adjacentTags: ["protective_ward", "contamination_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    resolution: {
      concept: "problem_resolution",
isComposite: true,
      description:
        "Broad resolution umbrella for spells that break curses, expel hostile presences, contain spread, purify contamination, or solve a supernatural problem at its source.",
      adjacentTags: ["curse_removal", "exorcism", "ritual_appeasement", "source_cleanup"],
      compositeOfAnyTags: [
        "curse_removal",
        "exorcism",
        "sanctification",
        "ritual_appeasement",
        "quarantine_containment",
        "contamination_cleanup",
        "source_revelation",
        "source_cleanup",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    ritual_appeasement: {
      description:
        "Ends a supernatural problem through offerings, restitution, funerary respect, ritual observance, or otherwise satisfying a spiritual demand rather than expelling the presence outright.",
      adjacentTags: ["sanctification", "exorcism"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    sanctification: {
      concept: "sacred_taint_sanctification",
      label: "sanctification",
      description:
        "Consecrates, hallowes, purifies, or spiritually cleanses a creature, object, or site to solve a malign supernatural problem.",
      adjacentTags: ["ritual_appeasement", "exorcism", "protective_ward"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    source_cleanup: {
      description:
        "Neutralizes, destroys, seals, or cleans up the cursed object, infected origin, corrupted site, or anchored source driving the problem.",
      adjacentTags: ["source_revelation", "contamination_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    source_revelation: {
      concept: "source_discovery",
      description:
        "Reveals the hidden source, curse anchor, carrier, infected origin, or spreading point of a supernatural or outbreak problem.",
      adjacentTags: ["problem_diagnosis", "source_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.UTILITY_REVELATION, {
    curse_revelation: {
      concept: "curse_discovery",
      description: "Identifies curses, spiritual corruption, or other malign supernatural bindings on a target.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard_revelation: {
      concept: "hazard_discovery",
      description:
        "Reveals hidden traps, secret wards, concealed passage dangers, or other obscured environmental threats.",
      appliesWhen: [
        "The spell is naturally retrieved to uncover traps, hidden dangers, secret magical wards, or dangerous concealed features in a location.",
        "Hazard discovery matters more than general magical detection or long-range scouting.",
      ],
      doesNotApplyWhen: [
        "The spell only detects magic, invisible creatures, or truth without specifically surfacing dangerous hidden features.",
        "The spell merely scouts an area from afar without exposing concealed trap logic or hazard placement.",
      ],
      adjacentTags: ["magic_detection", "scouting"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    invisibility_reveal: {
      concept: "invisibility_discovery",
      description: "Exposes invisible, hidden, concealed, or magically obscured creatures and objects.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    magic_detection: {
      concept: "magic_discovery",
      description: "Reveals magical auras, spell presence, active effects, or other supernatural signatures.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    revelation: {
      concept: "problem_discovery",
isComposite: true,
      description:
        "Broad reveal umbrella for spells that detect magic, uncover deceptions, expose invisible threats, or identify hidden supernatural problems.",
      adjacentTags: ["magic_detection", "truth_reveal", "hazard_revelation"],
      compositeOfAnyTags: [
        "magic_detection",
        "invisibility_reveal",
        "truth_reveal",
        "curse_revelation",
        "hazard_revelation",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    truth_reveal: {
      concept: "truth_discovery",
      description: "Forces honesty, exposes lies, or reveals disguised, false, or hidden truths.",
      appliesWhen: [
        "The spell's retrieval value comes from exposing deception, forcing truthful answers, or stripping away false presentation.",
        "A user would plausibly look for it when they need an answer spell rather than a sensor spell.",
      ],
      doesNotApplyWhen: [
        "The spell only detects magic, invisibility, or general auras without interrogating truth or deception.",
        "The spell mainly alters memory or emotions rather than revealing facts.",
      ],
      adjacentTags: ["magic_detection", "memory_manipulation"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.UTILITY_SECURITY, {
    alarm: {
      description: "Alerts you or others when a watched area, threshold, or ward is crossed.",
      appliesWhen: [
        "The spell is naturally retrieved to warn about intrusion, threshold crossing, tampering, or unwanted entry.",
        "Detection and notice matter more than directly stopping the intruder.",
      ],
      doesNotApplyWhen: [
        "The spell mainly protects, blocks, or hides the target without providing a warning function.",
        "The spell only reveals truth or magic generally rather than guarding a watched perimeter.",
      ],
      adjacentTags: ["protective_ward", "scrying_protection"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    scrying_protection: {
      description:
        "Blocks magical observation, remote viewing, divinatory surveillance, or other information leakage from a protected target or space.",
      appliesWhen: [
        "The spell is naturally retrieved to keep plans, sanctums, identities, or conversations hidden from magical spying.",
        "Its core value is denying observation or divination rather than only raising an intrusion alarm.",
      ],
      doesNotApplyWhen: [
        "The spell only improves mundane concealment or silence without real anti-divination protection.",
        "The spell counters magic broadly but is not specifically about surveillance or remote observation.",
      ],
      adjacentTags: ["alarm", "countermagic"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    security: {
isComposite: true,
      description:
        "Broad security umbrella for spells that warn about intrusion, protect private spaces, or harden a target against magical observation and interference.",
      adjacentTags: ["alarm", "scrying_protection", "protective_ward"],
      compositeOfAnyTags: ["alarm", "scrying_protection", "protective_ward", "countermagic"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.UTILITY_SENSORY_SUPPORT, {
    illumination: {
      description:
        "Produces practical light that brightens darkness, reveals an area, or lets creatures see more clearly.",
      adjacentTags: ["senses_support", "line_of_sight_control"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    senses_support: {
      description:
        "Enhances vision or other senses through darkvision, see invisible, sharpened perception, scent, or similar perceptual upgrades.",
      adjacentTags: ["scouting", "invisibility_reveal"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.UTILITY_TELEPORTATION, {
    extraction_teleport: {
      description: "Teleports a creature out of danger, through restraints, or away from immediate threat pressure.",
      appliesWhen: [
        "The spell is naturally retrieved as an escape, rescue, or anti-capture tool rather than only a movement spell.",
        "The reposition breaks danger, confinement, or immediate battlefield pressure.",
      ],
      doesNotApplyWhen: [
        "The spell is mostly a neutral short-range blink or a long-distance travel effect.",
        "The spell primarily opens planar movement rather than emergency extraction.",
      ],
      adjacentTags: ["short_range_teleport", "escape_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    long_range_teleport: {
      description: "Teleports creatures across major overland distances, settlements, or remote destinations.",
      appliesWhen: [
        "The spell is naturally retrieved for strategic travel, relocation, or bypassing long routes.",
        "The destination scale is substantially larger than one encounter map or immediate tactical space.",
      ],
      doesNotApplyWhen: [
        "The spell is mainly a tactical blink, extraction, or planar crossing effect.",
        "The spell only repositions creatures within the same immediate scene.",
      ],
      adjacentTags: ["short_range_teleport", "planar_travel"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    planar_travel: {
      description: "Moves creatures between planes, through planar routes, or into extraplanar destinations.",
      appliesWhen: [
        "Crossing into another plane, demiplane, or extraplanar route is central to the spell's retrieval value.",
        "The spell is naturally retrieved for cosmological travel rather than mundane relocation.",
      ],
      doesNotApplyWhen: [
        "The spell only teleports within the same plane or functions as a normal long-distance travel tool.",
        "Extradimensional storage or shelter is present without real plane-crossing travel.",
      ],
      adjacentTags: ["long_range_teleport", "field_shelter"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    short_range_teleport: {
      description:
        "Teleports a creature across a short tactical distance, usually within the same scene or encounter area.",
      appliesWhen: [
        "The spell repositions a creature within the current encounter or scene rather than serving as expedition travel.",
        "The tactical blink or reposition is itself a major reason to retrieve the spell.",
      ],
      doesNotApplyWhen: [
        "The spell is mainly about escaping custody, extracting allies, or long-distance transport.",
        "The spell primarily crosses planes or major overland distances.",
      ],
      adjacentTags: ["extraction_teleport", "long_range_teleport"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.UTILITY_WAYFINDING, {
    wayfinding: {
isComposite: true,
      description:
        "Broad route-and-destination umbrella for spells that orient travel, locate a target destination, or bypass distance through strategic movement magic.",
      adjacentTags: ["navigation", "tracking", "long_range_teleport"],
      compositeOfAnyTags: ["navigation", "tracking", "long_range_teleport", "planar_travel"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"spell">[];
