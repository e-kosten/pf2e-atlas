import type { DerivedTagCanonicalConcept } from "../../../domain/derived-tag-types.js";

export const DERIVED_TAG_AGGREGATE_CANONICAL_CONCEPTS_BY_ID: Record<string, DerivedTagCanonicalConcept> = 
{
  breaching: {
    domainId: "barrier",
    id: "breaching",
    label: "breaching",
    operation: "breach",
    schemaKind: "aggregate",
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces."
    }
  },
  communication: {
    id: "communication",
    label: "communication",
    primaryFacetKind: "capability",
    primaryFacetValue: "capability",
    schemaKind: "aggregate"
  },
  consultation: {
    id: "consultation",
    label: "consultation",
    primaryFacetKind: "capability",
    primaryFacetValue: "capability",
    schemaKind: "aggregate"
  },
  cosmic_framework_setting: {
    id: "cosmic_framework_setting",
    label: "cosmic_framework_setting",
    primaryFacetKind: "setting",
    primaryFacetValue: "planar",
    schemaKind: "aggregate"
  },
  displacement_application: {
    domainId: "displacement",
    id: "displacement_application",
    label: "displacement_application",
    operation: "apply",
    schemaKind: "aggregate",
    text: {
      definition: "Hazard effect and countermeasure tags."
    }
  },
  elemental_plane_setting: {
    id: "elemental_plane_setting",
    label: "elemental_plane_setting",
    primaryFacetKind: "setting",
    primaryFacetValue: "planar",
    schemaKind: "aggregate"
  },
  environmental_application: {
    domainId: "environmental",
    id: "environmental_application",
    label: "environmental_application",
    operation: "apply",
    schemaKind: "aggregate",
    text: {
      definition: "Hazard effect and countermeasure tags."
    }
  },
  expedition: {
    id: "expedition",
    label: "expedition",
    primaryFacetKind: "capability",
    primaryFacetValue: "capability",
    schemaKind: "aggregate"
  },
  guarding_hazard: {
    id: "guarding_hazard",
    label: "guarding_hazard",
    primaryFacetKind: "function",
    primaryFacetValue: "hazard_function",
    schemaKind: "aggregate"
  },
  lower_plane_setting: {
    id: "lower_plane_setting",
    label: "lower_plane_setting",
    primaryFacetKind: "setting",
    primaryFacetValue: "planar",
    schemaKind: "aggregate"
  },
  movement_traversal: {
    id: "movement_traversal",
    label: "movement_traversal",
    primaryFacetKind: "capability",
    primaryFacetValue: "capability",
    schemaKind: "aggregate"
  },
  perception_application: {
    domainId: "perception",
    id: "perception_application",
    label: "perception_application",
    operation: "apply",
    schemaKind: "aggregate",
    text: {
      definition: "Hazard effect and countermeasure tags."
    }
  },
  problem_discovery: {
    domainId: "problem",
    id: "problem_discovery",
    label: "problem_discovery",
    operation: "discover",
    schemaKind: "aggregate",
    text: {
      definition: "Discovery-side spell tags normalize as operational discover concepts."
    }
  },
  problem_resolution: {
    domainId: "problem",
    id: "problem_resolution",
    label: "problem_resolution",
    operation: "resolve",
    schemaKind: "aggregate",
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces."
    }
  },
  reconnaissance: {
    id: "reconnaissance",
    label: "reconnaissance",
    primaryFacetKind: "capability",
    primaryFacetValue: "capability",
    schemaKind: "aggregate"
  },
  security: {
    id: "security",
    label: "security",
    primaryFacetKind: "capability",
    primaryFacetValue: "capability",
    schemaKind: "aggregate"
  },
  transformation: {
    domainId: "form",
    id: "transformation",
    label: "transformation",
    operation: "transform",
    schemaKind: "aggregate",
    text: {
      definition: "Operational spell effects or answer paths."
    }
  },
  upper_plane_setting: {
    id: "upper_plane_setting",
    label: "upper_plane_setting",
    primaryFacetKind: "setting",
    primaryFacetValue: "planar",
    schemaKind: "aggregate"
  },
  wayfinding: {
    id: "wayfinding",
    label: "wayfinding",
    primaryFacetKind: "capability",
    primaryFacetValue: "capability",
    schemaKind: "aggregate"
  }
};
