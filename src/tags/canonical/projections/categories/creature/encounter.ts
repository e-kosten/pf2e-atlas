import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";

export const creatureEncounterProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.CREATURE.ENCOUNTER_COHORT_ROLE, {
    court_entourage: {
      description:
        "Naturally retrieved as part of a courtly retinue, noble household, ceremonial train, or political chamber roster around a central authority figure.",
      adjacentTags: ["authority_npc", "courtly_pageantry"],
    },
    crew_member: {
      description:
        "Naturally retrieved as part of a ship crew, dock crew, wreck complement, or other nautical working roster that belongs together in one scene.",
      adjacentTags: ["nautical_setting", "escort_npc"],
    },
    criminal_cell: {
      description:
        "Naturally retrieved as part of a gang, smuggling ring, burglary team, or other underworld roster that functions through a small coordinated cell.",
      adjacentTags: ["criminal_npc", "infiltrator_npc"],
    },
    cult_member: {
      description:
        "Naturally retrieved as one member of a ritual circle, hidden sect, temple conspiracy, or other cultic roster built around shared devotion or doctrine.",
      adjacentTags: ["religious_npc", "ritualist_creature"],
    },
    guardian_retinue: {
      description:
        "Naturally retrieved as one member of a posted defense roster around a leader, relic, sanctum, or protected threshold rather than as an independent threat.",
      adjacentTags: ["guardian_npc", "defender_combatant"],
    },
    infestation_member: {
      description:
        "Naturally retrieved as one body in a swarm-like infestation, burrowing colony, parasite outbreak, or other many-body nuisance roster.",
      adjacentTags: ["parasite_ridden", "plaguebearing"],
    },
    pack_hunter: {
      description:
        "Naturally retrieved because the creature hunts as part of a pack, coordinated ambush group, or pursuit cluster rather than as a solitary predator.",
      adjacentTags: ["ambusher_combatant", "skirmisher_combatant"],
    },
    patrol_member: {
      description:
        "Naturally retrieved as part of a watch patrol, sentry route, street sweep, border detail, or other recurring patrol formation.",
      adjacentTags: ["watcher_npc", "urban_setting"],
    },
    warband_member: {
      description:
        "Naturally retrieved as one body in a raiding party, battle line, war camp, or other organized hostile fighting band.",
      adjacentTags: ["commander_combatant", "battlefield_setting"],
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.CREATURE.ENCOUNTER_COMBAT_ROLE, {
    ambusher_combatant: {
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
    artillery_combatant: {
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
    brute_combatant: {
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
    commander_combatant: {
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
    controller_combatant: {
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
    defender_combatant: {
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
    harrier_combatant: {
      description:
        "Built to chip away from safety through repeated ranged harassment, flyby pressure, or evasive standoff attacks that force pursuit.",
      adjacentTags: ["skirmisher_combatant", "artillery_combatant"],
    },
    skirmisher_combatant: {
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
    support_combatant: {
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
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.CREATURE.ENCOUNTER_THREAT_PROFILE, {
    ambush_grabber: {
      concept: "ambush_grab",
      description:
        "Captures prey through grabbing, constriction, swallowing whole, webbing, or drag-off ambush tactics.",
    },
    curse_threat: {
      concept: "curse_application",
      description:
        "Threat defined by curses, doom effects, or other lingering supernatural afflictions imposed on victims.",
    },
    death_burst_threat: {
      concept: "death_burst",
      description:
        "Threat defined by explosive death effects, cursed aftermath, or punishing consequences when the creature is dropped.",
    },
    disease_vector: {
      concept: "disease_application",
      description:
        "Threat defined by spreading disease, curse-plague conditions, or infectious aftermath beyond immediate damage.",
    },
    infiltration_threat: {
      concept: "infiltration",
      description:
        "Threat defined by disguise, replacement, infiltration, or remaining embedded among victims before the danger fully reveals itself.",
      adjacentTags: ["disguised_pretender", "possession_threat"],
    },
    life_drain_threat: {
      concept: "life_drain_application",
      description: "Threat defined by draining blood, vitality, life force, or souls from victims.",
    },
    petrification_threat: {
      concept: "petrification_application",
      description: "Threat defined by petrifying victims or turning them to stone.",
    },
    poison_threat: {
      concept: "poison_application",
      description: "Threat defined by venom, toxic excretions, poisoned weapons, or other recurring poison delivery.",
    },
    possession_threat: {
      concept: "possession_application",
      description: "Can possess, body-snatch, or take control of a victim from within.",
      appliesWhen: [
        "Use when entering, riding, replacing, or controlling a host body is a major reason to retrieve the creature.",
        "The possession dynamic matters more than ordinary charm, domination, or haunting flavor.",
      ],
      doesNotApplyWhen: [
        "The creature only compels, frightens, or mentally influences targets without true body-occupying takeover.",
        "The stronger fit is curse_threat or reinforcement_threat because possession is not central to encounter prep.",
      ],
      adjacentTags: ["curse_threat", "reinforcement_threat"],
    },
    prey_control_threat: {
      concept: "prey_control",
      description:
        "Threat defined by holding prey in place through grabs, constriction, webbing, swallowing, or other ongoing body-control pressure.",
      adjacentTags: ["ambush_grabber", "terrain_control_threat"],
    },
    regeneration_threat: {
      concept: "regeneration",
      description: "Regenerates or requires special suppression or finishing countermeasures.",
    },
    reinforcement_threat: {
      concept: "reinforcement",
      description:
        "Threat defined by materially changing encounter structure through added bodies, activated subordinates, or sharply elevated allied creatures.",
      appliesWhen: [
        "Use when the creature's main prep significance is that it adds bodies, activates subordinates, or sharply force-multiplies nearby allies.",
        "The encounter meaningfully changes because of its reinforcement engine rather than just because it personally hits hard.",
      ],
      doesNotApplyWhen: [
        "The creature only has one minor ally-facing buff or an incidental summon without materially changing encounter structure.",
        "The stronger fit is support_combatant, commander_combatant, or spawn_creator because reinforcement is not the real threat hook.",
      ],
      adjacentTags: ["spawn_creator", "commander_combatant"],
    },
    spawn_creator: {
      concept: "spawn_creation",
      description: "Creates additional threats through infestation, spawn-making, conversion, or implanted offspring.",
    },
    terrain_control_threat: {
      concept: "terrain_control",
      description:
        "Threat defined by webs, walls, zones, hazards, or other space-shaping control that changes battlefield movement.",
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"creature">[];
