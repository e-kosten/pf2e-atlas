import type { DerivedTagCategoryProjection } from "../../../domain/derived-tag-types.js";

export const SPELL_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG: Record<string, DerivedTagCategoryProjection> = 
{
  action_denial: {
    assignmentMode: "hybrid",
    axis: "battlefield",
    category: "spell",
    conceptId: "action_denial",
    currentTag: "action_denial",
    description: "Denies actions through paralysis, stupefying shutdown, slowed tempo, or similarly severe turn disruption.",
    family: "control",
    id: "spell:action_denial",
    label: "action_denial",
    translationStatus: "provisional"
  },
  affliction_cleanup: {
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "affliction_cleanup",
    currentTag: "affliction_cleanup",
    description: "Cleanses, cures, neutralizes, or removes disease, poison, curse, or similar afflictions.",
    family: "support",
    id: "spell:affliction_cleanup",
    label: "affliction_cleanup",
    translationStatus: "provisional"
  },
  alarm: {
    adjacentTags: [
      "protective_ward",
      "scrying_protection"
    ],
    appliesWhen: [
      "The spell is naturally retrieved to warn about intrusion, threshold crossing, tampering, or unwanted entry.",
      "Detection and notice matter more than directly stopping the intruder."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "alarm",
    currentTag: "alarm",
    description: "Alerts you or others when a watched area, threshold, or ward is crossed.",
    doesNotApplyWhen: [
      "The spell mainly protects, blocks, or hides the target without providing a warning function.",
      "The spell only reveals truth or magic generally rather than guarding a watched perimeter."
    ],
    family: "security",
    id: "spell:alarm",
    label: "alarm",
    translationStatus: "provisional"
  },
  animal_form: {
    assignmentMode: "deterministic",
    axis: "transformation",
    category: "spell",
    conceptId: "animal_form",
    currentTag: "animal_form",
    description: "Transforms a creature into an animal, beast, pest, or similar natural form.",
    family: "transformation",
    id: "spell:animal_form",
    label: "animal_form",
    translationStatus: "provisional"
  },
  anti_bleed: {
    adjacentTags: [
      "healing_support",
      "condition_support"
    ],
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "bleed_remediation",
    currentTag: "anti_bleed",
    description: "Staunches bleeding, ends persistent bleed damage, or closes wounds that keep draining a target.",
    family: "support",
    id: "spell:anti_bleed",
    label: "bleed_remediation",
    translationStatus: "provisional"
  },
  anti_caster_disruption: {
    adjacentTags: [
      "countermagic",
      "silencing"
    ],
    assignmentMode: "hybrid",
    axis: "battlefield",
    category: "spell",
    conceptId: "caster_disruption_remediation",
    currentTag: "anti_caster_disruption",
    description: "Disrupts casting, punishes spell use, or specifically suppresses hostile spellcasters in the moment.",
    family: "control",
    id: "spell:anti_caster_disruption",
    label: "caster_disruption_remediation",
    translationStatus: "provisional"
  },
  anti_confusion: {
    adjacentTags: [
      "condition_support",
      "anti_fear"
    ],
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "confusion_remediation",
    currentTag: "anti_confusion",
    description: "Ends confusion, steadies a disordered mind, or protects a target from confusion-like mental unraveling.",
    family: "support",
    id: "spell:anti_confusion",
    label: "confusion_remediation",
    translationStatus: "provisional"
  },
  anti_disease: {
    adjacentTags: [
      "affliction_cleanup",
      "anti_poison"
    ],
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "disease_remediation",
    currentTag: "anti_disease",
    description: "Cures disease, counteracts infections, or protects a target against plague, fever, and similar disease effects.",
    family: "support",
    id: "spell:anti_disease",
    label: "disease_remediation",
    translationStatus: "provisional"
  },
  anti_fear: {
    adjacentTags: [
      "condition_support",
      "anti_confusion"
    ],
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "fear_remediation",
    currentTag: "anti_fear",
    description: "Counters frightened or fear effects, bolsters courage, or protects a target against fear.",
    family: "support",
    id: "spell:anti_fear",
    label: "fear_remediation",
    translationStatus: "provisional"
  },
  anti_paralysis: {
    adjacentTags: [
      "condition_support",
      "escape_support"
    ],
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "paralysis_remediation",
    currentTag: "anti_paralysis",
    description: "Ends paralysis, restores bodily function, or frees a target from magic or afflictions that leave it unable to move.",
    family: "support",
    id: "spell:anti_paralysis",
    label: "paralysis_remediation",
    translationStatus: "provisional"
  },
  anti_petrification: {
    adjacentTags: [
      "affliction_cleanup",
      "condition_support"
    ],
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "petrification_remediation",
    currentTag: "anti_petrification",
    description: "Prevents, reverses, or counteracts petrification and other turn-to-stone effects.",
    family: "support",
    id: "spell:anti_petrification",
    label: "petrification_remediation",
    translationStatus: "provisional"
  },
  anti_poison: {
    adjacentTags: [
      "affliction_cleanup",
      "anti_disease"
    ],
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "poison_remediation",
    currentTag: "anti_poison",
    description: "Cures poison, counters toxic afflictions, or protects a target against venom and similar poisoning effects.",
    family: "support",
    id: "spell:anti_poison",
    label: "poison_remediation",
    translationStatus: "provisional"
  },
  aquatic_support: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "spell",
    conceptId: "aquatic_support",
    currentTag: "aquatic_support",
    description: "Helps with swimming, underwater breathing, water-surface travel, or other aquatic movement.",
    family: "expedition",
    id: "spell:aquatic_support",
    label: "aquatic_support",
    translationStatus: "provisional"
  },
  barrier_bypass: {
    adjacentTags: [
      "lock_bypass",
      "countermagic"
    ],
    appliesWhen: [
      "The spell is naturally retrieved to pass through, nullify, or ignore a blocking wall, sealed threshold, force barrier, or magical ward.",
      "Crossing the obstruction matters more than simply traveling farther or counteracting magic in the abstract."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "barrier_bypass",
    currentTag: "barrier_bypass",
    description: "Gets a creature through a blocked threshold, wall, seal, force barrier, or magical ward that otherwise prevents passage.",
    doesNotApplyWhen: [
      "The spell only unlocks a door or manipulates a mechanism without really solving a barrier or ward.",
      "The spell's value is ordinary travel or relocation rather than penetrating a blocked passage."
    ],
    family: "access_bypass",
    id: "spell:barrier_bypass",
    label: "barrier_bypass",
    translationStatus: "provisional"
  },
  barrier_creation: {
    assignmentMode: "hybrid",
    axis: "battlefield",
    category: "spell",
    conceptId: "barrier_creation",
    currentTag: "barrier_creation",
    description: "Creates a wall, dome, cage, force barrier, or other discrete blocking structure that reshapes access lines.",
    family: "control",
    id: "spell:barrier_creation",
    label: "barrier_creation",
    translationStatus: "provisional"
  },
  battle_form: {
    assignmentMode: "deterministic",
    axis: "transformation",
    category: "spell",
    conceptId: "battle_form",
    currentTag: "battle_form",
    description: "Transforms a creature into a combat-ready form with new statistics or battle-form language.",
    family: "transformation",
    id: "spell:battle_form",
    label: "battle_form",
    translationStatus: "provisional"
  },
  battlefield_disruption: {
    assignmentMode: "deterministic",
    axis: "battlefield",
    category: "spell",
    conceptId: "battlefield_disruption",
    currentTag: "battlefield_disruption",
    description: "Creates area denial, difficult terrain, barriers, or other battlefield obstacles.",
    family: "control",
    id: "spell:battlefield_disruption",
    label: "battlefield_disruption",
    translationStatus: "provisional"
  },
  burst_damage: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "spell",
    conceptId: "burst_damage",
    currentTag: "burst_damage",
    description: "Delivers concentrated damage in a spike or splash pattern that users naturally retrieve as a damage-first answer.",
    family: "impact",
    id: "spell:burst_damage",
    label: "burst_damage",
    translationStatus: "provisional"
  },
  charm_influence: {
    adjacentTags: [
      "emotion_control",
      "compulsion_control"
    ],
    appliesWhen: [
      "The spell's main value is improving a target's attitude, trust, or willingness to cooperate.",
      "The spell changes social reception more than it scripts exact behavior."
    ],
    assignmentMode: "hybrid",
    axis: "influence",
    category: "spell",
    conceptId: "charm_influence",
    currentTag: "charm_influence",
    description: "Wins cooperation through friendliness, fascination, admiration, or magically altered social regard.",
    doesNotApplyWhen: [
      "The spell compels exact actions, overrides agency, or takes total control.",
      "The spell only manipulates mood without establishing a social bond or regard shift."
    ],
    family: "influence",
    id: "spell:charm_influence",
    label: "charm_influence",
    translationStatus: "provisional"
  },
  communication: {
    adjacentTags: [
      "signaling",
      "telepathic_communication",
      "message_delivery",
      "translation_support"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "spell",
    compositeOfAnyTags: [
      "signaling",
      "telepathic_communication",
      "message_delivery",
      "translation_support"
    ],
    conceptId: "communication",
    currentTag: "communication",
    description: "Broad communication umbrella for spells used to signal allies, relay messages, bridge language barriers, or coordinate silently.",
    family: "communication",
    id: "spell:communication",
    label: "communication",
    translationStatus: "provisional"
  },
  compulsion_control: {
    adjacentTags: [
      "charm_influence",
      "domination"
    ],
    appliesWhen: [
      "The spell explicitly pressures the target into doing something, moving somewhere, or obeying a commanded pattern.",
      "Loss of agency is more important than affection, calm, or broad mood change."
    ],
    assignmentMode: "hybrid",
    axis: "influence",
    category: "spell",
    conceptId: "compulsion_control",
    currentTag: "compulsion_control",
    description: "Forces scripted behavior, movement, or obedience against a target's normal will.",
    doesNotApplyWhen: [
      "The spell merely charms or emotionally softens the target.",
      "The spell fully dominates the target over sustained actions rather than issuing narrower commands."
    ],
    family: "influence",
    id: "spell:compulsion_control",
    label: "compulsion_control",
    translationStatus: "provisional"
  },
  concealment: {
    assignmentMode: "deterministic",
    axis: "battlefield",
    category: "spell",
    conceptId: "concealment",
    currentTag: "concealment",
    description: "Makes a creature hard to see, hidden, concealed, or undetected.",
    family: "control",
    id: "spell:concealment",
    label: "concealment",
    translationStatus: "provisional"
  },
  condition_support: {
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "condition_support",
    currentTag: "condition_support",
    description: "Delays, suppresses, or removes afflictions and conditions.",
    family: "support",
    id: "spell:condition_support",
    label: "condition_support",
    translationStatus: "provisional"
  },
  consultation: {
    adjacentTags: [
      "lore_consultation",
      "problem_diagnosis",
      "omen_guidance"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "spell",
    compositeOfAnyTags: [
      "lore_consultation",
      "problem_diagnosis",
      "omen_guidance"
    ],
    conceptId: "consultation",
    currentTag: "consultation",
    description: "Broad consultation umbrella for spells used to seek cosmic answers, diagnose mysteries, or gain non-sensory divinatory guidance.",
    family: "consultation",
    id: "spell:consultation",
    label: "consultation",
    translationStatus: "provisional"
  },
  contamination_cleanup: {
    adjacentTags: [
      "quarantine_containment",
      "source_cleanup"
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "contamination_cleanup",
    currentTag: "contamination_cleanup",
    description: "Cleanses tainted residue, neutralizes corrupted ground, removes lingering pollution, or purifies a contaminated space.",
    family: "resolution",
    id: "spell:contamination_cleanup",
    label: "contamination_cleanup",
    translationStatus: "provisional"
  },
  countermagic: {
    adjacentTags: [
      "magic_detection",
      "protective_ward"
    ],
    appliesWhen: [
      "The spell is naturally retrieved because stopping, unravelling, or suppressing existing magic is its main job.",
      "Anti-magic response matters more than simple protection, detection, or concealment."
    ],
    assignmentMode: "hybrid",
    axis: "battlefield",
    category: "spell",
    conceptId: "active_magic_counteraction",
    currentTag: "countermagic",
    description: "Counteracts, dispels, suppresses, or shuts down magic.",
    doesNotApplyWhen: [
      "The spell mainly protects targets from harm without actually disrupting hostile magic.",
      "The spell only reveals or warns about magic rather than counteracting it."
    ],
    family: "control",
    id: "spell:countermagic",
    label: "countermagic",
    translationStatus: "provisional"
  },
  creature_summoning: {
    assignmentMode: "hybrid",
    axis: "summoning",
    category: "spell",
    conceptId: "creature_summoning",
    currentTag: "creature_summoning",
    description: "Summons, conjures, or calls creatures to act as temporary allies, tools, or battlefield assets.",
    family: "summoning",
    id: "spell:creature_summoning",
    label: "creature_summoning",
    translationStatus: "provisional"
  },
  crowd_clearing: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "spell",
    conceptId: "crowd_clearing",
    currentTag: "crowd_clearing",
    description: "Damages or wipes clusters of weaker foes and is naturally retrieved as an anti-group answer.",
    family: "impact",
    id: "spell:crowd_clearing",
    label: "crowd_clearing",
    translationStatus: "provisional"
  },
  curse_removal: {
    adjacentTags: [
      "exorcism",
      "sanctification"
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "curse_remediation",
    currentTag: "curse_removal",
    description: "Breaks, removes, or counteracts curses as a direct answer path rather than only suppressing symptoms.",
    family: "resolution",
    id: "spell:curse_removal",
    label: "curse_remediation",
    translationStatus: "provisional"
  },
  curse_revelation: {
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "curse_discovery",
    currentTag: "curse_revelation",
    description: "Identifies curses, spiritual corruption, or other malign supernatural bindings on a target.",
    family: "revelation",
    id: "spell:curse_revelation",
    label: "curse_discovery",
    translationStatus: "provisional"
  },
  death_prevention: {
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "death_prevention",
    currentTag: "death_prevention",
    description: "Prevents death, stabilizes the dying, or brings a creature back from the brink.",
    family: "support",
    id: "spell:death_prevention",
    label: "death_prevention",
    translationStatus: "provisional"
  },
  disguise: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "spell",
    conceptId: "disguise",
    currentTag: "disguise",
    description: "Helps alter appearance or impersonate another identity.",
    family: "infiltration",
    id: "spell:disguise",
    label: "disguise",
    translationStatus: "provisional"
  },
  domination: {
    adjacentTags: [
      "compulsion_control",
      "action_denial"
    ],
    appliesWhen: [
      "The spell grants ongoing, high-authority control over what the target does rather than just one compelled action.",
      "A user would retrieve it as a takeover spell, not merely a charm or suggestion spell."
    ],
    assignmentMode: "hybrid",
    axis: "influence",
    category: "spell",
    conceptId: "domination",
    currentTag: "domination",
    description: "Seizes sustained control over a target's actions, body, or tactical decision-making.",
    doesNotApplyWhen: [
      "The spell only improves attitude, stirs emotion, or issues narrower one-off compulsions.",
      "The spell mainly suppresses actions without redirecting them under the caster's control."
    ],
    family: "influence",
    id: "spell:domination",
    label: "domination",
    translationStatus: "provisional"
  },
  eidolon_support: {
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "eidolon_support",
    currentTag: "eidolon_support",
    description: "Directly benefits an eidolon or the summoner-eidolon bond.",
    family: "support",
    id: "spell:eidolon_support",
    label: "eidolon_support",
    translationStatus: "provisional"
  },
  elemental_form: {
    assignmentMode: "deterministic",
    axis: "transformation",
    category: "spell",
    conceptId: "elemental_form",
    currentTag: "elemental_form",
    description: "Transforms a creature into an elemental form.",
    family: "transformation",
    id: "spell:elemental_form",
    label: "elemental_form",
    translationStatus: "provisional"
  },
  emotion_control: {
    adjacentTags: [
      "fear_pressure",
      "charm_influence"
    ],
    appliesWhen: [
      "The spell is naturally retrieved for changing a creature's feelings, morale, or emotional volatility.",
      "The emotional state change matters more than explicit obedience or truth extraction."
    ],
    assignmentMode: "hybrid",
    axis: "influence",
    category: "spell",
    conceptId: "emotion_control",
    currentTag: "emotion_control",
    description: "Directly manipulates fear, calm, rage, love, despair, or other emotional states.",
    doesNotApplyWhen: [
      "The spell chiefly compels discrete actions or sustained domination.",
      "The spell only inflicts fear as damage pressure without broader emotional steering."
    ],
    family: "influence",
    id: "spell:emotion_control",
    label: "emotion_control",
    translationStatus: "provisional"
  },
  environmental_adaptation: {
    adjacentTags: [
      "aquatic_support",
      "field_shelter",
      "resistance_support"
    ],
    appliesWhen: [
      "The spell is naturally retrieved to survive extreme heat, cold, altitude, smoke, pressure, or other punishing environmental conditions during travel or exploration.",
      "Environmental endurance matters more than only resisting one attack form or creating a place to rest."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "environmental_adaptation",
    currentTag: "environmental_adaptation",
    description: "Helps creatures endure hostile climates, thin air, smoke, pressure, vacuum, or other expedition-grade environmental extremes.",
    doesNotApplyWhen: [
      "The spell mainly grants combat resistance, a protective ward, or aquatic mobility without broader expedition-survival value.",
      "The spell only creates shelter or sustenance rather than adapting creatures to the surrounding environment."
    ],
    family: "expedition",
    id: "spell:environmental_adaptation",
    label: "environmental_adaptation",
    translationStatus: "provisional"
  },
  escape_support: {
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "escape_support",
    currentTag: "escape_support",
    description: "Helps a creature slip away, break free, flee, or evade pursuit.",
    family: "support",
    id: "spell:escape_support",
    label: "escape_support",
    translationStatus: "provisional"
  },
  exorcism: {
    adjacentTags: [
      "curse_removal",
      "sanctification"
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "hostile_presence_expulsion",
    currentTag: "exorcism",
    description: "Banishes, expels, or spiritually drives out a hostile spirit, possession, haunt, or invading supernatural presence.",
    family: "resolution",
    id: "spell:exorcism",
    label: "exorcism",
    translationStatus: "provisional"
  },
  expedition: {
    adjacentTags: [
      "navigation",
      "field_shelter",
      "environmental_adaptation"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "spell",
    compositeOfAnyTags: [
      "navigation",
      "flight",
      "aquatic_support",
      "sustenance",
      "field_shelter",
      "environmental_adaptation",
      "wayfinding"
    ],
    conceptId: "expedition",
    currentTag: "expedition",
    description: "Broad expedition umbrella for spells used for routefinding, travel-ready movement, shelter, sustainment, aquatic operations, and hostile-environment survival.",
    family: "expedition",
    id: "spell:expedition",
    label: "expedition",
    translationStatus: "provisional"
  },
  extraction_teleport: {
    adjacentTags: [
      "short_range_teleport",
      "escape_support"
    ],
    appliesWhen: [
      "The spell is naturally retrieved as an escape, rescue, or anti-capture tool rather than only a movement spell.",
      "The reposition breaks danger, confinement, or immediate battlefield pressure."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "extraction_teleport",
    currentTag: "extraction_teleport",
    description: "Teleports a creature out of danger, through restraints, or away from immediate threat pressure.",
    doesNotApplyWhen: [
      "The spell is mostly a neutral short-range blink or a long-distance travel effect.",
      "The spell primarily opens planar movement rather than emergency extraction."
    ],
    family: "teleportation",
    id: "spell:extraction_teleport",
    label: "extraction_teleport",
    translationStatus: "provisional"
  },
  fear_pressure: {
    assignmentMode: "deterministic",
    axis: "battlefield",
    category: "spell",
    conceptId: "fear_pressure",
    currentTag: "fear_pressure",
    description: "Forces fear, panic, dread, or morale collapse onto a target.",
    family: "control",
    id: "spell:fear_pressure",
    label: "fear_pressure",
    translationStatus: "provisional"
  },
  field_shelter: {
    adjacentTags: [
      "protective_ward",
      "planar_travel"
    ],
    appliesWhen: [
      "The spell is naturally retrieved to create a campsite refuge, safe resting place, or expedition shelter in hostile territory.",
      "Its value is prolonged field habitation or protected rest rather than momentary combat defense."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "field_shelter",
    currentTag: "field_shelter",
    description: "Creates shelter, refuge, or a protected resting place in the field.",
    doesNotApplyWhen: [
      "The spell only creates a brief combat ward, cover effect, or instant defensive barrier.",
      "The spell merely transports creatures away instead of establishing a place to rest."
    ],
    family: "expedition",
    id: "spell:field_shelter",
    label: "field_shelter",
    translationStatus: "provisional"
  },
  flight: {
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "flight",
    currentTag: "flight",
    description: "Grants flying movement, sustained aerial travel, or practical airborne maneuvering.",
    family: "expedition",
    id: "spell:flight",
    label: "flight",
    translationStatus: "provisional"
  },
  forced_movement: {
    assignmentMode: "deterministic",
    axis: "battlefield",
    category: "spell",
    conceptId: "forced_movement",
    currentTag: "forced_movement",
    description: "Pushes, pulls, drags, or otherwise repositions a target against its will.",
    family: "control",
    id: "spell:forced_movement",
    label: "forced_movement",
    translationStatus: "provisional"
  },
  hazard_revelation: {
    adjacentTags: [
      "magic_detection",
      "scouting"
    ],
    appliesWhen: [
      "The spell is naturally retrieved to uncover traps, hidden dangers, secret magical wards, or dangerous concealed features in a location.",
      "Hazard discovery matters more than general magical detection or long-range scouting."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "hazard_discovery",
    currentTag: "hazard_revelation",
    description: "Reveals hidden traps, secret wards, concealed passage dangers, or other obscured environmental threats.",
    doesNotApplyWhen: [
      "The spell only detects magic, invisible creatures, or truth without specifically surfacing dangerous hidden features.",
      "The spell merely scouts an area from afar without exposing concealed trap logic or hazard placement."
    ],
    family: "revelation",
    id: "spell:hazard_revelation",
    label: "hazard_discovery",
    translationStatus: "provisional"
  },
  healing_support: {
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "healing_support",
    currentTag: "healing_support",
    description: "Directly restores hit points or accelerates recovery.",
    family: "support",
    id: "spell:healing_support",
    label: "healing_support",
    translationStatus: "provisional"
  },
  illumination: {
    adjacentTags: [
      "senses_support",
      "line_of_sight_control"
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "illumination",
    currentTag: "illumination",
    description: "Produces practical light that brightens darkness, reveals an area, or lets creatures see more clearly.",
    family: "sensory_support",
    id: "spell:illumination",
    label: "illumination",
    translationStatus: "provisional"
  },
  infiltration: {
    adjacentTags: [
      "stealth_support",
      "disguise",
      "social_infiltration"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "spell",
    compositeOfAnyTags: [
      "stealth_support",
      "disguise",
      "social_infiltration"
    ],
    conceptId: "infiltration",
    currentTag: "infiltration",
    description: "Broad infiltration umbrella for quiet-entry, disguise, and covert social-passing spells.",
    family: "infiltration",
    id: "spell:infiltration",
    label: "infiltration",
    translationStatus: "provisional"
  },
  initiative_support: {
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "initiative_support",
    currentTag: "initiative_support",
    description: "Improves initiative, pre-combat readiness, or the party's opening tempo before the first turn.",
    family: "support",
    id: "spell:initiative_support",
    label: "initiative_support",
    translationStatus: "provisional"
  },
  invisibility_reveal: {
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "invisibility_discovery",
    currentTag: "invisibility_reveal",
    description: "Exposes invisible, hidden, concealed, or magically obscured creatures and objects.",
    family: "revelation",
    id: "spell:invisibility_reveal",
    label: "invisibility_discovery",
    translationStatus: "provisional"
  },
  line_of_sight_control: {
    assignmentMode: "deterministic",
    axis: "battlefield",
    category: "spell",
    conceptId: "line_of_sight_control",
    currentTag: "line_of_sight_control",
    description: "Blocks vision, obscures sight lines, or denies clear observation across an area.",
    family: "control",
    id: "spell:line_of_sight_control",
    label: "line_of_sight_control",
    translationStatus: "provisional"
  },
  lock_bypass: {
    adjacentTags: [
      "trap_bypass",
      "barrier_bypass"
    ],
    appliesWhen: [
      "The spell is naturally retrieved to unlock, unseal, or open a secured entry point, door, chest, manacle, or similar closure.",
      "Accessing something closed matters more than broad movement, damage, or generic anti-magic."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "lock_bypass",
    currentTag: "lock_bypass",
    description: "Opens locks, sealed containers, secured doors, or similar closed access points through magic rather than physical lockpicking.",
    doesNotApplyWhen: [
      "The spell mainly destroys the obstacle, bypasses the whole wall, or teleports past the problem without interacting with the locked access point.",
      "The spell only manipulates unattended objects generally and opening secured access is not a real retrieval hook."
    ],
    family: "access_bypass",
    id: "spell:lock_bypass",
    label: "lock_bypass",
    translationStatus: "provisional"
  },
  long_range_teleport: {
    adjacentTags: [
      "short_range_teleport",
      "planar_travel"
    ],
    appliesWhen: [
      "The spell is naturally retrieved for strategic travel, relocation, or bypassing long routes.",
      "The destination scale is substantially larger than one encounter map or immediate tactical space."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "long_range_teleport",
    currentTag: "long_range_teleport",
    description: "Teleports creatures across major overland distances, settlements, or remote destinations.",
    doesNotApplyWhen: [
      "The spell is mainly a tactical blink, extraction, or planar crossing effect.",
      "The spell only repositions creatures within the same immediate scene."
    ],
    family: "teleportation",
    id: "spell:long_range_teleport",
    label: "long_range_teleport",
    translationStatus: "provisional"
  },
  lore_consultation: {
    adjacentTags: [
      "truth_reveal",
      "problem_diagnosis"
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "lore_consultation",
    currentTag: "lore_consultation",
    description: "Provides interpretive insight, shared knowledge, or focused understanding about a subject, clue, history, or magical situation.",
    family: "consultation",
    id: "spell:lore_consultation",
    label: "lore_consultation",
    translationStatus: "provisional"
  },
  magic_detection: {
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "magic_discovery",
    currentTag: "magic_detection",
    description: "Reveals magical auras, spell presence, active effects, or other supernatural signatures.",
    family: "revelation",
    id: "spell:magic_detection",
    label: "magic_discovery",
    translationStatus: "provisional"
  },
  mechanism_manipulation: {
    adjacentTags: [
      "lock_bypass",
      "trap_bypass"
    ],
    appliesWhen: [
      "The spell is naturally retrieved to operate a lever, button, latch, control panel, pressure surface, or similar mechanism from a safe or unusual position.",
      "The mechanism interaction itself matters more than broad telekinesis, damage, or ordinary object movement."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "mechanism_manipulation",
    currentTag: "mechanism_manipulation",
    description: "Precisely triggers, moves, holds, or operates levers, buttons, switches, pressure plates, locks, or similar scene mechanisms.",
    doesNotApplyWhen: [
      "The spell only moves creatures or loose objects without a real access-, control-, or mechanism-facing use case.",
      "The spell bypasses the obstacle by teleporting or destroying it instead of operating the mechanism."
    ],
    family: "access_bypass",
    id: "spell:mechanism_manipulation",
    label: "mechanism_manipulation",
    translationStatus: "provisional"
  },
  memory_manipulation: {
    adjacentTags: [
      "truth_reveal",
      "charm_influence"
    ],
    appliesWhen: [
      "The spell is naturally retrieved for altering what a target remembers, forgets, or believes it experienced.",
      "Memory editing is more central than charm, emotion, or truth exposure."
    ],
    assignmentMode: "hybrid",
    axis: "influence",
    category: "spell",
    conceptId: "memory_manipulation",
    currentTag: "memory_manipulation",
    description: "Edits, suppresses, restores, or rewrites memories, recollection, and remembered events.",
    doesNotApplyWhen: [
      "The spell only reveals truth or emotions without changing stored recollection.",
      "The spell primarily imposes obedience or domination in the present moment."
    ],
    family: "influence",
    id: "spell:memory_manipulation",
    label: "memory_manipulation",
    translationStatus: "provisional"
  },
  mental_impairment: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "spell",
    conceptId: "mental_impairment",
    currentTag: "mental_impairment",
    description: "Impairs thought, composure, or agency through fear, confusion, or similarly hostile mental effects.",
    family: "impact",
    id: "spell:mental_impairment",
    label: "mental_impairment",
    translationStatus: "provisional"
  },
  message_delivery: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "spell",
    conceptId: "message_delivery",
    currentTag: "message_delivery",
    description: "Sends, stores, or relays actual content across time or distance.",
    family: "communication",
    id: "spell:message_delivery",
    label: "message_delivery",
    translationStatus: "provisional"
  },
  mobility: {
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "mobility",
    currentTag: "mobility",
    description: "Helps move faster, gain movement modes, or traverse terrain more effectively.",
    family: "expedition",
    id: "spell:mobility",
    label: "mobility",
    translationStatus: "provisional"
  },
  mobility_denial: {
    adjacentTags: [
      "forced_movement",
      "restraint_capture"
    ],
    assignmentMode: "hybrid",
    axis: "battlefield",
    category: "spell",
    conceptId: "mobility_denial",
    currentTag: "mobility_denial",
    description: "Pins, slows, grounds, or otherwise prevents normal repositioning without necessarily functioning as a full restraint effect.",
    family: "control",
    id: "spell:mobility_denial",
    label: "mobility_denial",
    translationStatus: "provisional"
  },
  navigation: {
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "navigation",
    currentTag: "navigation",
    description: "Helps orient, guide a route, or identify a destination's direction.",
    family: "expedition",
    id: "spell:navigation",
    label: "navigation",
    translationStatus: "provisional"
  },
  offensive_summons: {
    adjacentTags: [
      "creature_summoning",
      "screening_summons"
    ],
    assignmentMode: "hybrid",
    axis: "summoning",
    category: "spell",
    conceptId: "offensive_summons",
    currentTag: "offensive_summons",
    description: "Creates summons primarily retrieved for direct damage, flanking pressure, or aggressive battlefield threat.",
    family: "summoning",
    id: "spell:offensive_summons",
    label: "offensive_summons",
    translationStatus: "provisional"
  },
  omen_guidance: {
    adjacentTags: [
      "lore_consultation",
      "wayfinding"
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "omen_guidance",
    currentTag: "omen_guidance",
    description: "Asks for omens, directional guidance, or advisory insight about the best course of action, likely outcome, or strategic choice.",
    family: "consultation",
    id: "spell:omen_guidance",
    label: "omen_guidance",
    translationStatus: "provisional"
  },
  persistent_damage: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "spell",
    conceptId: "persistent_damage",
    currentTag: "persistent_damage",
    description: "Directly inflicts persistent damage or grants attacks that reliably impose persistent damage.",
    family: "impact",
    id: "spell:persistent_damage",
    label: "persistent_damage",
    translationStatus: "provisional"
  },
  planar_travel: {
    adjacentTags: [
      "long_range_teleport",
      "field_shelter"
    ],
    appliesWhen: [
      "Crossing into another plane, demiplane, or extraplanar route is central to the spell's retrieval value.",
      "The spell is naturally retrieved for cosmological travel rather than mundane relocation."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "planar_travel",
    currentTag: "planar_travel",
    description: "Moves creatures between planes, through planar routes, or into extraplanar destinations.",
    doesNotApplyWhen: [
      "The spell only teleports within the same plane or functions as a normal long-distance travel tool.",
      "Extradimensional storage or shelter is present without real plane-crossing travel."
    ],
    family: "teleportation",
    id: "spell:planar_travel",
    label: "planar_travel",
    translationStatus: "provisional"
  },
  problem_diagnosis: {
    adjacentTags: [
      "curse_revelation",
      "magic_detection"
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "problem_diagnosis",
    currentTag: "problem_diagnosis",
    description: "Helps determine what hidden magical, spiritual, cursed, or otherwise obscure problem is actually affecting a target, site, or situation.",
    family: "consultation",
    id: "spell:problem_diagnosis",
    label: "problem_diagnosis",
    translationStatus: "provisional"
  },
  protective_ward: {
    adjacentTags: [
      "alarm",
      "resistance_support"
    ],
    appliesWhen: [
      "The spell is naturally retrieved as a defensive ward, sanctuary, or protective boundary rather than only a resistance buff.",
      "Its value comes from shielding a creature, object, or space against incoming harm or intrusion."
    ],
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "protective_ward",
    currentTag: "protective_ward",
    description: "Places a ward, sanctuary, shield, or protective boundary.",
    doesNotApplyWhen: [
      "The spell only grants resistance, temporary Hit Points, or healing without a real warding or boundary element.",
      "The spell is mainly an alarm, anti-scrying, or mobility tool rather than direct protection."
    ],
    family: "support",
    id: "spell:protective_ward",
    label: "protective_ward",
    translationStatus: "provisional"
  },
  quarantine_containment: {
    adjacentTags: [
      "protective_ward",
      "contamination_cleanup"
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "outbreak_containment",
    currentTag: "quarantine_containment",
    description: "Helps isolate victims, secure a dangerous area, or impose protective boundaries that stop spread while the problem is being solved.",
    family: "resolution",
    id: "spell:quarantine_containment",
    label: "outbreak_containment",
    translationStatus: "provisional"
  },
  quickened_support: {
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "quickened_support",
    currentTag: "quickened_support",
    description: "Grants extra actions, quickened condition benefits, or similar action-economy acceleration.",
    family: "support",
    id: "spell:quickened_support",
    label: "quickened_support",
    translationStatus: "provisional"
  },
  reconnaissance: {
    adjacentTags: [
      "scouting",
      "tracking",
      "scouting_summons"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "spell",
    compositeOfAnyTags: [
      "scouting",
      "tracking",
      "scouting_summons"
    ],
    conceptId: "reconnaissance",
    currentTag: "reconnaissance",
    description: "Broad scouting umbrella for spells that gather remote information, extend senses, or track a target from afar.",
    family: "reconnaissance",
    id: "spell:reconnaissance",
    label: "reconnaissance",
    translationStatus: "provisional"
  },
  resistance_support: {
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "resistance_support",
    currentTag: "resistance_support",
    description: "Grants resistance or immunity against energy, damage, or hazards.",
    family: "support",
    id: "spell:resistance_support",
    label: "resistance_support",
    translationStatus: "provisional"
  },
  resolution: {
    adjacentTags: [
      "curse_removal",
      "exorcism",
      "ritual_appeasement",
      "source_cleanup"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "spell",
    compositeOfAnyTags: [
      "curse_removal",
      "exorcism",
      "sanctification",
      "ritual_appeasement",
      "quarantine_containment",
      "contamination_cleanup",
      "source_revelation",
      "source_cleanup"
    ],
    conceptId: "problem_resolution",
    currentTag: "resolution",
    description: "Broad resolution umbrella for spells that break curses, expel hostile presences, contain spread, purify contamination, or solve a supernatural problem at its source.",
    family: "resolution",
    id: "spell:resolution",
    label: "problem_resolution",
    translationStatus: "provisional"
  },
  restraint_capture: {
    assignmentMode: "deterministic",
    axis: "battlefield",
    category: "spell",
    conceptId: "restraint_capture",
    currentTag: "restraint_capture",
    description: "Restrains, immobilizes, entangles, or traps a target in place.",
    family: "control",
    id: "spell:restraint_capture",
    label: "restraint_capture",
    translationStatus: "provisional"
  },
  revelation: {
    adjacentTags: [
      "magic_detection",
      "truth_reveal",
      "hazard_revelation"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "spell",
    compositeOfAnyTags: [
      "magic_detection",
      "invisibility_reveal",
      "truth_reveal",
      "curse_revelation",
      "hazard_revelation"
    ],
    conceptId: "problem_discovery",
    currentTag: "revelation",
    description: "Broad reveal umbrella for spells that detect magic, uncover deceptions, expose invisible threats, or identify hidden supernatural problems.",
    family: "revelation",
    id: "spell:revelation",
    label: "problem_discovery",
    translationStatus: "provisional"
  },
  ritual_appeasement: {
    adjacentTags: [
      "sanctification",
      "exorcism"
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "ritual_appeasement",
    currentTag: "ritual_appeasement",
    description: "Ends a supernatural problem through offerings, restitution, funerary respect, ritual observance, or otherwise satisfying a spiritual demand rather than expelling the presence outright.",
    family: "resolution",
    id: "spell:ritual_appeasement",
    label: "ritual_appeasement",
    translationStatus: "provisional"
  },
  sanctification: {
    adjacentTags: [
      "ritual_appeasement",
      "exorcism",
      "protective_ward"
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "sacred_taint_sanctification",
    currentTag: "sanctification",
    description: "Consecrates, hallowes, purifies, or spiritually cleanses a creature, object, or site to solve a malign supernatural problem.",
    family: "resolution",
    id: "spell:sanctification",
    label: "sanctification",
    translationStatus: "provisional"
  },
  scouting: {
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "scouting",
    currentTag: "scouting",
    description: "Helps observe at a distance, extend senses, or locate a target.",
    family: "reconnaissance",
    id: "spell:scouting",
    label: "scouting",
    translationStatus: "provisional"
  },
  scouting_summons: {
    adjacentTags: [
      "summoned_servitor",
      "creature_summoning"
    ],
    assignmentMode: "hybrid",
    axis: "summoning",
    category: "spell",
    conceptId: "scouting_summons",
    currentTag: "scouting_summons",
    description: "Creates a summon primarily valued for reconnaissance, watch duty, sensing, or forward information gathering.",
    family: "summoning",
    id: "spell:scouting_summons",
    label: "scouting_summons",
    translationStatus: "provisional"
  },
  screening_summons: {
    adjacentTags: [
      "creature_summoning",
      "temporary_hp_support"
    ],
    assignmentMode: "hybrid",
    axis: "summoning",
    category: "spell",
    conceptId: "screening_summons",
    currentTag: "screening_summons",
    description: "Creates bodies mainly used to block space, absorb hits, or protect allies rather than maximize damage.",
    family: "summoning",
    id: "spell:screening_summons",
    label: "screening_summons",
    translationStatus: "provisional"
  },
  scrying_protection: {
    adjacentTags: [
      "alarm",
      "countermagic"
    ],
    appliesWhen: [
      "The spell is naturally retrieved to keep plans, sanctums, identities, or conversations hidden from magical spying.",
      "Its core value is denying observation or divination rather than only raising an intrusion alarm."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "scrying_protection",
    currentTag: "scrying_protection",
    description: "Blocks magical observation, remote viewing, divinatory surveillance, or other information leakage from a protected target or space.",
    doesNotApplyWhen: [
      "The spell only improves mundane concealment or silence without real anti-divination protection.",
      "The spell counters magic broadly but is not specifically about surveillance or remote observation."
    ],
    family: "security",
    id: "spell:scrying_protection",
    label: "scrying_protection",
    translationStatus: "provisional"
  },
  security: {
    adjacentTags: [
      "alarm",
      "scrying_protection",
      "protective_ward"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "spell",
    compositeOfAnyTags: [
      "alarm",
      "scrying_protection",
      "protective_ward",
      "countermagic"
    ],
    conceptId: "security",
    currentTag: "security",
    description: "Broad security umbrella for spells that warn about intrusion, protect private spaces, or harden a target against magical observation and interference.",
    family: "security",
    id: "spell:security",
    label: "security",
    translationStatus: "provisional"
  },
  senses_support: {
    adjacentTags: [
      "scouting",
      "invisibility_reveal"
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "senses_support",
    currentTag: "senses_support",
    description: "Enhances vision or other senses through darkvision, see invisible, sharpened perception, scent, or similar perceptual upgrades.",
    family: "sensory_support",
    id: "spell:senses_support",
    label: "senses_support",
    translationStatus: "provisional"
  },
  sensory_impairment: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "spell",
    conceptId: "sensory_impairment",
    currentTag: "sensory_impairment",
    description: "Blinds, deafens, or otherwise directly suppresses a creature's senses.",
    family: "impact",
    id: "spell:sensory_impairment",
    label: "sensory_impairment",
    translationStatus: "provisional"
  },
  short_range_teleport: {
    adjacentTags: [
      "extraction_teleport",
      "long_range_teleport"
    ],
    appliesWhen: [
      "The spell repositions a creature within the current encounter or scene rather than serving as expedition travel.",
      "The tactical blink or reposition is itself a major reason to retrieve the spell."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "short_range_teleport",
    currentTag: "short_range_teleport",
    description: "Teleports a creature across a short tactical distance, usually within the same scene or encounter area.",
    doesNotApplyWhen: [
      "The spell is mainly about escaping custody, extracting allies, or long-distance transport.",
      "The spell primarily crosses planes or major overland distances."
    ],
    family: "teleportation",
    id: "spell:short_range_teleport",
    label: "short_range_teleport",
    translationStatus: "provisional"
  },
  signaling: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "spell",
    conceptId: "signaling",
    currentTag: "signaling",
    description: "Helps draw attention, mark a location, or coordinate allies.",
    family: "communication",
    id: "spell:signaling",
    label: "signaling",
    translationStatus: "provisional"
  },
  silencing: {
    assignmentMode: "hybrid",
    axis: "battlefield",
    category: "spell",
    conceptId: "silencing",
    currentTag: "silencing",
    description: "Suppresses speech, sound production, verbal casting, or other voice-dependent action.",
    family: "control",
    id: "spell:silencing",
    label: "silencing",
    translationStatus: "provisional"
  },
  single_target_removal: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "spell",
    conceptId: "single_target_removal",
    currentTag: "single_target_removal",
    description: "Drops, banishes, petrifies, or otherwise decisively removes one important creature from the fight.",
    family: "impact",
    id: "spell:single_target_removal",
    label: "single_target_removal",
    translationStatus: "provisional"
  },
  sleep_magic: {
    assignmentMode: "deterministic",
    axis: "influence",
    category: "spell",
    conceptId: "sleep_magic",
    currentTag: "sleep_magic",
    description: "Puts creatures to sleep, into magical slumber, or into a similarly enforced dormant state.",
    family: "influence",
    id: "spell:sleep_magic",
    label: "sleep_magic",
    translationStatus: "provisional"
  },
  social_infiltration: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "spell",
    conceptId: "social_infiltration",
    currentTag: "social_infiltration",
    description: "Helps blend into a group or pass under social scrutiny.",
    family: "infiltration",
    id: "spell:social_infiltration",
    label: "social_infiltration",
    translationStatus: "provisional"
  },
  source_cleanup: {
    adjacentTags: [
      "source_revelation",
      "contamination_cleanup"
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "source_cleanup",
    currentTag: "source_cleanup",
    description: "Neutralizes, destroys, seals, or cleans up the cursed object, infected origin, corrupted site, or anchored source driving the problem.",
    family: "resolution",
    id: "spell:source_cleanup",
    label: "source_cleanup",
    translationStatus: "provisional"
  },
  source_revelation: {
    adjacentTags: [
      "problem_diagnosis",
      "source_cleanup"
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "source_discovery",
    currentTag: "source_revelation",
    description: "Reveals the hidden source, curse anchor, carrier, infected origin, or spreading point of a supernatural or outbreak problem.",
    family: "resolution",
    id: "spell:source_revelation",
    label: "source_discovery",
    translationStatus: "provisional"
  },
  stealth_support: {
    adjacentTags: [
      "concealment",
      "silencing"
    ],
    appliesWhen: [
      "The spell is naturally retrieved to help a creature move quietly, avoid notice, pass unseen, or keep a covert approach from drawing attention.",
      "The retrieval hook is quiet entry or low-profile movement rather than only broad battlefield obscurity."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "stealth_support",
    currentTag: "stealth_support",
    description: "Helps move quietly, avoid notice, suppress noisy presence, or otherwise support covert entry and low-profile movement.",
    doesNotApplyWhen: [
      "The spell only creates a combat concealment effect or visual obstruction without really supporting a covert approach.",
      "The spell changes appearance or social presentation without materially helping the target move unnoticed."
    ],
    family: "infiltration",
    id: "spell:stealth_support",
    label: "stealth_support",
    translationStatus: "provisional"
  },
  summoned_servitor: {
    adjacentTags: [
      "creature_summoning",
      "scouting"
    ],
    appliesWhen: [
      "The spell is naturally retrieved for utility help, labor, scouting, transport, or task performance rather than frontline combat stats.",
      "The conjured ally behaves more like a helper or specialist tool than a main battle summon."
    ],
    assignmentMode: "hybrid",
    axis: "summoning",
    category: "spell",
    conceptId: "summoned_servitor",
    currentTag: "summoned_servitor",
    description: "Creates a helper, laborer, scout, mount, or similarly task-focused magical servitor rather than a pure combat summon.",
    doesNotApplyWhen: [
      "The spell's main value is summoning a combat creature to attack, flank, or absorb hits.",
      "The spell only creates an object, barrier, or terrain effect without a real servant-like entity."
    ],
    family: "summoning",
    id: "spell:summoned_servitor",
    label: "summoned_servitor",
    translationStatus: "provisional"
  },
  sustenance: {
    assignmentMode: "deterministic",
    axis: "utility",
    category: "spell",
    conceptId: "sustenance",
    currentTag: "sustenance",
    description: "Provides food, water, rations, or practical nourishment for travel and survival.",
    family: "expedition",
    id: "spell:sustenance",
    label: "sustenance",
    translationStatus: "provisional"
  },
  telepathic_communication: {
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "telepathic_communication",
    currentTag: "telepathic_communication",
    description: "Creates direct mind-to-mind communication, silent tactical coordination, or psychic speech between creatures.",
    family: "communication",
    id: "spell:telepathic_communication",
    label: "telepathic_communication",
    translationStatus: "provisional"
  },
  temporary_hp_support: {
    assignmentMode: "hybrid",
    axis: "support",
    category: "spell",
    conceptId: "temporary_hp_support",
    currentTag: "temporary_hp_support",
    description: "Grants temporary Hit Points or similar buffer protection instead of restoring lost Hit Points.",
    family: "support",
    id: "spell:temporary_hp_support",
    label: "temporary_hp_support",
    translationStatus: "provisional"
  },
  tracking: {
    adjacentTags: [
      "scouting",
      "navigation"
    ],
    appliesWhen: [
      "The spell is naturally retrieved to find a named target, trace a quarry, or point the caster toward a specific creature, object, or place.",
      "Target location matters more than broad sensory surveillance or general route guidance."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "tracking",
    currentTag: "tracking",
    description: "Locates a specific creature, object, or destination, or follows a supernatural trail toward it.",
    doesNotApplyWhen: [
      "The spell mainly reveals an area, extends senses, or scouts without locking onto a specific target.",
      "The spell only helps orient a journey or choose a route once the destination is already known."
    ],
    family: "reconnaissance",
    id: "spell:tracking",
    label: "tracking",
    translationStatus: "provisional"
  },
  transformation: {
    adjacentTags: [
      "battle_form",
      "animal_form",
      "elemental_form"
    ],
    assignmentMode: "composite",
    axis: "transformation",
    category: "spell",
    compositeOfAnyTags: [
      "battle_form",
      "animal_form",
      "elemental_form"
    ],
    conceptId: "transformation",
    currentTag: "transformation",
    description: "Spells that alter a creature's body, form, or battle shape.",
    family: "transformation",
    id: "spell:transformation",
    label: "transformation",
    translationStatus: "provisional"
  },
  translation_support: {
    adjacentTags: [
      "telepathic_communication",
      "message_delivery"
    ],
    appliesWhen: [
      "The spell is naturally retrieved to understand, translate, or make oneself understood across otherwise incompatible languages or scripts.",
      "Language access matters more than merely sending a message or speaking silently."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "translation_support",
    currentTag: "translation_support",
    description: "Bridges spoken or written language barriers through translation, comprehension, deciphering, or magically shared understanding.",
    doesNotApplyWhen: [
      "The spell only transmits content farther or more privately without solving a language barrier.",
      "The spell reveals truth, thoughts, or memories without actually translating speech or writing."
    ],
    family: "communication",
    id: "spell:translation_support",
    label: "translation_support",
    translationStatus: "provisional"
  },
  trap_bypass: {
    adjacentTags: [
      "lock_bypass",
      "mechanism_manipulation"
    ],
    appliesWhen: [
      "The spell is naturally retrieved to disable, neutralize, or get past a trap or trapped access point without simply enduring the hazard.",
      "Trap-solving matters more than generic revelation, scouting, or damage prevention."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "trap_bypass",
    currentTag: "trap_bypass",
    description: "Disarms, suppresses, safely triggers, or helps bypass a trap, warded threshold, or similar trapped access problem.",
    doesNotApplyWhen: [
      "The spell only reveals that a trap exists without helping bypass or disable it.",
      "The spell mainly counters open combat hazards or battlefield effects rather than access-facing traps."
    ],
    family: "access_bypass",
    id: "spell:trap_bypass",
    label: "trap_bypass",
    translationStatus: "provisional"
  },
  truth_reveal: {
    adjacentTags: [
      "magic_detection",
      "memory_manipulation"
    ],
    appliesWhen: [
      "The spell's retrieval value comes from exposing deception, forcing truthful answers, or stripping away false presentation.",
      "A user would plausibly look for it when they need an answer spell rather than a sensor spell."
    ],
    assignmentMode: "hybrid",
    axis: "utility",
    category: "spell",
    conceptId: "truth_discovery",
    currentTag: "truth_reveal",
    description: "Forces honesty, exposes lies, or reveals disguised, false, or hidden truths.",
    doesNotApplyWhen: [
      "The spell only detects magic, invisibility, or general auras without interrogating truth or deception.",
      "The spell mainly alters memory or emotions rather than revealing facts."
    ],
    family: "revelation",
    id: "spell:truth_reveal",
    label: "truth_discovery",
    translationStatus: "provisional"
  },
  undead_summoning: {
    assignmentMode: "hybrid",
    axis: "summoning",
    category: "spell",
    conceptId: "undead_summoning",
    currentTag: "undead_summoning",
    description: "Summons, calls, or manifests undead entities, spirits of the dead, or corpse-driven servitors.",
    family: "summoning",
    id: "spell:undead_summoning",
    label: "undead_summoning",
    translationStatus: "provisional"
  },
  wayfinding: {
    adjacentTags: [
      "navigation",
      "tracking",
      "long_range_teleport"
    ],
    assignmentMode: "composite",
    axis: "utility",
    category: "spell",
    compositeOfAnyTags: [
      "navigation",
      "tracking",
      "long_range_teleport",
      "planar_travel"
    ],
    conceptId: "wayfinding",
    currentTag: "wayfinding",
    description: "Broad route-and-destination umbrella for spells that orient travel, locate a target destination, or bypass distance through strategic movement magic.",
    family: "wayfinding",
    id: "spell:wayfinding",
    label: "wayfinding",
    translationStatus: "provisional"
  }
};
