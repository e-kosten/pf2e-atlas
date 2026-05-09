import { defineFacetConcepts, mergeCanonicalConceptSeeds, type FacetlessConceptSeed } from "../../builders.js";
import { CANONICAL_FACETS } from "../../facets.js";

const effectProfileSeeds: Record<string, FacetlessConceptSeed> = {
  action_denial: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  cognitive_impairment: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  curse_marking: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  healing_suppression: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  hemorrhagic_failure: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  mental_impairment: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  mobility_impairment: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  nightmare_torment: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  petrification: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  physical_debilitation: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  possession_seed: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  respiratory_impairment: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  sedation: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  sensory_impairment: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  silencing: {},
  soul_binding: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  transformative_corruption: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  void_soul_corruption: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
  wasting_hunger: {
    text: {
      definition: "Effect/state descriptors.",
    },
  },
};

export const effectSeedsByKind = mergeCanonicalConceptSeeds([
  defineFacetConcepts(
    CANONICAL_FACETS.EFFECT.EFFECT_PROFILE,
    effectProfileSeeds,
  ),
]);
