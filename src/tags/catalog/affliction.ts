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
];
