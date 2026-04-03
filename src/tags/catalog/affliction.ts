import { DerivedTagCatalogEntry } from "../../types.js";

export const AFFLICTION_DERIVED_TAG_CATALOG: DerivedTagCatalogEntry[] = [
  {
    category: "affliction",
    family: "impact",
    description: "Affliction impact tags for practical downstream consequences.",
    tags: [
      { value: "mental_impairment", description: "Impairs judgment, emotions, or perception through confusion, fear, or delirium." },
      { value: "mobility_impairment", description: "Reduces speed, stiffens movement, or leaves the victim immobilized." },
      { value: "physical_debilitation", description: "Weakens the body through exhaustion, sickness, drained vitality, or similar bodily degradation." },
      { value: "healing_suppression", description: "Prevents normal healing or sharply reduces healing received." },
      { value: "cognitive_impairment", description: "Dulls thought, memory, decision-making, or mental clarity without being just fear or confusion." },
      { value: "sensory_impairment", description: "Blinds, deafens, or otherwise suppresses perception and the senses." },
      { value: "sedation", description: "Induces sleep, deep drowsiness, unconsciousness, or difficulty waking." },
    ],
  },
  {
    category: "affliction",
    family: "pathogenesis",
    description: "Affliction tags for recurring infection and bodily-transformation patterns.",
    tags: [
      { value: "rot_decay", description: "Defined by bodily rot, necrosis, blight, mummification, or similar physical decay." },
      { value: "infestation_implant", description: "Defined by eggs, larvae, spores, parasites, or other host-colonizing implantation." },
    ],
  },
  {
    category: "affliction",
    family: "behavioral_override",
    description: "Affliction tags for forced behavior and explicit agency override.",
    tags: [
      { value: "compulsion", description: "Overrides agency through commanded behavior, forced truth-telling, or similarly scripted actions." },
    ],
  },
  {
    category: "affliction",
    family: "physiology_override",
    description: "Affliction tags for forced breathing failure and body-changing corruption.",
    tags: [
      { value: "respiratory_impairment", description: "Prevents normal breathing or fills the victim's lungs with water, fluid, or similar suffocating effects." },
      { value: "transformative_corruption", description: "Progressively transforms the body into crystal, plant matter, or another corrupted form." },
    ],
  },
];
