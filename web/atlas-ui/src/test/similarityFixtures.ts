import type {
  SimilarityConfigurationView,
  SimilarityOptions,
  SimilarityPartsView,
} from "../generated/atlas";

export function similarityConfigurationFixture(
  options: SimilarityOptions,
): SimilarityConfigurationView {
  return {
    options,
    source_sections:
      options.method === "baseline" ? null : options.parts ? "custom" : "all",
    search_content: "all",
    candidate_identity: options.method === "baseline",
    core_recipe: null,
    resolved_selectors: options.method === "baseline" ? [] : (options.parts ?? []),
    core: null,
    recipe: "roles/v1",
    role_recipe: "typed-content-correspondence/v1",
    diagnostics: {
      seed_units: 1,
      knn_queries: 1,
      candidate_vectors: 1,
      pairwise_comparisons: 1,
      undefined_candidate_centroids: 0,
    },
  };
}

export function similarityPartsFixture(
  artifact_generation = "generation-b",
): SimilarityPartsView {
  return {
    seed: { kind: "record", record_key: "hazards:brown-mold" },
    artifact_generation,
    search_profile: "bge-base",
    role_recipe: "roles/v1",
    supports_corresponding_parts: true,
    core: {
      family: "hazard",
      selectors: ["description-b"],
      representation: "identity_and_prose",
      fallback_reason: null,
      recipe: "core-sections/v1",
    },
    parts: [
      {
        kind: "section",
        selector: "description-b",
        label: "Description",
        owner_label: "Brown Mold",
        owners: [],
        field: "/system/details/description",
        section_ordinal: 0,
        role: "description",
        role_label: "Description",
        has_vectors: true,
        label_only: false,
        navigation: null,
      },
      {
        kind: "section",
        selector: "reset-b",
        label: "Reset",
        owner_label: "Brown Mold",
        owners: [],
        field: "/system/details/reset",
        section_ordinal: 0,
        role: "reset",
        role_label: "Reset",
        has_vectors: true,
        label_only: false,
        navigation: null,
      },
    ],
  };
}
