import type { DerivedTagCategoryProjection } from "../../../domain/derived-tag-types.js";

export const CREATURE_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG: Record<string, DerivedTagCategoryProjection> = 
{
  abaddon_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "abaddon_setting",
    currentTag: "abaddon_setting",
    description: "Strongly associated with Abaddon, daemons, or soul-devouring lower-planar wastelands.",
    family: "planar_setting",
    id: "creature:abaddon_setting",
    label: "abaddon_setting",
    nativeOntologyPolicy: "aggregates_native_signals",
    translationStatus: "provisional"
  },
  abyss_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "abyss_setting",
    currentTag: "abyss_setting",
    description: "Strongly associated with the Abyss, demon realms, or qlippoth-infested outer rifts.",
    family: "planar_setting",
    id: "creature:abyss_setting",
    label: "abyss_setting",
    translationStatus: "provisional"
  },
  alien_technology_wasteland_setting: {
    adjacentTags: [
      "magic_blight_wasteland_setting",
      "animated_object"
    ],
    appliesWhen: [
      "Use when the creature is naturally retrieved for weird-tech wilderness, star-metal ruins, robot-haunted badlands, mutant frontiers, or barbarian-meets-super-science planning.",
      "The planning value comes from science-fantasy intrusion into the region's creature ecology rather than only from a single construct or technological gimmick."
    ],
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "alien_technology_wasteland_setting",
    currentTag: "alien_technology_wasteland_setting",
    description: "Strongly associated with alien technology, robots, mutants, strange metal ruins, and science-fantasy wastelands. In Golarion, this primarily corresponds to Numeria.",
    doesNotApplyWhen: [
      "The creature only uses one technological item or construct-like ability without a broader weird-tech regional frame.",
      "The stronger fit is bound_object or a combat-role tag because the planning value is tactical rather than regional or thematic."
    ],
    family: "regional_setting",
    id: "creature:alien_technology_wasteland_setting",
    label: "alien_technology_wasteland_setting",
    translationStatus: "mapped"
  },
  ambush_grabber: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "ambush_grab",
    currentTag: "ambush_grabber",
    description: "Captures prey through grabbing, constriction, swallowing whole, webbing, or drag-off ambush tactics.",
    family: "threat_profile",
    id: "creature:ambush_grabber",
    label: "ambush_grab",
    translationStatus: "mapped"
  },
  ambusher_combatant: {
    adjacentTags: [
      "skirmisher_combatant",
      "brute_combatant"
    ],
    appliesWhen: [
      "The creature is naturally retrieved for hidden approach, surprise attack, trapdoor positioning, or burst from concealment.",
      "Opening from stealth is central to how it functions in an encounter."
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "ambusher_combatant",
    currentTag: "ambusher_combatant",
    description: "Built around stealth openings, surprise rounds, sudden strike pressure, or hidden attack vectors.",
    doesNotApplyWhen: [
      "The creature only uses stealth incidentally before acting as another clearer combat role.",
      "The stronger identity is skirmisher_combatant or artillery_combatant rather than surprise-predator play."
    ],
    family: "combat_role",
    id: "creature:ambusher_combatant",
    label: "ambusher_combatant",
    translationStatus: "mapped"
  },
  ancestral_legacy: {
    adjacentTags: [
      "prophecy_omen",
      "courtly_pageantry"
    ],
    appliesWhen: [
      "The creature's presentation depends on lineage, inheritance, dynastic fate, ancestral memory, or family burden carried into the present.",
      "Retrieval value comes from inherited role, curse, or legacy rather than only social rank or prophecy."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "ancestral_legacy",
    currentTag: "ancestral_legacy",
    description: "Strongly associated with bloodline burdens, inherited duty, dynastic memory, haunting lineage, or the weight of family legacy.",
    doesNotApplyWhen: [
      "The creature merely belongs to a noble house, species line, or ancestry without legacy pressure being central.",
      "The stronger fit is prophecy_omen or courtly_pageantry because omen-bearing destiny or aristocratic presentation matters more than inherited burden."
    ],
    family: "story_motif",
    id: "creature:ancestral_legacy",
    label: "ancestral_legacy",
    translationStatus: "mapped"
  },
  animated_object: {
    assignmentMode: "hybrid",
    axis: "presentation",
    category: "creature",
    conceptId: "animated_object",
    currentTag: "animated_object",
    description: "Strongly associated with animated objects, furniture, tools, or other constructed items.",
    family: "bound_object",
    id: "creature:animated_object",
    label: "animated_object",
    translationStatus: "mapped"
  },
  animated_statue: {
    assignmentMode: "hybrid",
    axis: "presentation",
    category: "creature",
    conceptId: "animated_statue",
    currentTag: "animated_statue",
    description: "Strongly associated with animated statues, effigies, idols, or monuments.",
    family: "bound_object",
    id: "creature:animated_statue",
    label: "animated_statue",
    translationStatus: "mapped"
  },
  apocalypse_ruin: {
    adjacentTags: [
      "cosmic_dread",
      "prophecy_omen"
    ],
    appliesWhen: [
      "The creature evokes cataclysm, final collapse, prophesied ending, or ruin on a society-shaping scale.",
      "A GM would retrieve it for last-days storytelling, omens of collapse, or world-unmaking scenes rather than only for big threat level."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "apocalypse_ruin",
    currentTag: "apocalypse_ruin",
    description: "Strongly associated with end-times, civilizational collapse, world-ending omen, or the sense that the creature heralds broad unraveling.",
    doesNotApplyWhen: [
      "The creature is simply powerful, destructive, or extraplanar without a real end-times or collapse-of-order presentation.",
      "The stronger fit is cosmic_dread or prophecy_omen because existential scale or foretold signs matter more than ruin itself."
    ],
    family: "story_motif",
    id: "creature:apocalypse_ruin",
    label: "apocalypse_ruin",
    translationStatus: "mapped"
  },
  aquatic_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "aquatic_setting",
    currentTag: "aquatic_setting",
    description: "Strongly associated with open water, underwater spaces, or aquatic environments.",
    family: "habitat_setting",
    id: "creature:aquatic_setting",
    label: "aquatic_setting",
    translationStatus: "mapped"
  },
  arcane_spellcaster: {
    assignmentMode: "hybrid",
    axis: "specialization",
    category: "creature",
    conceptId: "arcane_spellcaster",
    currentTag: "arcane_spellcaster",
    description: "Creature whose spellcasting is substantially framed through arcane traditions, wizardry, runes, or similarly arcane technique.",
    family: "casting_profile",
    id: "creature:arcane_spellcaster",
    label: "arcane_spellcaster",
    translationStatus: "mapped"
  },
  arctic_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "arctic_setting",
    currentTag: "arctic_setting",
    description: "Strongly associated with snow, ice, tundra, or frozen reaches.",
    family: "habitat_setting",
    id: "creature:arctic_setting",
    label: "arctic_setting",
    translationStatus: "mapped"
  },
  artillery_combatant: {
    adjacentTags: [
      "harrier_combatant",
      "controller_combatant",
      "support_combatant"
    ],
    appliesWhen: [
      "The creature is naturally retrieved as a ranged damage or spell-barrage threat that prefers distance.",
      "Its tactical identity is standoff pressure more than command, mobility, or pure control."
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "artillery_combatant",
    currentTag: "artillery_combatant",
    description: "Built to pressure targets from range through volleys, spell barrages, breath attacks, or other standoff offense.",
    doesNotApplyWhen: [
      "The creature happens to have one ranged option but still fundamentally plays as a brute, controller, or support piece.",
      "Its main retrieval hook is casting support or battlefield control rather than ranged offense."
    ],
    family: "combat_role",
    id: "creature:artillery_combatant",
    label: "artillery_combatant",
    translationStatus: "mapped"
  },
  artisan_npc: {
    adjacentTags: [
      "merchant_npc",
      "profession_npc"
    ],
    appliesWhen: [
      "Craft production, repair labor, or skilled making work is central to the creature's world-facing identity.",
      "A GM would plausibly retrieve it as a blacksmith, mason, tailor, shipwright, or similarly maker-facing contact."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "artisan_npc",
    currentTag: "artisan_npc",
    description: "Presented as a smith, craftsperson, builder, artisan, or other maker-facing role-holder tied to production or skilled labor.",
    doesNotApplyWhen: [
      "The creature only sells goods or manages trade without being defined by making or repair work.",
      "The stronger fit is merchant_npc, profession_npc, or civic_npc without a clear craft-labor role."
    ],
    family: "social_role",
    id: "creature:artisan_npc",
    label: "artisan_npc",
    translationStatus: "mapped"
  },
  astral_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "astral_setting",
    currentTag: "astral_setting",
    description: "Strongly associated with Astral Plane scenes, silver-void travel, or stable portal routes.",
    family: "planar_setting",
    id: "creature:astral_setting",
    label: "astral_setting",
    translationStatus: "provisional"
  },
  authority_npc: {
    adjacentTags: [
      "profession_npc",
      "civic_npc"
    ],
    appliesWhen: [
      "The creature's retrieval value comes from official office, rank, command, or institutional authority.",
      "A GM would plausibly seek it as a leader, official, or governing figure.",
      "Formal office or rank is the main retrieval hook, even if the creature also serves as a civic_npc or enforcer_npc in the scene."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "authority_npc",
    currentTag: "authority_npc",
    description: "Presented as an officer, magistrate, noble, administrator, or other figure of formal social authority.",
    doesNotApplyWhen: [
      "The record is only a generic combatant without meaningful office or status.",
      "The stronger fit is profession_npc, civic_npc, or enforcer_npc because status is incidental."
    ],
    family: "social_role",
    id: "creature:authority_npc",
    label: "authority_npc",
    translationStatus: "mapped"
  },
  axis_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "axis_setting",
    currentTag: "axis_setting",
    description: "Strongly associated with Axis, the Eternal City, or its lawful planar order.",
    family: "planar_setting",
    id: "creature:axis_setting",
    label: "axis_setting",
    translationStatus: "provisional"
  },
  battlefield_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "battlefield_setting",
    currentTag: "battlefield_setting",
    description: "Strongly associated with battlefields, war zones, organized military deployments, or mass-combat scenes.",
    family: "site_setting",
    id: "creature:battlefield_setting",
    label: "battlefield_setting",
    translationStatus: "mapped"
  },
  blight_tainted: {
    adjacentTags: [
      "wasteland_setting",
      "body_horror"
    ],
    assignmentMode: "hybrid",
    axis: "specialization",
    category: "creature",
    conceptId: "blight_tainted",
    currentTag: "blight_tainted",
    description: "Creature strongly defined by ecological blight, withering nature, corrupted groves, or land-sickened wilderness.",
    family: "corruption_profile",
    id: "creature:blight_tainted",
    label: "blight_tainted",
    translationStatus: "mapped"
  },
  body_horror: {
    adjacentTags: [
      "stitched_horror",
      "disease_vector"
    ],
    appliesWhen: [
      "Distorted flesh, invasive alteration, exposed anatomy, or grotesque physical transformation is central to the creature's horror identity.",
      "A GM would retrieve the creature specifically for visceral corruption, mutation, or flesh-warp scenes."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "body_horror",
    currentTag: "body_horror",
    description: "Strongly associated with warped anatomy, invasive flesh transformation, surgical grotesquerie, or visceral physical corruption.",
    doesNotApplyWhen: [
      "The creature is merely monstrous, bloody, or physically powerful without a strong corruption-of-the-body motif.",
      "The stronger fit is stitched_horror or disease_vector because constructed patchwork or infection aftermath matters more than bodily grotesquerie as presentation."
    ],
    family: "genre_motif",
    id: "creature:body_horror",
    label: "body_horror",
    translationStatus: "mapped"
  },
  boneyard_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "boneyard_setting",
    currentTag: "boneyard_setting",
    description: "Strongly associated with the Boneyard, psychopomp duties, or soul-processing afterlife scenes.",
    family: "planar_setting",
    id: "creature:boneyard_setting",
    label: "boneyard_setting",
    translationStatus: "provisional"
  },
  brute_combatant: {
    adjacentTags: [
      "defender_combatant",
      "artillery_combatant"
    ],
    appliesWhen: [
      "The creature is naturally retrieved as a heavy hitter that advances, endures punishment, and threatens through direct force.",
      "Its tactical identity is more about sturdy pressure than mobility, command, or precision control."
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "brute_combatant",
    currentTag: "brute_combatant",
    description: "Built to pressure the front line through durability, direct damage, and straightforward melee threat.",
    doesNotApplyWhen: [
      "The creature mainly operates from range, through battlefield control, or through ally support.",
      "High damage is present but the stronger combat identity is ambush, artillery, or commander play."
    ],
    family: "combat_role",
    id: "creature:brute_combatant",
    label: "brute_combatant",
    translationStatus: "mapped"
  },
  canyon_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "canyon_setting",
    currentTag: "canyon_setting",
    description: "Strongly associated with canyons, gorges, mesas, or badlands.",
    family: "habitat_setting",
    id: "creature:canyon_setting",
    label: "canyon_setting",
    translationStatus: "mapped"
  },
  captive_npc: {
    adjacentTags: [
      "guardian_npc",
      "civic_npc"
    ],
    appliesWhen: [
      "The creature is naturally retrieved because it is held, imprisoned, restrained, threatened, or otherwise scenically constrained.",
      "Its immediate scenario function is being rescued, questioned, transported, or guarded rather than acting freely in the scene."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "captive_npc",
    currentTag: "captive_npc",
    description: "Immediate-scenario prisoner, hostage, detained witness, sacrifice target, or other constrained figure whose scene value comes from being held or imperiled.",
    doesNotApplyWhen: [
      "The creature is merely vulnerable, socially subordinate, or under pressure without actually being a captive or detained figure.",
      "The stronger fit is civic_npc or profession_npc because social embeddedness or world role matters more than captivity."
    ],
    family: "scene_role",
    id: "creature:captive_npc",
    label: "captive_npc",
    translationStatus: "mapped"
  },
  carnival_show: {
    assignmentMode: "hybrid",
    axis: "presentation",
    category: "creature",
    conceptId: "carnival_show",
    currentTag: "carnival_show",
    description: "Strongly associated with carnivals, circuses, clowns, jesters, or sideshow-style presentation.",
    family: "genre_motif",
    id: "creature:carnival_show",
    label: "carnival_show",
    translationStatus: "mapped"
  },
  civic_npc: {
    adjacentTags: [
      "profession_npc",
      "enforcer_npc"
    ],
    appliesWhen: [
      "The creature belongs to the civic, domestic, institutional, or everyday social fabric of the scene.",
      "A GM would plausibly retrieve it as a socially embedded NPC rather than as a combat-forward foe.",
      "This tag answers the creature's immediate scene slot, even if a separate social_role tag explains its profession or office."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "civic_npc",
    currentTag: "civic_npc",
    description: "Fits the civic or social fabric of a scene and usually isn't the primary monster answer.",
    doesNotApplyWhen: [
      "The record is primarily a hostile combatant or raider.",
      "The creature is only profession-labeled without clear social embeddedness, or the profession or office itself is the main retrieval hook."
    ],
    family: "scene_role",
    id: "creature:civic_npc",
    label: "civic_npc",
    translationStatus: "mapped"
  },
  coastal_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "coastal_setting",
    currentTag: "coastal_setting",
    description: "Strongly associated with coasts, shores, reefs, or littoral edges.",
    family: "habitat_setting",
    id: "creature:coastal_setting",
    label: "coastal_setting",
    translationStatus: "mapped"
  },
  commander_combatant: {
    adjacentTags: [
      "support_combatant",
      "reinforcement_threat"
    ],
    appliesWhen: [
      "The creature is naturally retrieved because its round-to-round battle role is directing allies, calling tactics, or coordinating a group.",
      "Leadership and coordination matter more than its own solo offense, brute durability, or passive aura support."
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "commander_combatant",
    currentTag: "commander_combatant",
    description: "Built to coordinate allies round to round through leadership, tactics, or command-driven positioning.",
    doesNotApplyWhen: [
      "The creature only has generic support effects with no clear command or tactical leadership role in the moment-to-moment fight.",
      "The stronger fit is support_combatant, artillery_combatant, or reinforcement_threat rather than leader play."
    ],
    family: "combat_role",
    id: "creature:commander_combatant",
    label: "commander_combatant",
    translationStatus: "mapped"
  },
  controller_combatant: {
    adjacentTags: [
      "support_combatant",
      "artillery_combatant"
    ],
    appliesWhen: [
      "The creature is naturally retrieved for immobilizing, repositioning, walling off, slowing, or otherwise dictating battlefield shape.",
      "Tactical denial matters more than direct damage output."
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "controller_combatant",
    currentTag: "controller_combatant",
    description: "Built to reshape the battlefield through debuffs, forced movement, terrain control, or other tactical denial.",
    doesNotApplyWhen: [
      "The creature only has one incidental debuff while otherwise fighting as a brute, defender, or artillery piece.",
      "The stronger identity is support or commander rather than enemy-space control."
    ],
    family: "combat_role",
    id: "creature:controller_combatant",
    label: "controller_combatant",
    translationStatus: "mapped"
  },
  corrupted_sacred: {
    adjacentTags: [
      "ritual_ceremony",
      "religious_npc"
    ],
    appliesWhen: [
      "The creature's presentation depends on desecrated ritual, fallen holiness, saintly imagery gone wrong, or sanctity turned threatening.",
      "Retrieval value comes from sacred symbolism being violated, inverted, or corrupted."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "corrupted_sacred",
    currentTag: "corrupted_sacred",
    description: "Strongly associated with profaned sanctity, fallen holiness, blasphemous devotion, or sacred imagery twisted into menace.",
    doesNotApplyWhen: [
      "The creature is merely evil, undead, or hostile in a temple without sacred corruption being central to its identity.",
      "The stronger fit is ritual_ceremony or religious_npc because the creature is ceremonial or clerical without profaned sanctity."
    ],
    family: "story_motif",
    id: "creature:corrupted_sacred",
    label: "corrupted_sacred",
    translationStatus: "mapped"
  },
  cosmic_dread: {
    adjacentTags: [
      "astral_setting",
      "dream_nightmare"
    ],
    appliesWhen: [
      "The creature evokes existential terror, starry abyssal scale, unknowable revelation, or the feeling of minds breaking before the universe.",
      "A GM would plausibly retrieve it for eldritch omen, cosmic terror, or revelation-of-the-void scenes."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "cosmic_dread",
    currentTag: "cosmic_dread",
    description: "Strongly associated with void vastness, star-born dread, incomprehensible revelation, or insignificance before the cosmos.",
    doesNotApplyWhen: [
      "The creature is merely alien, aberrant, or extraplanar without a strong cosmic-horror presentation.",
      "Astral or outer-planar placement alone is better captured by setting tags."
    ],
    family: "genre_motif",
    id: "creature:cosmic_dread",
    label: "cosmic_dread",
    translationStatus: "mapped"
  },
  cosmic_framework_setting: {
    adjacentTags: [
      "axis_setting",
      "boneyard_setting",
      "maelstrom_setting"
    ],
    assignmentMode: "composite",
    axis: "setting",
    category: "creature",
    compositeOfAnyTags: [
      "axis_setting",
      "boneyard_setting",
      "maelstrom_setting"
    ],
    conceptId: "cosmic_framework_setting",
    currentTag: "cosmic_framework_setting",
    description: "Strongly associated with the cosmic framework planes of Axis, the Boneyard, and the Maelstrom, which govern order, judgment, and transformative change.",
    family: "planar_setting",
    id: "creature:cosmic_framework_setting",
    label: "cosmic_framework_setting",
    nativeOntologyPolicy: "aggregates_native_signals",
    translationStatus: "provisional"
  },
  court_entourage: {
    adjacentTags: [
      "authority_npc",
      "courtly_pageantry"
    ],
    assignmentMode: "editorial",
    axis: "encounter",
    category: "creature",
    conceptId: "court_entourage",
    currentTag: "court_entourage",
    description: "Naturally retrieved as part of a courtly retinue, noble household, ceremonial train, or political chamber roster around a central authority figure.",
    family: "cohort_role",
    id: "creature:court_entourage",
    label: "court_entourage",
    translationStatus: "mapped"
  },
  courtly_pageantry: {
    adjacentTags: [
      "authority_npc",
      "mask_motif"
    ],
    appliesWhen: [
      "Court spectacle, heraldic pomp, ballroom tension, formal procession, or aristocratic display is central to the creature's presentation.",
      "A GM would plausibly retrieve the creature for palace intrigue, masquerades, or noble ceremonial scenes rather than for office alone."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "courtly_pageantry",
    currentTag: "courtly_pageantry",
    description: "Strongly associated with nobles, heraldry, formal spectacle, masquerade grandeur, or ceremonial court presentation.",
    doesNotApplyWhen: [
      "The creature merely holds rank or authority without strong pageantry, splendor, or court-scene presentation.",
      "The stronger fit is authority_npc or performer_npc rather than courtly spectacle."
    ],
    family: "story_motif",
    id: "creature:courtly_pageantry",
    label: "courtly_pageantry",
    translationStatus: "mapped"
  },
  crew_member: {
    adjacentTags: [
      "nautical_setting",
      "escort_npc"
    ],
    assignmentMode: "editorial",
    axis: "encounter",
    category: "creature",
    conceptId: "crew_member",
    currentTag: "crew_member",
    description: "Naturally retrieved as part of a ship crew, dock crew, wreck complement, or other nautical working roster that belongs together in one scene.",
    family: "cohort_role",
    id: "creature:crew_member",
    label: "crew_member",
    translationStatus: "mapped"
  },
  criminal_cell: {
    adjacentTags: [
      "criminal_npc",
      "infiltrator_npc"
    ],
    assignmentMode: "editorial",
    axis: "encounter",
    category: "creature",
    conceptId: "criminal_cell",
    currentTag: "criminal_cell",
    description: "Naturally retrieved as part of a gang, smuggling ring, burglary team, or other underworld roster that functions through a small coordinated cell.",
    family: "cohort_role",
    id: "creature:criminal_cell",
    label: "criminal_cell",
    translationStatus: "mapped"
  },
  criminal_npc: {
    adjacentTags: [
      "enforcer_npc",
      "infiltrator_npc"
    ],
    appliesWhen: [
      "The creature is defined by illicit trade, organized crime, covert violence, or other criminal social function.",
      "A GM would retrieve it as an underworld contact or criminal adversary rather than a generic soldier."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "criminal_npc",
    currentTag: "criminal_npc",
    description: "Presented as a thief, smuggler, assassin, gang operative, fence, or other explicitly underworld-coded role.",
    doesNotApplyWhen: [
      "The creature is merely hostile without underworld or crime-scene framing.",
      "The stronger fit is enforcer_npc without a distinct criminal identity."
    ],
    family: "social_role",
    id: "creature:criminal_npc",
    label: "criminal_npc",
    translationStatus: "mapped"
  },
  cult_member: {
    adjacentTags: [
      "religious_npc",
      "ritualist_creature"
    ],
    assignmentMode: "editorial",
    axis: "encounter",
    category: "creature",
    conceptId: "cult_member",
    currentTag: "cult_member",
    description: "Naturally retrieved as one member of a ritual circle, hidden sect, temple conspiracy, or other cultic roster built around shared devotion or doctrine.",
    family: "cohort_role",
    id: "creature:cult_member",
    label: "cult_member",
    translationStatus: "mapped"
  },
  curse_threat: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "curse_application",
    currentTag: "curse_threat",
    description: "Threat defined by curses, doom effects, or other lingering supernatural afflictions imposed on victims.",
    family: "threat_profile",
    id: "creature:curse_threat",
    label: "curse_application",
    translationStatus: "mapped"
  },
  cursed_transformation: {
    adjacentTags: [
      "body_horror",
      "ancestral_legacy"
    ],
    appliesWhen: [
      "The creature's story identity centers on an afflicting change, monstrous becoming, or transformation that threatens personhood.",
      "A GM would retrieve it for cursed metamorphosis, bestial change, or body-and-identity corruption rather than only raw mutation."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "cursed_transformation",
    currentTag: "cursed_transformation",
    description: "Strongly associated with involuntary metamorphosis, curse-driven loss of self, or identity eroded by becoming something else.",
    doesNotApplyWhen: [
      "The creature merely transforms, shapeshifts, or mutates as an ability without cursed or tragic transformation being the presentation hook.",
      "The stronger fit is body_horror or disguised_pretender because physical grotesquerie or impersonation matters more than involuntary becoming."
    ],
    family: "story_motif",
    id: "creature:cursed_transformation",
    label: "cursed_transformation",
    translationStatus: "mapped"
  },
  cursewarped: {
    adjacentTags: [
      "curse_threat",
      "cursed_transformation"
    ],
    assignmentMode: "hybrid",
    axis: "specialization",
    category: "creature",
    conceptId: "cursewarped",
    currentTag: "cursewarped",
    description: "Creature strongly defined by curse-driven distortion, doom-warping, or being transformed into its current state by a curse.",
    family: "corruption_profile",
    id: "creature:cursewarped",
    label: "cursewarped",
    translationStatus: "mapped"
  },
  darklands_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "darklands_setting",
    currentTag: "darklands_setting",
    description: "Strongly associated with the Darklands as a civilization-bearing underworld macro-region rather than generic underground terrain or one cave network.",
    family: "regional_setting",
    id: "creature:darklands_setting",
    label: "darklands_setting",
    translationStatus: "mapped"
  },
  death_burst_threat: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "death_burst",
    currentTag: "death_burst_threat",
    description: "Threat defined by explosive death effects, cursed aftermath, or punishing consequences when the creature is dropped.",
    family: "threat_profile",
    id: "creature:death_burst_threat",
    label: "death_burst",
    translationStatus: "mapped"
  },
  decadence_decline: {
    adjacentTags: [
      "courtly_pageantry",
      "revelry_excess"
    ],
    appliesWhen: [
      "The creature evokes indulgent splendor gone rotten, noble decline, decadent excess, or crumbling beauty concealing corruption.",
      "A GM would plausibly retrieve it for decaying courts, ruined salons, or luxury-turned-horror scenes."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "decadence_decline",
    currentTag: "decadence_decline",
    description: "Strongly associated with faded luxury, aristocratic rot, opulent ruin, or beauty collapsing into moral and material decay.",
    doesNotApplyWhen: [
      "The creature is merely rich, noble, or associated with ruins without a clear decline-and-rot presentation.",
      "The stronger fit is courtly_pageantry or revelry_excess because spectacle or celebration matters more than decay."
    ],
    family: "story_motif",
    id: "creature:decadence_decline",
    label: "decadence_decline",
    translationStatus: "mapped"
  },
  defender_combatant: {
    adjacentTags: [
      "brute_combatant",
      "commander_combatant"
    ],
    appliesWhen: [
      "The creature is naturally retrieved for guarding chokepoints, bodyguarding allies, or making passage costly.",
      "Space-holding matters more than raw pursuit, burst damage, or command."
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "defender_combatant",
    currentTag: "defender_combatant",
    description: "Built to hold space, intercept attacks, bodyguard allies, or punish passage through a defended line.",
    doesNotApplyWhen: [
      "The creature is merely durable without a real guard, intercept, or line-holding identity.",
      "The stronger fit is brute_combatant or another non-guardian role."
    ],
    family: "combat_role",
    id: "creature:defender_combatant",
    label: "defender_combatant",
    translationStatus: "mapped"
  },
  demonic_scar_frontier_setting: {
    adjacentTags: [
      "battlefield_setting",
      "void_tainted"
    ],
    appliesWhen: [
      "Use when the creature is naturally retrieved for post-invasion frontier planning: planar scars, reclaimed sacred sites, demon-torn wastelands, lingering abyssal corruption, or survival after a world-rending incursion.",
      "The planning value comes from a land still shaped by demonic devastation and reclamation rather than from generic fiendish wilderness or one demonic trait."
    ],
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "demonic_scar_frontier_setting",
    currentTag: "demonic_scar_frontier_setting",
    description: "Strongly associated with demon-scarred frontiers, reclaimed homelands, abyss-tainted wilderness, and recovery after planar catastrophe. In Golarion, this primarily corresponds to the Sarkoris Scar.",
    doesNotApplyWhen: [
      "The creature is merely demonic, corrupted, or extraplanar without a real frontier-of-reclamation regional frame.",
      "The stronger fit is planar_setting or corruption_profile because the retrieval hook is cosmological origin or body-horror taint rather than a scarred regional frontier."
    ],
    family: "regional_setting",
    id: "creature:demonic_scar_frontier_setting",
    label: "demonic_scar_frontier_setting",
    translationStatus: "mapped"
  },
  desert_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "desert_setting",
    currentTag: "desert_setting",
    description: "Strongly associated with dunes, sand, or arid wastes.",
    family: "habitat_setting",
    id: "creature:desert_setting",
    label: "desert_setting",
    translationStatus: "mapped"
  },
  disease_vector: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "disease_application",
    currentTag: "disease_vector",
    description: "Threat defined by spreading disease, curse-plague conditions, or infectious aftermath beyond immediate damage.",
    family: "threat_profile",
    id: "creature:disease_vector",
    label: "disease_application",
    translationStatus: "mapped"
  },
  disguised_pretender: {
    adjacentTags: [
      "faceless_horror",
      "mask_motif"
    ],
    appliesWhen: [
      "Impersonation, replacement, disguise, or false identity is a central retrieval hook.",
      "The creature is framed around passing as someone or something else."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "disguised_pretender",
    currentTag: "disguised_pretender",
    description: "Strongly associated with false identities, impersonation, infiltration, shapeshifting, or replacement.",
    doesNotApplyWhen: [
      "It merely uses stealth or deception without identity substitution.",
      "The stronger presentation tag is mask_motif or faceless_horror rather than impersonation."
    ],
    family: "story_motif",
    id: "creature:disguised_pretender",
    label: "disguised_pretender",
    translationStatus: "mapped"
  },
  divine_spellcaster: {
    assignmentMode: "hybrid",
    axis: "specialization",
    category: "creature",
    conceptId: "divine_spellcaster",
    currentTag: "divine_spellcaster",
    description: "Creature whose spellcasting is substantially framed through divine prayer, sacred miracles, or deity-facing magic.",
    family: "casting_profile",
    id: "creature:divine_spellcaster",
    label: "divine_spellcaster",
    translationStatus: "mapped"
  },
  dragon_spellcaster: {
    adjacentTags: [
      "arcane_spellcaster",
      "ritualist_creature"
    ],
    appliesWhen: [
      "Use when a dragon or archdragon variant explicitly presents meaningful spellcasting as part of its encounter identity.",
      "The spellcasting matters for prep and counterplay beyond incidental magical flavor."
    ],
    assignmentMode: "hybrid",
    axis: "specialization",
    category: "creature",
    conceptId: "dragon_spellcaster",
    currentTag: "dragon_spellcaster",
    description: "Dragon or archdragon variant with an explicit spellcaster stat block or named spellcaster presentation.",
    doesNotApplyWhen: [
      "The dragon only has innate magical flavor, one-off magical actions, or a few utility effects without real spellcaster framing.",
      "The stronger fit is only a tradition-specific spellcaster tag without a dragon-specific spellcaster presentation."
    ],
    family: "casting_profile",
    id: "creature:dragon_spellcaster",
    label: "dragon_spellcaster",
    translationStatus: "mapped"
  },
  dream_nightmare: {
    adjacentTags: [
      "dreamlands_setting",
      "cosmic_dread"
    ],
    appliesWhen: [
      "Dream logic, nightmare intrusion, sleep visitation, or surreal unreality is central to the creature's story identity.",
      "A GM would plausibly retrieve the creature for dreamscapes, night terrors, or oneiric scenes even outside a literal Dreamlands setting."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "dream_nightmare",
    currentTag: "dream_nightmare",
    description: "Strongly associated with dreams, nightmares, sleep-haunting, surreal unreality, or subconscious dread.",
    doesNotApplyWhen: [
      "The creature merely casts sleep or fear effects without a real dream or nightmare presentation theme.",
      "Dreamlands placement alone is better captured by dreamlands_setting."
    ],
    family: "genre_motif",
    id: "creature:dream_nightmare",
    label: "dream_nightmare",
    translationStatus: "mapped"
  },
  dreamlands_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "dreamlands_setting",
    currentTag: "dreamlands_setting",
    description: "Strongly associated with the Dreamlands, Leng-linked dream roads, or iconic denizens that dwell there.",
    family: "planar_setting",
    id: "creature:dreamlands_setting",
    label: "dreamlands_setting",
    translationStatus: "provisional"
  },
  elemental_plane_setting: {
    adjacentTags: [
      "plane_of_fire_setting",
      "plane_of_air_setting",
      "plane_of_water_setting",
      "plane_of_earth_setting"
    ],
    assignmentMode: "composite",
    axis: "setting",
    category: "creature",
    compositeOfAnyTags: [
      "plane_of_fire_setting",
      "plane_of_air_setting",
      "plane_of_water_setting",
      "plane_of_earth_setting"
    ],
    conceptId: "elemental_plane_setting",
    currentTag: "elemental_plane_setting",
    description: "Strongly associated with one of the elemental planes of Fire, Air, Water, or Earth.",
    family: "planar_setting",
    id: "creature:elemental_plane_setting",
    label: "elemental_plane_setting",
    nativeOntologyPolicy: "aggregates_native_signals",
    translationStatus: "provisional"
  },
  elysium_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "elysium_setting",
    currentTag: "elysium_setting",
    description: "Strongly associated with Elysium, azatas, or freedom-driven celestial heroism.",
    family: "planar_setting",
    id: "creature:elysium_setting",
    label: "elysium_setting",
    translationStatus: "provisional"
  },
  enforcer_npc: {
    adjacentTags: [
      "authority_npc",
      "civic_npc"
    ],
    appliesWhen: [
      "The creature is presented as a role-defined humanoid adversary whose immediate scene value is direct martial pressure, enforcement, or armed opposition.",
      "Retrieval value comes from being a fight-first scene answer rather than a civic, social, or watch-post role.",
      "This tag answers how the creature functions in the immediate scene, even if it separately has a profession, rank, or institutional role."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "enforcer_npc",
    currentTag: "enforcer_npc",
    description: "Scene-slot fight-first humanoid adversary such as a soldier, bandit, mercenary, or other overt martial enforcer.",
    doesNotApplyWhen: [
      "The record is mainly a social, occupational, escort, or posted-guard NPC whose scene identity is not primarily direct armed opposition.",
      "The creature is better captured by profession_npc, civic_npc, guardian_npc, or watcher_npc without a strong fight-first role.",
      "The office, profession, surveillance role, or posted protection duty is the stronger retrieval hook and combat readiness is only secondary."
    ],
    family: "scene_role",
    id: "creature:enforcer_npc",
    label: "enforcer_npc",
    translationStatus: "mapped"
  },
  escort_npc: {
    adjacentTags: [
      "guide_npc",
      "guardian_npc"
    ],
    appliesWhen: [
      "The creature is naturally retrieved because it actively escorts, transports, shepherds, or accompanies another figure through a dangerous scene or route.",
      "Its scene function is movement-with-charge or safe transit rather than only office, profession, or posted guard duty."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "escort_npc",
    currentTag: "escort_npc",
    description: "Immediate-scenario escort, courier companion, guide-on-mission, ward mover, or other figure whose scene value is accompanying or moving someone through danger.",
    doesNotApplyWhen: [
      "The creature merely knows the route, provides directions, or protects a place without actively accompanying someone.",
      "The stronger fit is guide_npc or guardian_npc because route expertise or posted guard duty matters more than accompaniment."
    ],
    family: "scene_role",
    id: "creature:escort_npc",
    label: "escort_npc",
    translationStatus: "mapped"
  },
  ethereal_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "ethereal_setting",
    currentTag: "ethereal_setting",
    description: "Strongly associated with the Ethereal Plane, its native wildlife, or recurring hunting and travel routes through it.",
    family: "planar_setting",
    id: "creature:ethereal_setting",
    label: "ethereal_setting",
    translationStatus: "provisional"
  },
  faceless_horror: {
    adjacentTags: [
      "disguised_pretender",
      "mask_motif"
    ],
    appliesWhen: [
      "Missing, stolen, hidden, or featureless faces are central to the creature's horror identity.",
      "Face absence or erasure is a recurring motif, not just a disguise tactic."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "faceless_horror",
    currentTag: "faceless_horror",
    description: "Strongly associated with missing, hidden, stolen, or featureless faces.",
    doesNotApplyWhen: [
      "The creature is mainly about impersonation or disguise rather than facial absence.",
      "A covered face appears only as costume or gear."
    ],
    family: "visual_motif",
    id: "creature:faceless_horror",
    label: "faceless_horror",
    translationStatus: "mapped"
  },
  first_world_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "first_world_setting",
    currentTag: "first_world_setting",
    description: "Strongly associated with the First World, fey realms, or thin-boundary crossings into that plane.",
    family: "planar_setting",
    id: "creature:first_world_setting",
    label: "first_world_setting",
    translationStatus: "provisional"
  },
  folk_horror: {
    adjacentTags: [
      "rural_setting",
      "funerary_mourning"
    ],
    appliesWhen: [
      "The creature evokes old-country fear, harvest rites gone wrong, scarecrow dread, witchcraft omen, or taboo-laden local folklore.",
      "Its retrieval value comes from uncanny communal belief and traditional dread, not only from being outdoors or rural."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "folk_horror",
    currentTag: "folk_horror",
    description: "Strongly associated with rural superstition, old customs, harvest dread, village taboos, or uncanny folklore menace.",
    doesNotApplyWhen: [
      "The creature is merely found in fields, forests, or villages without a real folklore or superstition-facing motif.",
      "The stronger fit is only rural_setting, swamp_setting, or another location tag."
    ],
    family: "genre_motif",
    id: "creature:folk_horror",
    label: "folk_horror",
    translationStatus: "mapped"
  },
  forbidden_knowledge: {
    adjacentTags: [
      "occult_conspiracy",
      "cosmic_dread"
    ],
    appliesWhen: [
      "The creature is framed around hidden texts, proscribed secrets, mind-breaking truths, or knowledge pursued at terrible cost.",
      "Retrieval value comes from the danger of revelation, not merely from scholarship or intelligence."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "forbidden_knowledge",
    currentTag: "forbidden_knowledge",
    description: "Strongly associated with taboo lore, dangerous revelation, blasphemous truth, or learning that should not be uncovered.",
    doesNotApplyWhen: [
      "The creature is simply smart, scholarly, occult, or mysterious without dangerous-knowledge presentation being central.",
      "The stronger fit is occult_conspiracy or cosmic_dread because hidden cabals or existential terror matters more than taboo learning itself."
    ],
    family: "story_motif",
    id: "creature:forbidden_knowledge",
    label: "forbidden_knowledge",
    translationStatus: "mapped"
  },
  forest_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "forest_setting",
    currentTag: "forest_setting",
    description: "Strongly associated with forests, jungles, groves, or briar-choked wilds.",
    family: "habitat_setting",
    id: "creature:forest_setting",
    label: "forest_setting",
    translationStatus: "mapped"
  },
  fortress_setting: {
    adjacentTags: [
      "urban_setting",
      "temple_setting"
    ],
    appliesWhen: [
      "Fortified structures are part of the creature's recurring encounter identity.",
      "The creature is framed around castles, citadels, watchtowers, keeps, or defensive strongholds."
    ],
    assignmentMode: "editorial",
    axis: "setting",
    category: "creature",
    conceptId: "fortress_setting",
    currentTag: "fortress_setting",
    description: "Strongly associated with castles, fortresses, citadels, watchtowers, or other fortified encounter sites.",
    doesNotApplyWhen: [
      "The record only uses a fortress as a one-off location.",
      "The broader identity is urban_setting or temple_setting rather than fortified-site specific."
    ],
    family: "site_setting",
    id: "creature:fortress_setting",
    label: "fortress_setting",
    translationStatus: "mapped"
  },
  freshwater_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "freshwater_setting",
    currentTag: "freshwater_setting",
    description: "Strongly associated with rivers, lakes, ponds, streams, springs, or other inland waters.",
    family: "habitat_setting",
    id: "creature:freshwater_setting",
    label: "freshwater_setting",
    translationStatus: "mapped"
  },
  funerary_mourning: {
    adjacentTags: [
      "ritual_ceremony",
      "mask_motif"
    ],
    appliesWhen: [
      "Funeral symbolism, grief-haunting, mourning observance, or memorial ritual is central to the creature's presentation.",
      "The creature is naturally retrieved for funerary scenes, dirges, wakes, or death-ritual storytelling rather than only because it is undead."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "funerary_mourning",
    currentTag: "funerary_mourning",
    description: "Strongly associated with grief, funeral rites, mourning processions, memorial haunting, or death-ritual solemnity.",
    doesNotApplyWhen: [
      "The creature is merely undead, ghostly, or graveyard-linked without a meaningful mourning or funerary presentation.",
      "The stronger fit is graveyard_setting, boneyard_setting, or undead_adjacent without a real ritualized grief presentation."
    ],
    family: "story_motif",
    id: "creature:funerary_mourning",
    label: "funerary_mourning",
    translationStatus: "mapped"
  },
  fungal_infested: {
    adjacentTags: [
      "disease_vector",
      "body_horror"
    ],
    assignmentMode: "hybrid",
    axis: "specialization",
    category: "creature",
    conceptId: "fungal_infested",
    currentTag: "fungal_infested",
    description: "Creature strongly defined by mycelium, spores, mushroom overgrowth, or fungus-driven bodily infestation.",
    family: "corruption_profile",
    id: "creature:fungal_infested",
    label: "fungal_infested",
    translationStatus: "mapped"
  },
  gothic_horror_land_setting: {
    adjacentTags: [
      "graveyard_setting",
      "folk_horror"
    ],
    appliesWhen: [
      "Use when the creature is naturally retrieved for Gothic-horror encounter planning: decaying manors, haunted villages, graveyard menace, superstitious communities, or classic monster-fiction mood.",
      "The planning value comes from atmospheric horror identity, cursed lineage, or old-world dread rather than only from one undead, fiend, or shapeshifter trait."
    ],
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "gothic_horror_land_setting",
    currentTag: "gothic_horror_land_setting",
    description: "Strongly associated with Gothic-horror lands of mist, superstition, graveyard dread, cursed nobility, and classic night monsters. In Golarion, this primarily corresponds to Ustalav.",
    doesNotApplyWhen: [
      "The creature is simply spooky, undead, or cursed without a real Gothic-horror social or atmospheric frame.",
      "The stronger fit is story_motif or genre_motif because the retrieval hook is a narrower narrative motif rather than a full regional horror lens."
    ],
    family: "regional_setting",
    id: "creature:gothic_horror_land_setting",
    label: "gothic_horror_land_setting",
    translationStatus: "mapped"
  },
  graveyard_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "graveyard_setting",
    currentTag: "graveyard_setting",
    description: "Strongly associated with cemeteries, tombs, barrows, or burial grounds.",
    family: "site_setting",
    id: "creature:graveyard_setting",
    label: "graveyard_setting",
    translationStatus: "mapped"
  },
  guardian_npc: {
    adjacentTags: [
      "enforcer_npc",
      "watcher_npc"
    ],
    appliesWhen: [
      "The creature is naturally retrieved because it is posted to guard, hold, jail, or protect a specific person, threshold, route, or space.",
      "Posted protection or interdiction matters more than general combat readiness, surveillance, or broad social authority."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "guardian_npc",
    currentTag: "guardian_npc",
    description: "Immediate-scenario guard, jailer, doorkeeper, bodyguard, or other posted protector whose scene value is physically holding or protecting a person, threshold, or place.",
    doesNotApplyWhen: [
      "The creature is merely combat-ready without a clear posted protection, bodyguard, or threshold-holding duty.",
      "The stronger fit is authority_npc, enforcer_npc, or watcher_npc because command status, generic martial opposition, or alarm duty matters more than guarding."
    ],
    family: "scene_role",
    id: "creature:guardian_npc",
    label: "guardian_npc",
    translationStatus: "mapped"
  },
  guardian_retinue: {
    adjacentTags: [
      "guardian_npc",
      "defender_combatant"
    ],
    assignmentMode: "editorial",
    axis: "encounter",
    category: "creature",
    conceptId: "guardian_retinue",
    currentTag: "guardian_retinue",
    description: "Naturally retrieved as one member of a posted defense roster around a leader, relic, sanctum, or protected threshold rather than as an independent threat.",
    family: "cohort_role",
    id: "creature:guardian_retinue",
    label: "guardian_retinue",
    translationStatus: "mapped"
  },
  guide_npc: {
    adjacentTags: [
      "profession_npc",
      "rural_setting"
    ],
    appliesWhen: [
      "Leading others through terrain, routes, borders, or dangerous travel spaces is central to the creature's world-facing identity.",
      "The record is naturally used as a scout, pathfinder, navigator, or local guide rather than only a generic outdoors person."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "guide_npc",
    currentTag: "guide_npc",
    description: "Presented as a scout, tracker, ferryman, caravan guide, wilderness pathfinder, or other route-leading specialist.",
    doesNotApplyWhen: [
      "The creature merely knows the area or has survival competence without a role-defined guiding function.",
      "The stronger fit is scholar_npc, civic_npc, or enforcer_npc rather than travel-leading expertise."
    ],
    family: "social_role",
    id: "creature:guide_npc",
    label: "guide_npc",
    translationStatus: "mapped"
  },
  harrier_combatant: {
    adjacentTags: [
      "skirmisher_combatant",
      "artillery_combatant"
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "harrier_combatant",
    currentTag: "harrier_combatant",
    description: "Built to chip away from safety through repeated ranged harassment, flyby pressure, or evasive standoff attacks that force pursuit.",
    family: "combat_role",
    id: "creature:harrier_combatant",
    label: "harrier_combatant",
    translationStatus: "mapped"
  },
  healer_npc: {
    adjacentTags: [
      "scholar_npc",
      "religious_npc"
    ],
    appliesWhen: [
      "Medical treatment, recovery support, or caretaker duty is central to the creature's world-facing identity.",
      "The record is naturally used as a healer, medic, apothecary, or restorative support NPC rather than only a generic scholar."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "healer_npc",
    currentTag: "healer_npc",
    description: "Presented as a physician, battlefield medic, herbalist, chirurgeon, caretaker, or other explicitly healing-facing role-holder.",
    doesNotApplyWhen: [
      "The creature merely has healing magic or restorative abilities without a role-defined caregiving identity.",
      "The stronger fit is scholar_npc, religious_npc, or civic_npc without a real healer-facing job."
    ],
    family: "social_role",
    id: "creature:healer_npc",
    label: "healer_npc",
    translationStatus: "mapped"
  },
  heaven_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "heaven_setting",
    currentTag: "heaven_setting",
    description: "Strongly associated with Heaven, archon hosts, or ordered celestial service.",
    family: "planar_setting",
    id: "creature:heaven_setting",
    label: "heaven_setting",
    translationStatus: "provisional"
  },
  hell_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "hell_setting",
    currentTag: "hell_setting",
    description: "Strongly associated with Hell, devil hosts, or infernal hierarchy.",
    family: "planar_setting",
    id: "creature:hell_setting",
    label: "hell_setting",
    translationStatus: "provisional"
  },
  industrial_grotesque: {
    adjacentTags: [
      "body_horror",
      "urban_setting"
    ],
    appliesWhen: [
      "The creature evokes factory horror, machine-maimed labor, furnace dread, or industrial systems turning flesh and society into raw material.",
      "Retrieval value comes from industrial corruption and mechanized degradation, not just from being a construct or urban creature."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "industrial_grotesque",
    currentTag: "industrial_grotesque",
    description: "Strongly associated with smoke, gears, furnaces, exploitation, mutilating machinery, or dehumanizing industrial corruption.",
    doesNotApplyWhen: [
      "The creature merely uses technology, lives in a city, or is a construct without industrial corruption or machine-horror presentation.",
      "The stronger fit is body_horror or urban_setting because visceral corruption or city placement matters more than industrial atmosphere."
    ],
    family: "genre_motif",
    id: "creature:industrial_grotesque",
    label: "industrial_grotesque",
    translationStatus: "mapped"
  },
  infestation_member: {
    adjacentTags: [
      "parasite_ridden",
      "plaguebearing"
    ],
    assignmentMode: "editorial",
    axis: "encounter",
    category: "creature",
    conceptId: "infestation_member",
    currentTag: "infestation_member",
    description: "Naturally retrieved as one body in a swarm-like infestation, burrowing colony, parasite outbreak, or other many-body nuisance roster.",
    family: "cohort_role",
    id: "creature:infestation_member",
    label: "infestation_member",
    translationStatus: "mapped"
  },
  infiltration_threat: {
    adjacentTags: [
      "disguised_pretender",
      "possession_threat"
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "infiltration",
    currentTag: "infiltration_threat",
    description: "Threat defined by disguise, replacement, infiltration, or remaining embedded among victims before the danger fully reveals itself.",
    family: "threat_profile",
    id: "creature:infiltration_threat",
    label: "infiltration",
    translationStatus: "mapped"
  },
  infiltrator_npc: {
    adjacentTags: [
      "enforcer_npc",
      "criminal_npc"
    ],
    appliesWhen: [
      "The creature is naturally retrieved as an embedded infiltrator, saboteur, impostor, spy, or quiet-entry specialist.",
      "This tag answers the creature's immediate scenario function rather than its broader profession, faction post, or criminal affiliation."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "infiltrator_npc",
    currentTag: "infiltrator_npc",
    description: "Scene-slot spy, saboteur, replacer, or quiet-entry specialist whose immediate scenario value comes from infiltration more than a straight fight.",
    doesNotApplyWhen: [
      "The creature is mainly an enforcer_npc or civic_npc and only uses stealth or deception incidentally.",
      "The stronger retrieval hook is criminal_npc or another social_role tag because the world-facing identity matters more than the scene slot."
    ],
    family: "scene_role",
    id: "creature:infiltrator_npc",
    label: "infiltrator_npc",
    translationStatus: "mapped"
  },
  innocence_twisted: {
    adjacentTags: [
      "living_toy",
      "carnival_show"
    ],
    appliesWhen: [
      "The creature's presentation depends on childlike, gentle, or comforting imagery becoming eerie, cruel, or dangerous.",
      "A GM would plausibly retrieve it for nursery horror, storybook menace, or innocence-corrupted scenes."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "innocence_twisted",
    currentTag: "innocence_twisted",
    description: "Strongly associated with childish innocence, nursery imagery, or comforting domestic symbols turned uncanny, cruel, or threatening.",
    doesNotApplyWhen: [
      "The creature is merely small, playful, or associated with toys without innocence-curdled-into-menace being central.",
      "The stronger fit is living_toy or carnival_show because animated playthings or spectacle explain the retrieval better."
    ],
    family: "genre_motif",
    id: "creature:innocence_twisted",
    label: "innocence_twisted",
    translationStatus: "mapped"
  },
  island_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "island_setting",
    currentTag: "island_setting",
    description: "Strongly associated with islands, archipelagos, or isolated isles.",
    family: "habitat_setting",
    id: "creature:island_setting",
    label: "island_setting",
    translationStatus: "mapped"
  },
  jungle_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "jungle_setting",
    currentTag: "jungle_setting",
    description: "Strongly associated with jungles, rainforests, or dense tropical canopies.",
    family: "habitat_setting",
    id: "creature:jungle_setting",
    label: "jungle_setting",
    translationStatus: "mapped"
  },
  life_drain_threat: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "life_drain_application",
    currentTag: "life_drain_threat",
    description: "Threat defined by draining blood, vitality, life force, or souls from victims.",
    family: "threat_profile",
    id: "creature:life_drain_threat",
    label: "life_drain_application",
    translationStatus: "mapped"
  },
  living_artwork: {
    assignmentMode: "hybrid",
    axis: "presentation",
    category: "creature",
    conceptId: "living_artwork",
    currentTag: "living_artwork",
    description: "Strongly associated with paintings, graffiti, murals, portraits, or other artworks brought to life.",
    family: "visual_motif",
    id: "creature:living_artwork",
    label: "living_artwork",
    translationStatus: "mapped"
  },
  living_toy: {
    assignmentMode: "hybrid",
    axis: "presentation",
    category: "creature",
    conceptId: "living_toy",
    currentTag: "living_toy",
    description: "Strongly associated with dolls, puppets, mannequins, or other animated playthings.",
    family: "visual_motif",
    id: "creature:living_toy",
    label: "living_toy",
    translationStatus: "mapped"
  },
  lower_plane_setting: {
    adjacentTags: [
      "hell_setting",
      "abyss_setting",
      "abaddon_setting"
    ],
    assignmentMode: "composite",
    axis: "setting",
    category: "creature",
    compositeOfAnyTags: [
      "hell_setting",
      "abyss_setting",
      "abaddon_setting"
    ],
    conceptId: "lower_plane_setting",
    currentTag: "lower_plane_setting",
    description: "Strongly associated with the lower planes of Hell, the Abyss, or Abaddon.",
    family: "planar_setting",
    id: "creature:lower_plane_setting",
    label: "lower_plane_setting",
    nativeOntologyPolicy: "aggregates_native_signals",
    translationStatus: "provisional"
  },
  maelstrom_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "maelstrom_setting",
    currentTag: "maelstrom_setting",
    description: "Strongly associated with the Maelstrom, proteans, or its chaotic planar fringes.",
    family: "planar_setting",
    id: "creature:maelstrom_setting",
    label: "maelstrom_setting",
    translationStatus: "provisional"
  },
  magic_blight_wasteland_setting: {
    adjacentTags: [
      "alien_technology_wasteland_setting",
      "wasteland_setting"
    ],
    appliesWhen: [
      "Use when the creature is naturally retrieved for dead-magic deserts, wild-magic badlands, spellscar survival, mutation-causing arcane fallout, or regions where magic itself has wounded the landscape.",
      "The planning value comes from magical environmental breakage and wasteland adaptation rather than only from one magical trait or one regional nation."
    ],
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "magic_blight_wasteland_setting",
    currentTag: "magic_blight_wasteland_setting",
    description: "Strongly associated with arcane devastation, dead-magic pockets, wild-magic scars, magical storms, and survival in a magically broken wasteland. In Golarion, this primarily corresponds to the Mana Wastes.",
    doesNotApplyWhen: [
      "The creature is merely magical, arcane, or mutated without a meaningful tie to a magic-blasted regional wasteland.",
      "The stronger fit is alien_technology_wasteland_setting because the retrieval hook is weird technology, robots, or alien ruins rather than magical landscape scarring."
    ],
    family: "regional_setting",
    id: "creature:magic_blight_wasteland_setting",
    label: "magic_blight_wasteland_setting",
    translationStatus: "mapped"
  },
  maritime_superstition: {
    adjacentTags: [
      "nautical_setting",
      "folk_horror"
    ],
    appliesWhen: [
      "The creature is naturally retrieved for ghost-ship rumor, sailor taboo, drowned prophecy, or sea-legend scenes where folklore matters as much as location.",
      "Use when nautical superstition and omen-laden seafaring culture are central to the creature's presentation."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "maritime_superstition",
    currentTag: "maritime_superstition",
    description: "Strongly associated with cursed voyages, sea omens, sailor folklore, drowned portents, or nautical dread shaped by legend and taboo.",
    doesNotApplyWhen: [
      "The creature is merely aquatic, coastal, or ship-linked without a real folklore-and-omen maritime motif.",
      "The stronger fit is nautical_setting or folk_horror because placement or generic rural superstition matters more than sailor legend."
    ],
    family: "genre_motif",
    id: "creature:maritime_superstition",
    label: "maritime_superstition",
    translationStatus: "mapped"
  },
  mask_motif: {
    adjacentTags: [
      "disguised_pretender",
      "faceless_horror"
    ],
    appliesWhen: [
      "Masks, veils, or deliberate face-covering are a salient presentation motif.",
      "The obscured face is part of the creature's recurring visual identity."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "mask_motif",
    currentTag: "mask_motif",
    description: "Strongly associated with masks, veils, ceremonial face-coverings, or deliberately obscured presentation.",
    doesNotApplyWhen: [
      "A mask appears once as minor equipment.",
      "The stronger semantic is faceless_horror or disguised_pretender rather than mask imagery."
    ],
    family: "visual_motif",
    id: "creature:mask_motif",
    label: "mask_motif",
    translationStatus: "mapped"
  },
  merchant_npc: {
    adjacentTags: [
      "profession_npc",
      "civic_npc"
    ],
    appliesWhen: [
      "Trade, selling, bargaining, or inventory-handling is central to the creature's world-facing identity.",
      "The creature naturally fills a market, shop, caravan, or supply-scene role."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "merchant_npc",
    currentTag: "merchant_npc",
    description: "Presented as a trader, broker, shopkeeper, caravan factor, or other commerce-facing role-holder.",
    doesNotApplyWhen: [
      "The record only implies wealth or possessions without an actual merchant role.",
      "The stronger fit is civic_npc or profession_npc without a clear commerce function."
    ],
    family: "social_role",
    id: "creature:merchant_npc",
    label: "merchant_npc",
    translationStatus: "mapped"
  },
  mirror_motif: {
    assignmentMode: "hybrid",
    axis: "presentation",
    category: "creature",
    conceptId: "mirror_motif",
    currentTag: "mirror_motif",
    description: "Strongly associated with mirrors, reflections, duplicated selves, or reflective surfaces as a core visual or horror identity.",
    family: "visual_motif",
    id: "creature:mirror_motif",
    label: "mirror_motif",
    translationStatus: "mapped"
  },
  mountain_setting: {
    adjacentTags: [
      "underground_setting",
      "sky_setting"
    ],
    appliesWhen: [
      "Mountain terrain is a recurring habitat or encounter frame.",
      "The creature is tied to peaks, passes, cliffs, or alpine strongholds."
    ],
    assignmentMode: "editorial",
    axis: "setting",
    category: "creature",
    conceptId: "mountain_setting",
    currentTag: "mountain_setting",
    description: "Strongly associated with cliffs, peaks, passes, or rocky heights.",
    doesNotApplyWhen: [
      "Rocky terrain is incidental rather than defining.",
      "The creature is better described as underground_setting or sky_setting."
    ],
    family: "habitat_setting",
    id: "creature:mountain_setting",
    label: "mountain_setting",
    translationStatus: "mapped"
  },
  mwangi_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "mwangi_setting",
    currentTag: "mwangi_setting",
    description: "Strongly associated with the Mwangi Expanse, its jungle polities, and Mwangi-rooted regional framing that materially affects creature planning and retrieval.",
    family: "regional_setting",
    id: "creature:mwangi_setting",
    label: "mwangi_setting",
    translationStatus: "mapped"
  },
  nautical_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "nautical_setting",
    currentTag: "nautical_setting",
    description: "Strongly associated with ships, sailors, wrecks, or harbors.",
    family: "site_setting",
    id: "creature:nautical_setting",
    label: "nautical_setting",
    translationStatus: "mapped"
  },
  nightmare_tainted: {
    adjacentTags: [
      "dream_nightmare",
      "dreamlands_setting"
    ],
    assignmentMode: "hybrid",
    axis: "specialization",
    category: "creature",
    conceptId: "nightmare_tainted",
    currentTag: "nightmare_tainted",
    description: "Creature strongly defined by dream corruption, sleep-haunting influence, or oneiric pollution leaking into the waking world.",
    family: "corruption_profile",
    id: "creature:nightmare_tainted",
    label: "nightmare_tainted",
    translationStatus: "mapped"
  },
  nirvana_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "nirvana_setting",
    currentTag: "nirvana_setting",
    description: "Strongly associated with Nirvana, enlightened celestials, or benevolent contemplative service.",
    family: "planar_setting",
    id: "creature:nirvana_setting",
    label: "nirvana_setting",
    translationStatus: "provisional"
  },
  obsession_fixation: {
    adjacentTags: [
      "forbidden_knowledge",
      "predatory_seduction"
    ],
    appliesWhen: [
      "The creature is framed around monomania, possessive attention, compulsive collecting, or a need it cannot release.",
      "Retrieval value comes from unhealthy fixation driving the story, not simply from a preference or goal."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "obsession_fixation",
    currentTag: "obsession_fixation",
    description: "Strongly associated with compulsive pursuit, jealous fixation, collecting mania, perfectionism, or a single consuming desire.",
    doesNotApplyWhen: [
      "The creature has a mission, desire, or recurring target without obsessive compulsion being central to its identity.",
      "The stronger fit is forbidden_knowledge or predatory_seduction because taboo learning or lure-based predation matters more than fixation."
    ],
    family: "story_motif",
    id: "creature:obsession_fixation",
    label: "obsession_fixation",
    translationStatus: "mapped"
  },
  occult_conspiracy: {
    adjacentTags: [
      "paranoia_surveillance",
      "forbidden_knowledge"
    ],
    appliesWhen: [
      "The creature is naturally retrieved for hidden cult cells, secret masters, conspiratorial rites, or layered occult plotting.",
      "Its presentation depends on covert structure and esoteric collusion, not only on individual deception or ritual practice."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "occult_conspiracy",
    currentTag: "occult_conspiracy",
    description: "Strongly associated with secret circles, hidden masters, esoteric cabals, or ritual networks manipulating events from the shadows.",
    doesNotApplyWhen: [
      "The creature merely participates in a ritual, infiltrates a group, or knows occult lore without a real cabal or conspiracy presentation.",
      "The stronger fit is ritual_ceremony, paranoia_surveillance, or forbidden_knowledge rather than hidden-network manipulation."
    ],
    family: "story_motif",
    id: "creature:occult_conspiracy",
    label: "occult_conspiracy",
    translationStatus: "mapped"
  },
  occult_spellcaster: {
    assignmentMode: "hybrid",
    axis: "specialization",
    category: "creature",
    conceptId: "occult_spellcaster",
    currentTag: "occult_spellcaster",
    description: "Creature whose spellcasting is substantially framed through occult lore, spirits, emotion, dreams, or esoteric mental power.",
    family: "casting_profile",
    id: "creature:occult_spellcaster",
    label: "occult_spellcaster",
    translationStatus: "mapped"
  },
  organized_undead_society_setting: {
    adjacentTags: [
      "undead_war_torn_region_setting",
      "urban_setting"
    ],
    appliesWhen: [
      "Use when the creature is naturally retrieved for an undead-ruled state, necromantic civil order, corpse-backed labor system, or other organized deathless society rather than an isolated tomb or graveyard.",
      "The planning value comes from undead institutions, court politics, civic structure, or civilized undead social roles as much as from simple undead presence."
    ],
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "organized_undead_society_setting",
    currentTag: "organized_undead_society_setting",
    description: "Strongly associated with organized undead societies, corpse labor, necromantic bureaucracy, and courtly undead institutions. In Golarion, this primarily corresponds to Geb.",
    doesNotApplyWhen: [
      "The creature is merely undead, tomb-dwelling, or cemetery-haunting without a meaningful link to a broader undead social order.",
      "The stronger fit is undead_war_torn_region_setting because the retrieval hook is an undead occupation zone or shattered crusader frontier rather than a stable undead society."
    ],
    family: "regional_setting",
    id: "creature:organized_undead_society_setting",
    label: "organized_undead_society_setting",
    translationStatus: "mapped"
  },
  pack_hunter: {
    adjacentTags: [
      "ambusher_combatant",
      "skirmisher_combatant"
    ],
    assignmentMode: "editorial",
    axis: "encounter",
    category: "creature",
    conceptId: "pack_hunter",
    currentTag: "pack_hunter",
    description: "Naturally retrieved because the creature hunts as part of a pack, coordinated ambush group, or pursuit cluster rather than as a solitary predator.",
    family: "cohort_role",
    id: "creature:pack_hunter",
    label: "pack_hunter",
    translationStatus: "mapped"
  },
  paranoia_surveillance: {
    adjacentTags: [
      "occult_conspiracy",
      "disguised_pretender"
    ],
    appliesWhen: [
      "The creature evokes stalking observation, unseen witnesses, informer networks, or dread rooted in being watched.",
      "Retrieval value comes from suspicion, surveillance, or omnipresent scrutiny rather than only stealth or infiltration."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "paranoia_surveillance",
    currentTag: "paranoia_surveillance",
    description: "Strongly associated with hidden watchers, constant scrutiny, being monitored, or the collapse of trust under observation.",
    doesNotApplyWhen: [
      "The creature merely scouts, spies, or infiltrates without a broader atmosphere of surveillance and distrust.",
      "The stronger fit is occult_conspiracy or disguised_pretender because hidden coordination or impersonation matters more than the watched feeling."
    ],
    family: "story_motif",
    id: "creature:paranoia_surveillance",
    label: "paranoia_surveillance",
    translationStatus: "mapped"
  },
  parasite_ridden: {
    adjacentTags: [
      "spawn_creator",
      "plaguebearing"
    ],
    assignmentMode: "hybrid",
    axis: "specialization",
    category: "creature",
    conceptId: "parasite_ridden",
    currentTag: "parasite_ridden",
    description: "Creature strongly defined by hosting burrowing larvae, parasitic colonies, implanted broods, or other invasive life within the body.",
    family: "corruption_profile",
    id: "creature:parasite_ridden",
    label: "parasite_ridden",
    translationStatus: "mapped"
  },
  patrol_member: {
    adjacentTags: [
      "watcher_npc",
      "urban_setting"
    ],
    assignmentMode: "editorial",
    axis: "encounter",
    category: "creature",
    conceptId: "patrol_member",
    currentTag: "patrol_member",
    description: "Naturally retrieved as part of a watch patrol, sentry route, street sweep, border detail, or other recurring patrol formation.",
    family: "cohort_role",
    id: "creature:patrol_member",
    label: "patrol_member",
    translationStatus: "mapped"
  },
  performer_npc: {
    adjacentTags: [
      "profession_npc",
      "carnival_show"
    ],
    appliesWhen: [
      "Performance, spectacle, or entertainment labor is central to the creature's world-facing identity.",
      "The creature would be retrieved for theater, carnival, court entertainment, or tavern-stage scenes."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "performer_npc",
    currentTag: "performer_npc",
    description: "Presented as a musician, actor, dancer, herald, jester, or other entertainment-facing role-holder.",
    doesNotApplyWhen: [
      "The creature is only whimsical or colorful without an explicit performer role.",
      "The stronger semantic is carnival_show rather than a role-defined entertainer."
    ],
    family: "social_role",
    id: "creature:performer_npc",
    label: "performer_npc",
    translationStatus: "mapped"
  },
  petrification_threat: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "petrification_application",
    currentTag: "petrification_threat",
    description: "Threat defined by petrifying victims or turning them to stone.",
    family: "threat_profile",
    id: "creature:petrification_threat",
    label: "petrification_application",
    translationStatus: "mapped"
  },
  plaguebearing: {
    adjacentTags: [
      "disease_vector",
      "parasite_ridden"
    ],
    assignmentMode: "hybrid",
    axis: "specialization",
    category: "creature",
    conceptId: "plaguebearing",
    currentTag: "plaguebearing",
    description: "Creature strongly defined by carrying, spreading, or embodying pestilence, fever, or outbreak-causing corruption.",
    family: "corruption_profile",
    id: "creature:plaguebearing",
    label: "plaguebearing",
    translationStatus: "mapped"
  },
  plains_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "plains_setting",
    currentTag: "plains_setting",
    description: "Strongly associated with open plains, grasslands, prairies, or savannas.",
    family: "habitat_setting",
    id: "creature:plains_setting",
    label: "plains_setting",
    translationStatus: "mapped"
  },
  plane_of_air_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "plane_of_air_setting",
    currentTag: "plane_of_air_setting",
    description: "Strongly associated with the Plane of Air, its native denizens, or its endless winds and cloud realms.",
    family: "planar_setting",
    id: "creature:plane_of_air_setting",
    label: "plane_of_air_setting",
    translationStatus: "provisional"
  },
  plane_of_earth_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "plane_of_earth_setting",
    currentTag: "plane_of_earth_setting",
    description: "Strongly associated with the Plane of Earth, its native denizens, or its crystal caverns and stonebound realms.",
    family: "planar_setting",
    id: "creature:plane_of_earth_setting",
    label: "plane_of_earth_setting",
    translationStatus: "provisional"
  },
  plane_of_fire_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "plane_of_fire_setting",
    currentTag: "plane_of_fire_setting",
    description: "Strongly associated with the Plane of Fire, its native denizens, or its infernal-bright cityscapes and battlefields.",
    family: "planar_setting",
    id: "creature:plane_of_fire_setting",
    label: "plane_of_fire_setting",
    translationStatus: "provisional"
  },
  plane_of_water_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "plane_of_water_setting",
    currentTag: "plane_of_water_setting",
    description: "Strongly associated with the Plane of Water, its native denizens, or its endless seas and oceanic realms.",
    family: "planar_setting",
    id: "creature:plane_of_water_setting",
    label: "plane_of_water_setting",
    translationStatus: "provisional"
  },
  poison_threat: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "poison_application",
    currentTag: "poison_threat",
    description: "Threat defined by venom, toxic excretions, poisoned weapons, or other recurring poison delivery.",
    family: "threat_profile",
    id: "creature:poison_threat",
    label: "poison_application",
    translationStatus: "mapped"
  },
  possessed_object: {
    adjacentTags: [
      "animated_object",
      "possession_threat"
    ],
    appliesWhen: [
      "Use when a spirit, ghost, curse, or other external presence is explicitly what animates the object.",
      "The inhabiting presence matters more than the object's construction, material, or generic animation."
    ],
    assignmentMode: "hybrid",
    axis: "presentation",
    category: "creature",
    conceptId: "possessed_object",
    currentTag: "possessed_object",
    description: "Strongly associated with an inhabiting spirit or curse animating an otherwise mundane object or suit of equipment.",
    doesNotApplyWhen: [
      "The object is simply animated by magic, clockwork, or sculpted animation with no real possessing force.",
      "The stronger fit is animated_object or animated_statue because possession is not central."
    ],
    family: "bound_object",
    id: "creature:possessed_object",
    label: "possessed_object",
    translationStatus: "mapped"
  },
  possession_threat: {
    adjacentTags: [
      "curse_threat",
      "reinforcement_threat"
    ],
    appliesWhen: [
      "Use when entering, riding, replacing, or controlling a host body is a major reason to retrieve the creature.",
      "The possession dynamic matters more than ordinary charm, domination, or haunting flavor."
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "possession_application",
    currentTag: "possession_threat",
    description: "Can possess, body-snatch, or take control of a victim from within.",
    doesNotApplyWhen: [
      "The creature only compels, frightens, or mentally influences targets without true body-occupying takeover.",
      "The stronger fit is curse_threat or reinforcement_threat because possession is not central to encounter prep."
    ],
    family: "threat_profile",
    id: "creature:possession_threat",
    label: "possession_application",
    translationStatus: "mapped"
  },
  predatory_seduction: {
    adjacentTags: [
      "seductive_temptation",
      "disguised_pretender"
    ],
    appliesWhen: [
      "The creature's presentation centers on luring prey through desire, intimacy, or false safety before the attack or betrayal lands.",
      "Use when the hunting or consuming dynamic matters more than general temptation or glamour."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "predatory_seduction",
    currentTag: "predatory_seduction",
    description: "Strongly associated with deliberate luring, honey-trap menace, erotic predation, or invitation used explicitly as a hunting tactic.",
    doesNotApplyWhen: [
      "The creature is alluring or corruptive without a strong predator-lure structure.",
      "The stronger fit is seductive_temptation because dangerous attraction matters more than an explicit hunt pattern."
    ],
    family: "genre_motif",
    id: "creature:predatory_seduction",
    label: "predatory_seduction",
    translationStatus: "mapped"
  },
  prey_control_threat: {
    adjacentTags: [
      "ambush_grabber",
      "terrain_control_threat"
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "prey_control",
    currentTag: "prey_control_threat",
    description: "Threat defined by holding prey in place through grabs, constriction, webbing, swallowing, or other ongoing body-control pressure.",
    family: "threat_profile",
    id: "creature:prey_control_threat",
    label: "prey_control",
    translationStatus: "mapped"
  },
  primal_spellcaster: {
    assignmentMode: "hybrid",
    axis: "specialization",
    category: "creature",
    conceptId: "primal_spellcaster",
    currentTag: "primal_spellcaster",
    description: "Creature whose spellcasting is substantially framed through nature, elemental power, druidic force, or instinctive primal magic.",
    family: "casting_profile",
    id: "creature:primal_spellcaster",
    label: "primal_spellcaster",
    translationStatus: "mapped"
  },
  profession_npc: {
    adjacentTags: [
      "authority_npc",
      "merchant_npc"
    ],
    appliesWhen: [
      "The creature is primarily presented through a social role, job, office, or profession label.",
      "Retrieval value comes from it being a role-defined NPC rather than a species-driven monster.",
      "This tag answers what the creature is in the world, even if a separate scene_role tag explains how it functions in the scene."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "profession_npc",
    currentTag: "profession_npc",
    description: "Role-defined NPC such as a captain, guard, merchant, priest, or commoner.",
    doesNotApplyWhen: [
      "The role label is incidental to a stronger monster or combat identity.",
      "The creature is better modeled only as enforcer_npc, infiltrator_npc, or civic_npc because the scene slot matters more than the job or office."
    ],
    family: "social_role",
    id: "creature:profession_npc",
    label: "profession_npc",
    translationStatus: "mapped"
  },
  prophecy_omen: {
    adjacentTags: [
      "apocalypse_ruin",
      "ancestral_legacy"
    ],
    appliesWhen: [
      "Portents, prophecy, omen-reading, or the creature's arrival as a sign of coming change is a central retrieval hook.",
      "A GM would plausibly retrieve the creature for foretold doom, chosen destiny, or fate-haunted story beats."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "prophecy_omen",
    currentTag: "prophecy_omen",
    description: "Strongly associated with foretelling, omen-bearing, apocalyptic signs, destiny, or a creature's role as a herald of what is to come.",
    doesNotApplyWhen: [
      "The creature merely predicts events, has divination magic, or is important to the plot without a strong omen-facing presentation.",
      "The stronger fit is apocalypse_ruin or ancestral_legacy because destiny-sign imagery is not the main presentation hook."
    ],
    family: "story_motif",
    id: "creature:prophecy_omen",
    label: "prophecy_omen",
    translationStatus: "mapped"
  },
  regeneration_threat: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "regeneration",
    currentTag: "regeneration_threat",
    description: "Regenerates or requires special suppression or finishing countermeasures.",
    family: "threat_profile",
    id: "creature:regeneration_threat",
    label: "regeneration",
    translationStatus: "mapped"
  },
  reinforcement_threat: {
    adjacentTags: [
      "spawn_creator",
      "commander_combatant"
    ],
    appliesWhen: [
      "Use when the creature's main prep significance is that it adds bodies, activates subordinates, or sharply force-multiplies nearby allies.",
      "The encounter meaningfully changes because of its reinforcement engine rather than just because it personally hits hard."
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "reinforcement",
    currentTag: "reinforcement_threat",
    description: "Threat defined by materially changing encounter structure through added bodies, activated subordinates, or sharply elevated allied creatures.",
    doesNotApplyWhen: [
      "The creature only has one minor ally-facing buff or an incidental summon without materially changing encounter structure.",
      "The stronger fit is support_combatant, commander_combatant, or spawn_creator because reinforcement is not the real threat hook."
    ],
    family: "threat_profile",
    id: "creature:reinforcement_threat",
    label: "reinforcement",
    translationStatus: "mapped"
  },
  religious_npc: {
    adjacentTags: [
      "profession_npc",
      "temple_setting"
    ],
    appliesWhen: [
      "Religious office, ritual duty, or custodianship of a faith space is central to the creature's world-facing identity.",
      "The creature is naturally retrieved as clergy, cult staff, or sacred-site personnel."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "religious_npc",
    currentTag: "religious_npc",
    description: "Presented as a priest, shrine keeper, cult officiant, monastic figure, or other explicitly religious role-holder.",
    doesNotApplyWhen: [
      "The creature merely has divine powers without a role-defined religious identity.",
      "Temple placement alone is better captured by temple_setting."
    ],
    family: "social_role",
    id: "creature:religious_npc",
    label: "religious_npc",
    translationStatus: "mapped"
  },
  revelry_excess: {
    adjacentTags: [
      "carnival_show",
      "seductive_temptation"
    ],
    appliesWhen: [
      "The creature is naturally retrieved for bacchanals, cursed feasts, debauched parties, or scenes of ecstatic excess.",
      "Celebration curdling into danger is part of the creature's recurring narrative identity."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "revelry_excess",
    currentTag: "revelry_excess",
    description: "Strongly associated with feasts, drunken revels, riotous celebration, gluttony, or ecstatic overindulgence.",
    doesNotApplyWhen: [
      "The creature only appears near taverns or festivals without excess, indulgence, or revelry being central.",
      "The stronger fit is carnival_show or seductive_temptation rather than feast-and-excess presentation."
    ],
    family: "story_motif",
    id: "creature:revelry_excess",
    label: "revelry_excess",
    translationStatus: "mapped"
  },
  ritual_ceremony: {
    adjacentTags: [
      "religious_npc",
      "ritualist_creature"
    ],
    appliesWhen: [
      "Ceremonial staging, sacrificial ritual, processional presentation, or formal observance is a major retrieval hook.",
      "The creature is naturally used in scenes defined by rites, altars, chants, offerings, or public ceremony."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "ritual_ceremony",
    currentTag: "ritual_ceremony",
    description: "Strongly associated with rites, sacrifices, processions, ceremonial observance, or cultic staging as a scene identity.",
    doesNotApplyWhen: [
      "The creature merely has ritual magic or divine powers without a strong ceremonial scene identity.",
      "Temple placement or religious office alone is better captured by temple_setting or religious_npc."
    ],
    family: "story_motif",
    id: "creature:ritual_ceremony",
    label: "ritual_ceremony",
    translationStatus: "mapped"
  },
  ritualist_creature: {
    adjacentTags: [
      "divine_spellcaster",
      "occult_spellcaster"
    ],
    appliesWhen: [
      "Use when ceremonial, circle-based, sacrificial, or extended-casting magic is a major reason to retrieve the creature.",
      "The creature is naturally used as a ritual leader, ritual threat, or ritual-supporting encounter element."
    ],
    assignmentMode: "hybrid",
    axis: "specialization",
    category: "creature",
    conceptId: "ritualist_creature",
    currentTag: "ritualist_creature",
    description: "Creature strongly associated with ritual casting, ceremonial magic, or extended occult or divine preparations.",
    doesNotApplyWhen: [
      "The creature merely casts normal encounter spells without a meaningful ritual identity.",
      "The stronger fit is a tradition spellcaster tag and ritual work is only incidental flavor."
    ],
    family: "casting_profile",
    id: "creature:ritualist_creature",
    label: "ritualist_creature",
    translationStatus: "mapped"
  },
  ruins_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "ruins_setting",
    currentTag: "ruins_setting",
    description: "Strongly associated with ancient ruins or derelict structures.",
    family: "site_setting",
    id: "creature:ruins_setting",
    label: "ruins_setting",
    translationStatus: "mapped"
  },
  rural_setting: {
    adjacentTags: [
      "small_settlement_setting",
      "plains_setting"
    ],
    appliesWhen: [
      "The creature is repeatedly framed around farms, fields, mills, roadsides, or countryside scenes.",
      "Agricultural or open-country placement is a recurring part of its encounter identity."
    ],
    assignmentMode: "editorial",
    axis: "setting",
    category: "creature",
    conceptId: "rural_setting",
    currentTag: "rural_setting",
    description: "Strongly associated with farms, pastures, croplands, countryside routes, mills, or other agricultural rural encounter scenes.",
    doesNotApplyWhen: [
      "The record only implies generic overland travel.",
      "The creature is better modeled by small_settlement_setting or plains_setting."
    ],
    family: "site_setting",
    id: "creature:rural_setting",
    label: "rural_setting",
    translationStatus: "mapped"
  },
  scholar_npc: {
    adjacentTags: [
      "profession_npc",
      "civic_npc"
    ],
    appliesWhen: [
      "Research, teaching, scholarship, or recordkeeping is central to the creature's world-facing identity.",
      "The creature is naturally retrieved as an academic or knowledge-scene NPC."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "scholar_npc",
    currentTag: "scholar_npc",
    description: "Presented as a sage, researcher, teacher, archivist, alchemist, or other knowledge-centered role-holder.",
    doesNotApplyWhen: [
      "Intelligence is incidental to a stronger combat or monster identity.",
      "The record only implies general competence without a knowledge-centered role."
    ],
    family: "social_role",
    id: "creature:scholar_npc",
    label: "scholar_npc",
    translationStatus: "mapped"
  },
  seasonal_festival: {
    adjacentTags: [
      "revelry_excess",
      "folk_horror"
    ],
    appliesWhen: [
      "The creature is naturally retrieved for midsummer revels, winter rites, harvest pageants, or holiday scenes defined by recurring festal tradition.",
      "Calendar-bound custom or festival atmosphere is a central part of the creature's presentation rather than incidental backdrop."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "seasonal_festival",
    currentTag: "seasonal_festival",
    description: "Strongly associated with solstice rites, harvest festivals, holiday customs, masked processions, or recurring calendar-bound celebration.",
    doesNotApplyWhen: [
      "The creature merely appears during a feast, fair, or celebration without a recurring seasonal or ritual festival identity.",
      "The stronger fit is revelry_excess, carnival_show, or folk_horror because indulgence, spectacle, or folklore carries the retrieval weight."
    ],
    family: "story_motif",
    id: "creature:seasonal_festival",
    label: "seasonal_festival",
    translationStatus: "mapped"
  },
  seductive_temptation: {
    adjacentTags: [
      "disguised_pretender",
      "courtly_pageantry"
    ],
    appliesWhen: [
      "The creature's presentation centers on allure, invitation, enchantment through desire, or baiting victims into a doomed choice.",
      "Retrieval value comes from temptation or dangerous attraction, not just social interaction or mechanical charm effects."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "seductive_temptation",
    currentTag: "seductive_temptation",
    description: "Strongly associated with allure, seduction, dangerous invitation, or temptation into vice, doom, or compromise.",
    doesNotApplyWhen: [
      "The creature is merely attractive, charismatic, or capable of charm without temptation being a real story hook.",
      "The stronger fit is disguised_pretender or courtly_pageantry rather than luring desire."
    ],
    family: "genre_motif",
    id: "creature:seductive_temptation",
    label: "seductive_temptation",
    translationStatus: "mapped"
  },
  shadow_plane_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "shadow_plane_setting",
    currentTag: "shadow_plane_setting",
    description: "Strongly associated with the Shadow Plane or the Plane of Shadow.",
    family: "planar_setting",
    id: "creature:shadow_plane_setting",
    label: "shadow_plane_setting",
    translationStatus: "provisional"
  },
  sinspawn_family: {
    assignmentMode: "hybrid",
    axis: "specialization",
    category: "creature",
    conceptId: "sinspawn_family",
    currentTag: "sinspawn_family",
    description: "Groups sinspawn and close runelord-bred sinspawn offshoots into one retrieval bucket.",
    family: "ontology_cluster",
    id: "creature:sinspawn_family",
    label: "sinspawn_family",
    nativeOntologyPolicy: "aggregates_native_signals",
    translationStatus: "mapped"
  },
  skirmisher_combatant: {
    adjacentTags: [
      "ambusher_combatant",
      "controller_combatant"
    ],
    appliesWhen: [
      "The creature's retrieval value comes from darting movement, flanking, repositioning, or repeated opportunistic attacks.",
      "Mobility and pressure cycling matter more than armor, command, or raw ranged bombardment."
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "skirmisher_combatant",
    currentTag: "skirmisher_combatant",
    description: "Built around mobility, repositioning, hit-and-run pressure, or opportunistic strikes.",
    doesNotApplyWhen: [
      "The creature mainly opens from stealth once and then behaves like another clearer combat role.",
      "The stronger identity is brute, ambusher, or artillery rather than mobile harassment."
    ],
    family: "combat_role",
    id: "creature:skirmisher_combatant",
    label: "skirmisher_combatant",
    translationStatus: "mapped"
  },
  sky_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "sky_setting",
    currentTag: "sky_setting",
    description: "Strongly associated with open skies, storm clouds, or high-altitude aerial habitats.",
    family: "habitat_setting",
    id: "creature:sky_setting",
    label: "sky_setting",
    translationStatus: "mapped"
  },
  small_settlement_setting: {
    adjacentTags: [
      "urban_setting",
      "rural_setting"
    ],
    appliesWhen: [
      "The creature is repeatedly framed around villages, hamlets, or small town life.",
      "Its encounter identity is tied to low-density community spaces rather than major urban districts."
    ],
    assignmentMode: "editorial",
    axis: "setting",
    category: "creature",
    conceptId: "small_settlement_setting",
    currentTag: "small_settlement_setting",
    description: "Strongly associated with villages, hamlets, small towns, or other low-density community settlements.",
    doesNotApplyWhen: [
      "The record only says the creature can appear near people.",
      "The creature is more strongly urban, rural, or fortress-coded than village-coded."
    ],
    family: "site_setting",
    id: "creature:small_settlement_setting",
    label: "small_settlement_setting",
    translationStatus: "mapped"
  },
  spawn_creator: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "spawn_creation",
    currentTag: "spawn_creator",
    description: "Creates additional threats through infestation, spawn-making, conversion, or implanted offspring.",
    family: "threat_profile",
    id: "creature:spawn_creator",
    label: "spawn_creation",
    translationStatus: "mapped"
  },
  stitched_horror: {
    assignmentMode: "hybrid",
    axis: "presentation",
    category: "creature",
    conceptId: "stitched_horror",
    currentTag: "stitched_horror",
    description: "Strongly associated with sutures, patchwork flesh, sewn bodies, or visibly assembled corpse craftsmanship.",
    family: "visual_motif",
    id: "creature:stitched_horror",
    label: "stitched_horror",
    translationStatus: "mapped"
  },
  support_combatant: {
    adjacentTags: [
      "commander_combatant",
      "controller_combatant"
    ],
    appliesWhen: [
      "The creature is naturally retrieved because it keeps allies alive, stronger, better positioned, or otherwise more dangerous.",
      "Ally enablement matters more than its own direct kill pressure."
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "support_combatant",
    currentTag: "support_combatant",
    description: "Built to heal, buff, protect, command, or otherwise enable allied creatures more than acting as the primary damage source.",
    doesNotApplyWhen: [
      "The creature has one incidental buff or heal but is otherwise a brute, artillery, or controller.",
      "The stronger identity is commander_combatant because leadership and coordination outweigh broad support."
    ],
    family: "combat_role",
    id: "creature:support_combatant",
    label: "support_combatant",
    translationStatus: "mapped"
  },
  swamp_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "swamp_setting",
    currentTag: "swamp_setting",
    description: "Strongly associated with bogs, marshes, fens, or mires.",
    family: "habitat_setting",
    id: "creature:swamp_setting",
    label: "swamp_setting",
    translationStatus: "mapped"
  },
  temple_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "temple_setting",
    currentTag: "temple_setting",
    description: "Strongly associated with temples, shrines, monasteries, or other sacred encounter sites.",
    family: "site_setting",
    id: "creature:temple_setting",
    label: "temple_setting",
    translationStatus: "mapped"
  },
  terrain_control_threat: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "creature",
    conceptId: "terrain_control",
    currentTag: "terrain_control_threat",
    description: "Threat defined by webs, walls, zones, hazards, or other space-shaping control that changes battlefield movement.",
    family: "threat_profile",
    id: "creature:terrain_control_threat",
    label: "terrain_control",
    translationStatus: "mapped"
  },
  tian_xia_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "tian_xia_setting",
    currentTag: "tian_xia_setting",
    description: "Strongly associated with Tian Xia and its major cultural subregions, where Tian-rooted regional framing materially affects creature planning and retrieval.",
    family: "regional_setting",
    id: "creature:tian_xia_setting",
    label: "tian_xia_setting",
    translationStatus: "mapped"
  },
  trickster_mischief: {
    adjacentTags: [
      "carnival_show",
      "disguised_pretender"
    ],
    appliesWhen: [
      "Pranks, baiting humor, whimsical menace, or deliberate trickster conduct are a central retrieval hook.",
      "The creature is naturally sought for playful-but-dangerous chaos rather than generic destruction or villainy."
    ],
    assignmentMode: "hybrid",
    axis: "presentation",
    category: "creature",
    conceptId: "trickster_mischief",
    currentTag: "trickster_mischief",
    description: "Strongly associated with pranks, capricious humor, gleeful sabotage, or explicit trickster behavior.",
    doesNotApplyWhen: [
      "The creature is merely chaotic, unpredictable, or destructive without a prankster or mischief-facing identity.",
      "Whimsical presentation appears only as surface flavor and another presentation tag or combat role better explains why the creature is being retrieved."
    ],
    family: "genre_motif",
    id: "creature:trickster_mischief",
    label: "trickster_mischief",
    translationStatus: "mapped"
  },
  undead_adjacent: {
    assignmentMode: "hybrid",
    axis: "specialization",
    category: "creature",
    conceptId: "undead_family",
    currentTag: "undead_adjacent",
    description: "Groups undead and closely undead-coded native signals into one retrieval bucket.",
    family: "ontology_cluster",
    id: "creature:undead_adjacent",
    label: "undead_family",
    nativeOntologyPolicy: "aggregates_native_signals",
    translationStatus: "mapped"
  },
  undead_war_torn_region_setting: {
    adjacentTags: [
      "organized_undead_society_setting",
      "battlefield_setting"
    ],
    appliesWhen: [
      "Use when the creature is naturally retrieved for undead war-frontier planning: ruined forts, haunted battlefields, occupied borderlands, refugee routes, or crusader-collapse aftermath.",
      "The planning value comes from a region still actively shaped by undead war, occupation, or collapse rather than by a stable undead social order."
    ],
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "undead_war_torn_region_setting",
    currentTag: "undead_war_torn_region_setting",
    description: "Strongly associated with undead occupation, shattered crusader lands, haunted battlefields, refugee pressure, and ruined strongholds under deathless siege. In Golarion, this primarily corresponds to the Gravelands.",
    doesNotApplyWhen: [
      "The creature is simply undead, martial, or grim without a meaningful tie to an undead-ravaged frontier or occupied war zone.",
      "The stronger fit is organized_undead_society_setting because the retrieval hook is necromantic civilization rather than a shattered war front."
    ],
    family: "regional_setting",
    id: "creature:undead_war_torn_region_setting",
    label: "undead_war_torn_region_setting",
    translationStatus: "mapped"
  },
  underground_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "underground_setting",
    currentTag: "underground_setting",
    description: "Strongly associated with caves, tunnels, crypts, or subterranean spaces.",
    family: "habitat_setting",
    id: "creature:underground_setting",
    label: "underground_setting",
    translationStatus: "mapped"
  },
  upper_plane_setting: {
    adjacentTags: [
      "heaven_setting",
      "nirvana_setting",
      "elysium_setting"
    ],
    assignmentMode: "composite",
    axis: "setting",
    category: "creature",
    compositeOfAnyTags: [
      "heaven_setting",
      "nirvana_setting",
      "elysium_setting"
    ],
    conceptId: "upper_plane_setting",
    currentTag: "upper_plane_setting",
    description: "Strongly associated with the upper planes of Heaven, Nirvana, or Elysium.",
    family: "planar_setting",
    id: "creature:upper_plane_setting",
    label: "upper_plane_setting",
    nativeOntologyPolicy: "aggregates_native_signals",
    translationStatus: "provisional"
  },
  urban_setting: {
    adjacentTags: [
      "small_settlement_setting",
      "fortress_setting"
    ],
    appliesWhen: [
      "The creature is primarily framed as belonging in city or sewer encounter spaces.",
      "Urban placement is a recurring part of its identity, role, or habitat."
    ],
    assignmentMode: "editorial",
    axis: "setting",
    category: "creature",
    conceptId: "urban_setting",
    currentTag: "urban_setting",
    description: "Strongly associated with urban encounter scenes such as cities, streets, alleys, dense buildings, markets, or sewers.",
    doesNotApplyWhen: [
      "The record merely mentions a city once.",
      "The creature can appear in towns but is not specifically urban-coded.",
      "The creature is better modeled by fortress_setting or small_settlement_setting."
    ],
    family: "site_setting",
    id: "creature:urban_setting",
    label: "urban_setting",
    negativeSignals: [
      "generic settlement mention",
      "single adventure location",
      "fortress-only residence"
    ],
    positiveSignals: [
      "city guard roles",
      "sewer habitat",
      "street or alley patrol scenes",
      "market or district anchoring"
    ],
    translationStatus: "mapped"
  },
  vengeful_tragedy: {
    adjacentTags: [
      "funerary_mourning",
      "disguised_pretender"
    ],
    appliesWhen: [
      "The creature is framed around betrayal, mourning, loss, or an unresolved wrong curdling into vengeance.",
      "A GM would retrieve it for tragic revenants, wronged spirits, or revenge-driven story beats rather than generic hostility."
    ],
    assignmentMode: "editorial",
    axis: "presentation",
    category: "creature",
    conceptId: "vengeful_tragedy",
    currentTag: "vengeful_tragedy",
    description: "Strongly associated with betrayal, grief, injustice, or a sorrowful wrong returning as vengeance.",
    doesNotApplyWhen: [
      "The creature is simply angry, hostile, or undead without a strong tragic-injustice presentation.",
      "The stronger fit is funerary_mourning because grief and ritual loss matter more than vengeance."
    ],
    family: "story_motif",
    id: "creature:vengeful_tragedy",
    label: "vengeful_tragedy",
    translationStatus: "mapped"
  },
  void_tainted: {
    adjacentTags: [
      "cosmic_dread",
      "forbidden_knowledge"
    ],
    assignmentMode: "hybrid",
    axis: "specialization",
    category: "creature",
    conceptId: "void_tainted",
    currentTag: "void_tainted",
    description: "Creature strongly defined by void corruption, cosmic hollowness, or metaphysical pollution that feels alien, cold, or reality-thinning.",
    family: "corruption_profile",
    id: "creature:void_tainted",
    label: "void_tainted",
    translationStatus: "mapped"
  },
  volcanic_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "volcanic_setting",
    currentTag: "volcanic_setting",
    description: "Strongly associated with volcanoes, calderas, lava, or magma.",
    family: "habitat_setting",
    id: "creature:volcanic_setting",
    label: "volcanic_setting",
    translationStatus: "mapped"
  },
  warband_member: {
    adjacentTags: [
      "commander_combatant",
      "battlefield_setting"
    ],
    assignmentMode: "editorial",
    axis: "encounter",
    category: "creature",
    conceptId: "warband_member",
    currentTag: "warband_member",
    description: "Naturally retrieved as one body in a raiding party, battle line, war camp, or other organized hostile fighting band.",
    family: "cohort_role",
    id: "creature:warband_member",
    label: "warband_member",
    translationStatus: "mapped"
  },
  wasteland_setting: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "creature",
    conceptId: "wasteland_setting",
    currentTag: "wasteland_setting",
    description: "Strongly associated with barren wastes, blasted wastelands, or desolate badlands.",
    family: "habitat_setting",
    id: "creature:wasteland_setting",
    label: "wasteland_setting",
    translationStatus: "mapped"
  },
  watcher_npc: {
    adjacentTags: [
      "guardian_npc",
      "infiltrator_npc"
    ],
    appliesWhen: [
      "The creature is naturally retrieved as a lookout, sentry, rooftop watcher, scout-on-post, or other early-warning presence.",
      "Observation and alarm value matter more than physically blocking passage, bodyguarding a charge, or acting as a general frontline enforcer."
    ],
    assignmentMode: "editorial",
    axis: "npc_role",
    category: "creature",
    conceptId: "watcher_npc",
    currentTag: "watcher_npc",
    description: "Immediate-scenario lookout, sentry, observer, or patrol-point watcher whose scene value is warning, spotting, or noticing intruders.",
    doesNotApplyWhen: [
      "The creature is simply a posted guard or enforcer without a strong surveillance, early-warning, or lookout function.",
      "The stronger fit is guide_npc or civic_npc because travel knowledge or social embedding matters more than active watch duty."
    ],
    family: "scene_role",
    id: "creature:watcher_npc",
    label: "watcher_npc",
    translationStatus: "mapped"
  }
};
