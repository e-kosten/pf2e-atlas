import type { DerivedTagCategoryProjection } from "../../../domain/derived-tag-types.js";

export const AFFLICTION_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG: Record<string, DerivedTagCategoryProjection> = 
{
  action_denial: {
    adjacentTags: [
      "mobility_impairment",
      "sedation"
    ],
    appliesWhen: [
      "The affliction is naturally retrieved because victims become unable to act, respond, or complete normal turns through paralysis, shutdown, or severe stupor.",
      "Operational lockout matters more than ordinary weakness, pain, or mood distortion."
    ],
    assignmentMode: "hybrid",
    axis: "effect",
    category: "affliction",
    conceptId: "action_denial",
    currentTag: "action_denial",
    description: "Prevents normal action-taking through paralysis, stupefying shutdown, or similarly severe operational lockout.",
    doesNotApplyWhen: [
      "The affliction only slows, weakens, or frightens the victim without truly preventing action-taking.",
      "The stronger fit is mobility_impairment, sedation, or mental_impairment rather than hard action denial."
    ],
    family: "impact",
    id: "affliction:action_denial",
    label: "action_denial",
    translationStatus: "mapped"
  },
  antidote_resolution: {
    adjacentTags: [
      "countermagic_resolution",
      "cure_clock_urgency"
    ],
    assignmentMode: "hybrid",
    axis: "response",
    category: "affliction",
    conceptId: "poison_remediation",
    currentTag: "antidote_resolution",
    description: "Naturally retrieved because antitoxins, antidotes, neutralizing medicine, or poison-specific treatment are central to solving it.",
    family: "resolution_profile",
    id: "affliction:antidote_resolution",
    label: "poison_remediation",
    translationStatus: "mapped"
  },
  bestial_transformation: {
    adjacentTags: [
      "transformative_corruption",
      "violence_compulsion"
    ],
    appliesWhen: [
      "The affliction's retrieval value comes from becoming feral, animalistic, lycanthropic, or beast-shaped over time.",
      "Behavioral and bodily slide toward a beast identity are both salient."
    ],
    assignmentMode: "hybrid",
    axis: "disease_model",
    category: "affliction",
    conceptId: "bestial_transformation",
    currentTag: "bestial_transformation",
    description: "Defined by animalistic mutation, feral reshaping, or a cursed slide into beastlike behavior and form.",
    doesNotApplyWhen: [
      "The transformation is general corruption without a real animalistic or feral endpoint.",
      "The affliction only compels violent behavior without reshaping the victim's form or identity."
    ],
    family: "pathogenesis",
    id: "affliction:bestial_transformation",
    label: "bestial_transformation",
    translationStatus: "mapped"
  },
  blood_rot: {
    adjacentTags: [
      "hemorrhagic_failure",
      "physical_debilitation"
    ],
    appliesWhen: [
      "Corrupted blood, blackened veins, or bloodstream collapse is the core thematic identity of the affliction.",
      "A user would retrieve it for blood-plague, hemorrhagic corruption, or vein-darkening disease imagery."
    ],
    assignmentMode: "hybrid",
    axis: "disease_model",
    category: "affliction",
    conceptId: "blood_rot",
    currentTag: "blood_rot",
    description: "Defined by corrupted blood, blackened veins, hemorrhagic poisoning, or other blood-borne bodily collapse.",
    doesNotApplyWhen: [
      "The affliction only causes bleeding as one symptom without a real blood-corruption identity.",
      "The stronger fit is hemorrhagic_failure because the focus is immediate bleeding outcome rather than disease theme."
    ],
    family: "pathogenesis",
    id: "affliction:blood_rot",
    label: "blood_rot",
    translationStatus: "mapped"
  },
  carrier_vector: {
    adjacentTags: [
      "injury_exposure",
      "waterborne_exposure"
    ],
    appliesWhen: [
      "The affliction's spread is materially tied to rats, mosquitoes, parasites, infected animals, or other living carriers.",
      "Tracing or controlling the living vector is central to understanding the disease."
    ],
    assignmentMode: "hybrid",
    axis: "disease_model",
    category: "affliction",
    conceptId: "carrier_vector",
    currentTag: "carrier_vector",
    description: "Spread through vermin, insects, bites, stings, or other living disease vectors.",
    doesNotApplyWhen: [
      "The disease spreads through air, water, dreams, or contaminated objects without a real living carrier.",
      "Injury transmission happens, but the record does not frame a recurring vector population or host species."
    ],
    family: "epidemiological_profile",
    id: "affliction:carrier_vector",
    label: "carrier_vector",
    translationStatus: "mapped"
  },
  cognitive_impairment: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "affliction",
    conceptId: "cognitive_impairment",
    currentTag: "cognitive_impairment",
    description: "Dulls thought, memory, decision-making, or mental clarity without being just fear or confusion.",
    family: "impact",
    id: "affliction:cognitive_impairment",
    label: "cognitive_impairment",
    translationStatus: "mapped"
  },
  community_outbreak: {
    adjacentTags: [
      "epidemic_pestilence",
      "carrier_vector"
    ],
    appliesWhen: [
      "The affliction is naturally retrieved as a spreading local crisis affecting a community, camp, ship, or institution.",
      "Outbreak management, quarantine pressure, or multi-victim spread matters more than a single infected host."
    ],
    assignmentMode: "hybrid",
    axis: "disease_model",
    category: "affliction",
    conceptId: "community_outbreak",
    currentTag: "community_outbreak",
    description: "Framed around camp-, village-, ship-, monastery-, or settlement-scale spread rather than one isolated victim.",
    doesNotApplyWhen: [
      "The affliction remains a one-target curse or isolated infection without wider spread framing.",
      "The stronger fit is epidemic_pestilence only because of plague flavor, but community-scale spread is not actually present."
    ],
    family: "epidemiological_profile",
    id: "affliction:community_outbreak",
    label: "community_outbreak",
    translationStatus: "mapped"
  },
  compulsion: {
    assignmentMode: "deterministic",
    axis: "behavior",
    category: "affliction",
    conceptId: "compulsion",
    currentTag: "compulsion",
    description: "Overrides agency through commanded behavior, forced truth-telling, or similarly scripted actions.",
    family: "behavioral_override",
    id: "affliction:compulsion",
    label: "compulsion",
    translationStatus: "mapped"
  },
  contact_exposure: {
    assignmentMode: "deterministic",
    axis: "disease_model",
    category: "affliction",
    conceptId: "contact_exposure",
    currentTag: "contact_exposure",
    description: "Spread by touch, skin contact, slime, ooze, or another surface-level transfer mechanism.",
    family: "delivery_profile",
    id: "affliction:contact_exposure",
    label: "contact_exposure",
    translationStatus: "mapped"
  },
  countermagic_resolution: {
    adjacentTags: [
      "antidote_resolution",
      "cursebreaking_resolution"
    ],
    assignmentMode: "hybrid",
    axis: "response",
    category: "affliction",
    conceptId: "active_magic_counteraction",
    currentTag: "countermagic_resolution",
    description: "Naturally retrieved because counteracting, suppressing, or dispelling an active magical affliction is central to solving it.",
    family: "resolution_profile",
    id: "affliction:countermagic_resolution",
    label: "countermagic",
    translationStatus: "mapped"
  },
  cumulative_transformation: {
    adjacentTags: [
      "transformative_corruption",
      "petrifying_corruption"
    ],
    appliesWhen: [
      "The stage-by-stage march toward a changed body is a major reason to retrieve the affliction.",
      "Transformation is progressive and cumulative rather than an instantaneous one-step effect."
    ],
    assignmentMode: "hybrid",
    axis: "disease_model",
    category: "affliction",
    conceptId: "cumulative_transformation",
    currentTag: "cumulative_transformation",
    description: "Affliction whose stages progressively rewrite the victim into a visibly altered or corrupted form.",
    doesNotApplyWhen: [
      "The affliction only imposes one stable transformed state without escalation.",
      "The stronger fit is delayed_onset, terminal_collapse, or a more specific pathogenesis tag."
    ],
    family: "progression_profile",
    id: "affliction:cumulative_transformation",
    label: "cumulative_transformation",
    translationStatus: "mapped"
  },
  cure_clock_urgency: {
    adjacentTags: [
      "terminal_collapse",
      "delayed_onset"
    ],
    assignmentMode: "hybrid",
    axis: "response",
    category: "affliction",
    conceptId: "time_critical_resolution",
    currentTag: "cure_clock_urgency",
    description: "Creates immediate pressure to diagnose and cure the affliction before a fast-moving catastrophic endpoint arrives.",
    family: "response_profile",
    id: "affliction:cure_clock_urgency",
    label: "time_critical_resolution",
    translationStatus: "mapped"
  },
  curse_marking: {
    adjacentTags: [
      "void_soul_corruption",
      "soul_binding"
    ],
    appliesWhen: [
      "The affliction leaves an explicit mark, sign, sigil, stain, or named curse-brand that matters to its identity.",
      "A user would retrieve it for visible or narratively explicit cursed marking, not just soul damage."
    ],
    assignmentMode: "hybrid",
    axis: "metaphysical",
    category: "affliction",
    conceptId: "curse_marking",
    currentTag: "curse_marking",
    description: "Brands the victim with a curse mark, doom sign, inherited hex, or similarly explicit supernatural stigma.",
    doesNotApplyWhen: [
      "The affliction is metaphysical but has no distinct branded or marked stigma.",
      "The stronger fit is void_soul_corruption or soul_binding without a visible curse sign."
    ],
    family: "metaphysical_profile",
    id: "affliction:curse_marking",
    label: "curse_marking",
    translationStatus: "mapped"
  },
  cursebreaking_resolution: {
    adjacentTags: [
      "countermagic_resolution",
      "ritual_appeasement_resolution"
    ],
    assignmentMode: "hybrid",
    axis: "response",
    category: "affliction",
    conceptId: "curse_remediation",
    currentTag: "cursebreaking_resolution",
    description: "Naturally retrieved because lifting a curse, breaking a doom, or ending a supernatural binding is central to recovery.",
    family: "resolution_profile",
    id: "affliction:cursebreaking_resolution",
    label: "curse_remediation",
    translationStatus: "mapped"
  },
  delayed_onset: {
    adjacentTags: [
      "recurrent_flare",
      "cumulative_transformation"
    ],
    appliesWhen: [
      "The affliction is naturally retrieved because the real danger appears after an incubation period, hidden delay, or deceptively quiet initial stage.",
      "The timing gap between exposure and serious consequence matters to prep, diagnosis, or quarantine decisions."
    ],
    assignmentMode: "hybrid",
    axis: "disease_model",
    category: "affliction",
    conceptId: "delayed_onset",
    currentTag: "delayed_onset",
    description: "Affliction whose symptoms, danger, or full transformation emerge after a notable delay rather than immediately.",
    doesNotApplyWhen: [
      "The affliction starts harming victims right away even if it worsens later.",
      "The stronger fit is recurrent_flare or cumulative_transformation because the main hook is cycling or progressive change rather than delayed emergence."
    ],
    family: "progression_profile",
    id: "affliction:delayed_onset",
    label: "delayed_onset",
    translationStatus: "mapped"
  },
  dreamborne_exposure: {
    adjacentTags: [
      "nightmare_torment",
      "inhaled_exposure"
    ],
    appliesWhen: [
      "The affliction is contracted or transmitted through dreams, sleep, shared nightmares, or oneiric contact.",
      "Dream-state exposure is central to how the condition reaches the victim."
    ],
    assignmentMode: "hybrid",
    axis: "disease_model",
    category: "affliction",
    conceptId: "dreamborne_exposure",
    currentTag: "dreamborne_exposure",
    description: "Spread through sleep, nightmares, dreaming contact, or other oneiric pathways.",
    doesNotApplyWhen: [
      "The affliction only causes nightmares after infection but is actually spread by wounds, air, or cursed objects.",
      "The stronger fit is nightmare_torment because the symptom profile matters more than the exposure path."
    ],
    family: "delivery_profile",
    id: "affliction:dreamborne_exposure",
    label: "dreamborne_exposure",
    translationStatus: "mapped"
  },
  epidemic_pestilence: {
    assignmentMode: "deterministic",
    axis: "disease_model",
    category: "affliction",
    conceptId: "epidemic_pestilence",
    currentTag: "epidemic_pestilence",
    description: "A named plague-, fever-, pox-, or pestilence-style disease with explicit outbreak or contagion framing.",
    family: "epidemiological_profile",
    id: "affliction:epidemic_pestilence",
    label: "epidemic_pestilence",
    translationStatus: "mapped"
  },
  exorcism_resolution: {
    adjacentTags: [
      "cursebreaking_resolution",
      "ritual_appeasement_resolution"
    ],
    assignmentMode: "hybrid",
    axis: "response",
    category: "affliction",
    conceptId: "hostile_presence_expulsion",
    currentTag: "exorcism_resolution",
    description: "Naturally retrieved because banishing, cleansing, or spiritually expelling a hostile presence is central to solving the affliction.",
    family: "resolution_profile",
    id: "affliction:exorcism_resolution",
    label: "exorcism",
    translationStatus: "mapped"
  },
  fungal_growth: {
    assignmentMode: "hybrid",
    axis: "disease_model",
    category: "affliction",
    conceptId: "fungal_growth",
    currentTag: "fungal_growth",
    description: "Defined by fungal bloom, spores, mycelial takeover, or mushroom-like growths spreading through the body.",
    family: "pathogenesis",
    id: "affliction:fungal_growth",
    label: "fungal_growth",
    translationStatus: "mapped"
  },
  healing_suppression: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "affliction",
    conceptId: "healing_suppression",
    currentTag: "healing_suppression",
    description: "Prevents normal healing or sharply reduces healing received.",
    family: "impact",
    id: "affliction:healing_suppression",
    label: "healing_suppression",
    translationStatus: "mapped"
  },
  hemorrhagic_failure: {
    adjacentTags: [
      "blood_rot",
      "physical_debilitation"
    ],
    appliesWhen: [
      "The affliction is naturally retrieved because bleeding, blood loss, ruptured vessels, or circulatory collapse are major consequences rather than incidental symptoms.",
      "The bodily failure pattern matters more than the source theme of the disease."
    ],
    assignmentMode: "hybrid",
    axis: "effect",
    category: "affliction",
    conceptId: "hemorrhagic_failure",
    currentTag: "hemorrhagic_failure",
    description: "Causes uncontrolled bleeding, blood loss, vessel rupture, or similar collapse of the body's circulatory integrity.",
    doesNotApplyWhen: [
      "The affliction only references corrupted blood as flavor without major bleeding or circulatory breakdown consequences.",
      "The stronger fit is blood_rot or physical_debilitation because hemorrhage is not the core downstream effect."
    ],
    family: "physiology_override",
    id: "affliction:hemorrhagic_failure",
    label: "hemorrhagic_failure",
    translationStatus: "mapped"
  },
  infestation_implant: {
    assignmentMode: "deterministic",
    axis: "disease_model",
    category: "affliction",
    conceptId: "infestation_implant",
    currentTag: "infestation_implant",
    description: "Defined by eggs, larvae, spores, parasites, or other host-colonizing implantation.",
    family: "pathogenesis",
    id: "affliction:infestation_implant",
    label: "infestation_implant",
    translationStatus: "mapped"
  },
  ingested_exposure: {
    assignmentMode: "deterministic",
    axis: "disease_model",
    category: "affliction",
    conceptId: "ingested_exposure",
    currentTag: "ingested_exposure",
    description: "Spread by swallowing contaminated food, drink, medicine, or another consumed substance.",
    family: "delivery_profile",
    id: "affliction:ingested_exposure",
    label: "ingested_exposure",
    translationStatus: "mapped"
  },
  inhaled_exposure: {
    assignmentMode: "deterministic",
    axis: "disease_model",
    category: "affliction",
    conceptId: "inhaled_exposure",
    currentTag: "inhaled_exposure",
    description: "Spread through smoke, breath, spores, vapor, dust, or another inhaled medium.",
    family: "delivery_profile",
    id: "affliction:inhaled_exposure",
    label: "inhaled_exposure",
    translationStatus: "mapped"
  },
  injury_exposure: {
    assignmentMode: "deterministic",
    axis: "disease_model",
    category: "affliction",
    conceptId: "injury_exposure",
    currentTag: "injury_exposure",
    description: "Spread through bites, stings, punctures, or other blood-entering injury vectors.",
    family: "delivery_profile",
    id: "affliction:injury_exposure",
    label: "injury_exposure",
    translationStatus: "mapped"
  },
  mental_impairment: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "affliction",
    conceptId: "mental_impairment",
    currentTag: "mental_impairment",
    description: "Impairs judgment, emotions, or perception through confusion, fear, or delirium.",
    family: "impact",
    id: "affliction:mental_impairment",
    label: "mental_impairment",
    translationStatus: "mapped"
  },
  mobility_impairment: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "affliction",
    conceptId: "mobility_impairment",
    currentTag: "mobility_impairment",
    description: "Reduces speed, stiffens movement, or leaves the victim immobilized.",
    family: "impact",
    id: "affliction:mobility_impairment",
    label: "mobility_impairment",
    translationStatus: "mapped"
  },
  nightmare_torment: {
    assignmentMode: "deterministic",
    axis: "metaphysical",
    category: "affliction",
    conceptId: "nightmare_torment",
    currentTag: "nightmare_torment",
    description: "Centers on nightmares, dream-torment, or similarly sleep-haunting affliction framing.",
    family: "metaphysical_profile",
    id: "affliction:nightmare_torment",
    label: "nightmare_torment",
    translationStatus: "mapped"
  },
  outbreak_management: {
    adjacentTags: [
      "community_outbreak",
      "quarantine_risk"
    ],
    assignmentMode: "hybrid",
    axis: "response",
    category: "affliction",
    conceptId: "outbreak_containment",
    currentTag: "outbreak_management",
    description: "Naturally retrieved as a disease or curse that creates a wider containment, treatment, and community-management problem rather than only an isolated victim.",
    family: "response_profile",
    id: "affliction:outbreak_management",
    label: "outbreak_containment",
    translationStatus: "mapped"
  },
  petrification: {
    adjacentTags: [
      "petrifying_corruption",
      "mobility_impairment"
    ],
    appliesWhen: [
      "The affliction is naturally retrieved because victims end up petrified, stone-locked, or mineralized.",
      "The end-state of stony immobilization matters more than the thematic cause."
    ],
    assignmentMode: "hybrid",
    axis: "effect",
    category: "affliction",
    conceptId: "petrification",
    currentTag: "petrification",
    description: "Turns flesh to stone or otherwise locks the body into a rigid mineralized state.",
    doesNotApplyWhen: [
      "The affliction only trends toward calcification without actually functioning as a petrifying condition.",
      "The stronger fit is petrifying_corruption because the gradual corruption process is the real hook."
    ],
    family: "physiology_override",
    id: "affliction:petrification",
    label: "petrification",
    translationStatus: "mapped"
  },
  petrifying_corruption: {
    adjacentTags: [
      "petrification",
      "cumulative_transformation"
    ],
    appliesWhen: [
      "The affliction is retrieved for gradual hardening, calcification, or stoneward corruption across stages.",
      "The process of becoming stone-like matters as much as the final condition."
    ],
    assignmentMode: "hybrid",
    axis: "disease_model",
    category: "affliction",
    conceptId: "petrifying_corruption",
    currentTag: "petrifying_corruption",
    description: "Defined by calcification, ossification, stone-turning, or gradual bodily hardening toward an inert form.",
    doesNotApplyWhen: [
      "The affliction chiefly imposes a final petrified state without broader progressive corruption framing.",
      "The stronger fit is transformative_corruption without specifically stoneward identity."
    ],
    family: "pathogenesis",
    id: "affliction:petrifying_corruption",
    label: "petrifying_corruption",
    translationStatus: "mapped"
  },
  physical_debilitation: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "affliction",
    conceptId: "physical_debilitation",
    currentTag: "physical_debilitation",
    description: "Weakens the body through exhaustion, sickness, drained vitality, blood loss, or similar bodily degradation.",
    family: "impact",
    id: "affliction:physical_debilitation",
    label: "physical_debilitation",
    translationStatus: "mapped"
  },
  possession_seed: {
    adjacentTags: [
      "compulsion",
      "soul_binding"
    ],
    appliesWhen: [
      "The affliction prepares a victim to be ridden, entered, replaced, or overtaken by another presence.",
      "A latent invading spirit or takeover-ready foothold is central to the condition's danger."
    ],
    assignmentMode: "hybrid",
    axis: "metaphysical",
    category: "affliction",
    conceptId: "possession_seed",
    currentTag: "possession_seed",
    description: "Plants an invading spirit, hostile presence, or takeover-ready metaphysical foothold inside the victim.",
    doesNotApplyWhen: [
      "The affliction only compels behavior without an actual possessing entity or metaphysical foothold.",
      "The stronger fit is soul_binding or compulsion because takeover is not really part of the disease model."
    ],
    family: "metaphysical_profile",
    id: "affliction:possession_seed",
    label: "possession_seed",
    translationStatus: "mapped"
  },
  quarantine_containment_resolution: {
    adjacentTags: [
      "outbreak_management",
      "quarantine_risk"
    ],
    assignmentMode: "hybrid",
    axis: "response",
    category: "affliction",
    conceptId: "outbreak_containment",
    currentTag: "quarantine_containment_resolution",
    description: "Naturally retrieved because isolation, contact control, and containment are core to preventing further spread while treatment proceeds.",
    family: "resolution_profile",
    id: "affliction:quarantine_containment_resolution",
    label: "outbreak_containment",
    translationStatus: "mapped"
  },
  quarantine_risk: {
    adjacentTags: [
      "community_outbreak",
      "inhaled_exposure"
    ],
    assignmentMode: "hybrid",
    axis: "response",
    category: "affliction",
    conceptId: "outbreak_containment",
    currentTag: "quarantine_risk",
    description: "Creates a strong need to isolate victims, restrict contact, or manage who can safely enter or leave an affected area.",
    family: "response_profile",
    id: "affliction:quarantine_risk",
    label: "outbreak_containment",
    translationStatus: "mapped"
  },
  recurrent_flare: {
    adjacentTags: [
      "delayed_onset",
      "terminal_collapse"
    ],
    appliesWhen: [
      "The affliction is naturally retrieved because symptoms repeatedly subside and then surge back in cycles, attacks, or flare-ups.",
      "Its pacing matters as a recurring problem rather than a single steady decline."
    ],
    assignmentMode: "hybrid",
    axis: "disease_model",
    category: "affliction",
    conceptId: "recurrent_flare",
    currentTag: "recurrent_flare",
    description: "Affliction that subsides and returns in episodes, repeating attacks, or cyclical symptom spikes.",
    doesNotApplyWhen: [
      "The affliction mainly incubates once and then worsens in a straight line.",
      "The stronger fit is delayed_onset or terminal_collapse because recurrence is not the main progression hook."
    ],
    family: "progression_profile",
    id: "affliction:recurrent_flare",
    label: "recurrent_flare",
    translationStatus: "mapped"
  },
  respiratory_impairment: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "affliction",
    conceptId: "respiratory_impairment",
    currentTag: "respiratory_impairment",
    description: "Prevents normal breathing or fills the victim's lungs with water, fluid, or similar suffocating effects.",
    family: "physiology_override",
    id: "affliction:respiratory_impairment",
    label: "respiratory_impairment",
    translationStatus: "mapped"
  },
  ritual_appeasement_resolution: {
    adjacentTags: [
      "cursebreaking_resolution",
      "exorcism_resolution"
    ],
    assignmentMode: "hybrid",
    axis: "response",
    category: "affliction",
    conceptId: "ritual_appeasement",
    currentTag: "ritual_appeasement_resolution",
    description: "Naturally retrieved because restitution, offerings, ritual respect, or meeting a spiritual demand is central to ending the affliction.",
    family: "resolution_profile",
    id: "affliction:ritual_appeasement_resolution",
    label: "ritual_appeasement",
    translationStatus: "mapped"
  },
  rot_decay: {
    assignmentMode: "deterministic",
    axis: "disease_model",
    category: "affliction",
    conceptId: "rot_decay",
    currentTag: "rot_decay",
    description: "Defined by bodily rot, necrosis, blight, mummification, or similar physical decay.",
    family: "pathogenesis",
    id: "affliction:rot_decay",
    label: "rot_decay",
    translationStatus: "mapped"
  },
  sedation: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "affliction",
    conceptId: "sedation",
    currentTag: "sedation",
    description: "Induces sleep, deep drowsiness, trance-like unconsciousness, or difficulty waking.",
    family: "impact",
    id: "affliction:sedation",
    label: "sedation",
    translationStatus: "mapped"
  },
  self_destructive_impulse: {
    assignmentMode: "hybrid",
    axis: "behavior",
    category: "affliction",
    conceptId: "self_destructive_impulse",
    currentTag: "self_destructive_impulse",
    description: "Drives reckless self-harm, suicidal behavior, or dangerous compulsions against the victim's own interests.",
    family: "behavioral_override",
    id: "affliction:self_destructive_impulse",
    label: "self_destructive_impulse",
    translationStatus: "mapped"
  },
  sensory_impairment: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "affliction",
    conceptId: "sensory_impairment",
    currentTag: "sensory_impairment",
    description: "Blinds, deafens, or otherwise suppresses perception and the senses.",
    family: "impact",
    id: "affliction:sensory_impairment",
    label: "sensory_impairment",
    translationStatus: "mapped"
  },
  soul_binding: {
    adjacentTags: [
      "curse_marking",
      "possession_seed"
    ],
    appliesWhen: [
      "The affliction's metaphysical identity depends on the victim's soul being anchored, trapped, pledged, or externally entangled.",
      "A user would retrieve it for ghost anchors, cursed bindings, oath-linked doom, or similar soul-tether effects."
    ],
    assignmentMode: "hybrid",
    axis: "metaphysical",
    category: "affliction",
    conceptId: "soul_binding",
    currentTag: "soul_binding",
    description: "Pins, traps, anchors, or otherwise entangles the victim's soul with an object, oath, place, or hostile power.",
    doesNotApplyWhen: [
      "The affliction only marks, weakens, or spiritually corrupts the victim without actually binding the soul to something.",
      "The stronger fit is curse_marking or possession_seed rather than tethering or anchoring."
    ],
    family: "metaphysical_profile",
    id: "affliction:soul_binding",
    label: "soul_binding",
    translationStatus: "mapped"
  },
  source_cleanup_resolution: {
    adjacentTags: [
      "source_tracing",
      "quarantine_containment_resolution"
    ],
    assignmentMode: "hybrid",
    axis: "response",
    category: "affliction",
    conceptId: "source_cleanup",
    currentTag: "source_cleanup_resolution",
    description: "Naturally retrieved because the contaminated site, cursed source, infected carrier chain, or environmental origin must be found and cleaned up.",
    family: "resolution_profile",
    id: "affliction:source_cleanup_resolution",
    label: "source_cleanup",
    translationStatus: "mapped"
  },
  source_tracing: {
    adjacentTags: [
      "carrier_vector",
      "community_outbreak"
    ],
    assignmentMode: "hybrid",
    axis: "response",
    category: "affliction",
    conceptId: "source_discovery",
    currentTag: "source_tracing",
    description: "Naturally retrieved because finding the contaminated source, carrier chain, cursed origin, or initial spread event is central to solving the problem.",
    family: "response_profile",
    id: "affliction:source_tracing",
    label: "source_discovery",
    translationStatus: "mapped"
  },
  surgical_extraction_resolution: {
    adjacentTags: [
      "source_cleanup_resolution",
      "infestation_implant"
    ],
    assignmentMode: "hybrid",
    axis: "response",
    category: "affliction",
    conceptId: "parasite_removal",
    currentTag: "surgical_extraction_resolution",
    description: "Naturally retrieved because removing eggs, larvae, parasites, implants, or invasive growth from the body is a central answer path.",
    family: "resolution_profile",
    id: "affliction:surgical_extraction_resolution",
    label: "parasite_removal",
    translationStatus: "mapped"
  },
  terminal_collapse: {
    adjacentTags: [
      "cumulative_transformation",
      "physical_debilitation"
    ],
    appliesWhen: [
      "The affliction is naturally retrieved because the later stages end in death, total ruin, or another catastrophic final break point.",
      "The inevitability of a terminal end state matters more than day-to-day impairment or transformation imagery."
    ],
    assignmentMode: "hybrid",
    axis: "disease_model",
    category: "affliction",
    conceptId: "terminal_collapse",
    currentTag: "terminal_collapse",
    description: "Affliction explicitly framed around catastrophic bodily failure, death, or a final ruinous end state if unchecked.",
    doesNotApplyWhen: [
      "The affliction is dangerous but does not build toward a distinct catastrophic endpoint.",
      "The stronger fit is cumulative_transformation or physical_debilitation because ongoing deterioration matters more than terminal collapse."
    ],
    family: "progression_profile",
    id: "affliction:terminal_collapse",
    label: "terminal_collapse",
    translationStatus: "mapped"
  },
  transformative_corruption: {
    assignmentMode: "deterministic",
    axis: "effect",
    category: "affliction",
    conceptId: "transformative_corruption",
    currentTag: "transformative_corruption",
    description: "Progressively transforms the body into crystal, plant matter, fungus, or another corrupted form.",
    family: "physiology_override",
    id: "affliction:transformative_corruption",
    label: "transformative_corruption",
    translationStatus: "mapped"
  },
  truth_compulsion: {
    adjacentTags: [
      "compulsion",
      "self_destructive_impulse"
    ],
    appliesWhen: [
      "The affliction is naturally retrieved because victims are forced to confess, answer honestly, or reveal hidden information.",
      "Involuntary disclosure matters more than broad obedience, mood change, or self-harm."
    ],
    assignmentMode: "hybrid",
    axis: "behavior",
    category: "affliction",
    conceptId: "truth_compulsion",
    currentTag: "truth_compulsion",
    description: "Forces confession, honesty, or involuntary revelation against the victim's will.",
    doesNotApplyWhen: [
      "The affliction only compels action or speech generally without a truth-telling or confession-facing hook.",
      "The stronger fit is compulsion or cognitive_impairment rather than forced honesty."
    ],
    family: "behavioral_override",
    id: "affliction:truth_compulsion",
    label: "truth_compulsion",
    translationStatus: "mapped"
  },
  violence_compulsion: {
    adjacentTags: [
      "compulsion",
      "self_destructive_impulse"
    ],
    appliesWhen: [
      "The affliction is naturally retrieved because it drives victims to attack, maul, murder, or lash out at others.",
      "Violent outward aggression matters more than truthful speech, self-harm, or generic loss of agency."
    ],
    assignmentMode: "hybrid",
    axis: "behavior",
    category: "affliction",
    conceptId: "violence_compulsion",
    currentTag: "violence_compulsion",
    description: "Forces hostile aggression, murderous rage, or other attack-driven loss of self-control.",
    doesNotApplyWhen: [
      "The affliction only makes the victim reckless, confused, or generally compelled without a real violence-forward pattern.",
      "The stronger fit is self_destructive_impulse or compulsion because aggression toward others is not central."
    ],
    family: "behavioral_override",
    id: "affliction:violence_compulsion",
    label: "violence_compulsion",
    translationStatus: "mapped"
  },
  void_soul_corruption: {
    assignmentMode: "deterministic",
    axis: "metaphysical",
    category: "affliction",
    conceptId: "void_soul_corruption",
    currentTag: "void_soul_corruption",
    description: "Attacks life force or the bond between body and soul through void or deathly corruption.",
    family: "metaphysical_profile",
    id: "affliction:void_soul_corruption",
    label: "void_soul_corruption",
    translationStatus: "mapped"
  },
  wasting_hunger: {
    assignmentMode: "hybrid",
    axis: "effect",
    category: "affliction",
    conceptId: "wasting_hunger",
    currentTag: "wasting_hunger",
    description: "Imposes unnatural starvation, ravenous craving, or a consuming metabolic drive that degrades the body over time.",
    family: "physiology_override",
    id: "affliction:wasting_hunger",
    label: "wasting_hunger",
    translationStatus: "mapped"
  },
  waterborne_exposure: {
    adjacentTags: [
      "ingested_exposure",
      "carrier_vector"
    ],
    appliesWhen: [
      "Contaminated water, immersion, flood contact, or cursed liquid exposure is central to how victims contract the affliction.",
      "A user would retrieve it for tainted wells, river contagion, drowning curses, or similar water-linked spread."
    ],
    assignmentMode: "hybrid",
    axis: "disease_model",
    category: "affliction",
    conceptId: "waterborne_exposure",
    currentTag: "waterborne_exposure",
    description: "Spread through tainted water, drowning contact, flood exposure, or cursed immersion.",
    doesNotApplyWhen: [
      "The affliction merely affects aquatic creatures without actually spreading through water contact.",
      "The stronger fit is inhaled_exposure, ingested_exposure, or carrier_vector instead of water-linked transmission."
    ],
    family: "delivery_profile",
    id: "affliction:waterborne_exposure",
    label: "waterborne_exposure",
    translationStatus: "mapped"
  }
};
