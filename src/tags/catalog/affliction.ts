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
    ],
  },
];
