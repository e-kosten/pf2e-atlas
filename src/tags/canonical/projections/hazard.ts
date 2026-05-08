import type { DerivedTagCategoryProjection } from "../../../domain/derived-tag-types.js";

export const HAZARD_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG: Record<string, DerivedTagCategoryProjection> = 
{
  acid_hazard: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "acid_application",
    currentTag: "acid_hazard",
    description: "Hazard centered on acid, corrosive spray, caustic runoff, or similar corrosive exposure.",
    family: "environmental_danger",
    id: "hazard:acid_hazard",
    label: "acid_application",
    translationStatus: "provisional"
  },
  alarm: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "hazard",
    conceptId: "alarm",
    currentTag: "alarm",
    description: "Alerts guardians, onlookers, or nearby creatures to an intrusion.",
    family: "function",
    id: "hazard:alarm",
    label: "alarm",
    translationStatus: "provisional"
  },
  ambush_burst: {
    adjacentTags: [
      "zone_denial",
      "attrition_pressure"
    ],
    appliesWhen: [
      "The hazard is naturally retrieved for a sudden opener, trap-spring punish, or first-contact burst that catches intruders before a longer fight develops.",
      "The surprise spike matters more than sustained zone control or prolonged attrition."
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "hazard",
    conceptId: "ambush_burst",
    currentTag: "ambush_burst",
    description: "Hazard designed to open with a sudden high-damage strike or surprise punish on first contact.",
    doesNotApplyWhen: [
      "The hazard mainly holds territory over time, guards a place persistently, or taxes resources across repeated rounds.",
      "The stronger fit is zone_denial or attrition_pressure rather than a front-loaded strike."
    ],
    family: "function",
    id: "hazard:ambush_burst",
    label: "ambush_burst",
    translationStatus: "provisional"
  },
  appeasement_countermeasure: {
    adjacentTags: [
      "exorcism_countermeasure",
      "judgment_haunt"
    ],
    appliesWhen: [
      "The hazard meaningfully invites negotiation-by-ritual, restitution, reverence, or satisfying an unmet dead or sacred demand.",
      "The nonviolent spiritual answer is more central than simply disabling mechanics or dispelling magic."
    ],
    assignmentMode: "hybrid",
    axis: "resolution",
    category: "hazard",
    conceptId: "ritual_appeasement",
    currentTag: "appeasement_countermeasure",
    description: "Hazard best resolved through offerings, ritual respect, social appeasement, or meeting a spiritual demand.",
    doesNotApplyWhen: [
      "The hazard only needs a standard disable check, counteract, or forceful destruction.",
      "The hazard is haunted but has no appeasement-style resolution path."
    ],
    family: "countermeasure_profile",
    id: "hazard:appeasement_countermeasure",
    label: "ritual_appeasement",
    translationStatus: "provisional"
  },
  aquatic_hazard: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "hazard",
    conceptId: "aquatic_hazard",
    currentTag: "aquatic_hazard",
    description: "Hazard strongly associated with flooded chambers, rivers, docks, ships, reefs, or underwater spaces.",
    family: "setting",
    id: "hazard:aquatic_hazard",
    label: "aquatic_hazard",
    translationStatus: "mapped"
  },
  area_denial: {
    adjacentTags: [
      "zone_denial",
      "sentinel_guardian"
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "hazard",
    conceptId: "area_denial",
    currentTag: "area_denial",
    description: "Legacy synonym for zone_denial preserved for compatibility while downstream hazard planning surfaces migrate to the simpler area-control vocabulary.",
    family: "function",
    id: "hazard:area_denial",
    label: "area_denial",
    translationStatus: "provisional"
  },
  attrition_pressure: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "hazard",
    conceptId: "attrition_pressure",
    currentTag: "attrition_pressure",
    description: "Hazard whose primary role is to wear the party down over time rather than deliver one decisive spike.",
    family: "function",
    id: "hazard:attrition_pressure",
    label: "attrition_pressure",
    translationStatus: "provisional"
  },
  barrier_lockdown: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "hazard",
    conceptId: "barrier_lockdown",
    currentTag: "barrier_lockdown",
    description: "Hazard that seals, closes, or blocks passage to trap or delay intruders.",
    family: "function",
    id: "hazard:barrier_lockdown",
    label: "barrier_lockdown",
    translationStatus: "provisional"
  },
  battlefield_disruption: {
    assignmentMode: "hybrid",
    axis: "haunt",
    category: "hazard",
    conceptId: "battlefield_disruption",
    currentTag: "battlefield_disruption",
    description: "Haunt that reshapes the scene with barriers, violent manifestations, or other encounter-disrupting effects.",
    family: "haunt_manifestation",
    id: "hazard:battlefield_disruption",
    label: "battlefield_disruption",
    translationStatus: "mapped"
  },
  battlefield_hazard: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "hazard",
    conceptId: "battlefield_hazard",
    currentTag: "battlefield_hazard",
    description: "Hazard strongly associated with siegeworks, trenches, killing grounds, war engines, or other battlefield scenes.",
    family: "setting",
    id: "hazard:battlefield_hazard",
    label: "battlefield_hazard",
    translationStatus: "mapped"
  },
  blight_hazard: {
    adjacentTags: [
      "contamination_hazard",
      "overgrowth_hazard"
    ],
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "blight_application",
    currentTag: "blight_hazard",
    description: "Hazard centered on ecological ruin, land-sickening corruption, withering growth, or terrain spoiled by supernatural blight.",
    family: "environmental_danger",
    id: "hazard:blight_hazard",
    label: "blight_application",
    translationStatus: "provisional"
  },
  bridge_passage_hazard: {
    adjacentTags: [
      "threshold_lockdown",
      "forced_movement"
    ],
    appliesWhen: [
      "The hazard is naturally retrieved for narrow crossings, gates, stairwells, or other spaces where intruders must pass through a constrained route.",
      "Forced-passage geometry matters more than a broader dungeon, urban, or wilderness setting identity."
    ],
    assignmentMode: "hybrid",
    axis: "setting",
    category: "hazard",
    conceptId: "bridge_passage_hazard",
    currentTag: "bridge_passage_hazard",
    description: "Hazard strongly associated with bridges, chokepoints, gates, stairwells, or forced passage bottlenecks.",
    doesNotApplyWhen: [
      "The hazard only happens to sit near a doorway or bridge once without the chokepoint being central to its design.",
      "The stronger fit is threshold_lockdown or forced_movement because the route bottleneck is not the main retrieval hook."
    ],
    family: "setting",
    id: "hazard:bridge_passage_hazard",
    label: "bridge_passage_hazard",
    translationStatus: "mapped"
  },
  cold_hazard: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "cold_application",
    currentTag: "cold_hazard",
    description: "Hazard centered on ice, frost, freezing, blizzards, or other cold exposure.",
    family: "environmental_danger",
    id: "hazard:cold_hazard",
    label: "cold_application",
    translationStatus: "provisional"
  },
  collapse_hazard: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "hazard",
    conceptId: "collapse_application",
    currentTag: "collapse_hazard",
    description: "Hazard built around collapsing structures, cave-ins, rockfalls, or crumbling ground.",
    family: "forced_position",
    id: "hazard:collapse_hazard",
    label: "collapse_application",
    translationStatus: "provisional"
  },
  contamination_cleanup_countermeasure: {
    adjacentTags: [
      "quarantine_containment_countermeasure",
      "source_cleanup_countermeasure"
    ],
    appliesWhen: [
      "The hazard is naturally retrieved because cleansing tainted ground, polluted air, cursed runoff, spores, or lingering residue is a core answer path.",
      "The cleanup process matters more than only suppressing the effect temporarily or bypassing the area."
    ],
    assignmentMode: "hybrid",
    axis: "resolution",
    category: "hazard",
    conceptId: "contamination_cleanup",
    currentTag: "contamination_cleanup_countermeasure",
    description: "Hazard best resolved through decontamination, purification, cleansing residue, or scrubbing the hazardous space back to safety.",
    doesNotApplyWhen: [
      "The hazard only has an immediate trigger or burst with no meaningful lingering contamination to clean up.",
      "The stronger fit is exorcism_countermeasure or dispel_countermeasure because the answer is purging a presence or ending an effect rather than cleaning a tainted site."
    ],
    family: "countermeasure_profile",
    id: "hazard:contamination_cleanup_countermeasure",
    label: "contamination_cleanup",
    translationStatus: "provisional"
  },
  contamination_hazard: {
    adjacentTags: [
      "poison_hazard",
      "respiratory_hazard"
    ],
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "contamination_application",
    currentTag: "contamination_hazard",
    description: "Hazard centered on tainted residue, corruptive seepage, drifting spores, cursed runoff, or other lingering contamination of a space.",
    family: "environmental_danger",
    id: "hazard:contamination_hazard",
    label: "contamination_application",
    translationStatus: "provisional"
  },
  control_interface: {
    assignmentMode: "hybrid",
    axis: "mechanism",
    category: "hazard",
    conceptId: "control_interface",
    currentTag: "control_interface",
    description: "Hazard operated through a button, lever, console, panel, switch, or similar control surface.",
    family: "mechanism",
    id: "hazard:control_interface",
    label: "control_interface",
    translationStatus: "mapped"
  },
  cursefield_hazard: {
    adjacentTags: [
      "contamination_hazard",
      "judgment_haunt"
    ],
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "cursefield_application",
    currentTag: "cursefield_hazard",
    description: "Hazard centered on cursed ground, spiritually poisoned space, or a zone whose danger comes from active supernatural contamination rather than one mechanism.",
    family: "environmental_danger",
    id: "hazard:cursefield_hazard",
    label: "cursefield_application",
    translationStatus: "provisional"
  },
  dispel_countermeasure: {
    adjacentTags: [
      "physical_disarm",
      "ward_trigger"
    ],
    appliesWhen: [
      "A user would plausibly retrieve the hazard because anti-magic answers are central to resolving it.",
      "Magical suppression matters more than physical disarm or spiritual appeasement."
    ],
    assignmentMode: "hybrid",
    axis: "resolution",
    category: "hazard",
    conceptId: "active_magic_counteraction",
    currentTag: "dispel_countermeasure",
    description: "Hazard meaningfully invites counteract, dispel, or magical suppression as a core answer path.",
    doesNotApplyWhen: [
      "The hazard is magical but best solved through rituals, offerings, or physical tampering instead.",
      "Counteracting is only a minor optional answer path."
    ],
    family: "countermeasure_profile",
    id: "hazard:dispel_countermeasure",
    label: "countermagic",
    translationStatus: "provisional"
  },
  displacement_hazard: {
    adjacentTags: [
      "pitfall",
      "collapse_hazard",
      "forced_separation"
    ],
    assignmentMode: "composite",
    axis: "effect",
    category: "hazard",
    compositeOfAnyTags: [
      "pitfall",
      "collapse_hazard",
      "forced_movement",
      "forced_separation"
    ],
    conceptId: "displacement_application",
    currentTag: "displacement_hazard",
    description: "Broad displacement umbrella for hazards that drop, collapse, shove, or split creatures apart through positional disruption.",
    family: "forced_position",
    id: "hazard:displacement_hazard",
    label: "displacement_application",
    translationStatus: "provisional"
  },
  dungeon_hazard: {
    adjacentTags: [
      "tomb_hazard",
      "temple_hazard"
    ],
    appliesWhen: [
      "The hazard is naturally retrieved as a classic corridor, chamber, or trap-complex defense in a dungeon environment.",
      "Its encounter identity is more about underground built-space adventuring than a narrower tomb or temple context."
    ],
    assignmentMode: "hybrid",
    axis: "setting",
    category: "hazard",
    conceptId: "dungeon_hazard",
    currentTag: "dungeon_hazard",
    description: "Hazard strongly associated with dungeon corridors, chambers, trapped passages, or underground complexes.",
    doesNotApplyWhen: [
      "The hazard is more specifically tomb-, temple-, bridge-, or aquatic-coded than generic dungeon-coded.",
      "The hazard merely happens to appear indoors once without dungeon-like scene identity."
    ],
    family: "setting",
    id: "hazard:dungeon_hazard",
    label: "dungeon_hazard",
    translationStatus: "mapped"
  },
  electric_hazard: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "electric_application",
    currentTag: "electric_hazard",
    description: "Hazard centered on lightning, shock, static discharge, or electrical exposure.",
    family: "environmental_danger",
    id: "hazard:electric_hazard",
    label: "electric_application",
    translationStatus: "provisional"
  },
  endurance_pressure: {
    assignmentMode: "hybrid",
    axis: "problem",
    category: "hazard",
    conceptId: "endurance_pressure",
    currentTag: "endurance_pressure",
    description: "Hazard whose main prep problem is surviving repeated exposure long enough to finish the scene rather than landing one clean solve immediately.",
    family: "problem_shape",
    id: "hazard:endurance_pressure",
    label: "endurance_pressure",
    translationStatus: "mapped"
  },
  environmental_hazard: {
    adjacentTags: [
      "fire_hazard",
      "poison_hazard",
      "contamination_hazard"
    ],
    assignmentMode: "composite",
    axis: "effect",
    category: "hazard",
    compositeOfAnyTags: [
      "acid_hazard",
      "cold_hazard",
      "fire_hazard",
      "electric_hazard",
      "poison_hazard",
      "respiratory_hazard",
      "sound_hazard",
      "water_hazard",
      "contamination_hazard",
      "blight_hazard",
      "overgrowth_hazard",
      "cursefield_hazard"
    ],
    conceptId: "environmental_application",
    currentTag: "environmental_hazard",
    description: "Broad environmental umbrella for elemental, toxic, contaminating, and terrain-corrupting hazards that threaten a space through recurring exposure.",
    family: "environmental_danger",
    id: "hazard:environmental_hazard",
    label: "environmental_application",
    translationStatus: "provisional"
  },
  exorcism_countermeasure: {
    adjacentTags: [
      "appeasement_countermeasure",
      "dispel_countermeasure"
    ],
    appliesWhen: [
      "The hazard is naturally retrieved because cleansing, banishing, or sanctifying the hostile presence is a core answer path.",
      "A spiritual purge matters more than appeasement, anti-magic suppression, or physical mechanism work."
    ],
    assignmentMode: "hybrid",
    axis: "resolution",
    category: "hazard",
    conceptId: "hostile_presence_expulsion",
    currentTag: "exorcism_countermeasure",
    description: "Hazard best resolved through banishment, exorcism, consecration, or another spirit-cleansing answer.",
    doesNotApplyWhen: [
      "The hazard mainly wants offerings, restitution, or ritual respect rather than expulsion.",
      "The hazard is magical or mechanical but not really spirit-cleansed out of existence."
    ],
    family: "countermeasure_profile",
    id: "hazard:exorcism_countermeasure",
    label: "exorcism",
    translationStatus: "provisional"
  },
  false_safe_route: {
    adjacentTags: [
      "navigation_disruption",
      "procedural_bypass"
    ],
    appliesWhen: [
      "The hazard actively misdirects intruders toward a route that looks protective, faster, or safer but is actually the danger.",
      "The retrieval hook is deceptive path choice rather than only illusion damage or generic navigation confusion."
    ],
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "false_safe_route",
    currentTag: "false_safe_route",
    description: "Hazard that tempts intruders toward a seemingly safer path or escape line that is itself the trap.",
    doesNotApplyWhen: [
      "The hazard merely scrambles orientation without presenting a tempting fake safe path.",
      "The hazard is solved through procedure, but misleading route presentation is not central to how it works."
    ],
    family: "perception_control",
    id: "hazard:false_safe_route",
    label: "false_safe_route",
    translationStatus: "provisional"
  },
  fire_hazard: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "fire_application",
    currentTag: "fire_hazard",
    description: "Hazard centered on open fire, flames, burning spread, or explosive ignition.",
    family: "environmental_danger",
    id: "hazard:fire_hazard",
    label: "fire_application",
    translationStatus: "provisional"
  },
  floor_eruption: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "floor_eruption",
    currentTag: "floor_eruption",
    description: "Hazard that attacks upward from the ground, floor, or a concealed underfoot chamber.",
    family: "attack_vector",
    id: "hazard:floor_eruption",
    label: "floor_eruption",
    translationStatus: "provisional"
  },
  forced_movement: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "hazard",
    conceptId: "forced_movement",
    currentTag: "forced_movement",
    description: "Hazard that pushes, pulls, drags, submerges, or otherwise forcibly repositions creatures.",
    family: "forced_position",
    id: "hazard:forced_movement",
    label: "forced_movement",
    translationStatus: "provisional"
  },
  forced_separation: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "hazard",
    conceptId: "forced_separation",
    currentTag: "forced_separation",
    description: "Hazard that splits allies apart through walls, drops, slides, teleports, or other positional disruption.",
    family: "function",
    id: "hazard:forced_separation",
    label: "forced_separation",
    translationStatus: "provisional"
  },
  forced_separation_hazard: {
    adjacentTags: [
      "forced_separation",
      "pursuit_punisher"
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "hazard",
    conceptId: "forced_separation_hazard",
    currentTag: "forced_separation_hazard",
    description: "Legacy synonym for forced_separation preserved for compatibility while downstream surfaces migrate to the simpler split-party vocabulary.",
    family: "function",
    id: "hazard:forced_separation_hazard",
    label: "forced_separation_hazard",
    translationStatus: "provisional"
  },
  guarding_hazard: {
    adjacentTags: [
      "alarm",
      "barrier_lockdown",
      "sentinel_guardian"
    ],
    assignmentMode: "composite",
    axis: "encounter",
    category: "hazard",
    compositeOfAnyTags: [
      "alarm",
      "barrier_lockdown",
      "sentinel_guardian"
    ],
    conceptId: "guarding_hazard",
    currentTag: "guarding_hazard",
    description: "Broad guarding umbrella for hazards that warn, lock down, or stand watch over a threshold, object, or protected space.",
    family: "function",
    id: "hazard:guarding_hazard",
    label: "guarding_hazard",
    translationStatus: "provisional"
  },
  illusion_assault: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "hazard",
    conceptId: "illusion_assault",
    currentTag: "illusion_assault",
    description: "Hazard that attacks through deceptive reflections, phantasms, or other hostile illusion-driven distortions.",
    family: "perception_control",
    id: "hazard:illusion_assault",
    label: "illusion_assault",
    translationStatus: "provisional"
  },
  judgment_haunt: {
    assignmentMode: "hybrid",
    axis: "haunt",
    category: "hazard",
    conceptId: "judgment_haunt",
    currentTag: "judgment_haunt",
    description: "Haunt that condemns trespassers through accusation, punishment, curse-like verdicts, or moral reckoning.",
    family: "haunt_manifestation",
    id: "hazard:judgment_haunt",
    label: "judgment_haunt",
    translationStatus: "mapped"
  },
  layered_resolution: {
    assignmentMode: "hybrid",
    axis: "problem",
    category: "hazard",
    conceptId: "multi_stage_resolution",
    currentTag: "layered_resolution",
    description: "Hazard that asks the party to solve multiple linked pieces rather than one single disable check or obvious answer.",
    family: "problem_shape",
    id: "hazard:layered_resolution",
    label: "multi_stage_resolution",
    translationStatus: "mapped"
  },
  life_drain_hazard: {
    assignmentMode: "hybrid",
    axis: "haunt",
    category: "hazard",
    conceptId: "life_drain_hazard",
    currentTag: "life_drain_hazard",
    description: "Haunt that drains life force, vitality, or souls from victims.",
    family: "haunt_manifestation",
    id: "hazard:life_drain_hazard",
    label: "life_drain_hazard",
    translationStatus: "mapped"
  },
  lure_compulsion: {
    assignmentMode: "hybrid",
    axis: "haunt",
    category: "hazard",
    conceptId: "lure_compulsion",
    currentTag: "lure_compulsion",
    description: "Haunt that beckons, lures, or compels creatures into moving or acting against their judgment.",
    family: "haunt_manifestation",
    id: "hazard:lure_compulsion",
    label: "lure_compulsion",
    translationStatus: "mapped"
  },
  mental_impairment: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "hazard",
    conceptId: "mental_impairment",
    currentTag: "mental_impairment",
    description: "Impairs judgment, emotions, or perception through fear, confusion, or similar effects.",
    family: "impact",
    id: "hazard:mental_impairment",
    label: "mental_impairment",
    translationStatus: "provisional"
  },
  mobility_impairment: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "hazard",
    conceptId: "mobility_impairment",
    currentTag: "mobility_impairment",
    description: "Paralyzes, immobilizes, or otherwise heavily hampers movement.",
    family: "impact",
    id: "hazard:mobility_impairment",
    label: "mobility_impairment",
    translationStatus: "provisional"
  },
  navigation_disruption: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "hazard",
    conceptId: "navigation_disruption",
    currentTag: "navigation_disruption",
    description: "Hazard that confounds routes, loops intruders, or scrambles navigation through distorted perception.",
    family: "perception_control",
    id: "hazard:navigation_disruption",
    label: "navigation_disruption",
    translationStatus: "provisional"
  },
  observation_first: {
    assignmentMode: "hybrid",
    axis: "problem",
    category: "hazard",
    conceptId: "observation_driven",
    currentTag: "observation_first",
    description: "Hazard that rewards careful watching, clue gathering, or reading the environment before a safe approach becomes obvious.",
    family: "problem_shape",
    id: "hazard:observation_first",
    label: "observation_driven",
    translationStatus: "mapped"
  },
  overgrowth_hazard: {
    adjacentTags: [
      "blight_hazard",
      "forced_movement"
    ],
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "overgrowth_application",
    currentTag: "overgrowth_hazard",
    description: "Hazard centered on choking roots, hostile vines, grasping thorns, or other dangerous living overgrowth that turns terrain against intruders.",
    family: "environmental_danger",
    id: "hazard:overgrowth_hazard",
    label: "overgrowth_application",
    translationStatus: "provisional"
  },
  overhead_strike: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "hazard",
    conceptId: "overhead_strike",
    currentTag: "overhead_strike",
    description: "Hazard that drops, crashes, or attacks from the ceiling or another overhead position.",
    family: "attack_vector",
    id: "hazard:overhead_strike",
    label: "overhead_strike",
    translationStatus: "provisional"
  },
  perception_hazard: {
    adjacentTags: [
      "navigation_disruption",
      "illusion_assault",
      "false_safe_route"
    ],
    assignmentMode: "composite",
    axis: "effect",
    category: "hazard",
    compositeOfAnyTags: [
      "navigation_disruption",
      "illusion_assault",
      "false_safe_route"
    ],
    conceptId: "perception_application",
    currentTag: "perception_hazard",
    description: "Broad perception umbrella for hazards that attack through distorted routes, hostile illusion, or deceptive pathing rather than direct force alone.",
    family: "perception_control",
    id: "hazard:perception_hazard",
    label: "perception_application",
    translationStatus: "provisional"
  },
  phantom_assailants: {
    assignmentMode: "hybrid",
    axis: "haunt",
    category: "hazard",
    conceptId: "phantom_assailants",
    currentTag: "phantom_assailants",
    description: "Haunt that manifests ghostly, spectral, or phantom attackers as separate assailants.",
    family: "haunt_manifestation",
    id: "hazard:phantom_assailants",
    label: "phantom_assailants",
    translationStatus: "mapped"
  },
  physical_disarm: {
    adjacentTags: [
      "procedural_bypass",
      "dispel_countermeasure"
    ],
    appliesWhen: [
      "The hazard is naturally retrieved because tools, hands-on disable work, or mechanism access are central to solving it.",
      "Mechanical tampering matters more than safe sequencing, anti-magic, or spiritual negotiation."
    ],
    assignmentMode: "hybrid",
    axis: "resolution",
    category: "hazard",
    conceptId: "physical_disarm",
    currentTag: "physical_disarm",
    description: "Hazard meaningfully invites physical mechanism tampering, disassembly, or trigger-blocking as the core answer path.",
    doesNotApplyWhen: [
      "The hazard is mainly solved by learning the right pattern, dispelling an effect, or meeting a spiritual demand.",
      "Physical interaction exists only as one optional fallback rather than the main resolution mode."
    ],
    family: "countermeasure_profile",
    id: "hazard:physical_disarm",
    label: "physical_disarm",
    translationStatus: "provisional"
  },
  pitfall: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "hazard",
    conceptId: "pitfall",
    currentTag: "pitfall",
    description: "Hazard built around a concealed pit, drop, or similar vertical fall trap.",
    family: "forced_position",
    id: "hazard:pitfall",
    label: "pitfall",
    translationStatus: "provisional"
  },
  planar_breach: {
    adjacentTags: [
      "dispel_countermeasure",
      "procedural_bypass"
    ],
    appliesWhen: [
      "An unstable portal, planar tear, extradimensional rupture, or reality breach is the central mechanism of the hazard.",
      "The hazard is naturally retrieved for cosmological leakage, portal instability, or something dangerous coming through a breach."
    ],
    assignmentMode: "hybrid",
    axis: "mechanism",
    category: "hazard",
    conceptId: "planar_breach",
    currentTag: "planar_breach",
    description: "Hazard centered on a portal, rift, tear, breach, or other unstable opening in reality.",
    doesNotApplyWhen: [
      "The hazard is merely magical or teleportive without a real opening in reality as the core hazard engine.",
      "The stronger fit is dispel_countermeasure or procedural_bypass because the planar flavor is incidental."
    ],
    family: "mechanism",
    id: "hazard:planar_breach",
    label: "planar_breach",
    translationStatus: "mapped"
  },
  poison_hazard: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "poison_application",
    currentTag: "poison_hazard",
    description: "Hazard centered on poison gas, toxic delivery, or other poisonous exposure.",
    family: "environmental_danger",
    id: "hazard:poison_hazard",
    label: "poison_application",
    translationStatus: "provisional"
  },
  possession_haunt: {
    assignmentMode: "hybrid",
    axis: "haunt",
    category: "hazard",
    conceptId: "possession_haunt",
    currentTag: "possession_haunt",
    description: "Haunt that enters, rides, or briefly controls a victim rather than only attacking them externally.",
    family: "haunt_manifestation",
    id: "hazard:possession_haunt",
    label: "possession_haunt",
    translationStatus: "mapped"
  },
  pressure_trigger: {
    assignmentMode: "hybrid",
    axis: "mechanism",
    category: "hazard",
    conceptId: "pressure_trigger",
    currentTag: "pressure_trigger",
    description: "Hazard triggered by stepping on, weighing down, or depressing a pressure surface.",
    family: "mechanism",
    id: "hazard:pressure_trigger",
    label: "pressure_trigger",
    translationStatus: "mapped"
  },
  procedural_bypass: {
    adjacentTags: [
      "physical_disarm",
      "false_safe_route"
    ],
    appliesWhen: [
      "The clean answer is learning and executing the hazard's safe procedure, sequence, or pattern rather than destroying it.",
      "A GM would plausibly retrieve the hazard for puzzle-like bypass, safe-route discovery, or passphrase-style navigation."
    ],
    assignmentMode: "hybrid",
    axis: "resolution",
    category: "hazard",
    conceptId: "procedural_bypass",
    currentTag: "procedural_bypass",
    description: "Hazard best bypassed through the right route, timing, command phrase, ritual sequence, or other correct procedure rather than direct disarm.",
    doesNotApplyWhen: [
      "The main answer is simple mechanism tampering, counteracting magic, or appeasing a spirit.",
      "The hazard only has a minor caution or tactical workaround without a real procedural solution."
    ],
    family: "countermeasure_profile",
    id: "hazard:procedural_bypass",
    label: "procedural_bypass",
    translationStatus: "provisional"
  },
  projectile_emitter: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "hazard",
    conceptId: "projectile_emitter",
    currentTag: "projectile_emitter",
    description: "Hazard that fires bolts, beams, jets, sprays, or similar directed emissions from a fixed emitter.",
    family: "attack_vector",
    id: "hazard:projectile_emitter",
    label: "projectile_emitter",
    translationStatus: "provisional"
  },
  proximity_burst: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "proximity_burst",
    currentTag: "proximity_burst",
    description: "Hazard that erupts in an immediate burst, blast, or detonation when a victim comes near or crosses a point.",
    family: "attack_vector",
    id: "hazard:proximity_burst",
    label: "proximity_burst",
    translationStatus: "provisional"
  },
  pursuit_punisher: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "hazard",
    conceptId: "pursuit_punisher",
    currentTag: "pursuit_punisher",
    description: "Hazard that punishes retreat, pursuit, escape routes, or movement through chase-style spaces.",
    family: "function",
    id: "hazard:pursuit_punisher",
    label: "pursuit_punisher",
    translationStatus: "provisional"
  },
  quarantine_containment_countermeasure: {
    adjacentTags: [
      "contamination_cleanup_countermeasure",
      "source_cleanup_countermeasure"
    ],
    appliesWhen: [
      "The hazard is naturally retrieved because the first meaningful answer is locking down spread, controlling access, or containing dangerous exposure.",
      "Containment procedures matter more than immediately dispelling, disarming, or appeasing the hazard."
    ],
    assignmentMode: "hybrid",
    axis: "resolution",
    category: "hazard",
    conceptId: "outbreak_containment",
    currentTag: "quarantine_containment_countermeasure",
    description: "Hazard best managed by isolating victims, sealing off the site, or imposing containment boundaries that stop spread while the danger is being handled.",
    doesNotApplyWhen: [
      "The hazard is dangerous but does not meaningfully spread, linger, or demand isolation and access control.",
      "The stronger fit is barrier_lockdown or sentinel_guardian because preventing passage is the hazard's function, not the party's resolution plan."
    ],
    family: "countermeasure_profile",
    id: "hazard:quarantine_containment_countermeasure",
    label: "outbreak_containment",
    translationStatus: "provisional"
  },
  replayed_tragedy: {
    adjacentTags: [
      "judgment_haunt",
      "appeasement_countermeasure"
    ],
    appliesWhen: [
      "The haunt is naturally retrieved because it replays a specific past calamity, crime, execution, or emotional flashpoint as its core manifestation.",
      "The narrative repetition of an old event matters more than generic life drain, possession, or battlefield disruption."
    ],
    assignmentMode: "hybrid",
    axis: "haunt",
    category: "hazard",
    conceptId: "replayed_tragedy",
    currentTag: "replayed_tragedy",
    description: "Haunt that re-enacts a murder, betrayal, execution, disaster, or other fixed traumatic event.",
    doesNotApplyWhen: [
      "The haunt is only sad, angry, or spiritually active without reenacting a fixed historical scene.",
      "The stronger fit is judgment_haunt or lure_compulsion because the recurring tragedy itself is not the central hook."
    ],
    family: "haunt_manifestation",
    id: "hazard:replayed_tragedy",
    label: "replayed_tragedy",
    translationStatus: "mapped"
  },
  resource_drain: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "hazard",
    conceptId: "resource_drain",
    currentTag: "resource_drain",
    description: "Hazard that taxes healing, spellcasting, equipment durability, or other party resources over time.",
    family: "function",
    id: "hazard:resource_drain",
    label: "resource_drain",
    translationStatus: "provisional"
  },
  respiratory_hazard: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "respiratory_application",
    currentTag: "respiratory_hazard",
    description: "Hazard centered on smoke, choking vapor, breathlessness, or impaired breathing.",
    family: "environmental_danger",
    id: "hazard:respiratory_hazard",
    label: "respiratory_application",
    translationStatus: "provisional"
  },
  restraint_capture: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "hazard",
    conceptId: "restraint_capture",
    currentTag: "restraint_capture",
    description: "Hazard that binds, restrains, or holds intruders in place.",
    family: "function",
    id: "hazard:restraint_capture",
    label: "restraint_capture",
    translationStatus: "provisional"
  },
  sensory_impairment: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "sensory_impairment",
    currentTag: "sensory_impairment",
    description: "Blinds, deafens, dazzles, or otherwise suppresses a victim's ability to perceive the environment clearly.",
    family: "impact",
    id: "hazard:sensory_impairment",
    label: "sensory_impairment",
    translationStatus: "provisional"
  },
  sentinel_guardian: {
    adjacentTags: [
      "alarm",
      "zone_denial"
    ],
    appliesWhen: [
      "The hazard is naturally retrieved as a guardian layer protecting a place, object, or route from intrusion.",
      "Its value is in persistent watchfulness or defensive coverage, not just burst damage."
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "hazard",
    conceptId: "guard_post",
    currentTag: "sentinel_guardian",
    description: "Hazard whose role is to guard an area, treasure, threshold, or sanctum as a standing defense layer.",
    doesNotApplyWhen: [
      "The hazard mainly creates open-area denial with no strong guard-post identity.",
      "The hazard is mostly an ambush opener or chase-punishment device."
    ],
    family: "function",
    id: "hazard:sentinel_guardian",
    label: "guard_post",
    translationStatus: "provisional"
  },
  sound_hazard: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "sound_application",
    currentTag: "sound_hazard",
    description: "Hazard centered on sonic force, deafening noise, vibration, or resonant disruption.",
    family: "environmental_danger",
    id: "hazard:sound_hazard",
    label: "sound_application",
    translationStatus: "provisional"
  },
  source_cleanup_countermeasure: {
    adjacentTags: [
      "quarantine_containment_countermeasure",
      "contamination_cleanup_countermeasure"
    ],
    appliesWhen: [
      "The hazard is naturally retrieved because the real answer is finding and dealing with the source object, origin point, or contamination engine.",
      "Neutralizing the origin matters more than only enduring the space or treating downstream symptoms."
    ],
    assignmentMode: "hybrid",
    axis: "resolution",
    category: "hazard",
    conceptId: "source_cleanup",
    currentTag: "source_cleanup_countermeasure",
    description: "Hazard best resolved by locating and neutralizing the cursed source, leaking node, corrupted remains, or other origin driving the dangerous field.",
    doesNotApplyWhen: [
      "The hazard has no meaningful source object, leak point, or origin to clean up beyond the hazard itself.",
      "The stronger fit is procedural_bypass or physical_disarm because the answer is executing a sequence or tampering with a mechanism rather than eliminating an origin source."
    ],
    family: "countermeasure_profile",
    id: "hazard:source_cleanup_countermeasure",
    label: "source_cleanup",
    translationStatus: "provisional"
  },
  source_tracing: {
    adjacentTags: [
      "observation_first",
      "source_cleanup_countermeasure"
    ],
    appliesWhen: [
      "The hazard is naturally retrieved because identifying the source object, origin point, or contamination engine is a major part of solving it.",
      "Finding what is powering the danger matters more than merely surviving exposure or executing a known disable procedure."
    ],
    assignmentMode: "hybrid",
    axis: "problem",
    category: "hazard",
    conceptId: "source_discovery",
    currentTag: "source_tracing",
    description: "Hazard whose real puzzle is locating the hidden anchor, leak point, cursed source, contamination engine, or origin node before a clean solution becomes possible.",
    doesNotApplyWhen: [
      "The hazard is fully understandable up front and the real challenge is timing, endurance, or multi-step execution rather than finding an origin.",
      "The stronger fit is source_cleanup_countermeasure because the source is already obvious and the remaining task is neutralizing it."
    ],
    family: "problem_shape",
    id: "hazard:source_tracing",
    label: "source_discovery",
    translationStatus: "mapped"
  },
  spawned_attackers: {
    adjacentTags: [
      "alarm",
      "phantom_assailants"
    ],
    appliesWhen: [
      "The hazard is naturally retrieved because it adds new hostile creatures, constructs, swarms, or manifestations to the scene.",
      "The extra attackers matter as separate encounter pressure rather than only as a damage effect."
    ],
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "hazard",
    conceptId: "spawned_attackers",
    currentTag: "spawned_attackers",
    description: "Hazard that summons, creates, or releases separate attackers into the scene.",
    doesNotApplyWhen: [
      "The hazard only deals direct damage, restrains victims, or creates a temporary illusion without generating distinct assailants.",
      "The stronger fit is phantom_assailants when the threat is specifically a haunt manifestation rather than a broader hazard function."
    ],
    family: "function",
    id: "hazard:spawned_attackers",
    label: "spawned_attackers",
    translationStatus: "provisional"
  },
  temple_hazard: {
    adjacentTags: [
      "dungeon_hazard",
      "appeasement_countermeasure"
    ],
    appliesWhen: [
      "Sacred architecture, ritual sanctums, or faith-site defense is central to the hazard's scene identity.",
      "The hazard is naturally retrieved for shrine, sanctum, or temple protection rather than generic indoor defense."
    ],
    assignmentMode: "hybrid",
    axis: "setting",
    category: "hazard",
    conceptId: "temple_hazard",
    currentTag: "temple_hazard",
    description: "Hazard strongly associated with shrines, sanctums, cathedrals, monasteries, or consecrated sites.",
    doesNotApplyWhen: [
      "The hazard is merely magical or haunted without specific sacred-site framing.",
      "The stronger setting is dungeon_hazard or urban_hazard rather than temple-focused."
    ],
    family: "setting",
    id: "hazard:temple_hazard",
    label: "temple_hazard",
    translationStatus: "mapped"
  },
  threshold_lockdown: {
    assignmentMode: "hybrid",
    axis: "mechanism",
    category: "hazard",
    conceptId: "threshold_lockdown",
    currentTag: "threshold_lockdown",
    description: "Hazard that seals, locks, or bars a threshold, doorway, or gate.",
    family: "mechanism",
    id: "hazard:threshold_lockdown",
    label: "threshold_lockdown",
    translationStatus: "mapped"
  },
  timing_window: {
    assignmentMode: "hybrid",
    axis: "problem",
    category: "hazard",
    conceptId: "timing_window",
    currentTag: "timing_window",
    description: "Hazard that is best handled by acting during the right cycle, opening, lull, or repeating timing pattern.",
    family: "problem_shape",
    id: "hazard:timing_window",
    label: "timing_window",
    translationStatus: "mapped"
  },
  tomb_hazard: {
    adjacentTags: [
      "dungeon_hazard",
      "temple_hazard"
    ],
    appliesWhen: [
      "Burial sites, funerary wards, grave robbing countermeasures, or undead-rest protections are central to the hazard.",
      "A user would retrieve it for crypts, tombs, barrows, or similar funerary scenes."
    ],
    assignmentMode: "hybrid",
    axis: "setting",
    category: "hazard",
    conceptId: "tomb_hazard",
    currentTag: "tomb_hazard",
    description: "Hazard strongly associated with tombs, barrows, mausoleums, crypts, or burial defenses.",
    doesNotApplyWhen: [
      "The hazard is only underground or haunted without real funerary or burial-site identity.",
      "The stronger placement is temple_hazard or generic dungeon_hazard."
    ],
    family: "setting",
    id: "hazard:tomb_hazard",
    label: "tomb_hazard",
    translationStatus: "mapped"
  },
  tripwire_trigger: {
    assignmentMode: "hybrid",
    axis: "mechanism",
    category: "hazard",
    conceptId: "tripwire_trigger",
    currentTag: "tripwire_trigger",
    description: "Hazard triggered by tugging, crossing, or disturbing a tripwire.",
    family: "mechanism",
    id: "hazard:tripwire_trigger",
    label: "tripwire_trigger",
    translationStatus: "mapped"
  },
  urban_hazard: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "hazard",
    conceptId: "urban_hazard",
    currentTag: "urban_hazard",
    description: "Hazard strongly associated with civic districts, alleys, rooftops, sewers, shops, or other built urban spaces.",
    family: "setting",
    id: "hazard:urban_hazard",
    label: "urban_hazard",
    translationStatus: "mapped"
  },
  ward_trigger: {
    assignmentMode: "hybrid",
    axis: "mechanism",
    category: "hazard",
    conceptId: "ward_trigger",
    currentTag: "ward_trigger",
    description: "Hazard triggered by a rune, glyph, sigil, ward, or similar inscribed mechanism.",
    family: "mechanism",
    id: "hazard:ward_trigger",
    label: "ward_trigger",
    translationStatus: "mapped"
  },
  water_hazard: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "hazard",
    conceptId: "water_application",
    currentTag: "water_hazard",
    description: "Hazard centered on floods, geysers, waves, surges, or other dangerous water exposure.",
    family: "environmental_danger",
    id: "hazard:water_hazard",
    label: "water_application",
    translationStatus: "provisional"
  },
  wilderness_hazard: {
    assignmentMode: "hybrid",
    axis: "setting",
    category: "hazard",
    conceptId: "wilderness_hazard",
    currentTag: "wilderness_hazard",
    description: "Hazard strongly associated with forests, trails, camps, ruins in the wild, or other outdoor expedition scenes.",
    family: "setting",
    id: "hazard:wilderness_hazard",
    label: "wilderness_hazard",
    translationStatus: "mapped"
  },
  zone_denial: {
    assignmentMode: "hybrid",
    axis: "encounter",
    category: "hazard",
    conceptId: "zone_denial",
    currentTag: "zone_denial",
    description: "Hazard that makes an area costly to enter, cross, or remain inside.",
    family: "function",
    id: "hazard:zone_denial",
    label: "zone_denial",
    translationStatus: "provisional"
  }
};
