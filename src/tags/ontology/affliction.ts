import type { DerivedTagOntologyFamily, DerivedTagOntologyTag } from "../../types.js";

export const AFFLICTION_DERIVED_TAG_ONTOLOGY_FAMILIES: DerivedTagOntologyFamily[] = [
  {
    category: "affliction",
    family: "impact",
    description: "Affliction impact tags for practical downstream consequences."
  },
  {
    category: "affliction",
    family: "pathogenesis",
    description: "Affliction tags for recurring infection and bodily-transformation patterns."
  },
  {
    category: "affliction",
    family: "epidemiological_profile",
    description: "Affliction tags for widespread disease patterns and contagion-facing retrieval."
  },
  {
    category: "affliction",
    family: "behavioral_override",
    description: "Affliction tags for forced behavior and explicit agency override."
  },
  {
    category: "affliction",
    family: "physiology_override",
    description: "Affliction tags for forced breathing failure and body-changing corruption."
  },
  {
    category: "affliction",
    family: "metaphysical_profile",
    description: "Affliction tags for soul-straining corruption and nightmare-facing torment."
  }
];

export const AFFLICTION_DERIVED_TAG_ONTOLOGY_TAGS: DerivedTagOntologyTag[] = [
  {
    category: "affliction",
    family: "impact",
    tag: "mental_impairment",
    description: "Impairs judgment, emotions, or perception through confusion, fear, or delirium.",
    assignmentMode: "deterministic"
  },
  {
    category: "affliction",
    family: "impact",
    tag: "mobility_impairment",
    description: "Reduces speed, stiffens movement, or leaves the victim immobilized.",
    assignmentMode: "deterministic"
  },
  {
    category: "affliction",
    family: "impact",
    tag: "physical_debilitation",
    description: "Weakens the body through exhaustion, sickness, drained vitality, blood loss, or similar bodily degradation.",
    assignmentMode: "deterministic"
  },
  {
    category: "affliction",
    family: "impact",
    tag: "healing_suppression",
    description: "Prevents normal healing or sharply reduces healing received.",
    assignmentMode: "deterministic"
  },
  {
    category: "affliction",
    family: "impact",
    tag: "cognitive_impairment",
    description: "Dulls thought, memory, decision-making, or mental clarity without being just fear or confusion.",
    assignmentMode: "deterministic"
  },
  {
    category: "affliction",
    family: "impact",
    tag: "sensory_impairment",
    description: "Blinds, deafens, or otherwise suppresses perception and the senses.",
    assignmentMode: "deterministic"
  },
  {
    category: "affliction",
    family: "impact",
    tag: "sedation",
    description: "Induces sleep, deep drowsiness, trance-like unconsciousness, or difficulty waking.",
    assignmentMode: "deterministic"
  },
  {
    category: "affliction",
    family: "pathogenesis",
    tag: "rot_decay",
    description: "Defined by bodily rot, necrosis, blight, mummification, or similar physical decay.",
    assignmentMode: "deterministic"
  },
  {
    category: "affliction",
    family: "pathogenesis",
    tag: "infestation_implant",
    description: "Defined by eggs, larvae, spores, parasites, or other host-colonizing implantation.",
    assignmentMode: "deterministic"
  },
  {
    category: "affliction",
    family: "epidemiological_profile",
    tag: "epidemic_pestilence",
    description: "A named plague-, fever-, pox-, or pestilence-style disease with explicit outbreak or contagion framing.",
    assignmentMode: "deterministic"
  },
  {
    category: "affliction",
    family: "behavioral_override",
    tag: "compulsion",
    description: "Overrides agency through commanded behavior, forced truth-telling, or similarly scripted actions.",
    assignmentMode: "deterministic"
  },
  {
    category: "affliction",
    family: "physiology_override",
    tag: "respiratory_impairment",
    description: "Prevents normal breathing or fills the victim's lungs with water, fluid, or similar suffocating effects.",
    assignmentMode: "deterministic"
  },
  {
    category: "affliction",
    family: "physiology_override",
    tag: "transformative_corruption",
    description: "Progressively transforms the body into crystal, plant matter, fungus, or another corrupted form.",
    assignmentMode: "deterministic"
  },
  {
    category: "affliction",
    family: "metaphysical_profile",
    tag: "void_soul_corruption",
    description: "Attacks life force or the bond between body and soul through void or deathly corruption.",
    assignmentMode: "deterministic"
  },
  {
    category: "affliction",
    family: "metaphysical_profile",
    tag: "nightmare_torment",
    description: "Centers on nightmares, dream-torment, or similarly sleep-haunting affliction framing.",
    assignmentMode: "deterministic"
  }
];
