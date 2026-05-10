import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const DescriptiveRoleProjectionDeclarations = [
  defineConceptProjections("ambusher_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Built around stealth openings, surprise rounds, sudden strike pressure, or hidden attack vectors.",
      appliesWhen: [
        "The creature is naturally retrieved for hidden approach, surprise attack, trapdoor positioning, or burst from concealment.",
        "Opening from stealth is central to how it functions in an encounter.",
      ],
      doesNotApplyWhen: [
        "The creature only uses stealth incidentally before acting as another clearer combat role.",
        "The stronger identity is skirmisher_combatant or artillery_combatant rather than surprise-predator play.",
      ],
      adjacentTags: ["skirmisher_combatant", "brute_combatant"],
    },
  }),
  defineConceptProjections("artillery_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Built to pressure targets from range through volleys, spell barrages, breath attacks, or other standoff offense.",
      appliesWhen: [
        "The creature is naturally retrieved as a ranged damage or spell-barrage threat that prefers distance.",
        "Its tactical identity is standoff pressure more than command, mobility, or pure control.",
      ],
      doesNotApplyWhen: [
        "The creature happens to have one ranged option but still fundamentally plays as a brute, controller, or support piece.",
        "Its main retrieval hook is casting support or battlefield control rather than ranged offense.",
      ],
      adjacentTags: ["harrier_combatant", "controller_combatant", "support_combatant"],
    },
  }),
  defineConceptProjections("artisan_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Presented as a smith, craftsperson, builder, artisan, or other maker-facing role-holder tied to production or skilled labor.",
      appliesWhen: [
        "Craft production, repair labor, or skilled making work is central to the creature's world-facing identity.",
        "A GM would plausibly retrieve it as a blacksmith, mason, tailor, shipwright, or similarly maker-facing contact.",
      ],
      doesNotApplyWhen: [
        "The creature only sells goods or manages trade without being defined by making or repair work.",
        "The stronger fit is merchant_npc, profession_npc, or civic_npc without a clear craft-labor role.",
      ],
      adjacentTags: ["merchant_npc", "profession_npc"],
    },
  }),
  defineConceptProjections("authority_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Presented as an officer, magistrate, noble, administrator, or other figure of formal social authority.",
      appliesWhen: [
        "The creature's retrieval value comes from official office, rank, command, or institutional authority.",
        "A GM would plausibly seek it as a leader, official, or governing figure.",
        "Formal office or rank is the main retrieval hook, even if the creature also serves as a civic_npc or enforcer_npc in the scene.",
      ],
      doesNotApplyWhen: [
        "The record is only a generic combatant without meaningful office or status.",
        "The stronger fit is profession_npc, civic_npc, or enforcer_npc because status is incidental.",
      ],
      adjacentTags: ["profession_npc", "civic_npc"],
    },
  }),
  defineConceptProjections("brute_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Built to pressure the front line through durability, direct damage, and straightforward melee threat.",
      appliesWhen: [
        "The creature is naturally retrieved as a heavy hitter that advances, endures punishment, and threatens through direct force.",
        "Its tactical identity is more about sturdy pressure than mobility, command, or precision control.",
      ],
      doesNotApplyWhen: [
        "The creature mainly operates from range, through battlefield control, or through ally support.",
        "High damage is present but the stronger combat identity is ambush, artillery, or commander play.",
      ],
      adjacentTags: ["defender_combatant", "artillery_combatant"],
    },
  }),
  defineConceptProjections("captive_npc", {
    creature: {
      axis: "npc_role",
      family: "scene_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Immediate-scenario prisoner, hostage, detained witness, sacrifice target, or other constrained figure whose scene value comes from being held or imperiled.",
      appliesWhen: [
        "The creature is naturally retrieved because it is held, imprisoned, restrained, threatened, or otherwise scenically constrained.",
        "Its immediate scenario function is being rescued, questioned, transported, or guarded rather than acting freely in the scene.",
      ],
      doesNotApplyWhen: [
        "The creature is merely vulnerable, socially subordinate, or under pressure without actually being a captive or detained figure.",
        "The stronger fit is civic_npc or profession_npc because social embeddedness or world role matters more than captivity.",
      ],
      adjacentTags: ["guardian_npc", "civic_npc"],
    },
  }),
  defineConceptProjections("civic_npc", {
    creature: {
      axis: "npc_role",
      family: "scene_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Fits the civic or social fabric of a scene and usually isn't the primary monster answer.",
      appliesWhen: [
        "The creature belongs to the civic, domestic, institutional, or everyday social fabric of the scene.",
        "A GM would plausibly retrieve it as a socially embedded NPC rather than as a combat-forward foe.",
        "This tag answers the creature's immediate scene slot, even if a separate social_role tag explains its profession or office.",
      ],
      doesNotApplyWhen: [
        "The record is primarily a hostile combatant or raider.",
        "The creature is only profession-labeled without clear social embeddedness, or the profession or office itself is the main retrieval hook.",
      ],
      adjacentTags: ["profession_npc", "enforcer_npc"],
    },
  }),
  defineConceptProjections("commander_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Built to coordinate allies round to round through leadership, tactics, or command-driven positioning.",
      appliesWhen: [
        "The creature is naturally retrieved because its round-to-round battle role is directing allies, calling tactics, or coordinating a group.",
        "Leadership and coordination matter more than its own solo offense, brute durability, or passive aura support.",
      ],
      doesNotApplyWhen: [
        "The creature only has generic support effects with no clear command or tactical leadership role in the moment-to-moment fight.",
        "The stronger fit is support_combatant, artillery_combatant, or reinforcement_threat rather than leader play.",
      ],
      adjacentTags: ["support_combatant", "reinforcement_threat"],
    },
  }),
  defineConceptProjections("controller_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Built to reshape the battlefield through debuffs, forced movement, terrain control, or other tactical denial.",
      appliesWhen: [
        "The creature is naturally retrieved for immobilizing, repositioning, walling off, slowing, or otherwise dictating battlefield shape.",
        "Tactical denial matters more than direct damage output.",
      ],
      doesNotApplyWhen: [
        "The creature only has one incidental debuff while otherwise fighting as a brute, defender, or artillery piece.",
        "The stronger identity is support or commander rather than enemy-space control.",
      ],
      adjacentTags: ["support_combatant", "artillery_combatant"],
    },
  }),
  defineConceptProjections("court_entourage", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Naturally retrieved as part of a courtly retinue, noble household, ceremonial train, or political chamber roster around a central authority figure.",
      adjacentTags: ["authority_npc", "courtly_pageantry"],
    },
  }),
  defineConceptProjections("crew_member", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Naturally retrieved as part of a ship crew, dock crew, wreck complement, or other nautical working roster that belongs together in one scene.",
      adjacentTags: ["nautical_setting", "escort_npc"],
    },
  }),
  defineConceptProjections("criminal_cell", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Naturally retrieved as part of a gang, smuggling ring, burglary team, or other underworld roster that functions through a small coordinated cell.",
      adjacentTags: ["criminal_npc", "infiltrator_npc"],
    },
  }),
  defineConceptProjections("criminal_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Presented as a thief, smuggler, assassin, gang operative, fence, or other explicitly underworld-coded role.",
      appliesWhen: [
        "The creature is defined by illicit trade, organized crime, covert violence, or other criminal social function.",
        "A GM would retrieve it as an underworld contact or criminal adversary rather than a generic soldier.",
      ],
      doesNotApplyWhen: [
        "The creature is merely hostile without underworld or crime-scene framing.",
        "The stronger fit is enforcer_npc without a distinct criminal identity.",
      ],
      adjacentTags: ["enforcer_npc", "infiltrator_npc"],
    },
  }),
  defineConceptProjections("cult_member", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Naturally retrieved as one member of a ritual circle, hidden sect, temple conspiracy, or other cultic roster built around shared devotion or doctrine.",
      adjacentTags: ["religious_npc", "ritualist_creature"],
    },
  }),
  defineConceptProjections("defender_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Built to hold space, intercept attacks, bodyguard allies, or punish passage through a defended line.",
      appliesWhen: [
        "The creature is naturally retrieved for guarding chokepoints, bodyguarding allies, or making passage costly.",
        "Space-holding matters more than raw pursuit, burst damage, or command.",
      ],
      doesNotApplyWhen: [
        "The creature is merely durable without a real guard, intercept, or line-holding identity.",
        "The stronger fit is brute_combatant or another non-guardian role.",
      ],
      adjacentTags: ["brute_combatant", "commander_combatant"],
    },
  }),
  defineConceptProjections("enforcer_npc", {
    creature: {
      axis: "npc_role",
      family: "scene_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Scene-slot fight-first humanoid adversary such as a soldier, bandit, mercenary, or other overt martial enforcer.",
      appliesWhen: [
        "The creature is presented as a role-defined humanoid adversary whose immediate scene value is direct martial pressure, enforcement, or armed opposition.",
        "Retrieval value comes from being a fight-first scene answer rather than a civic, social, or watch-post role.",
        "This tag answers how the creature functions in the immediate scene, even if it separately has a profession, rank, or institutional role.",
      ],
      doesNotApplyWhen: [
        "The record is mainly a social, occupational, escort, or posted-guard NPC whose scene identity is not primarily direct armed opposition.",
        "The creature is better captured by profession_npc, civic_npc, guardian_npc, or watcher_npc without a strong fight-first role.",
        "The office, profession, surveillance role, or posted protection duty is the stronger retrieval hook and combat readiness is only secondary.",
      ],
      adjacentTags: ["authority_npc", "civic_npc"],
    },
  }),
  defineConceptProjections("escort_npc", {
    creature: {
      axis: "npc_role",
      family: "scene_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Immediate-scenario escort, courier companion, guide-on-mission, ward mover, or other figure whose scene value is accompanying or moving someone through danger.",
      appliesWhen: [
        "The creature is naturally retrieved because it actively escorts, transports, shepherds, or accompanies another figure through a dangerous scene or route.",
        "Its scene function is movement-with-charge or safe transit rather than only office, profession, or posted guard duty.",
      ],
      doesNotApplyWhen: [
        "The creature merely knows the route, provides directions, or protects a place without actively accompanying someone.",
        "The stronger fit is guide_npc or guardian_npc because route expertise or posted guard duty matters more than accompaniment.",
      ],
      adjacentTags: ["guide_npc", "guardian_npc"],
    },
  }),
  defineConceptProjections("guardian_npc", {
    creature: {
      axis: "npc_role",
      family: "scene_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Immediate-scenario guard, jailer, doorkeeper, bodyguard, or other posted protector whose scene value is physically holding or protecting a person, threshold, or place.",
      appliesWhen: [
        "The creature is naturally retrieved because it is posted to guard, hold, jail, or protect a specific person, threshold, route, or space.",
        "Posted protection or interdiction matters more than general combat readiness, surveillance, or broad social authority.",
      ],
      doesNotApplyWhen: [
        "The creature is merely combat-ready without a clear posted protection, bodyguard, or threshold-holding duty.",
        "The stronger fit is authority_npc, enforcer_npc, or watcher_npc because command status, generic martial opposition, or alarm duty matters more than guarding.",
      ],
      adjacentTags: ["enforcer_npc", "watcher_npc"],
    },
  }),
  defineConceptProjections("guardian_retinue", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Naturally retrieved as one member of a posted defense roster around a leader, relic, sanctum, or protected threshold rather than as an independent threat.",
      adjacentTags: ["guardian_npc", "defender_combatant"],
    },
  }),
  defineConceptProjections("guide_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Presented as a scout, tracker, ferryman, caravan guide, wilderness pathfinder, or other route-leading specialist.",
      appliesWhen: [
        "Leading others through terrain, routes, borders, or dangerous travel spaces is central to the creature's world-facing identity.",
        "The record is naturally used as a scout, pathfinder, navigator, or local guide rather than only a generic outdoors person.",
      ],
      doesNotApplyWhen: [
        "The creature merely knows the area or has survival competence without a role-defined guiding function.",
        "The stronger fit is scholar_npc, civic_npc, or enforcer_npc rather than travel-leading expertise.",
      ],
      adjacentTags: ["profession_npc", "rural_setting"],
    },
  }),
  defineConceptProjections("harrier_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Built to chip away from safety through repeated ranged harassment, flyby pressure, or evasive standoff attacks that force pursuit.",
      adjacentTags: ["skirmisher_combatant", "artillery_combatant"],
    },
  }),
  defineConceptProjections("healer_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Presented as a physician, battlefield medic, herbalist, chirurgeon, caretaker, or other explicitly healing-facing role-holder.",
      appliesWhen: [
        "Medical treatment, recovery support, or caretaker duty is central to the creature's world-facing identity.",
        "The record is naturally used as a healer, medic, apothecary, or restorative support NPC rather than only a generic scholar.",
      ],
      doesNotApplyWhen: [
        "The creature merely has healing magic or restorative abilities without a role-defined caregiving identity.",
        "The stronger fit is scholar_npc, religious_npc, or civic_npc without a real healer-facing job.",
      ],
      adjacentTags: ["scholar_npc", "religious_npc"],
    },
  }),
  defineConceptProjections("infestation_member", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Naturally retrieved as one body in a swarm-like infestation, burrowing colony, parasite outbreak, or other many-body nuisance roster.",
      adjacentTags: ["parasite_ridden", "plaguebearing"],
    },
  }),
  defineConceptProjections("infiltrator_npc", {
    creature: {
      axis: "npc_role",
      family: "scene_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Scene-slot spy, saboteur, replacer, or quiet-entry specialist whose immediate scenario value comes from infiltration more than a straight fight.",
      appliesWhen: [
        "The creature is naturally retrieved as an embedded infiltrator, saboteur, impostor, spy, or quiet-entry specialist.",
        "This tag answers the creature's immediate scenario function rather than its broader profession, faction post, or criminal affiliation.",
      ],
      doesNotApplyWhen: [
        "The creature is mainly an enforcer_npc or civic_npc and only uses stealth or deception incidentally.",
        "The stronger retrieval hook is criminal_npc or another social_role tag because the world-facing identity matters more than the scene slot.",
      ],
      adjacentTags: ["enforcer_npc", "criminal_npc"],
    },
  }),
  defineConceptProjections("merchant_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Presented as a trader, broker, shopkeeper, caravan factor, or other commerce-facing role-holder.",
      appliesWhen: [
        "Trade, selling, bargaining, or inventory-handling is central to the creature's world-facing identity.",
        "The creature naturally fills a market, shop, caravan, or supply-scene role.",
      ],
      doesNotApplyWhen: [
        "The record only implies wealth or possessions without an actual merchant role.",
        "The stronger fit is civic_npc or profession_npc without a clear commerce function.",
      ],
      adjacentTags: ["profession_npc", "civic_npc"],
    },
  }),
  defineConceptProjections("pack_hunter", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Naturally retrieved because the creature hunts as part of a pack, coordinated ambush group, or pursuit cluster rather than as a solitary predator.",
      adjacentTags: ["ambusher_combatant", "skirmisher_combatant"],
    },
  }),
  defineConceptProjections("patrol_member", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Naturally retrieved as part of a watch patrol, sentry route, street sweep, border detail, or other recurring patrol formation.",
      adjacentTags: ["watcher_npc", "urban_setting"],
    },
  }),
  defineConceptProjections("performer_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Presented as a musician, actor, dancer, herald, jester, or other entertainment-facing role-holder.",
      appliesWhen: [
        "Performance, spectacle, or entertainment labor is central to the creature's world-facing identity.",
        "The creature would be retrieved for theater, carnival, court entertainment, or tavern-stage scenes.",
      ],
      doesNotApplyWhen: [
        "The creature is only whimsical or colorful without an explicit performer role.",
        "The stronger semantic is carnival_show rather than a role-defined entertainer.",
      ],
      adjacentTags: ["profession_npc", "carnival_show"],
    },
  }),
  defineConceptProjections("profession_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Role-defined NPC such as a captain, guard, merchant, priest, or commoner.",
      appliesWhen: [
        "The creature is primarily presented through a social role, job, office, or profession label.",
        "Retrieval value comes from it being a role-defined NPC rather than a species-driven monster.",
        "This tag answers what the creature is in the world, even if a separate scene_role tag explains how it functions in the scene.",
      ],
      doesNotApplyWhen: [
        "The role label is incidental to a stronger monster or combat identity.",
        "The creature is better modeled only as enforcer_npc, infiltrator_npc, or civic_npc because the scene slot matters more than the job or office.",
      ],
      adjacentTags: ["authority_npc", "merchant_npc"],
    },
  }),
  defineConceptProjections("religious_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Presented as a priest, shrine keeper, cult officiant, monastic figure, or other explicitly religious role-holder.",
      appliesWhen: [
        "Religious office, ritual duty, or custodianship of a faith space is central to the creature's world-facing identity.",
        "The creature is naturally retrieved as clergy, cult staff, or sacred-site personnel.",
      ],
      doesNotApplyWhen: [
        "The creature merely has divine powers without a role-defined religious identity.",
        "Temple placement alone is better captured by temple_setting.",
      ],
      adjacentTags: ["profession_npc", "temple_setting"],
    },
  }),
  defineConceptProjections("scholar_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Presented as a sage, researcher, teacher, archivist, alchemist, or other knowledge-centered role-holder.",
      appliesWhen: [
        "Research, teaching, scholarship, or recordkeeping is central to the creature's world-facing identity.",
        "The creature is naturally retrieved as an academic or knowledge-scene NPC.",
      ],
      doesNotApplyWhen: [
        "Intelligence is incidental to a stronger combat or monster identity.",
        "The record only implies general competence without a knowledge-centered role.",
      ],
      adjacentTags: ["profession_npc", "civic_npc"],
    },
  }),
  defineConceptProjections("skirmisher_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Built around mobility, repositioning, hit-and-run pressure, or opportunistic strikes.",
      appliesWhen: [
        "The creature's retrieval value comes from darting movement, flanking, repositioning, or repeated opportunistic attacks.",
        "Mobility and pressure cycling matter more than armor, command, or raw ranged bombardment.",
      ],
      doesNotApplyWhen: [
        "The creature mainly opens from stealth once and then behaves like another clearer combat role.",
        "The stronger identity is brute, ambusher, or artillery rather than mobile harassment.",
      ],
      adjacentTags: ["ambusher_combatant", "controller_combatant"],
    },
  }),
  defineConceptProjections("support_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Built to heal, buff, protect, command, or otherwise enable allied creatures more than acting as the primary damage source.",
      appliesWhen: [
        "The creature is naturally retrieved because it keeps allies alive, stronger, better positioned, or otherwise more dangerous.",
        "Ally enablement matters more than its own direct kill pressure.",
      ],
      doesNotApplyWhen: [
        "The creature has one incidental buff or heal but is otherwise a brute, artillery, or controller.",
        "The stronger identity is commander_combatant because leadership and coordination outweigh broad support.",
      ],
      adjacentTags: ["commander_combatant", "controller_combatant"],
    },
  }),
  defineConceptProjections("warband_member", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Naturally retrieved as one body in a raiding party, battle line, war camp, or other organized hostile fighting band.",
      adjacentTags: ["commander_combatant", "battlefield_setting"],
    },
  }),
  defineConceptProjections("watcher_npc", {
    creature: {
      axis: "npc_role",
      family: "scene_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Immediate-scenario lookout, sentry, observer, or patrol-point watcher whose scene value is warning, spotting, or noticing intruders.",
      appliesWhen: [
        "The creature is naturally retrieved as a lookout, sentry, rooftop watcher, scout-on-post, or other early-warning presence.",
        "Observation and alarm value matter more than physically blocking passage, bodyguarding a charge, or acting as a general frontline enforcer.",
      ],
      doesNotApplyWhen: [
        "The creature is simply a posted guard or enforcer without a strong surveillance, early-warning, or lookout function.",
        "The stronger fit is guide_npc or civic_npc because travel knowledge or social embedding matters more than active watch duty.",
      ],
      adjacentTags: ["guardian_npc", "infiltrator_npc"],
    },
  }),
] satisfies ConceptProjectionDeclaration[];
