import { defineConceptProjections } from "../../builders.js";
import { CANONICAL_VOCABULARY } from "../../vocabulary.js";

export const descriptiveProjectionDeclarations = [
  defineConceptProjections("abaddon_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Abaddon, daemons, or soul-devouring lower-planar wastelands.",
      nativeOntologyPolicy: "aggregates_native_signals",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("abyss_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Abyss, demon realms, or qlippoth-infested outer rifts.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("action_denial", {
    affliction: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Prevents normal action-taking through paralysis, stupefying shutdown, or similarly severe operational lockout.",
      appliesWhen: [
        "The affliction is naturally retrieved because victims become unable to act, respond, or complete normal turns through paralysis, shutdown, or severe stupor.",
        "Operational lockout matters more than ordinary weakness, pain, or mood distortion.",
      ],
      doesNotApplyWhen: [
        "The affliction only slows, weakens, or frightens the victim without truly preventing action-taking.",
        "The stronger fit is mobility_impairment, sedation, or mental_impairment rather than hard action denial.",
      ],
      adjacentTags: [
        "mobility_impairment",
        "sedation",
      ],
    },
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Denies actions through paralysis, stupefying shutdown, slowed tempo, or similarly severe turn disruption.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("action_economy_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable because it compresses setup, speeds access, or meaningfully improves in-combat action efficiency.",
    },
  }),
  defineConceptProjections("alarm", {
    equipment: {
      axis: "utility",
      family: "security",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Alerts you or others when a watched area, threshold, or device is triggered.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Alerts guardians, onlookers, or nearby creatures to an intrusion.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "security",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Alerts you or others when a watched area, threshold, or ward is crossed.",
      appliesWhen: [
        "The spell is naturally retrieved to warn about intrusion, threshold crossing, tampering, or unwanted entry.",
        "Detection and notice matter more than directly stopping the intruder.",
      ],
      doesNotApplyWhen: [
        "The spell mainly protects, blocks, or hides the target without providing a warning function.",
        "The spell only reveals truth or magic generally rather than guarding a watched perimeter.",
      ],
      adjacentTags: [
        "protective_ward",
        "scrying_protection",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("alchemical_crafting", {
    equipment: {
      axis: "utility",
      family: "crafting_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports alchemical preparation, formula work, reagent handling, or crafting-related field setup.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "consumable",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("alien_technology_wasteland_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with alien technology, robots, mutants, strange metal ruins, and science-fantasy wastelands. In Golarion, this primarily corresponds to Numeria.",
      appliesWhen: [
        "Use when the creature is naturally retrieved for weird-tech wilderness, star-metal ruins, robot-haunted badlands, mutant frontiers, or barbarian-meets-super-science planning.",
        "The planning value comes from science-fantasy intrusion into the region's creature ecology rather than only from a single construct or technological gimmick.",
      ],
      doesNotApplyWhen: [
        "The creature only uses one technological item or construct-like ability without a broader weird-tech regional frame.",
        "The stronger fit is bound_object or a combat-role tag because the planning value is tactical rather than regional or thematic.",
      ],
      adjacentTags: [
        "magic_blight_wasteland_setting",
        "animated_object",
      ],
    },
  }),
  defineConceptProjections("ally_cover", {
    equipment: {
      axis: "item_mechanical",
      family: "defense_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Provides cover or upgraded cover to nearby allies.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "shield",
      ],
    },
  }),
  defineConceptProjections("ally_support", {
    equipment: {
      axis: "effect",
      family: "consumable_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Support consumable that can directly benefit another creature.",
      subcategories: [
        "consumable",
      ],
    },
  }),
  defineConceptProjections("ambush_burst", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard designed to open with a sudden high-damage strike or surprise punish on first contact.",
      appliesWhen: [
        "The hazard is naturally retrieved for a sudden opener, trap-spring punish, or first-contact burst that catches intruders before a longer fight develops.",
        "The surprise spike matters more than sustained zone control or prolonged attrition.",
      ],
      doesNotApplyWhen: [
        "The hazard mainly holds territory over time, guards a place persistently, or taxes resources across repeated rounds.",
        "The stronger fit is zone_denial or attrition_pressure rather than a front-loaded strike.",
      ],
      adjacentTags: [
        "zone_denial",
        "attrition_pressure",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
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
      adjacentTags: [
        "skirmisher_combatant",
        "brute_combatant",
      ],
    },
  }),
  defineConceptProjections("ammo_management", {
    equipment: {
      axis: "item_mechanical",
      family: "access_system",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Magazines or related gear that manage repeating-weapon ammunition or reload workflow.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "ammo",
        "armor",
        "weapon",
      ],
    },
  }),
  defineConceptProjections("ancestral_legacy", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with bloodline burdens, inherited duty, dynastic memory, haunting lineage, or the weight of family legacy.",
      appliesWhen: [
        "The creature's presentation depends on lineage, inheritance, dynastic fate, ancestral memory, or family burden carried into the present.",
        "Retrieval value comes from inherited role, curse, or legacy rather than only social rank or prophecy.",
      ],
      doesNotApplyWhen: [
        "The creature merely belongs to a noble house, species line, or ancestry without legacy pressure being central.",
        "The stronger fit is prophecy_omen or courtly_pageantry because omen-bearing destiny or aristocratic presentation matters more than inherited burden.",
      ],
      adjacentTags: [
        "prophecy_omen",
        "courtly_pageantry",
      ],
    },
  }),
  defineConceptProjections("animated_object", {
    creature: {
      axis: "presentation",
      family: "bound_object",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with animated objects, furniture, tools, or other constructed items.",
    },
  }),
  defineConceptProjections("animated_statue", {
    creature: {
      axis: "presentation",
      family: "bound_object",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with animated statues, effigies, idols, or monuments.",
    },
  }),
  defineConceptProjections("anti_tracking", {
    equipment: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps hide your trail, mask scent, or make pursuit harder.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("apocalypse_ruin", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with end-times, civilizational collapse, world-ending omen, or the sense that the creature heralds broad unraveling.",
      appliesWhen: [
        "The creature evokes cataclysm, final collapse, prophesied ending, or ruin on a society-shaping scale.",
        "A GM would retrieve it for last-days storytelling, omens of collapse, or world-unmaking scenes rather than only for big threat level.",
      ],
      doesNotApplyWhen: [
        "The creature is simply powerful, destructive, or extraplanar without a real end-times or collapse-of-order presentation.",
        "The stronger fit is cosmic_dread or prophecy_omen because existential scale or foretold signs matter more than ruin itself.",
      ],
      adjacentTags: [
        "cosmic_dread",
        "prophecy_omen",
      ],
    },
  }),
  defineConceptProjections("aquatic_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard strongly associated with flooded chambers, rivers, docks, ships, reefs, or underwater spaces.",
    },
  }),
  defineConceptProjections("aquatic_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with open water, underwater spaces, or aquatic environments.",
    },
  }),
  defineConceptProjections("aquatic_support", {
    equipment: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps with swimming, underwater breathing, flotation, water-surface travel, or watercraft use.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
        "armor",
        "shield",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps with swimming, underwater breathing, water-surface travel, or other aquatic movement.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("arcane_spellcaster", {
    creature: {
      axis: "specialization",
      family: "casting_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creature whose spellcasting is substantially framed through arcane traditions, wizardry, runes, or similarly arcane technique.",
    },
  }),
  defineConceptProjections("arctic_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with snow, ice, tundra, or frozen reaches.",
    },
  }),
  defineConceptProjections("area_denial", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Legacy synonym for zone_denial preserved for compatibility while downstream hazard planning surfaces migrate to the simpler area-control vocabulary.",
      adjacentTags: [
        "zone_denial",
        "sentinel_guardian",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("artillery_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Built to pressure targets from range through volleys, spell barrages, breath attacks, or other standoff offense.",
      appliesWhen: [
        "The creature is naturally retrieved as a ranged damage or spell-barrage threat that prefers distance.",
        "Its tactical identity is standoff pressure more than command, mobility, or pure control.",
      ],
      doesNotApplyWhen: [
        "The creature happens to have one ranged option but still fundamentally plays as a brute, controller, or support piece.",
        "Its main retrieval hook is casting support or battlefield control rather than ranged offense.",
      ],
      adjacentTags: [
        "harrier_combatant",
        "controller_combatant",
        "support_combatant",
      ],
    },
  }),
  defineConceptProjections("artisan_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Presented as a smith, craftsperson, builder, artisan, or other maker-facing role-holder tied to production or skilled labor.",
      appliesWhen: [
        "Craft production, repair labor, or skilled making work is central to the creature's world-facing identity.",
        "A GM would plausibly retrieve it as a blacksmith, mason, tailor, shipwright, or similarly maker-facing contact.",
      ],
      doesNotApplyWhen: [
        "The creature only sells goods or manages trade without being defined by making or repair work.",
        "The stronger fit is merchant_npc, profession_npc, or civic_npc without a clear craft-labor role.",
      ],
      adjacentTags: [
        "merchant_npc",
        "profession_npc",
      ],
    },
  }),
  defineConceptProjections("astral_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Astral Plane scenes, silver-void travel, or stable portal routes.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("attrition_pressure", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard whose primary role is to wear the party down over time rather than deliver one decisive spike.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("authority_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Presented as an officer, magistrate, noble, administrator, or other figure of formal social authority.",
      appliesWhen: [
        "The creature's retrieval value comes from official office, rank, command, or institutional authority.",
        "A GM would plausibly seek it as a leader, official, or governing figure.",
        "Formal office or rank is the main retrieval hook, even if the creature also serves as a civic_npc or enforcer_npc in the scene.",
      ],
      doesNotApplyWhen: [
        "The record is only a generic combatant without meaningful office or status.",
        "The stronger fit is profession_npc, civic_npc, or enforcer_npc because status is incidental.",
      ],
      adjacentTags: [
        "profession_npc",
        "civic_npc",
      ],
    },
  }),
  defineConceptProjections("axis_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Axis, the Eternal City, or its lawful planar order.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("barrier_lockdown", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that seals, closes, or blocks passage to trap or delay intruders.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("battlefield_disruption", {
    hazard: {
      axis: "haunt",
      family: "haunt_manifestation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that reshapes the scene with barriers, violent manifestations, or other encounter-disrupting effects.",
    },
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Creates area denial, difficult terrain, barriers, or other battlefield obstacles.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("battlefield_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard strongly associated with siegeworks, trenches, killing grounds, war engines, or other battlefield scenes.",
    },
  }),
  defineConceptProjections("battlefield_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with battlefields, war zones, organized military deployments, or mass-combat scenes.",
    },
  }),
  defineConceptProjections("bestial_transformation", {
    affliction: {
      axis: "disease_model",
      family: "pathogenesis",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Defined by animalistic mutation, feral reshaping, or a cursed slide into beastlike behavior and form.",
      appliesWhen: [
        "The affliction's retrieval value comes from becoming feral, animalistic, lycanthropic, or beast-shaped over time.",
        "Behavioral and bodily slide toward a beast identity are both salient.",
      ],
      doesNotApplyWhen: [
        "The transformation is general corruption without a real animalistic or feral endpoint.",
        "The affliction only compels violent behavior without reshaping the victim's form or identity.",
      ],
      adjacentTags: [
        "transformative_corruption",
        "violence_compulsion",
      ],
    },
  }),
  defineConceptProjections("blight_tainted", {
    creature: {
      axis: "specialization",
      family: "corruption_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creature strongly defined by ecological blight, withering nature, corrupted groves, or land-sickened wilderness.",
      adjacentTags: [
        "wasteland_setting",
        "body_horror",
      ],
    },
  }),
  defineConceptProjections("blood_rot", {
    affliction: {
      axis: "disease_model",
      family: "pathogenesis",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Defined by corrupted blood, blackened veins, hemorrhagic poisoning, or other blood-borne bodily collapse.",
      appliesWhen: [
        "Corrupted blood, blackened veins, or bloodstream collapse is the core thematic identity of the affliction.",
        "A user would retrieve it for blood-plague, hemorrhagic corruption, or vein-darkening disease imagery.",
      ],
      doesNotApplyWhen: [
        "The affliction only causes bleeding as one symptom without a real blood-corruption identity.",
        "The stronger fit is hemorrhagic_failure because the focus is immediate bleeding outcome rather than disease theme.",
      ],
      adjacentTags: [
        "hemorrhagic_failure",
        "physical_debilitation",
      ],
    },
  }),
  defineConceptProjections("body_horror", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with warped anatomy, invasive flesh transformation, surgical grotesquerie, or visceral physical corruption.",
      appliesWhen: [
        "Distorted flesh, invasive alteration, exposed anatomy, or grotesque physical transformation is central to the creature's horror identity.",
        "A GM would retrieve the creature specifically for visceral corruption, mutation, or flesh-warp scenes.",
      ],
      doesNotApplyWhen: [
        "The creature is merely monstrous, bloody, or physically powerful without a strong corruption-of-the-body motif.",
        "The stronger fit is stitched_horror or disease_vector because constructed patchwork or infection aftermath matters more than bodily grotesquerie as presentation.",
      ],
      adjacentTags: [
        "stitched_horror",
        "disease_vector",
      ],
    },
  }),
  defineConceptProjections("boneyard_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Boneyard, psychopomp duties, or soul-processing afterlife scenes.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("bridge_passage_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard strongly associated with bridges, chokepoints, gates, stairwells, or forced passage bottlenecks.",
      appliesWhen: [
        "The hazard is naturally retrieved for narrow crossings, gates, stairwells, or other spaces where intruders must pass through a constrained route.",
        "Forced-passage geometry matters more than a broader dungeon, urban, or wilderness setting identity.",
      ],
      doesNotApplyWhen: [
        "The hazard only happens to sit near a doorway or bridge once without the chokepoint being central to its design.",
        "The stronger fit is threshold_lockdown or forced_movement because the route bottleneck is not the main retrieval hook.",
      ],
      adjacentTags: [
        "threshold_lockdown",
        "forced_movement",
      ],
    },
  }),
  defineConceptProjections("brute_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Built to pressure the front line through durability, direct damage, and straightforward melee threat.",
      appliesWhen: [
        "The creature is naturally retrieved as a heavy hitter that advances, endures punishment, and threatens through direct force.",
        "Its tactical identity is more about sturdy pressure than mobility, command, or precision control.",
      ],
      doesNotApplyWhen: [
        "The creature mainly operates from range, through battlefield control, or through ally support.",
        "High damage is present but the stronger combat identity is ambush, artillery, or commander play.",
      ],
      adjacentTags: [
        "defender_combatant",
        "artillery_combatant",
      ],
    },
  }),
  defineConceptProjections("camp_setup", {
    equipment: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports campsite creation, resting infrastructure, shelter setup, or extended overland staging.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
        "armor",
        "shield",
        "weapon",
      ],
      appliesWhen: [
        "The item is naturally retrieved for making camp, resting outdoors, or supporting expedition downtime.",
        "Its value is in field habitation rather than only carrying gear or feeding travelers.",
      ],
      doesNotApplyWhen: [
        "The item only provides sustenance, mobility, or transport without campsite infrastructure.",
        "The item is merely general survival gear with no setup or shelter-facing role.",
      ],
      adjacentTags: [
        "sustenance",
        "carry_support",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("canyon_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with canyons, gorges, mesas, or badlands.",
    },
  }),
  defineConceptProjections("captive_npc", {
    creature: {
      axis: "npc_role",
      family: "scene_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Immediate-scenario prisoner, hostage, detained witness, sacrifice target, or other constrained figure whose scene value comes from being held or imperiled.",
      appliesWhen: [
        "The creature is naturally retrieved because it is held, imprisoned, restrained, threatened, or otherwise scenically constrained.",
        "Its immediate scenario function is being rescued, questioned, transported, or guarded rather than acting freely in the scene.",
      ],
      doesNotApplyWhen: [
        "The creature is merely vulnerable, socially subordinate, or under pressure without actually being a captive or detained figure.",
        "The stronger fit is civic_npc or profession_npc because social embeddedness or world role matters more than captivity.",
      ],
      adjacentTags: [
        "guardian_npc",
        "civic_npc",
      ],
    },
  }),
  defineConceptProjections("carnival_show", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with carnivals, circuses, clowns, jesters, or sideshow-style presentation.",
    },
  }),
  defineConceptProjections("carrier_vector", {
    affliction: {
      axis: "disease_model",
      family: "epidemiological_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Spread through vermin, insects, bites, stings, or other living disease vectors.",
      appliesWhen: [
        "The affliction's spread is materially tied to rats, mosquitoes, parasites, infected animals, or other living carriers.",
        "Tracing or controlling the living vector is central to understanding the disease.",
      ],
      doesNotApplyWhen: [
        "The disease spreads through air, water, dreams, or contaminated objects without a real living carrier.",
        "Injury transmission happens, but the record does not frame a recurring vector population or host species.",
      ],
      adjacentTags: [
        "injury_exposure",
        "waterborne_exposure",
      ],
    },
  }),
  defineConceptProjections("carry_support", {
    equipment: {
      axis: "utility",
      family: "carry_logistics",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps stow, carry, or organize equipment.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("caster_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to spellcasters for magical prep, casting reliability, spell defense, or spell-adjacent utility.",
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
      adjacentTags: [
        "profession_npc",
        "enforcer_npc",
      ],
    },
  }),
  defineConceptProjections("climbing", {
    equipment: {
      axis: "utility",
      family: "movement_traversal",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps climb, rappel, or navigate vertical obstacles.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("coastal_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with coasts, shores, reefs, or littoral edges.",
    },
  }),
  defineConceptProjections("cognitive_impairment", {
    affliction: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Dulls thought, memory, decision-making, or mental clarity without being just fear or confusion.",
    },
  }),
  defineConceptProjections("combat_maneuver_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to builds that rely on grappling, tripping, shoving, disarming, or other combat maneuvers to control enemies.",
    },
  }),
  defineConceptProjections("commander_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Built to coordinate allies round to round through leadership, tactics, or command-driven positioning.",
      appliesWhen: [
        "The creature is naturally retrieved because its round-to-round battle role is directing allies, calling tactics, or coordinating a group.",
        "Leadership and coordination matter more than its own solo offense, brute durability, or passive aura support.",
      ],
      doesNotApplyWhen: [
        "The creature only has generic support effects with no clear command or tactical leadership role in the moment-to-moment fight.",
        "The stronger fit is support_combatant, artillery_combatant, or reinforcement_threat rather than leader play.",
      ],
      adjacentTags: [
        "support_combatant",
        "reinforcement_threat",
      ],
    },
  }),
  defineConceptProjections("community_outbreak", {
    affliction: {
      axis: "disease_model",
      family: "epidemiological_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Framed around camp-, village-, ship-, monastery-, or settlement-scale spread rather than one isolated victim.",
      appliesWhen: [
        "The affliction is naturally retrieved as a spreading local crisis affecting a community, camp, ship, or institution.",
        "Outbreak management, quarantine pressure, or multi-victim spread matters more than a single infected host.",
      ],
      doesNotApplyWhen: [
        "The affliction remains a one-target curse or isolated infection without wider spread framing.",
        "The stronger fit is epidemic_pestilence only because of plague flavor, but community-scale spread is not actually present.",
      ],
      adjacentTags: [
        "epidemic_pestilence",
        "carrier_vector",
      ],
    },
  }),
  defineConceptProjections("companion_handling_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable when a character's tactical workflow depends on commanding, outfitting, transporting, or protecting companions and mounts.",
    },
  }),
  defineConceptProjections("companion_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to handlers of animal companions, mounts, familiars, or other creature-side support play patterns.",
    },
  }),
  defineConceptProjections("compulsion", {
    affliction: {
      axis: "behavior",
      family: "behavioral_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Overrides agency through commanded behavior, forced truth-telling, or similarly scripted actions.",
    },
  }),
  defineConceptProjections("concealable", {
    equipment: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Easy to hide on the person or carry discreetly.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("concealment", {
    equipment: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps obscure a creature, item, or area from sight or make it harder to perceive.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Makes a creature hard to see, hidden, concealed, or undetected.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("contact_exposure", {
    affliction: {
      axis: "disease_model",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Spread by touch, skin contact, slime, ooze, or another surface-level transfer mechanism.",
    },
  }),
  defineConceptProjections("contact_offense", {
    equipment: {
      axis: "effect",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Offensive consumable delivered through touch or skin contact.",
      subcategories: [
        "consumable",
      ],
    },
  }),
  defineConceptProjections("control_interface", {
    hazard: {
      axis: "mechanism",
      family: "mechanism",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard operated through a button, lever, console, panel, switch, or similar control surface.",
    },
  }),
  defineConceptProjections("controller_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Built to reshape the battlefield through debuffs, forced movement, terrain control, or other tactical denial.",
      appliesWhen: [
        "The creature is naturally retrieved for immobilizing, repositioning, walling off, slowing, or otherwise dictating battlefield shape.",
        "Tactical denial matters more than direct damage output.",
      ],
      doesNotApplyWhen: [
        "The creature only has one incidental debuff while otherwise fighting as a brute, defender, or artillery piece.",
        "The stronger identity is support or commander rather than enemy-space control.",
      ],
      adjacentTags: [
        "support_combatant",
        "artillery_combatant",
      ],
    },
  }),
  defineConceptProjections("corrupted_sacred", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with profaned sanctity, fallen holiness, blasphemous devotion, or sacred imagery twisted into menace.",
      appliesWhen: [
        "The creature's presentation depends on desecrated ritual, fallen holiness, saintly imagery gone wrong, or sanctity turned threatening.",
        "Retrieval value comes from sacred symbolism being violated, inverted, or corrupted.",
      ],
      doesNotApplyWhen: [
        "The creature is merely evil, undead, or hostile in a temple without sacred corruption being central to its identity.",
        "The stronger fit is ritual_ceremony or religious_npc because the creature is ceremonial or clerical without profaned sanctity.",
      ],
      adjacentTags: [
        "ritual_ceremony",
        "religious_npc",
      ],
    },
  }),
  defineConceptProjections("cosmic_dread", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with void vastness, star-born dread, incomprehensible revelation, or insignificance before the cosmos.",
      appliesWhen: [
        "The creature evokes existential terror, starry abyssal scale, unknowable revelation, or the feeling of minds breaking before the universe.",
        "A GM would plausibly retrieve it for eldritch omen, cosmic terror, or revelation-of-the-void scenes.",
      ],
      doesNotApplyWhen: [
        "The creature is merely alien, aberrant, or extraplanar without a strong cosmic-horror presentation.",
        "Astral or outer-planar placement alone is better captured by setting tags.",
      ],
      adjacentTags: [
        "astral_setting",
        "dream_nightmare",
      ],
    },
  }),
  defineConceptProjections("court_entourage", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Naturally retrieved as part of a courtly retinue, noble household, ceremonial train, or political chamber roster around a central authority figure.",
      adjacentTags: [
        "authority_npc",
        "courtly_pageantry",
      ],
    },
  }),
  defineConceptProjections("courtly_pageantry", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with nobles, heraldry, formal spectacle, masquerade grandeur, or ceremonial court presentation.",
      appliesWhen: [
        "Court spectacle, heraldic pomp, ballroom tension, formal procession, or aristocratic display is central to the creature's presentation.",
        "A GM would plausibly retrieve the creature for palace intrigue, masquerades, or noble ceremonial scenes rather than for office alone.",
      ],
      doesNotApplyWhen: [
        "The creature merely holds rank or authority without strong pageantry, splendor, or court-scene presentation.",
        "The stronger fit is authority_npc or performer_npc rather than courtly spectacle.",
      ],
      adjacentTags: [
        "authority_npc",
        "mask_motif",
      ],
    },
  }),
  defineConceptProjections("creature_bane", {
    equipment: {
      axis: "item_mechanical",
      family: "ammunition_payload",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Tailored ammunition for a selected creature type or trait.",
      subcategories: [
        "ammo",
      ],
    },
  }),
  defineConceptProjections("crew_member", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Naturally retrieved as part of a ship crew, dock crew, wreck complement, or other nautical working roster that belongs together in one scene.",
      adjacentTags: [
        "nautical_setting",
        "escort_npc",
      ],
    },
  }),
  defineConceptProjections("criminal_cell", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Naturally retrieved as part of a gang, smuggling ring, burglary team, or other underworld roster that functions through a small coordinated cell.",
      adjacentTags: [
        "criminal_npc",
        "infiltrator_npc",
      ],
    },
  }),
  defineConceptProjections("criminal_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Presented as a thief, smuggler, assassin, gang operative, fence, or other explicitly underworld-coded role.",
      appliesWhen: [
        "The creature is defined by illicit trade, organized crime, covert violence, or other criminal social function.",
        "A GM would retrieve it as an underworld contact or criminal adversary rather than a generic soldier.",
      ],
      doesNotApplyWhen: [
        "The creature is merely hostile without underworld or crime-scene framing.",
        "The stronger fit is enforcer_npc without a distinct criminal identity.",
      ],
      adjacentTags: [
        "enforcer_npc",
        "infiltrator_npc",
      ],
    },
  }),
  defineConceptProjections("cult_member", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Naturally retrieved as one member of a ritual circle, hidden sect, temple conspiracy, or other cultic roster built around shared devotion or doctrine.",
      adjacentTags: [
        "religious_npc",
        "ritualist_creature",
      ],
    },
  }),
  defineConceptProjections("cumulative_transformation", {
    affliction: {
      axis: "disease_model",
      family: "progression_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Affliction whose stages progressively rewrite the victim into a visibly altered or corrupted form.",
      appliesWhen: [
        "The stage-by-stage march toward a changed body is a major reason to retrieve the affliction.",
        "Transformation is progressive and cumulative rather than an instantaneous one-step effect.",
      ],
      doesNotApplyWhen: [
        "The affliction only imposes one stable transformed state without escalation.",
        "The stronger fit is delayed_onset, terminal_collapse, or a more specific pathogenesis tag.",
      ],
      adjacentTags: [
        "transformative_corruption",
        "petrifying_corruption",
      ],
    },
  }),
  defineConceptProjections("curse_marking", {
    affliction: {
      axis: "metaphysical",
      family: "metaphysical_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Brands the victim with a curse mark, doom sign, inherited hex, or similarly explicit supernatural stigma.",
      appliesWhen: [
        "The affliction leaves an explicit mark, sign, sigil, stain, or named curse-brand that matters to its identity.",
        "A user would retrieve it for visible or narratively explicit cursed marking, not just soul damage.",
      ],
      doesNotApplyWhen: [
        "The affliction is metaphysical but has no distinct branded or marked stigma.",
        "The stronger fit is void_soul_corruption or soul_binding without a visible curse sign.",
      ],
      adjacentTags: [
        "void_soul_corruption",
        "soul_binding",
      ],
    },
  }),
  defineConceptProjections("cursed_transformation", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with involuntary metamorphosis, curse-driven loss of self, or identity eroded by becoming something else.",
      appliesWhen: [
        "The creature's story identity centers on an afflicting change, monstrous becoming, or transformation that threatens personhood.",
        "A GM would retrieve it for cursed metamorphosis, bestial change, or body-and-identity corruption rather than only raw mutation.",
      ],
      doesNotApplyWhen: [
        "The creature merely transforms, shapeshifts, or mutates as an ability without cursed or tragic transformation being the presentation hook.",
        "The stronger fit is body_horror or disguised_pretender because physical grotesquerie or impersonation matters more than involuntary becoming.",
      ],
      adjacentTags: [
        "body_horror",
        "ancestral_legacy",
      ],
    },
  }),
  defineConceptProjections("cursewarped", {
    creature: {
      axis: "specialization",
      family: "corruption_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creature strongly defined by curse-driven distortion, doom-warping, or being transformed into its current state by a curse.",
      adjacentTags: [
        "curse_threat",
        "cursed_transformation",
      ],
    },
  }),
  defineConceptProjections("darklands_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Darklands as a civilization-bearing underworld macro-region rather than generic underground terrain or one cave network.",
    },
  }),
  defineConceptProjections("decadence_decline", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with faded luxury, aristocratic rot, opulent ruin, or beauty collapsing into moral and material decay.",
      appliesWhen: [
        "The creature evokes indulgent splendor gone rotten, noble decline, decadent excess, or crumbling beauty concealing corruption.",
        "A GM would plausibly retrieve it for decaying courts, ruined salons, or luxury-turned-horror scenes.",
      ],
      doesNotApplyWhen: [
        "The creature is merely rich, noble, or associated with ruins without a clear decline-and-rot presentation.",
        "The stronger fit is courtly_pageantry or revelry_excess because spectacle or celebration matters more than decay.",
      ],
      adjacentTags: [
        "courtly_pageantry",
        "revelry_excess",
      ],
    },
  }),
  defineConceptProjections("defender_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Built to hold space, intercept attacks, bodyguard allies, or punish passage through a defended line.",
      appliesWhen: [
        "The creature is naturally retrieved for guarding chokepoints, bodyguarding allies, or making passage costly.",
        "Space-holding matters more than raw pursuit, burst damage, or command.",
      ],
      doesNotApplyWhen: [
        "The creature is merely durable without a real guard, intercept, or line-holding identity.",
        "The stronger fit is brute_combatant or another non-guardian role.",
      ],
      adjacentTags: [
        "brute_combatant",
        "commander_combatant",
      ],
    },
  }),
  defineConceptProjections("defender_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to shield users, bodyguards, line-holders, or other defender-style characters.",
    },
  }),
  defineConceptProjections("delayed_onset", {
    affliction: {
      axis: "disease_model",
      family: "progression_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Affliction whose symptoms, danger, or full transformation emerge after a notable delay rather than immediately.",
      appliesWhen: [
        "The affliction is naturally retrieved because the real danger appears after an incubation period, hidden delay, or deceptively quiet initial stage.",
        "The timing gap between exposure and serious consequence matters to prep, diagnosis, or quarantine decisions.",
      ],
      doesNotApplyWhen: [
        "The affliction starts harming victims right away even if it worsens later.",
        "The stronger fit is recurrent_flare or cumulative_transformation because the main hook is cycling or progressive change rather than delayed emergence.",
      ],
      adjacentTags: [
        "recurrent_flare",
        "cumulative_transformation",
      ],
    },
  }),
  defineConceptProjections("demonic_scar_frontier_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with demon-scarred frontiers, reclaimed homelands, abyss-tainted wilderness, and recovery after planar catastrophe. In Golarion, this primarily corresponds to the Sarkoris Scar.",
      appliesWhen: [
        "Use when the creature is naturally retrieved for post-invasion frontier planning: planar scars, reclaimed sacred sites, demon-torn wastelands, lingering abyssal corruption, or survival after a world-rending incursion.",
        "The planning value comes from a land still shaped by demonic devastation and reclamation rather than from generic fiendish wilderness or one demonic trait.",
      ],
      doesNotApplyWhen: [
        "The creature is merely demonic, corrupted, or extraplanar without a real frontier-of-reclamation regional frame.",
        "The stronger fit is planar_setting or corruption_profile because the retrieval hook is cosmological origin or body-horror taint rather than a scarred regional frontier.",
      ],
      adjacentTags: [
        "battlefield_setting",
        "void_tainted",
      ],
    },
  }),
  defineConceptProjections("desert_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with dunes, sand, or arid wastes.",
    },
  }),
  defineConceptProjections("disguise", {
    equipment: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps alter appearance or impersonate another identity.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps alter appearance or impersonate another identity.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("disguised_pretender", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with false identities, impersonation, infiltration, shapeshifting, or replacement.",
      appliesWhen: [
        "Impersonation, replacement, disguise, or false identity is a central retrieval hook.",
        "The creature is framed around passing as someone or something else.",
      ],
      doesNotApplyWhen: [
        "It merely uses stealth or deception without identity substitution.",
        "The stronger presentation tag is mask_motif or faceless_horror rather than impersonation.",
      ],
      adjacentTags: [
        "faceless_horror",
        "mask_motif",
      ],
    },
  }),
  defineConceptProjections("divine_spellcaster", {
    creature: {
      axis: "specialization",
      family: "casting_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creature whose spellcasting is substantially framed through divine prayer, sacred miracles, or deity-facing magic.",
    },
  }),
  defineConceptProjections("dragon_spellcaster", {
    creature: {
      axis: "specialization",
      family: "casting_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Dragon or archdragon variant with an explicit spellcaster stat block or named spellcaster presentation.",
      appliesWhen: [
        "Use when a dragon or archdragon variant explicitly presents meaningful spellcasting as part of its encounter identity.",
        "The spellcasting matters for prep and counterplay beyond incidental magical flavor.",
      ],
      doesNotApplyWhen: [
        "The dragon only has innate magical flavor, one-off magical actions, or a few utility effects without real spellcaster framing.",
        "The stronger fit is only a tradition-specific spellcaster tag without a dragon-specific spellcaster presentation.",
      ],
      adjacentTags: [
        "arcane_spellcaster",
        "ritualist_creature",
      ],
    },
  }),
  defineConceptProjections("dream_nightmare", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with dreams, nightmares, sleep-haunting, surreal unreality, or subconscious dread.",
      appliesWhen: [
        "Dream logic, nightmare intrusion, sleep visitation, or surreal unreality is central to the creature's story identity.",
        "A GM would plausibly retrieve the creature for dreamscapes, night terrors, or oneiric scenes even outside a literal Dreamlands setting.",
      ],
      doesNotApplyWhen: [
        "The creature merely casts sleep or fear effects without a real dream or nightmare presentation theme.",
        "Dreamlands placement alone is better captured by dreamlands_setting.",
      ],
      adjacentTags: [
        "dreamlands_setting",
        "cosmic_dread",
      ],
    },
  }),
  defineConceptProjections("dreamborne_exposure", {
    affliction: {
      axis: "disease_model",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Spread through sleep, nightmares, dreaming contact, or other oneiric pathways.",
      appliesWhen: [
        "The affliction is contracted or transmitted through dreams, sleep, shared nightmares, or oneiric contact.",
        "Dream-state exposure is central to how the condition reaches the victim.",
      ],
      doesNotApplyWhen: [
        "The affliction only causes nightmares after infection but is actually spread by wounds, air, or cursed objects.",
        "The stronger fit is nightmare_torment because the symptom profile matters more than the exposure path.",
      ],
      adjacentTags: [
        "nightmare_torment",
        "inhaled_exposure",
      ],
    },
  }),
  defineConceptProjections("dreamlands_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Dreamlands, Leng-linked dream roads, or iconic denizens that dwell there.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("dungeon_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard strongly associated with dungeon corridors, chambers, trapped passages, or underground complexes.",
      appliesWhen: [
        "The hazard is naturally retrieved as a classic corridor, chamber, or trap-complex defense in a dungeon environment.",
        "Its encounter identity is more about underground built-space adventuring than a narrower tomb or temple context.",
      ],
      doesNotApplyWhen: [
        "The hazard is more specifically tomb-, temple-, bridge-, or aquatic-coded than generic dungeon-coded.",
        "The hazard merely happens to appear indoors once without dungeon-like scene identity.",
      ],
      adjacentTags: [
        "tomb_hazard",
        "temple_hazard",
      ],
    },
  }),
  defineConceptProjections("elemental_payload", {
    equipment: {
      axis: "item_mechanical",
      family: "ammunition_payload",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Ammunition that delivers an elemental or reagent-based payload on impact.",
      subcategories: [
        "ammo",
      ],
    },
  }),
  defineConceptProjections("elysium_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Elysium, azatas, or freedom-driven celestial heroism.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("emergency_recovery", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable as a panic-button item for sudden healing, escape, stabilization, or critical condition rescue.",
    },
  }),
  defineConceptProjections("endurance_pressure", {
    hazard: {
      axis: "problem",
      family: "problem_shape",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard whose main prep problem is surviving repeated exposure long enough to finish the scene rather than landing one clean solve immediately.",
    },
  }),
  defineConceptProjections("energy_resistance", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Grants resistance against one or more energy types.",
      subcategories: [
        "consumable",
      ],
    },
  }),
  defineConceptProjections("enforcer_npc", {
    creature: {
      axis: "npc_role",
      family: "scene_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Scene-slot fight-first humanoid adversary such as a soldier, bandit, mercenary, or other overt martial enforcer.",
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
      adjacentTags: [
        "authority_npc",
        "civic_npc",
      ],
    },
  }),
  defineConceptProjections("environmental_adaptation", {
    equipment: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps travelers endure extreme weather, thin air, smoke, pressure, or other dangerous environmental exposure.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
        "armor",
        "shield",
        "weapon",
      ],
      appliesWhen: [
        "The item's retrieval value comes from surviving punishing climate, altitude, breathing hazards, immersion pressure, or similar expedition environments.",
        "It is naturally sought as environmental survival gear rather than a general defense item or campsite tool.",
      ],
      doesNotApplyWhen: [
        "The item only protects against one incoming attack or hazard burst without broader travel-survival use.",
        "The item mainly creates camp infrastructure, carries provisions, or improves aquatic movement instead of adapting the user to the environment.",
      ],
      adjacentTags: [
        "aquatic_support",
        "camp_setup",
        "hazard_shielding",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Helps creatures endure hostile climates, thin air, smoke, pressure, vacuum, or other expedition-grade environmental extremes.",
      appliesWhen: [
        "The spell is naturally retrieved to survive extreme heat, cold, altitude, smoke, pressure, or other punishing environmental conditions during travel or exploration.",
        "Environmental endurance matters more than only resisting one attack form or creating a place to rest.",
      ],
      doesNotApplyWhen: [
        "The spell mainly grants combat resistance, a protective ward, or aquatic mobility without broader expedition-survival value.",
        "The spell only creates shelter or sustenance rather than adapting creatures to the surrounding environment.",
      ],
      adjacentTags: [
        "aquatic_support",
        "field_shelter",
        "resistance_support",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("epidemic_pestilence", {
    affliction: {
      axis: "disease_model",
      family: "epidemiological_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "A named plague-, fever-, pox-, or pestilence-style disease with explicit outbreak or contagion framing.",
    },
  }),
  defineConceptProjections("escort_npc", {
    creature: {
      axis: "npc_role",
      family: "scene_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Immediate-scenario escort, courier companion, guide-on-mission, ward mover, or other figure whose scene value is accompanying or moving someone through danger.",
      appliesWhen: [
        "The creature is naturally retrieved because it actively escorts, transports, shepherds, or accompanies another figure through a dangerous scene or route.",
        "Its scene function is movement-with-charge or safe transit rather than only office, profession, or posted guard duty.",
      ],
      doesNotApplyWhen: [
        "The creature merely knows the route, provides directions, or protects a place without actively accompanying someone.",
        "The stronger fit is guide_npc or guardian_npc because route expertise or posted guard duty matters more than accompaniment.",
      ],
      adjacentTags: [
        "guide_npc",
        "guardian_npc",
      ],
    },
  }),
  defineConceptProjections("ethereal_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Ethereal Plane, its native wildlife, or recurring hunting and travel routes through it.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("explosive_payload", {
    equipment: {
      axis: "item_mechanical",
      family: "ammunition_payload",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Ammunition that detonates or scatters area damage on impact.",
      subcategories: [
        "ammo",
      ],
    },
  }),
  defineConceptProjections("extraction_teleport", {
    spell: {
      axis: "utility",
      family: "teleportation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Teleports a creature out of danger, through restraints, or away from immediate threat pressure.",
      appliesWhen: [
        "The spell is naturally retrieved as an escape, rescue, or anti-capture tool rather than only a movement spell.",
        "The reposition breaks danger, confinement, or immediate battlefield pressure.",
      ],
      doesNotApplyWhen: [
        "The spell is mostly a neutral short-range blink or a long-distance travel effect.",
        "The spell primarily opens planar movement rather than emergency extraction.",
      ],
      adjacentTags: [
        "short_range_teleport",
        "escape_support",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("extradimensional_storage", {
    equipment: {
      axis: "item_mechanical",
      family: "access_system",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Provides bag-of-holding-style storage through extradimensional or magically expanded space.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "ammo",
        "armor",
        "weapon",
      ],
    },
  }),
  defineConceptProjections("face_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to negotiators, deceivers, diplomats, or other socially forward characters who need influence, disguise, or presentation support.",
    },
  }),
  defineConceptProjections("faceless_horror", {
    creature: {
      axis: "presentation",
      family: "visual_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with missing, hidden, stolen, or featureless faces.",
      appliesWhen: [
        "Missing, stolen, hidden, or featureless faces are central to the creature's horror identity.",
        "Face absence or erasure is a recurring motif, not just a disguise tactic.",
      ],
      doesNotApplyWhen: [
        "The creature is mainly about impersonation or disguise rather than facial absence.",
        "A covered face appears only as costume or gear.",
      ],
      adjacentTags: [
        "disguised_pretender",
        "mask_motif",
      ],
    },
  }),
  defineConceptProjections("fall_protection", {
    equipment: {
      axis: "item_mechanical",
      family: "defense_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Reduces falling harm, cushions impact, or protects against vertical movement accidents and collapse.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "shield",
      ],
    },
  }),
  defineConceptProjections("field_shelter", {
    spell: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creates shelter, refuge, or a protected resting place in the field.",
      appliesWhen: [
        "The spell is naturally retrieved to create a campsite refuge, safe resting place, or expedition shelter in hostile territory.",
        "Its value is prolonged field habitation or protected rest rather than momentary combat defense.",
      ],
      doesNotApplyWhen: [
        "The spell only creates a brief combat ward, cover effect, or instant defensive barrier.",
        "The spell merely transports creatures away instead of establishing a place to rest.",
      ],
      adjacentTags: [
        "protective_ward",
        "planar_travel",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("first_world_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the First World, fey realms, or thin-boundary crossings into that plane.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("flight", {
    spell: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Grants flying movement, sustained aerial travel, or practical airborne maneuvering.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("focus_magic_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to focus-spell-heavy or magic-routine-driven builds that want more reliable magical cadence and recovery.",
    },
  }),
  defineConceptProjections("folk_horror", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with rural superstition, old customs, harvest dread, village taboos, or uncanny folklore menace.",
      appliesWhen: [
        "The creature evokes old-country fear, harvest rites gone wrong, scarecrow dread, witchcraft omen, or taboo-laden local folklore.",
        "Its retrieval value comes from uncanny communal belief and traditional dread, not only from being outdoors or rural.",
      ],
      doesNotApplyWhen: [
        "The creature is merely found in fields, forests, or villages without a real folklore or superstition-facing motif.",
        "The stronger fit is only rural_setting, swamp_setting, or another location tag.",
      ],
      adjacentTags: [
        "rural_setting",
        "funerary_mourning",
      ],
    },
  }),
  defineConceptProjections("forbidden_knowledge", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with taboo lore, dangerous revelation, blasphemous truth, or learning that should not be uncovered.",
      appliesWhen: [
        "The creature is framed around hidden texts, proscribed secrets, mind-breaking truths, or knowledge pursued at terrible cost.",
        "Retrieval value comes from the danger of revelation, not merely from scholarship or intelligence.",
      ],
      doesNotApplyWhen: [
        "The creature is simply smart, scholarly, occult, or mysterious without dangerous-knowledge presentation being central.",
        "The stronger fit is occult_conspiracy or cosmic_dread because hidden cabals or existential terror matters more than taboo learning itself.",
      ],
      adjacentTags: [
        "occult_conspiracy",
        "cosmic_dread",
      ],
    },
  }),
  defineConceptProjections("forced_separation", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that splits allies apart through walls, drops, slides, teleports, or other positional disruption.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("forced_separation_hazard", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Legacy synonym for forced_separation preserved for compatibility while downstream surfaces migrate to the simpler split-party vocabulary.",
      adjacentTags: [
        "forced_separation",
        "pursuit_punisher",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("forest_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with forests, jungles, groves, or briar-choked wilds.",
    },
  }),
  defineConceptProjections("forgery_support", {
    equipment: {
      axis: "utility",
      family: "crafting_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports document falsification, seal imitation, signature copying, or bureaucratic deception.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "consumable",
      ],
      appliesWhen: [
        "The item's retrieval value comes from imitating documents, seals, credentials, or official paperwork.",
        "It supports passing administrative scrutiny rather than just changing clothing or physical appearance.",
      ],
      doesNotApplyWhen: [
        "The item only changes appearance or supports social disguise without document work.",
        "The item is a normal writing or archival tool with no deception-facing use.",
      ],
      adjacentTags: [
        "disguise",
        "writing_recordkeeping",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("fortress_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with castles, fortresses, citadels, watchtowers, or other fortified encounter sites.",
      appliesWhen: [
        "Fortified structures are part of the creature's recurring encounter identity.",
        "The creature is framed around castles, citadels, watchtowers, keeps, or defensive strongholds.",
      ],
      doesNotApplyWhen: [
        "The record only uses a fortress as a one-off location.",
        "The broader identity is urban_setting or temple_setting rather than fortified-site specific.",
      ],
      adjacentTags: [
        "urban_setting",
        "temple_setting",
      ],
    },
  }),
  defineConceptProjections("freshwater_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with rivers, lakes, ponds, streams, springs, or other inland waters.",
    },
  }),
  defineConceptProjections("funerary_mourning", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with grief, funeral rites, mourning processions, memorial haunting, or death-ritual solemnity.",
      appliesWhen: [
        "Funeral symbolism, grief-haunting, mourning observance, or memorial ritual is central to the creature's presentation.",
        "The creature is naturally retrieved for funerary scenes, dirges, wakes, or death-ritual storytelling rather than only because it is undead.",
      ],
      doesNotApplyWhen: [
        "The creature is merely undead, ghostly, or graveyard-linked without a meaningful mourning or funerary presentation.",
        "The stronger fit is graveyard_setting, boneyard_setting, or undead_adjacent without a real ritualized grief presentation.",
      ],
      adjacentTags: [
        "ritual_ceremony",
        "mask_motif",
      ],
    },
  }),
  defineConceptProjections("fungal_growth", {
    affliction: {
      axis: "disease_model",
      family: "pathogenesis",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Defined by fungal bloom, spores, mycelial takeover, or mushroom-like growths spreading through the body.",
    },
  }),
  defineConceptProjections("fungal_infested", {
    creature: {
      axis: "specialization",
      family: "corruption_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creature strongly defined by mycelium, spores, mushroom overgrowth, or fungus-driven bodily infestation.",
      adjacentTags: [
        "disease_vector",
        "body_horror",
      ],
    },
  }),
  defineConceptProjections("gothic_horror_land_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Gothic-horror lands of mist, superstition, graveyard dread, cursed nobility, and classic night monsters. In Golarion, this primarily corresponds to Ustalav.",
      appliesWhen: [
        "Use when the creature is naturally retrieved for Gothic-horror encounter planning: decaying manors, haunted villages, graveyard menace, superstitious communities, or classic monster-fiction mood.",
        "The planning value comes from atmospheric horror identity, cursed lineage, or old-world dread rather than only from one undead, fiend, or shapeshifter trait.",
      ],
      doesNotApplyWhen: [
        "The creature is simply spooky, undead, or cursed without a real Gothic-horror social or atmospheric frame.",
        "The stronger fit is story_motif or genre_motif because the retrieval hook is a narrower narrative motif rather than a full regional horror lens.",
      ],
      adjacentTags: [
        "graveyard_setting",
        "folk_horror",
      ],
    },
  }),
  defineConceptProjections("graveyard_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with cemeteries, tombs, barrows, or burial grounds.",
    },
  }),
  defineConceptProjections("guard_post", {
    hazard: {
      tag: "sentinel_guardian",
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard whose role is to guard an area, treasure, threshold, or sanctum as a standing defense layer.",
      appliesWhen: [
        "The hazard is naturally retrieved as a guardian layer protecting a place, object, or route from intrusion.",
        "Its value is in persistent watchfulness or defensive coverage, not just burst damage.",
      ],
      doesNotApplyWhen: [
        "The hazard mainly creates open-area denial with no strong guard-post identity.",
        "The hazard is mostly an ambush opener or chase-punishment device.",
      ],
      adjacentTags: [
        "alarm",
        "zone_denial",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("guardian_npc", {
    creature: {
      axis: "npc_role",
      family: "scene_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Immediate-scenario guard, jailer, doorkeeper, bodyguard, or other posted protector whose scene value is physically holding or protecting a person, threshold, or place.",
      appliesWhen: [
        "The creature is naturally retrieved because it is posted to guard, hold, jail, or protect a specific person, threshold, route, or space.",
        "Posted protection or interdiction matters more than general combat readiness, surveillance, or broad social authority.",
      ],
      doesNotApplyWhen: [
        "The creature is merely combat-ready without a clear posted protection, bodyguard, or threshold-holding duty.",
        "The stronger fit is authority_npc, enforcer_npc, or watcher_npc because command status, generic martial opposition, or alarm duty matters more than guarding.",
      ],
      adjacentTags: [
        "enforcer_npc",
        "watcher_npc",
      ],
    },
  }),
  defineConceptProjections("guardian_retinue", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Naturally retrieved as one member of a posted defense roster around a leader, relic, sanctum, or protected threshold rather than as an independent threat.",
      adjacentTags: [
        "guardian_npc",
        "defender_combatant",
      ],
    },
  }),
  defineConceptProjections("guide_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Presented as a scout, tracker, ferryman, caravan guide, wilderness pathfinder, or other route-leading specialist.",
      appliesWhen: [
        "Leading others through terrain, routes, borders, or dangerous travel spaces is central to the creature's world-facing identity.",
        "The record is naturally used as a scout, pathfinder, navigator, or local guide rather than only a generic outdoors person.",
      ],
      doesNotApplyWhen: [
        "The creature merely knows the area or has survival competence without a role-defined guiding function.",
        "The stronger fit is scholar_npc, civic_npc, or enforcer_npc rather than travel-leading expertise.",
      ],
      adjacentTags: [
        "profession_npc",
        "rural_setting",
      ],
    },
  }),
  defineConceptProjections("harrier_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Built to chip away from safety through repeated ranged harassment, flyby pressure, or evasive standoff attacks that force pursuit.",
      adjacentTags: [
        "skirmisher_combatant",
        "artillery_combatant",
      ],
    },
  }),
  defineConceptProjections("hazard_shielding", {
    equipment: {
      axis: "item_mechanical",
      family: "defense_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Protects against environmental hazards, area effects, or other damaging exposures.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "shield",
      ],
      appliesWhen: [
        "The item is naturally retrieved for surviving traps, breath weapons, alchemical blasts, or other hazardous exposure.",
        "Protection against scenes and effects matters more than only deflecting direct weapon attacks.",
      ],
      doesNotApplyWhen: [
        "The item's main value is cover against arrows or weapon strikes rather than wider hazard exposure.",
        "The item mainly protects against spells specifically, making magic_protection the stronger hook.",
      ],
      adjacentTags: [
        "projectile_defense",
        "magic_protection",
      ],
    },
  }),
  defineConceptProjections("healer_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Presented as a physician, battlefield medic, herbalist, chirurgeon, caretaker, or other explicitly healing-facing role-holder.",
      appliesWhen: [
        "Medical treatment, recovery support, or caretaker duty is central to the creature's world-facing identity.",
        "The record is naturally used as a healer, medic, apothecary, or restorative support NPC rather than only a generic scholar.",
      ],
      doesNotApplyWhen: [
        "The creature merely has healing magic or restorative abilities without a role-defined caregiving identity.",
        "The stronger fit is scholar_npc, religious_npc, or civic_npc without a real healer-facing job.",
      ],
      adjacentTags: [
        "scholar_npc",
        "religious_npc",
      ],
    },
  }),
  defineConceptProjections("healer_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to medics, battlefield healers, or builds expected to stabilize, treat, or recover allies under pressure.",
    },
  }),
  defineConceptProjections("healing_suppression", {
    affliction: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Prevents normal healing or sharply reduces healing received.",
    },
  }),
  defineConceptProjections("heaven_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Heaven, archon hosts, or ordered celestial service.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("hell_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Hell, devil hosts, or infernal hierarchy.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("hemorrhagic_failure", {
    affliction: {
      axis: "effect",
      family: "physiology_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Causes uncontrolled bleeding, blood loss, vessel rupture, or similar collapse of the body's circulatory integrity.",
      appliesWhen: [
        "The affliction is naturally retrieved because bleeding, blood loss, ruptured vessels, or circulatory collapse are major consequences rather than incidental symptoms.",
        "The bodily failure pattern matters more than the source theme of the disease.",
      ],
      doesNotApplyWhen: [
        "The affliction only references corrupted blood as flavor without major bleeding or circulatory breakdown consequences.",
        "The stronger fit is blood_rot or physical_debilitation because hemorrhage is not the core downstream effect.",
      ],
      adjacentTags: [
        "blood_rot",
        "physical_debilitation",
      ],
    },
  }),
  defineConceptProjections("illumination", {
    equipment: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Produces or improves light in dark environments.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "sensory_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Produces practical light that brightens darkness, reveals an area, or lets creatures see more clearly.",
      adjacentTags: [
        "senses_support",
        "line_of_sight_control",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("industrial_grotesque", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with smoke, gears, furnaces, exploitation, mutilating machinery, or dehumanizing industrial corruption.",
      appliesWhen: [
        "The creature evokes factory horror, machine-maimed labor, furnace dread, or industrial systems turning flesh and society into raw material.",
        "Retrieval value comes from industrial corruption and mechanized degradation, not just from being a construct or urban creature.",
      ],
      doesNotApplyWhen: [
        "The creature merely uses technology, lives in a city, or is a construct without industrial corruption or machine-horror presentation.",
        "The stronger fit is body_horror or urban_setting because visceral corruption or city placement matters more than industrial atmosphere.",
      ],
      adjacentTags: [
        "body_horror",
        "urban_setting",
      ],
    },
  }),
  defineConceptProjections("infestation_implant", {
    affliction: {
      axis: "disease_model",
      family: "pathogenesis",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Defined by eggs, larvae, spores, parasites, or other host-colonizing implantation.",
    },
  }),
  defineConceptProjections("infestation_member", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Naturally retrieved as one body in a swarm-like infestation, burrowing colony, parasite outbreak, or other many-body nuisance roster.",
      adjacentTags: [
        "parasite_ridden",
        "plaguebearing",
      ],
    },
  }),
  defineConceptProjections("infiltrator_npc", {
    creature: {
      axis: "npc_role",
      family: "scene_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Scene-slot spy, saboteur, replacer, or quiet-entry specialist whose immediate scenario value comes from infiltration more than a straight fight.",
      appliesWhen: [
        "The creature is naturally retrieved as an embedded infiltrator, saboteur, impostor, spy, or quiet-entry specialist.",
        "This tag answers the creature's immediate scenario function rather than its broader profession, faction post, or criminal affiliation.",
      ],
      doesNotApplyWhen: [
        "The creature is mainly an enforcer_npc or civic_npc and only uses stealth or deception incidentally.",
        "The stronger retrieval hook is criminal_npc or another social_role tag because the world-facing identity matters more than the scene slot.",
      ],
      adjacentTags: [
        "enforcer_npc",
        "criminal_npc",
      ],
    },
  }),
  defineConceptProjections("ingested_exposure", {
    affliction: {
      axis: "disease_model",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Spread by swallowing contaminated food, drink, medicine, or another consumed substance.",
    },
  }),
  defineConceptProjections("ingested_offense", {
    equipment: {
      axis: "effect",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Offensive consumable delivered when swallowed or consumed.",
      subcategories: [
        "consumable",
      ],
    },
  }),
  defineConceptProjections("inhaled_exposure", {
    affliction: {
      axis: "disease_model",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Spread through smoke, breath, spores, vapor, dust, or another inhaled medium.",
    },
  }),
  defineConceptProjections("injury_exposure", {
    affliction: {
      axis: "disease_model",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Spread through bites, stings, punctures, or other blood-entering injury vectors.",
    },
  }),
  defineConceptProjections("innocence_twisted", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with childish innocence, nursery imagery, or comforting domestic symbols turned uncanny, cruel, or threatening.",
      appliesWhen: [
        "The creature's presentation depends on childlike, gentle, or comforting imagery becoming eerie, cruel, or dangerous.",
        "A GM would plausibly retrieve it for nursery horror, storybook menace, or innocence-corrupted scenes.",
      ],
      doesNotApplyWhen: [
        "The creature is merely small, playful, or associated with toys without innocence-curdled-into-menace being central.",
        "The stronger fit is living_toy or carnival_show because animated playthings or spectacle explain the retrieval better.",
      ],
      adjacentTags: [
        "living_toy",
        "carnival_show",
      ],
    },
  }),
  defineConceptProjections("island_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with islands, archipelagos, or isolated isles.",
    },
  }),
  defineConceptProjections("judgment_haunt", {
    hazard: {
      axis: "haunt",
      family: "haunt_manifestation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that condemns trespassers through accusation, punishment, curse-like verdicts, or moral reckoning.",
    },
  }),
  defineConceptProjections("jungle_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with jungles, rainforests, or dense tropical canopies.",
    },
  }),
  defineConceptProjections("life_drain_hazard", {
    hazard: {
      axis: "haunt",
      family: "haunt_manifestation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that drains life force, vitality, or souls from victims.",
    },
  }),
  defineConceptProjections("living_artwork", {
    creature: {
      axis: "presentation",
      family: "visual_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with paintings, graffiti, murals, portraits, or other artworks brought to life.",
    },
  }),
  defineConceptProjections("living_toy", {
    creature: {
      axis: "presentation",
      family: "visual_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with dolls, puppets, mannequins, or other animated playthings.",
    },
  }),
  defineConceptProjections("long_range_teleport", {
    spell: {
      axis: "utility",
      family: "teleportation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Teleports creatures across major overland distances, settlements, or remote destinations.",
      appliesWhen: [
        "The spell is naturally retrieved for strategic travel, relocation, or bypassing long routes.",
        "The destination scale is substantially larger than one encounter map or immediate tactical space.",
      ],
      doesNotApplyWhen: [
        "The spell is mainly a tactical blink, extraction, or planar crossing effect.",
        "The spell only repositions creatures within the same immediate scene.",
      ],
      adjacentTags: [
        "short_range_teleport",
        "planar_travel",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("lore_consultation", {
    spell: {
      axis: "utility",
      family: "consultation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Provides interpretive insight, shared knowledge, or focused understanding about a subject, clue, history, or magical situation.",
      adjacentTags: [
        "truth_reveal",
        "problem_diagnosis",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("lure_compulsion", {
    hazard: {
      axis: "haunt",
      family: "haunt_manifestation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that beckons, lures, or compels creatures into moving or acting against their judgment.",
    },
  }),
  defineConceptProjections("maelstrom_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Maelstrom, proteans, or its chaotic planar fringes.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("magic_blight_wasteland_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with arcane devastation, dead-magic pockets, wild-magic scars, magical storms, and survival in a magically broken wasteland. In Golarion, this primarily corresponds to the Mana Wastes.",
      appliesWhen: [
        "Use when the creature is naturally retrieved for dead-magic deserts, wild-magic badlands, spellscar survival, mutation-causing arcane fallout, or regions where magic itself has wounded the landscape.",
        "The planning value comes from magical environmental breakage and wasteland adaptation rather than only from one magical trait or one regional nation.",
      ],
      doesNotApplyWhen: [
        "The creature is merely magical, arcane, or mutated without a meaningful tie to a magic-blasted regional wasteland.",
        "The stronger fit is alien_technology_wasteland_setting because the retrieval hook is weird technology, robots, or alien ruins rather than magical landscape scarring.",
      ],
      adjacentTags: [
        "alien_technology_wasteland_setting",
        "wasteland_setting",
      ],
    },
  }),
  defineConceptProjections("maritime_superstition", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with cursed voyages, sea omens, sailor folklore, drowned portents, or nautical dread shaped by legend and taboo.",
      appliesWhen: [
        "The creature is naturally retrieved for ghost-ship rumor, sailor taboo, drowned prophecy, or sea-legend scenes where folklore matters as much as location.",
        "Use when nautical superstition and omen-laden seafaring culture are central to the creature's presentation.",
      ],
      doesNotApplyWhen: [
        "The creature is merely aquatic, coastal, or ship-linked without a real folklore-and-omen maritime motif.",
        "The stronger fit is nautical_setting or folk_horror because placement or generic rural superstition matters more than sailor legend.",
      ],
      adjacentTags: [
        "nautical_setting",
        "folk_horror",
      ],
    },
  }),
  defineConceptProjections("mask_motif", {
    creature: {
      axis: "presentation",
      family: "visual_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with masks, veils, ceremonial face-coverings, or deliberately obscured presentation.",
      appliesWhen: [
        "Masks, veils, or deliberate face-covering are a salient presentation motif.",
        "The obscured face is part of the creature's recurring visual identity.",
      ],
      doesNotApplyWhen: [
        "A mask appears once as minor equipment.",
        "The stronger semantic is faceless_horror or disguised_pretender rather than mask imagery.",
      ],
      adjacentTags: [
        "disguised_pretender",
        "faceless_horror",
      ],
    },
  }),
  defineConceptProjections("medical_support", {
    equipment: {
      axis: "utility",
      family: "crafting_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports first aid, diagnosis, treatment, or ongoing medical care outside direct magical healing.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "consumable",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("mental_impairment", {
    affliction: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs judgment, emotions, or perception through confusion, fear, or delirium.",
    },
    equipment: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs thought, judgment, composure, or behavior through fear, confusion, or similar effects.",
      subcategories: [
        "ammo",
        "consumable",
      ],
    },
    hazard: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs judgment, emotions, or perception through fear, confusion, or similar effects.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs thought, composure, or agency through fear, confusion, or similarly hostile mental effects.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
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
      adjacentTags: [
        "profession_npc",
        "civic_npc",
      ],
    },
  }),
  defineConceptProjections("message_delivery", {
    equipment: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Sends, stores, or relays actual content across time or distance.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Sends, stores, or relays actual content across time or distance.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("mirror_motif", {
    creature: {
      axis: "presentation",
      family: "visual_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with mirrors, reflections, duplicated selves, or reflective surfaces as a core visual or horror identity.",
    },
  }),
  defineConceptProjections("mobility", {
    equipment: {
      axis: "utility",
      family: "movement_traversal",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Improves movement or traversal flexibility.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Helps move faster, gain movement modes, or traverse terrain more effectively.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("mobility_impairment", {
    affliction: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Reduces speed, stiffens movement, or leaves the victim immobilized.",
    },
    equipment: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs movement through slowing, restraining, sticking, or immobilizing effects.",
      subcategories: [
        "ammo",
        "consumable",
      ],
    },
    hazard: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Paralyzes, immobilizes, or otherwise heavily hampers movement.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("mountain_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with cliffs, peaks, passes, or rocky heights.",
      appliesWhen: [
        "Mountain terrain is a recurring habitat or encounter frame.",
        "The creature is tied to peaks, passes, cliffs, or alpine strongholds.",
      ],
      doesNotApplyWhen: [
        "Rocky terrain is incidental rather than defining.",
        "The creature is better described as underground_setting or sky_setting.",
      ],
      adjacentTags: [
        "underground_setting",
        "sky_setting",
      ],
    },
  }),
  defineConceptProjections("mounted_support", {
    equipment: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports mounted combat, rider control, saddle use, or mount-specific loadouts.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
        "armor",
        "shield",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("multi_stage_resolution", {
    hazard: {
      tag: "layered_resolution",
      axis: "problem",
      family: "problem_shape",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that asks the party to solve multiple linked pieces rather than one single disable check or obvious answer.",
    },
  }),
  defineConceptProjections("mwangi_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Mwangi Expanse, its jungle polities, and Mwangi-rooted regional framing that materially affects creature planning and retrieval.",
    },
  }),
  defineConceptProjections("nautical_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with ships, sailors, wrecks, or harbors.",
    },
  }),
  defineConceptProjections("navigation", {
    equipment: {
      axis: "utility",
      family: "movement_traversal",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps track direction, route, or position.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Helps orient, guide a route, or identify a destination's direction.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("nightmare_tainted", {
    creature: {
      axis: "specialization",
      family: "corruption_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creature strongly defined by dream corruption, sleep-haunting influence, or oneiric pollution leaking into the waking world.",
      adjacentTags: [
        "dream_nightmare",
        "dreamlands_setting",
      ],
    },
  }),
  defineConceptProjections("nightmare_torment", {
    affliction: {
      axis: "metaphysical",
      family: "metaphysical_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Centers on nightmares, dream-torment, or similarly sleep-haunting affliction framing.",
    },
  }),
  defineConceptProjections("nirvana_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Nirvana, enlightened celestials, or benevolent contemplative service.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("observation_driven", {
    hazard: {
      tag: "observation_first",
      axis: "problem",
      family: "problem_shape",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that rewards careful watching, clue gathering, or reading the environment before a safe approach becomes obvious.",
    },
  }),
  defineConceptProjections("obsession_fixation", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with compulsive pursuit, jealous fixation, collecting mania, perfectionism, or a single consuming desire.",
      appliesWhen: [
        "The creature is framed around monomania, possessive attention, compulsive collecting, or a need it cannot release.",
        "Retrieval value comes from unhealthy fixation driving the story, not simply from a preference or goal.",
      ],
      doesNotApplyWhen: [
        "The creature has a mission, desire, or recurring target without obsessive compulsion being central to its identity.",
        "The stronger fit is forbidden_knowledge or predatory_seduction because taboo learning or lure-based predation matters more than fixation.",
      ],
      adjacentTags: [
        "forbidden_knowledge",
        "predatory_seduction",
      ],
    },
  }),
  defineConceptProjections("occult_conspiracy", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with secret circles, hidden masters, esoteric cabals, or ritual networks manipulating events from the shadows.",
      appliesWhen: [
        "The creature is naturally retrieved for hidden cult cells, secret masters, conspiratorial rites, or layered occult plotting.",
        "Its presentation depends on covert structure and esoteric collusion, not only on individual deception or ritual practice.",
      ],
      doesNotApplyWhen: [
        "The creature merely participates in a ritual, infiltrates a group, or knows occult lore without a real cabal or conspiracy presentation.",
        "The stronger fit is ritual_ceremony, paranoia_surveillance, or forbidden_knowledge rather than hidden-network manipulation.",
      ],
      adjacentTags: [
        "paranoia_surveillance",
        "forbidden_knowledge",
      ],
    },
  }),
  defineConceptProjections("occult_spellcaster", {
    creature: {
      axis: "specialization",
      family: "casting_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creature whose spellcasting is substantially framed through occult lore, spirits, emotion, dreams, or esoteric mental power.",
    },
  }),
  defineConceptProjections("offensive", {
    equipment: {
      axis: "effect",
      family: "consumable_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hostile consumable primarily meant to harm or debilitate a target.",
      subcategories: [
        "consumable",
      ],
    },
  }),
  defineConceptProjections("omen_guidance", {
    spell: {
      axis: "utility",
      family: "consultation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Asks for omens, directional guidance, or advisory insight about the best course of action, likely outcome, or strategic choice.",
      adjacentTags: [
        "lore_consultation",
        "wayfinding",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("organized_undead_society_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with organized undead societies, corpse labor, necromantic bureaucracy, and courtly undead institutions. In Golarion, this primarily corresponds to Geb.",
      appliesWhen: [
        "Use when the creature is naturally retrieved for an undead-ruled state, necromantic civil order, corpse-backed labor system, or other organized deathless society rather than an isolated tomb or graveyard.",
        "The planning value comes from undead institutions, court politics, civic structure, or civilized undead social roles as much as from simple undead presence.",
      ],
      doesNotApplyWhen: [
        "The creature is merely undead, tomb-dwelling, or cemetery-haunting without a meaningful link to a broader undead social order.",
        "The stronger fit is undead_war_torn_region_setting because the retrieval hook is an undead occupation zone or shattered crusader frontier rather than a stable undead society.",
      ],
      adjacentTags: [
        "undead_war_torn_region_setting",
        "urban_setting",
      ],
    },
  }),
  defineConceptProjections("pack_hunter", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Naturally retrieved because the creature hunts as part of a pack, coordinated ambush group, or pursuit cluster rather than as a solitary predator.",
      adjacentTags: [
        "ambusher_combatant",
        "skirmisher_combatant",
      ],
    },
  }),
  defineConceptProjections("paranoia_surveillance", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with hidden watchers, constant scrutiny, being monitored, or the collapse of trust under observation.",
      appliesWhen: [
        "The creature evokes stalking observation, unseen witnesses, informer networks, or dread rooted in being watched.",
        "Retrieval value comes from suspicion, surveillance, or omnipresent scrutiny rather than only stealth or infiltration.",
      ],
      doesNotApplyWhen: [
        "The creature merely scouts, spies, or infiltrates without a broader atmosphere of surveillance and distrust.",
        "The stronger fit is occult_conspiracy or disguised_pretender because hidden coordination or impersonation matters more than the watched feeling.",
      ],
      adjacentTags: [
        "occult_conspiracy",
        "disguised_pretender",
      ],
    },
  }),
  defineConceptProjections("parasite_ridden", {
    creature: {
      axis: "specialization",
      family: "corruption_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creature strongly defined by hosting burrowing larvae, parasitic colonies, implanted broods, or other invasive life within the body.",
      adjacentTags: [
        "spawn_creator",
        "plaguebearing",
      ],
    },
  }),
  defineConceptProjections("patrol_member", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Naturally retrieved as part of a watch patrol, sentry route, street sweep, border detail, or other recurring patrol formation.",
      adjacentTags: [
        "watcher_npc",
        "urban_setting",
      ],
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
      adjacentTags: [
        "profession_npc",
        "carnival_show",
      ],
    },
  }),
  defineConceptProjections("petrification", {
    affliction: {
      axis: "effect",
      family: "physiology_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Turns flesh to stone or otherwise locks the body into a rigid mineralized state.",
      appliesWhen: [
        "The affliction is naturally retrieved because victims end up petrified, stone-locked, or mineralized.",
        "The end-state of stony immobilization matters more than the thematic cause.",
      ],
      doesNotApplyWhen: [
        "The affliction only trends toward calcification without actually functioning as a petrifying condition.",
        "The stronger fit is petrifying_corruption because the gradual corruption process is the real hook.",
      ],
      adjacentTags: [
        "petrifying_corruption",
        "mobility_impairment",
      ],
    },
  }),
  defineConceptProjections("petrifying_corruption", {
    affliction: {
      axis: "disease_model",
      family: "pathogenesis",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Defined by calcification, ossification, stone-turning, or gradual bodily hardening toward an inert form.",
      appliesWhen: [
        "The affliction is retrieved for gradual hardening, calcification, or stoneward corruption across stages.",
        "The process of becoming stone-like matters as much as the final condition.",
      ],
      doesNotApplyWhen: [
        "The affliction chiefly imposes a final petrified state without broader progressive corruption framing.",
        "The stronger fit is transformative_corruption without specifically stoneward identity.",
      ],
      adjacentTags: [
        "petrification",
        "cumulative_transformation",
      ],
    },
  }),
  defineConceptProjections("phantom_assailants", {
    hazard: {
      axis: "haunt",
      family: "haunt_manifestation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that manifests ghostly, spectral, or phantom attackers as separate assailants.",
    },
  }),
  defineConceptProjections("physical_debilitation", {
    affliction: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Weakens the body through exhaustion, sickness, drained vitality, blood loss, or similar bodily degradation.",
    },
    equipment: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Weakens the body through drained vitality, sickness, fatigue, clumsiness, or similar bodily degradation.",
      subcategories: [
        "ammo",
        "consumable",
      ],
    },
  }),
  defineConceptProjections("plaguebearing", {
    creature: {
      axis: "specialization",
      family: "corruption_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creature strongly defined by carrying, spreading, or embodying pestilence, fever, or outbreak-causing corruption.",
      adjacentTags: [
        "disease_vector",
        "parasite_ridden",
      ],
    },
  }),
  defineConceptProjections("plains_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with open plains, grasslands, prairies, or savannas.",
    },
  }),
  defineConceptProjections("planar_breach", {
    hazard: {
      axis: "mechanism",
      family: "mechanism",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on a portal, rift, tear, breach, or other unstable opening in reality.",
      appliesWhen: [
        "An unstable portal, planar tear, extradimensional rupture, or reality breach is the central mechanism of the hazard.",
        "The hazard is naturally retrieved for cosmological leakage, portal instability, or something dangerous coming through a breach.",
      ],
      doesNotApplyWhen: [
        "The hazard is merely magical or teleportive without a real opening in reality as the core hazard engine.",
        "The stronger fit is dispel_countermeasure or procedural_bypass because the planar flavor is incidental.",
      ],
      adjacentTags: [
        "dispel_countermeasure",
        "procedural_bypass",
      ],
    },
  }),
  defineConceptProjections("planar_travel", {
    spell: {
      axis: "utility",
      family: "teleportation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Moves creatures between planes, through planar routes, or into extraplanar destinations.",
      appliesWhen: [
        "Crossing into another plane, demiplane, or extraplanar route is central to the spell's retrieval value.",
        "The spell is naturally retrieved for cosmological travel rather than mundane relocation.",
      ],
      doesNotApplyWhen: [
        "The spell only teleports within the same plane or functions as a normal long-distance travel tool.",
        "Extradimensional storage or shelter is present without real plane-crossing travel.",
      ],
      adjacentTags: [
        "long_range_teleport",
        "field_shelter",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("plane_of_air_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Plane of Air, its native denizens, or its endless winds and cloud realms.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("plane_of_earth_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Plane of Earth, its native denizens, or its crystal caverns and stonebound realms.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("plane_of_fire_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Plane of Fire, its native denizens, or its infernal-bright cityscapes and battlefields.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("plane_of_water_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Plane of Water, its native denizens, or its endless seas and oceanic realms.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("possessed_object", {
    creature: {
      axis: "presentation",
      family: "bound_object",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with an inhabiting spirit or curse animating an otherwise mundane object or suit of equipment.",
      appliesWhen: [
        "Use when a spirit, ghost, curse, or other external presence is explicitly what animates the object.",
        "The inhabiting presence matters more than the object's construction, material, or generic animation.",
      ],
      doesNotApplyWhen: [
        "The object is simply animated by magic, clockwork, or sculpted animation with no real possessing force.",
        "The stronger fit is animated_object or animated_statue because possession is not central.",
      ],
      adjacentTags: [
        "animated_object",
        "possession_threat",
      ],
    },
  }),
  defineConceptProjections("possession_haunt", {
    hazard: {
      axis: "haunt",
      family: "haunt_manifestation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that enters, rides, or briefly controls a victim rather than only attacking them externally.",
    },
  }),
  defineConceptProjections("possession_seed", {
    affliction: {
      axis: "metaphysical",
      family: "metaphysical_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Plants an invading spirit, hostile presence, or takeover-ready metaphysical foothold inside the victim.",
      appliesWhen: [
        "The affliction prepares a victim to be ridden, entered, replaced, or overtaken by another presence.",
        "A latent invading spirit or takeover-ready foothold is central to the condition's danger.",
      ],
      doesNotApplyWhen: [
        "The affliction only compels behavior without an actual possessing entity or metaphysical foothold.",
        "The stronger fit is soul_binding or compulsion because takeover is not really part of the disease model.",
      ],
      adjacentTags: [
        "compulsion",
        "soul_binding",
      ],
    },
  }),
  defineConceptProjections("predatory_seduction", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with deliberate luring, honey-trap menace, erotic predation, or invitation used explicitly as a hunting tactic.",
      appliesWhen: [
        "The creature's presentation centers on luring prey through desire, intimacy, or false safety before the attack or betrayal lands.",
        "Use when the hunting or consuming dynamic matters more than general temptation or glamour.",
      ],
      doesNotApplyWhen: [
        "The creature is alluring or corruptive without a strong predator-lure structure.",
        "The stronger fit is seductive_temptation because dangerous attraction matters more than an explicit hunt pattern.",
      ],
      adjacentTags: [
        "seductive_temptation",
        "disguised_pretender",
      ],
    },
  }),
  defineConceptProjections("pressure_trigger", {
    hazard: {
      axis: "mechanism",
      family: "mechanism",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard triggered by stepping on, weighing down, or depressing a pressure surface.",
    },
  }),
  defineConceptProjections("primal_spellcaster", {
    creature: {
      axis: "specialization",
      family: "casting_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creature whose spellcasting is substantially framed through nature, elemental power, druidic force, or instinctive primal magic.",
    },
  }),
  defineConceptProjections("problem_diagnosis", {
    spell: {
      axis: "utility",
      family: "consultation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Helps determine what hidden magical, spiritual, cursed, or otherwise obscure problem is actually affecting a target, site, or situation.",
      adjacentTags: [
        "curse_revelation",
        "magic_detection",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
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
      adjacentTags: [
        "authority_npc",
        "merchant_npc",
      ],
    },
  }),
  defineConceptProjections("projectile_defense", {
    equipment: {
      axis: "item_mechanical",
      family: "defense_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Intercepts, redirects, or absorbs ranged attacks and projectiles.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "shield",
      ],
    },
  }),
  defineConceptProjections("prophecy_omen", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with foretelling, omen-bearing, apocalyptic signs, destiny, or a creature's role as a herald of what is to come.",
      appliesWhen: [
        "Portents, prophecy, omen-reading, or the creature's arrival as a sign of coming change is a central retrieval hook.",
        "A GM would plausibly retrieve the creature for foretold doom, chosen destiny, or fate-haunted story beats.",
      ],
      doesNotApplyWhen: [
        "The creature merely predicts events, has divination magic, or is important to the plot without a strong omen-facing presentation.",
        "The stronger fit is apocalypse_ruin or ancestral_legacy because destiny-sign imagery is not the main presentation hook.",
      ],
      adjacentTags: [
        "apocalypse_ruin",
        "ancestral_legacy",
      ],
    },
  }),
  defineConceptProjections("pursuit_punisher", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that punishes retreat, pursuit, escape routes, or movement through chase-style spaces.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("ranged_striker_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to archers, gunners, or other builds that pressure from range through repeated attacks or reload workflows.",
    },
  }),
  defineConceptProjections("recurrent_flare", {
    affliction: {
      axis: "disease_model",
      family: "progression_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Affliction that subsides and returns in episodes, repeating attacks, or cyclical symptom spikes.",
      appliesWhen: [
        "The affliction is naturally retrieved because symptoms repeatedly subside and then surge back in cycles, attacks, or flare-ups.",
        "Its pacing matters as a recurring problem rather than a single steady decline.",
      ],
      doesNotApplyWhen: [
        "The affliction mainly incubates once and then worsens in a straight line.",
        "The stronger fit is delayed_onset or terminal_collapse because recurrence is not the main progression hook.",
      ],
      adjacentTags: [
        "delayed_onset",
        "terminal_collapse",
      ],
    },
  }),
  defineConceptProjections("religious_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Presented as a priest, shrine keeper, cult officiant, monastic figure, or other explicitly religious role-holder.",
      appliesWhen: [
        "Religious office, ritual duty, or custodianship of a faith space is central to the creature's world-facing identity.",
        "The creature is naturally retrieved as clergy, cult staff, or sacred-site personnel.",
      ],
      doesNotApplyWhen: [
        "The creature merely has divine powers without a role-defined religious identity.",
        "Temple placement alone is better captured by temple_setting.",
      ],
      adjacentTags: [
        "profession_npc",
        "temple_setting",
      ],
    },
  }),
  defineConceptProjections("reload_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to ranged builds whose action flow depends on reloading efficiently or keeping ammunition ready.",
    },
  }),
  defineConceptProjections("repair_support", {
    equipment: {
      axis: "utility",
      family: "crafting_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports item repair, patchwork, upkeep, or restoring damaged gear and structures.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "consumable",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("replayed_tragedy", {
    hazard: {
      axis: "haunt",
      family: "haunt_manifestation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that re-enacts a murder, betrayal, execution, disaster, or other fixed traumatic event.",
      appliesWhen: [
        "The haunt is naturally retrieved because it replays a specific past calamity, crime, execution, or emotional flashpoint as its core manifestation.",
        "The narrative repetition of an old event matters more than generic life drain, possession, or battlefield disruption.",
      ],
      doesNotApplyWhen: [
        "The haunt is only sad, angry, or spiritually active without reenacting a fixed historical scene.",
        "The stronger fit is judgment_haunt or lure_compulsion because the recurring tragedy itself is not the central hook.",
      ],
      adjacentTags: [
        "judgment_haunt",
        "appeasement_countermeasure",
      ],
    },
  }),
  defineConceptProjections("resource_drain", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that taxes healing, spellcasting, equipment durability, or other party resources over time.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("respiratory_impairment", {
    affliction: {
      axis: "effect",
      family: "physiology_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Prevents normal breathing or fills the victim's lungs with water, fluid, or similar suffocating effects.",
    },
  }),
  defineConceptProjections("revelry_excess", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with feasts, drunken revels, riotous celebration, gluttony, or ecstatic overindulgence.",
      appliesWhen: [
        "The creature is naturally retrieved for bacchanals, cursed feasts, debauched parties, or scenes of ecstatic excess.",
        "Celebration curdling into danger is part of the creature's recurring narrative identity.",
      ],
      doesNotApplyWhen: [
        "The creature only appears near taverns or festivals without excess, indulgence, or revelry being central.",
        "The stronger fit is carnival_show or seductive_temptation rather than feast-and-excess presentation.",
      ],
      adjacentTags: [
        "carnival_show",
        "seductive_temptation",
      ],
    },
  }),
  defineConceptProjections("ritual_ceremony", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with rites, sacrifices, processions, ceremonial observance, or cultic staging as a scene identity.",
      appliesWhen: [
        "Ceremonial staging, sacrificial ritual, processional presentation, or formal observance is a major retrieval hook.",
        "The creature is naturally used in scenes defined by rites, altars, chants, offerings, or public ceremony.",
      ],
      doesNotApplyWhen: [
        "The creature merely has ritual magic or divine powers without a strong ceremonial scene identity.",
        "Temple placement or religious office alone is better captured by temple_setting or religious_npc.",
      ],
      adjacentTags: [
        "religious_npc",
        "ritualist_creature",
      ],
    },
  }),
  defineConceptProjections("ritual_support", {
    equipment: {
      axis: "utility",
      family: "crafting_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports ritual casting, ceremonial setup, circles, offerings, or other extended magical preparation.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "consumable",
      ],
      appliesWhen: [
        "The item meaningfully supports ceremonial, circle-based, offering-based, or extended-casting magic work.",
        "A user would retrieve it for magic preparation rather than ordinary crafting or adventuring gear.",
      ],
      doesNotApplyWhen: [
        "The item is only generally magical without helping ritual process or setup.",
        "The item is mainly a focus of worship or symbolism rather than ritual procedure.",
      ],
      adjacentTags: [
        "alchemical_crafting",
        "magic_protection",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("ritualist_creature", {
    creature: {
      axis: "specialization",
      family: "casting_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creature strongly associated with ritual casting, ceremonial magic, or extended occult or divine preparations.",
      appliesWhen: [
        "Use when ceremonial, circle-based, sacrificial, or extended-casting magic is a major reason to retrieve the creature.",
        "The creature is naturally used as a ritual leader, ritual threat, or ritual-supporting encounter element.",
      ],
      doesNotApplyWhen: [
        "The creature merely casts normal encounter spells without a meaningful ritual identity.",
        "The stronger fit is a tradition spellcaster tag and ritual work is only incidental flavor.",
      ],
      adjacentTags: [
        "divine_spellcaster",
        "occult_spellcaster",
      ],
    },
  }),
  defineConceptProjections("rot_decay", {
    affliction: {
      axis: "disease_model",
      family: "pathogenesis",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Defined by bodily rot, necrosis, blight, mummification, or similar physical decay.",
    },
  }),
  defineConceptProjections("ruins_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with ancient ruins or derelict structures.",
    },
  }),
  defineConceptProjections("rural_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with farms, pastures, croplands, countryside routes, mills, or other agricultural rural encounter scenes.",
      appliesWhen: [
        "The creature is repeatedly framed around farms, fields, mills, roadsides, or countryside scenes.",
        "Agricultural or open-country placement is a recurring part of its encounter identity.",
      ],
      doesNotApplyWhen: [
        "The record only implies generic overland travel.",
        "The creature is better modeled by small_settlement_setting or plains_setting.",
      ],
      adjacentTags: [
        "small_settlement_setting",
        "plains_setting",
      ],
    },
  }),
  defineConceptProjections("scholar_npc", {
    creature: {
      axis: "npc_role",
      family: "social_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Presented as a sage, researcher, teacher, archivist, alchemist, or other knowledge-centered role-holder.",
      appliesWhen: [
        "Research, teaching, scholarship, or recordkeeping is central to the creature's world-facing identity.",
        "The creature is naturally retrieved as an academic or knowledge-scene NPC.",
      ],
      doesNotApplyWhen: [
        "Intelligence is incidental to a stronger combat or monster identity.",
        "The record only implies general competence without a knowledge-centered role.",
      ],
      adjacentTags: [
        "profession_npc",
        "civic_npc",
      ],
    },
  }),
  defineConceptProjections("scout_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to scouts, infiltrators, or advance-party play through quiet entry, recon, and information-gathering support.",
    },
  }),
  defineConceptProjections("scouting", {
    equipment: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps observe, survey, or reconnoiter an area.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Helps observe at a distance, extend senses, or locate a target.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("scrying_protection", {
    equipment: {
      axis: "utility",
      family: "security",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Blocks magical observation, remote viewing, or information leakage through divination-like effects.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
      ],
      appliesWhen: [
        "The item's main value is preventing magical spying, remote observation, or divination-led tracking.",
        "A user would retrieve it to keep plans, rooms, or identities hidden from magical surveillance.",
      ],
      doesNotApplyWhen: [
        "The item only improves ordinary stealth or concealment without anti-divination protection.",
        "The item counters magic generally but is not particularly about observation or information leakage.",
      ],
      adjacentTags: [
        "alarm",
        "concealment",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "security",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Blocks magical observation, remote viewing, divinatory surveillance, or other information leakage from a protected target or space.",
      appliesWhen: [
        "The spell is naturally retrieved to keep plans, sanctums, identities, or conversations hidden from magical spying.",
        "Its core value is denying observation or divination rather than only raising an intrusion alarm.",
      ],
      doesNotApplyWhen: [
        "The spell only improves mundane concealment or silence without real anti-divination protection.",
        "The spell counters magic broadly but is not specifically about surveillance or remote observation.",
      ],
      adjacentTags: [
        "alarm",
        "countermagic",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("seasonal_festival", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with solstice rites, harvest festivals, holiday customs, masked processions, or recurring calendar-bound celebration.",
      appliesWhen: [
        "The creature is naturally retrieved for midsummer revels, winter rites, harvest pageants, or holiday scenes defined by recurring festal tradition.",
        "Calendar-bound custom or festival atmosphere is a central part of the creature's presentation rather than incidental backdrop.",
      ],
      doesNotApplyWhen: [
        "The creature merely appears during a feast, fair, or celebration without a recurring seasonal or ritual festival identity.",
        "The stronger fit is revelry_excess, carnival_show, or folk_horror because indulgence, spectacle, or folklore carries the retrieval weight.",
      ],
      adjacentTags: [
        "revelry_excess",
        "folk_horror",
      ],
    },
  }),
  defineConceptProjections("sedation", {
    affliction: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Induces sleep, deep drowsiness, trance-like unconsciousness, or difficulty waking.",
    },
    equipment: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Induces sleep, lethargy, unconsciousness, or similar incapacitating drowsiness.",
      subcategories: [
        "ammo",
        "consumable",
      ],
    },
  }),
  defineConceptProjections("seductive_temptation", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with allure, seduction, dangerous invitation, or temptation into vice, doom, or compromise.",
      appliesWhen: [
        "The creature's presentation centers on allure, invitation, enchantment through desire, or baiting victims into a doomed choice.",
        "Retrieval value comes from temptation or dangerous attraction, not just social interaction or mechanical charm effects.",
      ],
      doesNotApplyWhen: [
        "The creature is merely attractive, charismatic, or capable of charm without temptation being a real story hook.",
        "The stronger fit is disguised_pretender or courtly_pageantry rather than luring desire.",
      ],
      adjacentTags: [
        "disguised_pretender",
        "courtly_pageantry",
      ],
    },
  }),
  defineConceptProjections("self_buff", {
    equipment: {
      axis: "effect",
      family: "consumable_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Support consumable primarily applied to the user.",
      subcategories: [
        "consumable",
      ],
    },
  }),
  defineConceptProjections("self_destructive_impulse", {
    affliction: {
      axis: "behavior",
      family: "behavioral_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Drives reckless self-harm, suicidal behavior, or dangerous compulsions against the victim's own interests.",
    },
  }),
  defineConceptProjections("senses_support", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Improves vision or other senses.",
      subcategories: [
        "consumable",
      ],
    },
    spell: {
      axis: "utility",
      family: "sensory_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Enhances vision or other senses through darkvision, see invisible, sharpened perception, scent, or similar perceptual upgrades.",
      adjacentTags: [
        "scouting",
        "invisibility_reveal",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("sensory_impairment", {
    affliction: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Blinds, deafens, or otherwise suppresses perception and the senses.",
    },
    equipment: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Impairs sight, hearing, or other senses through blinding, dazzling, or deafening effects.",
      subcategories: [
        "ammo",
        "consumable",
      ],
    },
    hazard: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Blinds, deafens, dazzles, or otherwise suppresses a victim's ability to perceive the environment clearly.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Blinds, deafens, or otherwise directly suppresses a creature's senses.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("shadow_plane_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with the Shadow Plane or the Plane of Shadow.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("shield_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to shield-forward play through improved blocking, readiness, or defensive shield workflow.",
    },
  }),
  defineConceptProjections("short_range_teleport", {
    spell: {
      axis: "utility",
      family: "teleportation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Teleports a creature across a short tactical distance, usually within the same scene or encounter area.",
      appliesWhen: [
        "The spell repositions a creature within the current encounter or scene rather than serving as expedition travel.",
        "The tactical blink or reposition is itself a major reason to retrieve the spell.",
      ],
      doesNotApplyWhen: [
        "The spell is mainly about escaping custody, extracting allies, or long-distance transport.",
        "The spell primarily crosses planes or major overland distances.",
      ],
      adjacentTags: [
        "extraction_teleport",
        "long_range_teleport",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("signaling", {
    equipment: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps draw attention, mark a location, or coordinate allies.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps draw attention, mark a location, or coordinate allies.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("silencing", {
    equipment: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Suppresses speech, voice, or other sound-dependent action through gagging, muting, or numbing effects.",
      subcategories: [
        "ammo",
        "consumable",
      ],
    },
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Suppresses speech, sound production, verbal casting, or other voice-dependent action.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("sinspawn_family", {
    creature: {
      axis: "specialization",
      family: "ontology_cluster",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Groups sinspawn and close runelord-bred sinspawn offshoots into one retrieval bucket.",
      nativeOntologyPolicy: "aggregates_native_signals",
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
      adjacentTags: [
        "ambusher_combatant",
        "controller_combatant",
      ],
    },
  }),
  defineConceptProjections("skirmisher_support", {
    equipment: {
      axis: "party_role",
      family: "party_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to mobile flankers, hit-and-run melee characters, or other skirmisher-style builds.",
    },
  }),
  defineConceptProjections("sky_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with open skies, storm clouds, or high-altitude aerial habitats.",
    },
  }),
  defineConceptProjections("small_settlement_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with villages, hamlets, small towns, or other low-density community settlements.",
      appliesWhen: [
        "The creature is repeatedly framed around villages, hamlets, or small town life.",
        "Its encounter identity is tied to low-density community spaces rather than major urban districts.",
      ],
      doesNotApplyWhen: [
        "The record only says the creature can appear near people.",
        "The creature is more strongly urban, rural, or fortress-coded than village-coded.",
      ],
      adjacentTags: [
        "urban_setting",
        "rural_setting",
      ],
    },
  }),
  defineConceptProjections("social_infiltration", {
    equipment: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps blend into a group or pass under social scrutiny.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps blend into a group or pass under social scrutiny.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("soul_binding", {
    affliction: {
      axis: "metaphysical",
      family: "metaphysical_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Pins, traps, anchors, or otherwise entangles the victim's soul with an object, oath, place, or hostile power.",
      appliesWhen: [
        "The affliction's metaphysical identity depends on the victim's soul being anchored, trapped, pledged, or externally entangled.",
        "A user would retrieve it for ghost anchors, cursed bindings, oath-linked doom, or similar soul-tether effects.",
      ],
      doesNotApplyWhen: [
        "The affliction only marks, weakens, or spiritually corrupts the victim without actually binding the soul to something.",
        "The stronger fit is curse_marking or possession_seed rather than tethering or anchoring.",
      ],
      adjacentTags: [
        "curse_marking",
        "possession_seed",
      ],
    },
  }),
  defineConceptProjections("spawned_attackers", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that summons, creates, or releases separate attackers into the scene.",
      appliesWhen: [
        "The hazard is naturally retrieved because it adds new hostile creatures, constructs, swarms, or manifestations to the scene.",
        "The extra attackers matter as separate encounter pressure rather than only as a damage effect.",
      ],
      doesNotApplyWhen: [
        "The hazard only deals direct damage, restrains victims, or creates a temporary illusion without generating distinct assailants.",
        "The stronger fit is phantom_assailants when the threat is specifically a haunt manifestation rather than a broader hazard function.",
      ],
      adjacentTags: [
        "alarm",
        "phantom_assailants",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("spell_payload", {
    equipment: {
      axis: "item_mechanical",
      family: "ammunition_payload",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Ammunition that delivers, casts, or imposes a spell effect on hit.",
      subcategories: [
        "ammo",
      ],
    },
  }),
  defineConceptProjections("stealth_entry_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to quiet-entry, infiltration, burglary, or reconnaissance play that depends on avoiding notice and solving access problems discreetly.",
    },
  }),
  defineConceptProjections("stealth_support", {
    equipment: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps move quietly, avoid notice, muffle noise, or otherwise support covert entry and low-profile movement.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Helps move quietly, avoid notice, suppress noisy presence, or otherwise support covert entry and low-profile movement.",
      appliesWhen: [
        "The spell is naturally retrieved to help a creature move quietly, avoid notice, pass unseen, or keep a covert approach from drawing attention.",
        "The retrieval hook is quiet entry or low-profile movement rather than only broad battlefield obscurity.",
      ],
      doesNotApplyWhen: [
        "The spell only creates a combat concealment effect or visual obstruction without really supporting a covert approach.",
        "The spell changes appearance or social presentation without materially helping the target move unnoticed.",
      ],
      adjacentTags: [
        "concealment",
        "silencing",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("stitched_horror", {
    creature: {
      axis: "presentation",
      family: "visual_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with sutures, patchwork flesh, sewn bodies, or visibly assembled corpse craftsmanship.",
    },
  }),
  defineConceptProjections("support_combatant", {
    creature: {
      axis: "encounter",
      family: "combat_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Built to heal, buff, protect, command, or otherwise enable allied creatures more than acting as the primary damage source.",
      appliesWhen: [
        "The creature is naturally retrieved because it keeps allies alive, stronger, better positioned, or otherwise more dangerous.",
        "Ally enablement matters more than its own direct kill pressure.",
      ],
      doesNotApplyWhen: [
        "The creature has one incidental buff or heal but is otherwise a brute, artillery, or controller.",
        "The stronger identity is commander_combatant because leadership and coordination outweigh broad support.",
      ],
      adjacentTags: [
        "commander_combatant",
        "controller_combatant",
      ],
    },
  }),
  defineConceptProjections("surveillance_recording", {
    equipment: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Captures, stores, or replays images, sound, or other evidence for later review.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon",
      ],
      appliesWhen: [
        "The item's retrieval value comes from preserving sights, sounds, or observations for later replay, proof, or analysis.",
        "It is naturally sought as evidence capture, remote monitoring, or watch-post support rather than live conversation gear.",
      ],
      doesNotApplyWhen: [
        "The item only sends a live message or helps coordinate allies without retaining evidence.",
        "The item protects against observation instead of performing it.",
      ],
      adjacentTags: [
        "scouting",
        "tamper_evidence",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("survival", {
    equipment: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports wilderness travel, shelter, or long-term field use.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
        "armor",
        "shield",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("sustenance", {
    equipment: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Provides food, feed, water, or other practical nourishment for travel and survival.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
        "armor",
        "shield",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Provides food, water, rations, or practical nourishment for travel and survival.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("swamp_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with bogs, marshes, fens, or mires.",
    },
  }),
  defineConceptProjections("tamper_evidence", {
    equipment: {
      axis: "utility",
      family: "security",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Makes intrusion, opening, theft, or interference easier to notice after the fact.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
      ],
      appliesWhen: [
        "The item is naturally retrieved to reveal whether a lock, crate, seal, letter, cache, or room has been disturbed.",
        "Evidence of interference matters more than immediate warning, direct defense, or anti-scrying.",
      ],
      doesNotApplyWhen: [
        "The item mainly alerts in real time when someone crosses a boundary.",
        "The item hides or protects a target without preserving signs of intrusion.",
      ],
      adjacentTags: [
        "alarm",
        "surveillance_recording",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("telepathic_communication", {
    equipment: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Enables silent mind-to-mind coordination, psychic speech, or communication that bypasses normal hearing.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
      ],
      appliesWhen: [
        "The item's retrieval value comes from silent psychic coordination, mind-to-mind speech, or communication that bypasses ordinary sound.",
        "It is naturally sought when stealth, silence, distance, or noise would make spoken coordination unreliable.",
      ],
      doesNotApplyWhen: [
        "The item only boosts ordinary signaling, writing, or message relay without true mind-to-mind communication.",
        "The item is mainly about surveillance or recording rather than live coordination.",
      ],
      adjacentTags: [
        "signaling",
        "message_delivery",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creates direct mind-to-mind communication, silent tactical coordination, or psychic speech between creatures.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("temple_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard strongly associated with shrines, sanctums, cathedrals, monasteries, or consecrated sites.",
      appliesWhen: [
        "Sacred architecture, ritual sanctums, or faith-site defense is central to the hazard's scene identity.",
        "The hazard is naturally retrieved for shrine, sanctum, or temple protection rather than generic indoor defense.",
      ],
      doesNotApplyWhen: [
        "The hazard is merely magical or haunted without specific sacred-site framing.",
        "The stronger setting is dungeon_hazard or urban_hazard rather than temple-focused.",
      ],
      adjacentTags: [
        "dungeon_hazard",
        "appeasement_countermeasure",
      ],
    },
  }),
  defineConceptProjections("temple_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with temples, shrines, monasteries, or other sacred encounter sites.",
    },
  }),
  defineConceptProjections("terminal_collapse", {
    affliction: {
      axis: "disease_model",
      family: "progression_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Affliction explicitly framed around catastrophic bodily failure, death, or a final ruinous end state if unchecked.",
      appliesWhen: [
        "The affliction is naturally retrieved because the later stages end in death, total ruin, or another catastrophic final break point.",
        "The inevitability of a terminal end state matters more than day-to-day impairment or transformation imagery.",
      ],
      doesNotApplyWhen: [
        "The affliction is dangerous but does not build toward a distinct catastrophic endpoint.",
        "The stronger fit is cumulative_transformation or physical_debilitation because ongoing deterioration matters more than terminal collapse.",
      ],
      adjacentTags: [
        "cumulative_transformation",
        "physical_debilitation",
      ],
    },
  }),
  defineConceptProjections("threshold_lockdown", {
    hazard: {
      axis: "mechanism",
      family: "mechanism",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that seals, locks, or bars a threshold, doorway, or gate.",
    },
  }),
  defineConceptProjections("thrown_offense", {
    equipment: {
      axis: "effect",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Offensive consumable delivered by throwing it.",
      subcategories: [
        "consumable",
      ],
    },
  }),
  defineConceptProjections("thrown_weapon_support", {
    equipment: {
      axis: "party_role",
      family: "play_pattern",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to builds that solve problems through thrown weapons, quick draws, or repeatable thrown-item pressure.",
    },
  }),
  defineConceptProjections("tian_xia_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with Tian Xia and its major cultural subregions, where Tian-rooted regional framing materially affects creature planning and retrieval.",
    },
  }),
  defineConceptProjections("time_critical_resolution", {
    affliction: {
      tag: "cure_clock_urgency",
      axis: "response",
      family: "response_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creates immediate pressure to diagnose and cure the affliction before a fast-moving catastrophic endpoint arrives.",
      adjacentTags: [
        "terminal_collapse",
        "delayed_onset",
      ],
    },
  }),
  defineConceptProjections("timing_window", {
    hazard: {
      axis: "problem",
      family: "problem_shape",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that is best handled by acting during the right cycle, opening, lull, or repeating timing pattern.",
    },
  }),
  defineConceptProjections("tomb_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard strongly associated with tombs, barrows, mausoleums, crypts, or burial defenses.",
      appliesWhen: [
        "Burial sites, funerary wards, grave robbing countermeasures, or undead-rest protections are central to the hazard.",
        "A user would retrieve it for crypts, tombs, barrows, or similar funerary scenes.",
      ],
      doesNotApplyWhen: [
        "The hazard is only underground or haunted without real funerary or burial-site identity.",
        "The stronger placement is temple_hazard or generic dungeon_hazard.",
      ],
      adjacentTags: [
        "dungeon_hazard",
        "temple_hazard",
      ],
    },
  }),
  defineConceptProjections("tracking", {
    equipment: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps follow trails, mark a target, or relocate something later.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Locates a specific creature, object, or destination, or follows a supernatural trail toward it.",
      appliesWhen: [
        "The spell is naturally retrieved to find a named target, trace a quarry, or point the caster toward a specific creature, object, or place.",
        "Target location matters more than broad sensory surveillance or general route guidance.",
      ],
      doesNotApplyWhen: [
        "The spell mainly reveals an area, extends senses, or scouts without locking onto a specific target.",
        "The spell only helps orient a journey or choose a route once the destination is already known.",
      ],
      adjacentTags: [
        "scouting",
        "navigation",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("transformative_corruption", {
    affliction: {
      axis: "effect",
      family: "physiology_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Progressively transforms the body into crystal, plant matter, fungus, or another corrupted form.",
    },
  }),
  defineConceptProjections("translation_support", {
    equipment: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Bridges language barriers through translation, deciphering, script interpretation, or speech-understanding aids.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
      ],
      appliesWhen: [
        "The item's retrieval value comes from understanding foreign languages, translating speech, or decoding otherwise unreadable text or symbols.",
        "It is naturally sought when communication fails because of language barriers rather than distance or secrecy.",
      ],
      doesNotApplyWhen: [
        "The item only stores, relays, or broadcasts messages without solving comprehension.",
        "The item only provides psychic communication between already understood participants.",
      ],
      adjacentTags: [
        "telepathic_communication",
        "message_delivery",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Bridges spoken or written language barriers through translation, comprehension, deciphering, or magically shared understanding.",
      appliesWhen: [
        "The spell is naturally retrieved to understand, translate, or make oneself understood across otherwise incompatible languages or scripts.",
        "Language access matters more than merely sending a message or speaking silently.",
      ],
      doesNotApplyWhen: [
        "The spell only transmits content farther or more privately without solving a language barrier.",
        "The spell reveals truth, thoughts, or memories without actually translating speech or writing.",
      ],
      adjacentTags: [
        "telepathic_communication",
        "message_delivery",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("transport", {
    equipment: {
      axis: "utility",
      family: "movement_traversal",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps move creatures or cargo from place to place.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("trickster_mischief", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with pranks, capricious humor, gleeful sabotage, or explicit trickster behavior.",
      appliesWhen: [
        "Pranks, baiting humor, whimsical menace, or deliberate trickster conduct are a central retrieval hook.",
        "The creature is naturally sought for playful-but-dangerous chaos rather than generic destruction or villainy.",
      ],
      doesNotApplyWhen: [
        "The creature is merely chaotic, unpredictable, or destructive without a prankster or mischief-facing identity.",
        "Whimsical presentation appears only as surface flavor and another presentation tag or combat role better explains why the creature is being retrieved.",
      ],
      adjacentTags: [
        "carnival_show",
        "disguised_pretender",
      ],
    },
  }),
  defineConceptProjections("tripwire_trigger", {
    hazard: {
      axis: "mechanism",
      family: "mechanism",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard triggered by tugging, crossing, or disturbing a tripwire.",
    },
  }),
  defineConceptProjections("truth_compulsion", {
    affliction: {
      axis: "behavior",
      family: "behavioral_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Forces confession, honesty, or involuntary revelation against the victim's will.",
      appliesWhen: [
        "The affliction is naturally retrieved because victims are forced to confess, answer honestly, or reveal hidden information.",
        "Involuntary disclosure matters more than broad obedience, mood change, or self-harm.",
      ],
      doesNotApplyWhen: [
        "The affliction only compels action or speech generally without a truth-telling or confession-facing hook.",
        "The stronger fit is compulsion or cognitive_impairment rather than forced honesty.",
      ],
      adjacentTags: [
        "compulsion",
        "self_destructive_impulse",
      ],
    },
  }),
  defineConceptProjections("undead_family", {
    creature: {
      tag: "undead_adjacent",
      axis: "specialization",
      family: "ontology_cluster",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Groups undead and closely undead-coded native signals into one retrieval bucket.",
      nativeOntologyPolicy: "aggregates_native_signals",
    },
  }),
  defineConceptProjections("undead_war_torn_region_setting", {
    creature: {
      axis: "setting",
      family: "regional_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with undead occupation, shattered crusader lands, haunted battlefields, refugee pressure, and ruined strongholds under deathless siege. In Golarion, this primarily corresponds to the Gravelands.",
      appliesWhen: [
        "Use when the creature is naturally retrieved for undead war-frontier planning: ruined forts, haunted battlefields, occupied borderlands, refugee routes, or crusader-collapse aftermath.",
        "The planning value comes from a region still actively shaped by undead war, occupation, or collapse rather than by a stable undead social order.",
      ],
      doesNotApplyWhen: [
        "The creature is simply undead, martial, or grim without a meaningful tie to an undead-ravaged frontier or occupied war zone.",
        "The stronger fit is organized_undead_society_setting because the retrieval hook is necromantic civilization rather than a shattered war front.",
      ],
      adjacentTags: [
        "organized_undead_society_setting",
        "battlefield_setting",
      ],
    },
  }),
  defineConceptProjections("underground_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with caves, tunnels, crypts, or subterranean spaces.",
    },
  }),
  defineConceptProjections("urban_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard strongly associated with civic districts, alleys, rooftops, sewers, shops, or other built urban spaces.",
    },
  }),
  defineConceptProjections("urban_setting", {
    creature: {
      axis: "setting",
      family: "site_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with urban encounter scenes such as cities, streets, alleys, dense buildings, markets, or sewers.",
      appliesWhen: [
        "The creature is primarily framed as belonging in city or sewer encounter spaces.",
        "Urban placement is a recurring part of its identity, role, or habitat.",
      ],
      doesNotApplyWhen: [
        "The record merely mentions a city once.",
        "The creature can appear in towns but is not specifically urban-coded.",
        "The creature is better modeled by fortress_setting or small_settlement_setting.",
      ],
      positiveSignals: [
        "city guard roles",
        "sewer habitat",
        "street or alley patrol scenes",
        "market or district anchoring",
      ],
      negativeSignals: [
        "generic settlement mention",
        "single adventure location",
        "fortress-only residence",
      ],
      adjacentTags: [
        "small_settlement_setting",
        "fortress_setting",
      ],
    },
  }),
  defineConceptProjections("vengeful_tragedy", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with betrayal, grief, injustice, or a sorrowful wrong returning as vengeance.",
      appliesWhen: [
        "The creature is framed around betrayal, mourning, loss, or an unresolved wrong curdling into vengeance.",
        "A GM would retrieve it for tragic revenants, wronged spirits, or revenge-driven story beats rather than generic hostility.",
      ],
      doesNotApplyWhen: [
        "The creature is simply angry, hostile, or undead without a strong tragic-injustice presentation.",
        "The stronger fit is funerary_mourning because grief and ritual loss matter more than vengeance.",
      ],
      adjacentTags: [
        "funerary_mourning",
        "disguised_pretender",
      ],
    },
  }),
  defineConceptProjections("violence_compulsion", {
    affliction: {
      axis: "behavior",
      family: "behavioral_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Forces hostile aggression, murderous rage, or other attack-driven loss of self-control.",
      appliesWhen: [
        "The affliction is naturally retrieved because it drives victims to attack, maul, murder, or lash out at others.",
        "Violent outward aggression matters more than truthful speech, self-harm, or generic loss of agency.",
      ],
      doesNotApplyWhen: [
        "The affliction only makes the victim reckless, confused, or generally compelled without a real violence-forward pattern.",
        "The stronger fit is self_destructive_impulse or compulsion because aggression toward others is not central.",
      ],
      adjacentTags: [
        "compulsion",
        "self_destructive_impulse",
      ],
    },
  }),
  defineConceptProjections("void_soul_corruption", {
    affliction: {
      axis: "metaphysical",
      family: "metaphysical_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Attacks life force or the bond between body and soul through void or deathly corruption.",
    },
  }),
  defineConceptProjections("void_tainted", {
    creature: {
      axis: "specialization",
      family: "corruption_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creature strongly defined by void corruption, cosmic hollowness, or metaphysical pollution that feels alien, cold, or reality-thinning.",
      adjacentTags: [
        "cosmic_dread",
        "forbidden_knowledge",
      ],
    },
  }),
  defineConceptProjections("volcanic_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with volcanoes, calderas, lava, or magma.",
    },
  }),
  defineConceptProjections("warband_member", {
    creature: {
      axis: "encounter",
      family: "cohort_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Naturally retrieved as one body in a raiding party, battle line, war camp, or other organized hostile fighting band.",
      adjacentTags: [
        "commander_combatant",
        "battlefield_setting",
      ],
    },
  }),
  defineConceptProjections("ward_trigger", {
    hazard: {
      axis: "mechanism",
      family: "mechanism",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard triggered by a rune, glyph, sigil, ward, or similar inscribed mechanism.",
    },
  }),
  defineConceptProjections("wasteland_setting", {
    creature: {
      axis: "setting",
      family: "habitat_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with barren wastes, blasted wastelands, or desolate badlands.",
    },
  }),
  defineConceptProjections("wasting_hunger", {
    affliction: {
      axis: "effect",
      family: "physiology_override",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Imposes unnatural starvation, ravenous craving, or a consuming metabolic drive that degrades the body over time.",
    },
  }),
  defineConceptProjections("watcher_npc", {
    creature: {
      axis: "npc_role",
      family: "scene_role",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Immediate-scenario lookout, sentry, observer, or patrol-point watcher whose scene value is warning, spotting, or noticing intruders.",
      appliesWhen: [
        "The creature is naturally retrieved as a lookout, sentry, rooftop watcher, scout-on-post, or other early-warning presence.",
        "Observation and alarm value matter more than physically blocking passage, bodyguarding a charge, or acting as a general frontline enforcer.",
      ],
      doesNotApplyWhen: [
        "The creature is simply a posted guard or enforcer without a strong surveillance, early-warning, or lookout function.",
        "The stronger fit is guide_npc or civic_npc because travel knowledge or social embedding matters more than active watch duty.",
      ],
      adjacentTags: [
        "guardian_npc",
        "infiltrator_npc",
      ],
    },
  }),
  defineConceptProjections("waterborne_exposure", {
    affliction: {
      axis: "disease_model",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Spread through tainted water, drowning contact, flood exposure, or cursed immersion.",
      appliesWhen: [
        "Contaminated water, immersion, flood contact, or cursed liquid exposure is central to how victims contract the affliction.",
        "A user would retrieve it for tainted wells, river contagion, drowning curses, or similar water-linked spread.",
      ],
      doesNotApplyWhen: [
        "The affliction merely affects aquatic creatures without actually spreading through water contact.",
        "The stronger fit is inhaled_exposure, ingested_exposure, or carrier_vector instead of water-linked transmission.",
      ],
      adjacentTags: [
        "ingested_exposure",
        "carrier_vector",
      ],
    },
  }),
  defineConceptProjections("weapon_applied", {
    equipment: {
      axis: "effect",
      family: "delivery_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Offensive consumable applied to a weapon before use.",
      subcategories: [
        "consumable",
      ],
    },
  }),
  defineConceptProjections("weapon_staging", {
    equipment: {
      axis: "item_mechanical",
      family: "access_system",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Holsters, sheaths, scabbards, or bandoliers that stage weapons for quick draw or organized carry.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "ammo",
        "armor",
        "weapon",
      ],
    },
  }),
  defineConceptProjections("wilderness_hazard", {
    hazard: {
      axis: "setting",
      family: "setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard strongly associated with forests, trails, camps, ruins in the wild, or other outdoor expedition scenes.",
    },
  }),
  defineConceptProjections("writing_recordkeeping", {
    equipment: {
      axis: "utility",
      family: "crafting_support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports note-taking, mapmaking, copying text, archival work, or durable information storage.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "consumable",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("zone_denial", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that makes an area costly to enter, cross, or remain inside.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
];
