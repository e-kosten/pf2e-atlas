import { defineConceptProjections } from "../../builders.js";
import { CANONICAL_VOCABULARY } from "../../vocabulary.js";

export const operationalProjectionDeclarations = [
  defineConceptProjections("acid_application", {
    hazard: {
      tag: "acid_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on acid, corrosive spray, caustic runoff, or similar corrosive exposure.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("active_magic_counteraction", {
    affliction: {
      tag: "countermagic_resolution",
      label: "countermagic",
      axis: "response",
      family: "resolution_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Naturally retrieved because counteracting, suppressing, or dispelling an active magical affliction is central to solving it.",
      adjacentTags: [
        "antidote_resolution",
        "cursebreaking_resolution",
      ],
    },
    equipment: {
      tag: "countermagic",
      label: "countermagic",
      axis: "utility",
      family: "anti_magic",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Counteracts, dispels, suppresses, or shuts down magic.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
      ],
      appliesWhen: [
        "The item's main value is actively cancelling, suppressing, or interfering with hostile or ongoing magic.",
        "A user would retrieve it as an anti-magic tool rather than a general protective charm.",
      ],
      doesNotApplyWhen: [
        "The item only protects the wearer from magical harm without disrupting the spell itself.",
        "The item focuses on blocking surveillance or hiding information rather than broader anti-magic interference.",
      ],
      adjacentTags: [
        "magic_protection",
        "scrying_protection",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard: {
      tag: "dispel_countermeasure",
      label: "countermagic",
      axis: "resolution",
      family: "countermeasure_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard meaningfully invites counteract, dispel, or magical suppression as a core answer path.",
      appliesWhen: [
        "A user would plausibly retrieve the hazard because anti-magic answers are central to resolving it.",
        "Magical suppression matters more than physical disarm or spiritual appeasement.",
      ],
      doesNotApplyWhen: [
        "The hazard is magical but best solved through rituals, offerings, or physical tampering instead.",
        "Counteracting is only a minor optional answer path.",
      ],
      adjacentTags: [
        "physical_disarm",
        "ward_trigger",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      tag: "countermagic",
      label: "countermagic",
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Counteracts, dispels, suppresses, or shuts down magic.",
      appliesWhen: [
        "The spell is naturally retrieved because stopping, unravelling, or suppressing existing magic is its main job.",
        "Anti-magic response matters more than simple protection, detection, or concealment.",
      ],
      doesNotApplyWhen: [
        "The spell mainly protects targets from harm without actually disrupting hostile magic.",
        "The spell only reveals or warns about magic rather than counteracting it.",
      ],
      adjacentTags: [
        "magic_detection",
        "protective_ward",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("affliction_cleanup", {
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Cleanses, cures, neutralizes, or removes disease, poison, curse, or similar afflictions.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("ambush_grab", {
    creature: {
      tag: "ambush_grabber",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Captures prey through grabbing, constriction, swallowing whole, webbing, or drag-off ambush tactics.",
    },
  }),
  defineConceptProjections("animal_form", {
    spell: {
      axis: "transformation",
      family: "transformation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Transforms a creature into an animal, beast, pest, or similar natural form.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("barrier_breaking", {
    equipment: {
      axis: "utility",
      family: "breaching",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Designed to tear through walls, barricades, ice, webs, or other physical obstructions.",
      subcategories: [
        "gear",
        "kit",
        "consumable",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("barrier_bypass", {
    equipment: {
      axis: "utility",
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps get through barred windows, grates, force screens, or other blocked passage without relying on brute-force breaching.",
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
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Gets a creature through a blocked threshold, wall, seal, force barrier, or magical ward that otherwise prevents passage.",
      appliesWhen: [
        "The spell is naturally retrieved to pass through, nullify, or ignore a blocking wall, sealed threshold, force barrier, or magical ward.",
        "Crossing the obstruction matters more than simply traveling farther or counteracting magic in the abstract.",
      ],
      doesNotApplyWhen: [
        "The spell only unlocks a door or manipulates a mechanism without really solving a barrier or ward.",
        "The spell's value is ordinary travel or relocation rather than penetrating a blocked passage.",
      ],
      adjacentTags: [
        "lock_bypass",
        "countermagic",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("barrier_creation", {
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creates a wall, dome, cage, force barrier, or other discrete blocking structure that reshapes access lines.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("battle_form", {
    spell: {
      axis: "transformation",
      family: "transformation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Transforms a creature into a combat-ready form with new statistics or battle-form language.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("bleed_remediation", {
    equipment: {
      tag: "anti_bleed",
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps staunch bleeding, end persistent bleed damage, or close ongoing wounds.",
      subcategories: [
        "consumable",
      ],
    },
    spell: {
      tag: "anti_bleed",
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Staunches bleeding, ends persistent bleed damage, or closes wounds that keep draining a target.",
      adjacentTags: [
        "healing_support",
        "condition_support",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("blight_application", {
    hazard: {
      tag: "blight_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on ecological ruin, land-sickening corruption, withering growth, or terrain spoiled by supernatural blight.",
      adjacentTags: [
        "contamination_hazard",
        "overgrowth_hazard",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("buff_support", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Provides a general beneficial enhancement or bonus.",
      subcategories: [
        "consumable",
      ],
    },
  }),
  defineConceptProjections("burst_damage", {
    equipment: {
      axis: "effect",
      family: "offensive_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Delivers a front-loaded blast, detonation, or splash pattern that users naturally retrieve as immediate damage.",
      subcategories: [
        "ammo",
        "consumable",
      ],
      adjacentTags: [
        "crowd_clearing",
        "persistent_damage",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Delivers concentrated damage in a spike or splash pattern that users naturally retrieve as a damage-first answer.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("caster_disruption_remediation", {
    equipment: {
      tag: "anti_caster_disruption",
      axis: "effect",
      family: "offensive_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Punishes casting, disrupts concentration, or is naturally retrieved to make enemy spell use unreliable.",
      subcategories: [
        "ammo",
        "consumable",
      ],
      adjacentTags: [
        "silencing",
        "countermagic",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      tag: "anti_caster_disruption",
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Disrupts casting, punishes spell use, or specifically suppresses hostile spellcasters in the moment.",
      adjacentTags: [
        "countermagic",
        "silencing",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("charm_influence", {
    spell: {
      axis: "influence",
      family: "influence",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Wins cooperation through friendliness, fascination, admiration, or magically altered social regard.",
      appliesWhen: [
        "The spell's main value is improving a target's attitude, trust, or willingness to cooperate.",
        "The spell changes social reception more than it scripts exact behavior.",
      ],
      doesNotApplyWhen: [
        "The spell compels exact actions, overrides agency, or takes total control.",
        "The spell only manipulates mood without establishing a social bond or regard shift.",
      ],
      adjacentTags: [
        "emotion_control",
        "compulsion_control",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("cold_application", {
    hazard: {
      tag: "cold_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on ice, frost, freezing, blizzards, or other cold exposure.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("collapse_application", {
    hazard: {
      tag: "collapse_hazard",
      axis: "effect",
      family: "forced_position",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hazard built around collapsing structures, cave-ins, rockfalls, or crumbling ground.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("compulsion_control", {
    spell: {
      axis: "influence",
      family: "influence",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Forces scripted behavior, movement, or obedience against a target's normal will.",
      appliesWhen: [
        "The spell explicitly pressures the target into doing something, moving somewhere, or obeying a commanded pattern.",
        "Loss of agency is more important than affection, calm, or broad mood change.",
      ],
      doesNotApplyWhen: [
        "The spell merely charms or emotionally softens the target.",
        "The spell fully dominates the target over sustained actions rather than issuing narrower commands.",
      ],
      adjacentTags: [
        "charm_influence",
        "domination",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("condition_support", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps clear or mitigate harmful conditions.",
      subcategories: [
        "consumable",
      ],
    },
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Delays, suppresses, or removes afflictions and conditions.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("confusion_remediation", {
    equipment: {
      tag: "anti_confusion",
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps clear confusion, restore mental steadiness, or recover from disordered thinking.",
      subcategories: [
        "consumable",
      ],
    },
    spell: {
      tag: "anti_confusion",
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Ends confusion, steadies a disordered mind, or protects a target from confusion-like mental unraveling.",
      adjacentTags: [
        "condition_support",
        "anti_fear",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("contamination_application", {
    hazard: {
      tag: "contamination_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on tainted residue, corruptive seepage, drifting spores, cursed runoff, or other lingering contamination of a space.",
      adjacentTags: [
        "poison_hazard",
        "respiratory_hazard",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("contamination_cleanup", {
    equipment: {
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps neutralize tainted residue, clean corrupted surfaces, purify contaminated supplies, or scrub a dangerous site back to safety.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "consumable",
      ],
      adjacentTags: [
        "quarantine_containment",
        "source_cleanup",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard: {
      tag: "contamination_cleanup_countermeasure",
      axis: "resolution",
      family: "countermeasure_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard best resolved through decontamination, purification, cleansing residue, or scrubbing the hazardous space back to safety.",
      appliesWhen: [
        "The hazard is naturally retrieved because cleansing tainted ground, polluted air, cursed runoff, spores, or lingering residue is a core answer path.",
        "The cleanup process matters more than only suppressing the effect temporarily or bypassing the area.",
      ],
      doesNotApplyWhen: [
        "The hazard only has an immediate trigger or burst with no meaningful lingering contamination to clean up.",
        "The stronger fit is exorcism_countermeasure or dispel_countermeasure because the answer is purging a presence or ending an effect rather than cleaning a tainted site.",
      ],
      adjacentTags: [
        "quarantine_containment_countermeasure",
        "source_cleanup_countermeasure",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Cleanses tainted residue, neutralizes corrupted ground, removes lingering pollution, or purifies a contaminated space.",
      adjacentTags: [
        "quarantine_containment",
        "source_cleanup",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("creature_summoning", {
    spell: {
      axis: "summoning",
      family: "summoning",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Summons, conjures, or calls creatures to act as temporary allies, tools, or battlefield assets.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("crowd_clearing", {
    equipment: {
      axis: "effect",
      family: "offensive_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Excels at damaging clusters, swarms, or tightly packed weaker enemies rather than focusing on one target.",
      subcategories: [
        "ammo",
        "consumable",
      ],
      adjacentTags: [
        "burst_damage",
        "line_of_sight_control",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Damages or wipes clusters of weaker foes and is naturally retrieved as an anti-group answer.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("curse_application", {
    creature: {
      tag: "curse_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Threat defined by curses, doom effects, or other lingering supernatural afflictions imposed on victims.",
    },
  }),
  defineConceptProjections("curse_discovery", {
    spell: {
      tag: "curse_revelation",
      axis: "utility",
      family: "revelation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Identifies curses, spiritual corruption, or other malign supernatural bindings on a target.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("curse_remediation", {
    affliction: {
      tag: "cursebreaking_resolution",
      axis: "response",
      family: "resolution_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Naturally retrieved because lifting a curse, breaking a doom, or ending a supernatural binding is central to recovery.",
      adjacentTags: [
        "countermagic_resolution",
        "ritual_appeasement_resolution",
      ],
    },
    equipment: {
      tag: "curse_removal",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps remove, break, or counteract curses as a direct answer path rather than only easing symptoms.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "consumable",
      ],
      adjacentTags: [
        "sanctification",
        "source_cleanup",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      tag: "curse_removal",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Breaks, removes, or counteracts curses as a direct answer path rather than only suppressing symptoms.",
      adjacentTags: [
        "exorcism",
        "sanctification",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("cursefield_application", {
    hazard: {
      tag: "cursefield_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on cursed ground, spiritually poisoned space, or a zone whose danger comes from active supernatural contamination rather than one mechanism.",
      adjacentTags: [
        "contamination_hazard",
        "judgment_haunt",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("death_burst", {
    creature: {
      tag: "death_burst_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Threat defined by explosive death effects, cursed aftermath, or punishing consequences when the creature is dropped.",
    },
  }),
  defineConceptProjections("death_prevention", {
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Prevents death, stabilizes the dying, or brings a creature back from the brink.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("demolition", {
    equipment: {
      axis: "utility",
      family: "breaching",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Designed for blasting, collapsing, or otherwise violently dismantling structures and obstacles.",
      subcategories: [
        "gear",
        "kit",
        "consumable",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("disease_application", {
    creature: {
      tag: "disease_vector",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Threat defined by spreading disease, curse-plague conditions, or infectious aftermath beyond immediate damage.",
    },
  }),
  defineConceptProjections("disease_remediation", {
    equipment: {
      tag: "anti_disease",
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps resist, prevent, or recover from disease.",
      subcategories: [
        "consumable",
      ],
    },
    spell: {
      tag: "anti_disease",
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Cures disease, counteracts infections, or protects a target against plague, fever, and similar disease effects.",
      adjacentTags: [
        "affliction_cleanup",
        "anti_poison",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("domination", {
    spell: {
      axis: "influence",
      family: "influence",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Seizes sustained control over a target's actions, body, or tactical decision-making.",
      appliesWhen: [
        "The spell grants ongoing, high-authority control over what the target does rather than just one compelled action.",
        "A user would retrieve it as a takeover spell, not merely a charm or suggestion spell.",
      ],
      doesNotApplyWhen: [
        "The spell only improves attitude, stirs emotion, or issues narrower one-off compulsions.",
        "The spell mainly suppresses actions without redirecting them under the caster's control.",
      ],
      adjacentTags: [
        "compulsion_control",
        "action_denial",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("door_breaching", {
    equipment: {
      axis: "utility",
      family: "breaching",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps force doors, shutters, gates, or similar entry points open by strength, impact, or destructive entry.",
      subcategories: [
        "gear",
        "kit",
        "consumable",
        "weapon",
      ],
      appliesWhen: [
        "The item's retrieval value is getting through doors, shutters, gates, or secured entry points.",
        "It solves access by force rather than by keys, stealth, or lock tools.",
      ],
      doesNotApplyWhen: [
        "The item is for larger demolition or siegework rather than point-of-entry breach.",
        "The item bypasses access quietly through locks or trickery instead of force.",
      ],
      adjacentTags: [
        "lock_bypass",
        "barrier_breaking",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("eidolon_support", {
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Directly benefits an eidolon or the summoner-eidolon bond.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("electric_application", {
    hazard: {
      tag: "electric_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on lightning, shock, static discharge, or electrical exposure.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("elemental_form", {
    spell: {
      axis: "transformation",
      family: "transformation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Transforms a creature into an elemental form.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("emotion_control", {
    spell: {
      axis: "influence",
      family: "influence",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Directly manipulates fear, calm, rage, love, despair, or other emotional states.",
      appliesWhen: [
        "The spell is naturally retrieved for changing a creature's feelings, morale, or emotional volatility.",
        "The emotional state change matters more than explicit obedience or truth extraction.",
      ],
      doesNotApplyWhen: [
        "The spell chiefly compels discrete actions or sustained domination.",
        "The spell only inflicts fear as damage pressure without broader emotional steering.",
      ],
      adjacentTags: [
        "fear_pressure",
        "charm_influence",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("escape_support", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps flee, slip away, or break free.",
      subcategories: [
        "consumable",
      ],
    },
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Helps a creature slip away, break free, flee, or evade pursuit.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("excavation", {
    equipment: {
      axis: "utility",
      family: "breaching",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps dig, cut through earth or stone, or otherwise open a route by excavation or practical earth-moving work.",
      subcategories: [
        "gear",
        "kit",
        "consumable",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("false_safe_route", {
    hazard: {
      axis: "effect",
      family: "perception_control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that tempts intruders toward a seemingly safer path or escape line that is itself the trap.",
      appliesWhen: [
        "The hazard actively misdirects intruders toward a route that looks protective, faster, or safer but is actually the danger.",
        "The retrieval hook is deceptive path choice rather than only illusion damage or generic navigation confusion.",
      ],
      doesNotApplyWhen: [
        "The hazard merely scrambles orientation without presenting a tempting fake safe path.",
        "The hazard is solved through procedure, but misleading route presentation is not central to how it works.",
      ],
      adjacentTags: [
        "navigation_disruption",
        "procedural_bypass",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("fear_pressure", {
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Forces fear, panic, dread, or morale collapse onto a target.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("fear_remediation", {
    equipment: {
      tag: "anti_fear",
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps resist fear, recover from frightened effects, or steady courage.",
      subcategories: [
        "consumable",
      ],
    },
    spell: {
      tag: "anti_fear",
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Counters frightened or fear effects, bolsters courage, or protects a target against fear.",
      adjacentTags: [
        "condition_support",
        "anti_confusion",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("fire_application", {
    hazard: {
      tag: "fire_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on open fire, flames, burning spread, or explosive ignition.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("floor_eruption", {
    hazard: {
      axis: "effect",
      family: "attack_vector",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that attacks upward from the ground, floor, or a concealed underfoot chamber.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("forced_movement", {
    hazard: {
      axis: "effect",
      family: "forced_position",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hazard that pushes, pulls, drags, submerges, or otherwise forcibly repositions creatures.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Pushes, pulls, drags, or otherwise repositions a target against its will.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("fortune_support", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Improves a creature's odds with rerolls, better-result effects, or failure rescue.",
      subcategories: [
        "consumable",
      ],
    },
  }),
  defineConceptProjections("hazard_discovery", {
    spell: {
      tag: "hazard_revelation",
      axis: "utility",
      family: "revelation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Reveals hidden traps, secret wards, concealed passage dangers, or other obscured environmental threats.",
      appliesWhen: [
        "The spell is naturally retrieved to uncover traps, hidden dangers, secret magical wards, or dangerous concealed features in a location.",
        "Hazard discovery matters more than general magical detection or long-range scouting.",
      ],
      doesNotApplyWhen: [
        "The spell only detects magic, invisible creatures, or truth without specifically surfacing dangerous hidden features.",
        "The spell merely scouts an area from afar without exposing concealed trap logic or hazard placement.",
      ],
      adjacentTags: [
        "magic_detection",
        "scouting",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("healing_support", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Restores hit points or provides direct healing.",
      subcategories: [
        "consumable",
      ],
    },
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Directly restores hit points or accelerates recovery.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("hostile_presence_expulsion", {
    affliction: {
      tag: "exorcism_resolution",
      label: "exorcism",
      axis: "response",
      family: "resolution_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Naturally retrieved because banishing, cleansing, or spiritually expelling a hostile presence is central to solving the affliction.",
      adjacentTags: [
        "cursebreaking_resolution",
        "ritual_appeasement_resolution",
      ],
    },
    hazard: {
      tag: "exorcism_countermeasure",
      label: "exorcism",
      axis: "resolution",
      family: "countermeasure_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard best resolved through banishment, exorcism, consecration, or another spirit-cleansing answer.",
      appliesWhen: [
        "The hazard is naturally retrieved because cleansing, banishing, or sanctifying the hostile presence is a core answer path.",
        "A spiritual purge matters more than appeasement, anti-magic suppression, or physical mechanism work.",
      ],
      doesNotApplyWhen: [
        "The hazard mainly wants offerings, restitution, or ritual respect rather than expulsion.",
        "The hazard is magical or mechanical but not really spirit-cleansed out of existence.",
      ],
      adjacentTags: [
        "appeasement_countermeasure",
        "dispel_countermeasure",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      tag: "exorcism",
      label: "exorcism",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Banishes, expels, or spiritually drives out a hostile spirit, possession, haunt, or invading supernatural presence.",
      adjacentTags: [
        "curse_removal",
        "sanctification",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("illusion_assault", {
    hazard: {
      axis: "effect",
      family: "perception_control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hazard that attacks through deceptive reflections, phantasms, or other hostile illusion-driven distortions.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("infiltration", {
    creature: {
      tag: "infiltration_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Threat defined by disguise, replacement, infiltration, or remaining embedded among victims before the danger fully reveals itself.",
      adjacentTags: [
        "disguised_pretender",
        "possession_threat",
      ],
    },
    equipment: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description: "Broad infiltration umbrella for quiet-entry, discreet-carry, disguise, and covert-passing equipment.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
      ],
      adjacentTags: [
        "stealth_support",
        "disguise",
        "social_infiltration",
      ],
      compositeOfAnyTags: [
        "stealth_support",
        "concealable",
        "disguise",
        "social_infiltration",
        "concealment",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "infiltration",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description: "Broad infiltration umbrella for quiet-entry, disguise, and covert social-passing spells.",
      adjacentTags: [
        "stealth_support",
        "disguise",
        "social_infiltration",
      ],
      compositeOfAnyTags: [
        "stealth_support",
        "disguise",
        "social_infiltration",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("initiative_support", {
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Improves initiative, pre-combat readiness, or the party's opening tempo before the first turn.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("invisibility_discovery", {
    spell: {
      tag: "invisibility_reveal",
      axis: "utility",
      family: "revelation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Exposes invisible, hidden, concealed, or magically obscured creatures and objects.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("life_drain_application", {
    creature: {
      tag: "life_drain_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Threat defined by draining blood, vitality, life force, or souls from victims.",
    },
  }),
  defineConceptProjections("line_of_sight_control", {
    equipment: {
      axis: "effect",
      family: "offensive_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Obscures vision, fills an area with smoke, or otherwise denies clear sight lines as the item's main offensive or tactical job.",
      subcategories: [
        "ammo",
        "consumable",
      ],
      adjacentTags: [
        "sensory_impairment",
        "restraint_capture",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Blocks vision, obscures sight lines, or denies clear observation across an area.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("lock_bypass", {
    equipment: {
      axis: "utility",
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps open locks or bypass secured entry points.",
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
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Opens locks, sealed containers, secured doors, or similar closed access points through magic rather than physical lockpicking.",
      appliesWhen: [
        "The spell is naturally retrieved to unlock, unseal, or open a secured entry point, door, chest, manacle, or similar closure.",
        "Accessing something closed matters more than broad movement, damage, or generic anti-magic.",
      ],
      doesNotApplyWhen: [
        "The spell mainly destroys the obstacle, bypasses the whole wall, or teleports past the problem without interacting with the locked access point.",
        "The spell only manipulates unattended objects generally and opening secured access is not a real retrieval hook.",
      ],
      adjacentTags: [
        "trap_bypass",
        "barrier_bypass",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("magic_discovery", {
    spell: {
      tag: "magic_detection",
      axis: "utility",
      family: "revelation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Reveals magical auras, spell presence, active effects, or other supernatural signatures.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("magic_protection", {
    equipment: {
      axis: "utility",
      family: "anti_magic",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Protects the user or target against hostile magical effects.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
      ],
      appliesWhen: [
        "The item's value comes from warding the bearer against curses, spells, hostile magical conditions, or magical damage.",
        "Protection matters more than actually counteracting or suppressing the incoming magic.",
      ],
      doesNotApplyWhen: [
        "The item mainly shuts down active magic rather than defending a wearer or target.",
        "The stronger fit is scrying_protection because surveillance denial is the specific retrieval hook.",
      ],
      adjacentTags: [
        "countermagic",
        "hazard_shielding",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("mechanism_manipulation", {
    equipment: {
      axis: "utility",
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps operate levers, latches, panels, pressure surfaces, or similar scene mechanisms from a safer or more advantageous position.",
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
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Precisely triggers, moves, holds, or operates levers, buttons, switches, pressure plates, locks, or similar scene mechanisms.",
      appliesWhen: [
        "The spell is naturally retrieved to operate a lever, button, latch, control panel, pressure surface, or similar mechanism from a safe or unusual position.",
        "The mechanism interaction itself matters more than broad telekinesis, damage, or ordinary object movement.",
      ],
      doesNotApplyWhen: [
        "The spell only moves creatures or loose objects without a real access-, control-, or mechanism-facing use case.",
        "The spell bypasses the obstacle by teleporting or destroying it instead of operating the mechanism.",
      ],
      adjacentTags: [
        "lock_bypass",
        "trap_bypass",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("memory_manipulation", {
    spell: {
      axis: "influence",
      family: "influence",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Edits, suppresses, restores, or rewrites memories, recollection, and remembered events.",
      appliesWhen: [
        "The spell is naturally retrieved for altering what a target remembers, forgets, or believes it experienced.",
        "Memory editing is more central than charm, emotion, or truth exposure.",
      ],
      doesNotApplyWhen: [
        "The spell only reveals truth or emotions without changing stored recollection.",
        "The spell primarily imposes obedience or domination in the present moment.",
      ],
      adjacentTags: [
        "truth_reveal",
        "charm_influence",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("mental_recovery", {
    equipment: {
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps stabilize emotions or recover from mental conditions.",
      subcategories: [
        "consumable",
      ],
    },
  }),
  defineConceptProjections("mobility_denial", {
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Pins, slows, grounds, or otherwise prevents normal repositioning without necessarily functioning as a full restraint effect.",
      adjacentTags: [
        "forced_movement",
        "restraint_capture",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("navigation_disruption", {
    hazard: {
      axis: "effect",
      family: "perception_control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hazard that confounds routes, loops intruders, or scrambles navigation through distorted perception.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("offensive_summons", {
    spell: {
      axis: "summoning",
      family: "summoning",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creates summons primarily retrieved for direct damage, flanking pressure, or aggressive battlefield threat.",
      adjacentTags: [
        "creature_summoning",
        "screening_summons",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("outbreak_containment", {
    affliction: [
      {
            tag: "outbreak_management",
            axis: "response",
            family: "response_profile",
            assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
            description: "Naturally retrieved as a disease or curse that creates a wider containment, treatment, and community-management problem rather than only an isolated victim.",
            adjacentTags: [
              "community_outbreak",
              "quarantine_risk",
            ],
          },
      {
            tag: "quarantine_containment_resolution",
            axis: "response",
            family: "resolution_profile",
            assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
            description: "Naturally retrieved because isolation, contact control, and containment are core to preventing further spread while treatment proceeds.",
            adjacentTags: [
              "outbreak_management",
              "quarantine_risk",
            ],
          },
      {
            tag: "quarantine_risk",
            axis: "response",
            family: "response_profile",
            assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
            description: "Creates a strong need to isolate victims, restrict contact, or manage who can safely enter or leave an affected area.",
            adjacentTags: [
              "community_outbreak",
              "inhaled_exposure",
            ],
          },
    ],
    equipment: {
      tag: "quarantine_containment",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps isolate victims, secure contaminated areas, or impose practical containment procedures that stop spread while treatment proceeds.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "consumable",
      ],
      adjacentTags: [
        "contamination_cleanup",
        "source_cleanup",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard: {
      tag: "quarantine_containment_countermeasure",
      axis: "resolution",
      family: "countermeasure_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard best managed by isolating victims, sealing off the site, or imposing containment boundaries that stop spread while the danger is being handled.",
      appliesWhen: [
        "The hazard is naturally retrieved because the first meaningful answer is locking down spread, controlling access, or containing dangerous exposure.",
        "Containment procedures matter more than immediately dispelling, disarming, or appeasing the hazard.",
      ],
      doesNotApplyWhen: [
        "The hazard is dangerous but does not meaningfully spread, linger, or demand isolation and access control.",
        "The stronger fit is barrier_lockdown or sentinel_guardian because preventing passage is the hazard's function, not the party's resolution plan.",
      ],
      adjacentTags: [
        "contamination_cleanup_countermeasure",
        "source_cleanup_countermeasure",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      tag: "quarantine_containment",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Helps isolate victims, secure a dangerous area, or impose protective boundaries that stop spread while the problem is being solved.",
      adjacentTags: [
        "protective_ward",
        "contamination_cleanup",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("overgrowth_application", {
    hazard: {
      tag: "overgrowth_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on choking roots, hostile vines, grasping thorns, or other dangerous living overgrowth that turns terrain against intruders.",
      adjacentTags: [
        "blight_hazard",
        "forced_movement",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("overhead_strike", {
    hazard: {
      axis: "effect",
      family: "attack_vector",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hazard that drops, crashes, or attacks from the ceiling or another overhead position.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("paralysis_remediation", {
    equipment: {
      tag: "anti_paralysis",
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps break paralysis, restore movement, or free a creature from immobilizing body shutdown.",
      subcategories: [
        "consumable",
      ],
    },
    spell: {
      tag: "anti_paralysis",
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Ends paralysis, restores bodily function, or frees a target from magic or afflictions that leave it unable to move.",
      adjacentTags: [
        "condition_support",
        "escape_support",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("parasite_removal", {
    affliction: {
      tag: "surgical_extraction_resolution",
      axis: "response",
      family: "resolution_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Naturally retrieved because removing eggs, larvae, parasites, implants, or invasive growth from the body is a central answer path.",
      adjacentTags: [
        "source_cleanup_resolution",
        "infestation_implant",
      ],
    },
  }),
  defineConceptProjections("persistent_damage", {
    equipment: {
      axis: "effect",
      family: "offensive_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Imposes ongoing damage through burning, acid, bleed, poison, or another lingering payload that keeps hurting after the initial hit.",
      subcategories: [
        "ammo",
        "consumable",
      ],
      adjacentTags: [
        "burst_damage",
        "physical_debilitation",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Directly inflicts persistent damage or grants attacks that reliably impose persistent damage.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("petrification_application", {
    creature: {
      tag: "petrification_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Threat defined by petrifying victims or turning them to stone.",
    },
  }),
  defineConceptProjections("petrification_remediation", {
    equipment: {
      tag: "anti_petrification",
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps prevent or reverse petrification and other stone-turning effects.",
      subcategories: [
        "consumable",
      ],
    },
    spell: {
      tag: "anti_petrification",
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Prevents, reverses, or counteracts petrification and other turn-to-stone effects.",
      adjacentTags: [
        "affliction_cleanup",
        "condition_support",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("physical_disarm", {
    hazard: {
      axis: "resolution",
      family: "countermeasure_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard meaningfully invites physical mechanism tampering, disassembly, or trigger-blocking as the core answer path.",
      appliesWhen: [
        "The hazard is naturally retrieved because tools, hands-on disable work, or mechanism access are central to solving it.",
        "Mechanical tampering matters more than safe sequencing, anti-magic, or spiritual negotiation.",
      ],
      doesNotApplyWhen: [
        "The hazard is mainly solved by learning the right pattern, dispelling an effect, or meeting a spiritual demand.",
        "Physical interaction exists only as one optional fallback rather than the main resolution mode.",
      ],
      adjacentTags: [
        "procedural_bypass",
        "dispel_countermeasure",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("pitfall", {
    hazard: {
      axis: "effect",
      family: "forced_position",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hazard built around a concealed pit, drop, or similar vertical fall trap.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("poison_application", {
    creature: {
      tag: "poison_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Threat defined by venom, toxic excretions, poisoned weapons, or other recurring poison delivery.",
    },
    hazard: {
      tag: "poison_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on poison gas, toxic delivery, or other poisonous exposure.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("poison_remediation", {
    affliction: {
      tag: "antidote_resolution",
      axis: "response",
      family: "resolution_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Naturally retrieved because antitoxins, antidotes, neutralizing medicine, or poison-specific treatment are central to solving it.",
      adjacentTags: [
        "countermagic_resolution",
        "cure_clock_urgency",
      ],
    },
    equipment: {
      tag: "anti_poison",
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps resist, prevent, or recover from poison.",
      subcategories: [
        "consumable",
      ],
    },
    spell: {
      tag: "anti_poison",
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Cures poison, counters toxic afflictions, or protects a target against venom and similar poisoning effects.",
      adjacentTags: [
        "affliction_cleanup",
        "anti_disease",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("possession_application", {
    creature: {
      tag: "possession_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Can possess, body-snatch, or take control of a victim from within.",
      appliesWhen: [
        "Use when entering, riding, replacing, or controlling a host body is a major reason to retrieve the creature.",
        "The possession dynamic matters more than ordinary charm, domination, or haunting flavor.",
      ],
      doesNotApplyWhen: [
        "The creature only compels, frightens, or mentally influences targets without true body-occupying takeover.",
        "The stronger fit is curse_threat or reinforcement_threat because possession is not central to encounter prep.",
      ],
      adjacentTags: [
        "curse_threat",
        "reinforcement_threat",
      ],
    },
  }),
  defineConceptProjections("prey_control", {
    creature: {
      tag: "prey_control_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Threat defined by holding prey in place through grabs, constriction, webbing, swallowing, or other ongoing body-control pressure.",
      adjacentTags: [
        "ambush_grabber",
        "terrain_control_threat",
      ],
    },
  }),
  defineConceptProjections("procedural_bypass", {
    hazard: {
      axis: "resolution",
      family: "countermeasure_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard best bypassed through the right route, timing, command phrase, ritual sequence, or other correct procedure rather than direct disarm.",
      appliesWhen: [
        "The clean answer is learning and executing the hazard's safe procedure, sequence, or pattern rather than destroying it.",
        "A GM would plausibly retrieve the hazard for puzzle-like bypass, safe-route discovery, or passphrase-style navigation.",
      ],
      doesNotApplyWhen: [
        "The main answer is simple mechanism tampering, counteracting magic, or appeasing a spirit.",
        "The hazard only has a minor caution or tactical workaround without a real procedural solution.",
      ],
      adjacentTags: [
        "physical_disarm",
        "false_safe_route",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("projectile_emitter", {
    hazard: {
      axis: "effect",
      family: "attack_vector",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hazard that fires bolts, beams, jets, sprays, or similar directed emissions from a fixed emitter.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("protective_ward", {
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Places a ward, sanctuary, shield, or protective boundary.",
      appliesWhen: [
        "The spell is naturally retrieved as a defensive ward, sanctuary, or protective boundary rather than only a resistance buff.",
        "Its value comes from shielding a creature, object, or space against incoming harm or intrusion.",
      ],
      doesNotApplyWhen: [
        "The spell only grants resistance, temporary Hit Points, or healing without a real warding or boundary element.",
        "The spell is mainly an alarm, anti-scrying, or mobility tool rather than direct protection.",
      ],
      adjacentTags: [
        "alarm",
        "resistance_support",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("proximity_burst", {
    hazard: {
      axis: "effect",
      family: "attack_vector",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that erupts in an immediate burst, blast, or detonation when a victim comes near or crosses a point.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("quickened_support", {
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Grants extra actions, quickened condition benefits, or similar action-economy acceleration.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("regeneration", {
    creature: {
      tag: "regeneration_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Regenerates or requires special suppression or finishing countermeasures.",
    },
  }),
  defineConceptProjections("reinforcement", {
    creature: {
      tag: "reinforcement_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Threat defined by materially changing encounter structure through added bodies, activated subordinates, or sharply elevated allied creatures.",
      appliesWhen: [
        "Use when the creature's main prep significance is that it adds bodies, activates subordinates, or sharply force-multiplies nearby allies.",
        "The encounter meaningfully changes because of its reinforcement engine rather than just because it personally hits hard.",
      ],
      doesNotApplyWhen: [
        "The creature only has one minor ally-facing buff or an incidental summon without materially changing encounter structure.",
        "The stronger fit is support_combatant, commander_combatant, or spawn_creator because reinforcement is not the real threat hook.",
      ],
      adjacentTags: [
        "spawn_creator",
        "commander_combatant",
      ],
    },
  }),
  defineConceptProjections("resistance_support", {
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Grants resistance or immunity against energy, damage, or hazards.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("respiratory_application", {
    hazard: {
      tag: "respiratory_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on smoke, choking vapor, breathlessness, or impaired breathing.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("restraint_capture", {
    equipment: {
      axis: "utility",
      family: "restraint",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps capture, bind, or keep a target restrained.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
        "armor",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that binds, restrains, or holds intruders in place.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Restrains, immobilizes, entangles, or traps a target in place.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("restraint_escape", {
    equipment: {
      axis: "utility",
      family: "restraint",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps break free from grabs, restraints, or similar immobilizing holds.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
        "armor",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("ritual_appeasement", {
    affliction: {
      tag: "ritual_appeasement_resolution",
      axis: "response",
      family: "resolution_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Naturally retrieved because restitution, offerings, ritual respect, or meeting a spiritual demand is central to ending the affliction.",
      adjacentTags: [
        "cursebreaking_resolution",
        "exorcism_resolution",
      ],
    },
    equipment: {
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports offerings, restitution, funerary observance, or appeasement ceremonies used to satisfy a spirit, haunt, curse, or sacred demand without directly expelling it.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "consumable",
      ],
      appliesWhen: [
        "The item's retrieval value comes from helping perform offerings, appeasement rites, restitution rituals, or ceremonial observance meant to settle a supernatural grievance.",
        "It is naturally sought for placation or ritual satisfaction rather than direct cleansing, banishment, or ordinary worship.",
      ],
      doesNotApplyWhen: [
        "The item only supports broad ritual process with no real appeasement, offering, or restitution-facing role.",
        "The stronger fit is sanctification or ritual_support because the item purifies generally or supports any rite rather than a placation answer path.",
      ],
      adjacentTags: [
        "ritual_support",
        "sanctification",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard: {
      tag: "appeasement_countermeasure",
      axis: "resolution",
      family: "countermeasure_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard best resolved through offerings, ritual respect, social appeasement, or meeting a spiritual demand.",
      appliesWhen: [
        "The hazard meaningfully invites negotiation-by-ritual, restitution, reverence, or satisfying an unmet dead or sacred demand.",
        "The nonviolent spiritual answer is more central than simply disabling mechanics or dispelling magic.",
      ],
      doesNotApplyWhen: [
        "The hazard only needs a standard disable check, counteract, or forceful destruction.",
        "The hazard is haunted but has no appeasement-style resolution path.",
      ],
      adjacentTags: [
        "exorcism_countermeasure",
        "judgment_haunt",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Ends a supernatural problem through offerings, restitution, funerary respect, ritual observance, or otherwise satisfying a spiritual demand rather than expelling the presence outright.",
      adjacentTags: [
        "sanctification",
        "exorcism",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("sacred_taint_sanctification", {
    equipment: {
      tag: "sanctification",
      label: "sanctification",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports hallowing, consecration, spiritual purification, or cleansing rites applied to a creature, object, or site.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "consumable",
      ],
      appliesWhen: [
        "The item's retrieval value comes from consecrating, hallowing, purifying, or spiritually cleansing a target or place.",
        "It is naturally sought as part of sacred-site cleanup, anti-haunt work, or ritual purification rather than general divine symbolism.",
      ],
      doesNotApplyWhen: [
        "The item is only religious, ceremonial, or devotional without materially helping purification or consecration.",
        "The stronger fit is ritual_support because the item supports a broad rite rather than sanctification in particular.",
      ],
      adjacentTags: [
        "ritual_support",
        "ritual_appeasement",
        "curse_removal",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      tag: "sanctification",
      label: "sanctification",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Consecrates, hallowes, purifies, or spiritually cleanses a creature, object, or site to solve a malign supernatural problem.",
      adjacentTags: [
        "ritual_appeasement",
        "exorcism",
        "protective_ward",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("scouting_summons", {
    spell: {
      axis: "summoning",
      family: "summoning",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creates a summon primarily valued for reconnaissance, watch duty, sensing, or forward information gathering.",
      adjacentTags: [
        "summoned_servitor",
        "creature_summoning",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("screening_summons", {
    spell: {
      axis: "summoning",
      family: "summoning",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creates bodies mainly used to block space, absorb hits, or protect allies rather than maximize damage.",
      adjacentTags: [
        "creature_summoning",
        "temporary_hp_support",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("siege_support", {
    equipment: {
      axis: "utility",
      family: "breaching",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Supports attacking gates, fortifications, vehicles, or other larger hardened targets.",
      subcategories: [
        "gear",
        "kit",
        "consumable",
        "weapon",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("single_target_removal", {
    spell: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Drops, banishes, petrifies, or otherwise decisively removes one important creature from the fight.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("sleep_magic", {
    spell: {
      axis: "influence",
      family: "influence",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Puts creatures to sleep, into magical slumber, or into a similarly enforced dormant state.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("sound_application", {
    hazard: {
      tag: "sound_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on sonic force, deafening noise, vibration, or resonant disruption.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("source_cleanup", {
    affliction: {
      tag: "source_cleanup_resolution",
      axis: "response",
      family: "resolution_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Naturally retrieved because the contaminated site, cursed source, infected carrier chain, or environmental origin must be found and cleaned up.",
      adjacentTags: [
        "source_tracing",
        "quarantine_containment_resolution",
      ],
    },
    equipment: {
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps find, remove, neutralize, or safely dispose of the cursed object, infected material, corrupted remains, or other source driving the problem.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "consumable",
      ],
      adjacentTags: [
        "source_revelation",
        "contamination_cleanup",
        "ritual_support",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard: {
      tag: "source_cleanup_countermeasure",
      axis: "resolution",
      family: "countermeasure_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard best resolved by locating and neutralizing the cursed source, leaking node, corrupted remains, or other origin driving the dangerous field.",
      appliesWhen: [
        "The hazard is naturally retrieved because the real answer is finding and dealing with the source object, origin point, or contamination engine.",
        "Neutralizing the origin matters more than only enduring the space or treating downstream symptoms.",
      ],
      doesNotApplyWhen: [
        "The hazard has no meaningful source object, leak point, or origin to clean up beyond the hazard itself.",
        "The stronger fit is procedural_bypass or physical_disarm because the answer is executing a sequence or tampering with a mechanism rather than eliminating an origin source.",
      ],
      adjacentTags: [
        "quarantine_containment_countermeasure",
        "contamination_cleanup_countermeasure",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Neutralizes, destroys, seals, or cleans up the cursed object, infected origin, corrupted site, or anchored source driving the problem.",
      adjacentTags: [
        "source_revelation",
        "contamination_cleanup",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("source_discovery", {
    affliction: {
      tag: "source_tracing",
      axis: "response",
      family: "response_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Naturally retrieved because finding the contaminated source, carrier chain, cursed origin, or initial spread event is central to solving the problem.",
      adjacentTags: [
        "carrier_vector",
        "community_outbreak",
      ],
    },
    equipment: {
      tag: "source_revelation",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps identify the cursed anchor, contaminated material, infected origin, hidden carrier, or other source driving the problem before cleanup begins.",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "consumable",
      ],
      appliesWhen: [
        "The item's retrieval value comes from finding or confirming the hidden source of a curse, contamination, outbreak, or spiritually tainted problem.",
        "It is naturally sought for tracing the origin or anchor rather than directly cleansing or disposing of it.",
      ],
      doesNotApplyWhen: [
        "The item only helps perform a cleanup, disposal, or purification step after the source is already known.",
        "The stronger fit is medical_support, tracking, or ritual_support because the item supports a broader process without specifically revealing the source.",
      ],
      adjacentTags: [
        "source_cleanup",
        "contamination_cleanup",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard: {
      tag: "source_tracing",
      axis: "problem",
      family: "problem_shape",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard whose real puzzle is locating the hidden anchor, leak point, cursed source, contamination engine, or origin node before a clean solution becomes possible.",
      appliesWhen: [
        "The hazard is naturally retrieved because identifying the source object, origin point, or contamination engine is a major part of solving it.",
        "Finding what is powering the danger matters more than merely surviving exposure or executing a known disable procedure.",
      ],
      doesNotApplyWhen: [
        "The hazard is fully understandable up front and the real challenge is timing, endurance, or multi-step execution rather than finding an origin.",
        "The stronger fit is source_cleanup_countermeasure because the source is already obvious and the remaining task is neutralizing it.",
      ],
      adjacentTags: [
        "observation_first",
        "source_cleanup_countermeasure",
      ],
    },
    spell: {
      tag: "source_revelation",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Reveals the hidden source, curse anchor, carrier, infected origin, or spreading point of a supernatural or outbreak problem.",
      adjacentTags: [
        "problem_diagnosis",
        "source_cleanup",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("spawn_creation", {
    creature: {
      tag: "spawn_creator",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creates additional threats through infestation, spawn-making, conversion, or implanted offspring.",
    },
  }),
  defineConceptProjections("summoned_servitor", {
    spell: {
      axis: "summoning",
      family: "summoning",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Creates a helper, laborer, scout, mount, or similarly task-focused magical servitor rather than a pure combat summon.",
      appliesWhen: [
        "The spell is naturally retrieved for utility help, labor, scouting, transport, or task performance rather than frontline combat stats.",
        "The conjured ally behaves more like a helper or specialist tool than a main battle summon.",
      ],
      doesNotApplyWhen: [
        "The spell's main value is summoning a combat creature to attack, flank, or absorb hits.",
        "The spell only creates an object, barrier, or terrain effect without a real servant-like entity.",
      ],
      adjacentTags: [
        "creature_summoning",
        "scouting",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("temporary_hp_support", {
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Grants temporary Hit Points or similar buffer protection instead of restoring lost Hit Points.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("terrain_control", {
    creature: {
      tag: "terrain_control_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Threat defined by webs, walls, zones, hazards, or other space-shaping control that changes battlefield movement.",
    },
  }),
  defineConceptProjections("trap_bypass", {
    equipment: {
      axis: "utility",
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps disarm, disable, or get past traps.",
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
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Disarms, suppresses, safely triggers, or helps bypass a trap, warded threshold, or similar trapped access problem.",
      appliesWhen: [
        "The spell is naturally retrieved to disable, neutralize, or get past a trap or trapped access point without simply enduring the hazard.",
        "Trap-solving matters more than generic revelation, scouting, or damage prevention.",
      ],
      doesNotApplyWhen: [
        "The spell only reveals that a trap exists without helping bypass or disable it.",
        "The spell mainly counters open combat hazards or battlefield effects rather than access-facing traps.",
      ],
      adjacentTags: [
        "lock_bypass",
        "mechanism_manipulation",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("truth_discovery", {
    spell: {
      tag: "truth_reveal",
      axis: "utility",
      family: "revelation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Forces honesty, exposes lies, or reveals disguised, false, or hidden truths.",
      appliesWhen: [
        "The spell's retrieval value comes from exposing deception, forcing truthful answers, or stripping away false presentation.",
        "A user would plausibly look for it when they need an answer spell rather than a sensor spell.",
      ],
      doesNotApplyWhen: [
        "The spell only detects magic, invisibility, or general auras without interrogating truth or deception.",
        "The spell mainly alters memory or emotions rather than revealing facts.",
      ],
      adjacentTags: [
        "magic_detection",
        "memory_manipulation",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("undead_summoning", {
    spell: {
      axis: "summoning",
      family: "summoning",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Summons, calls, or manifests undead entities, spirits of the dead, or corpse-driven servitors.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("water_application", {
    hazard: {
      tag: "water_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on floods, geysers, waves, surges, or other dangerous water exposure.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
];
