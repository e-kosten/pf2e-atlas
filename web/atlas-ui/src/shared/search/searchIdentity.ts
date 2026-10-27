import { AtlasApiError } from "../../api/atlasApi";
import type {
  ResultSort,
  SearchResultsRequest,
  TraitRecordsRequest,
  SimilarityOptions,
  SimilaritySourceSections,
  SearchContentScope,
  SimilaritySeed,
  SimilarityPartsView,
} from "../../generated/atlas";

/** Ordering and paging may retain a labelled prior page; eligibility and source identity may not. */
export function sameSearchPageContext(
  previous: SearchResultsRequest | TraitRecordsRequest,
  next: SearchResultsRequest | TraitRecordsRequest,
): boolean {
  return (
    JSON.stringify({ ...previous, sort: undefined, page: undefined }) ===
    JSON.stringify({ ...next, sort: undefined, page: undefined })
  );
}

export function assertSimilarityIdentity(
  response: {
    similarity?: {
      options: SimilarityOptions;
      source_sections?: SimilaritySourceSections | null;
      search_content?: SearchContentScope;
    };
  },
  options?: SimilarityOptions,
  sourceSections?: SimilaritySourceSections,
  searchContent?: SearchContentScope,
) {
  if (!options) return;
  const key = (value?: SimilarityOptions) =>
    value?.method === "baseline"
      ? JSON.stringify([value.method])
      : value
        ? JSON.stringify([
            value.method,
            value.method === "passages" ? value.decay : null,
            value.candidate_parts,
            value.parts,
          ])
        : null;
  if (
    key(response.similarity?.options) !== key(options) ||
    (sourceSections !== undefined &&
      options.method !== "baseline" &&
      response.similarity?.source_sections !== sourceSections) ||
    (searchContent !== undefined &&
      response.similarity?.search_content !== searchContent)
  )
    throw new Error(
      "The search response used different similarity settings. Retry the search.",
    );
}

/** Never display retrieval evidence under a different model or collection. */
export function assertSearchIdentity(
  response: {
    search_profile?: string;
    artifact_generation: string;
    sort?: ResultSort;
    search_content?: SearchContentScope;
  },
  profile: string | null | undefined,
  generation?: string | null,
  sort?: ResultSort,
  searchContent?: SearchContentScope,
) {
  if (searchContent != null && response.search_content !== searchContent)
    throw new Error(
      "The search response used different search content. Retry the search.",
    );
  if (profile != null && response.search_profile !== profile)
    throw new Error(
      "The search response does not match this request. Retry the search.",
    );
  if (generation != null && response.artifact_generation !== generation)
    throw new AtlasApiError(409, "The reference collection changed. Refresh results.", {
      code: "artifact_generation_changed",
      message: "The reference collection changed. Refresh results.",
      recoverable_action: "refresh_results",
    });
  if (sort != null && !sameResultSort(response.sort, sort))
    throw new Error("The search response used a different order. Retry the search.");
}

/** Generated sort DTOs are values; response object identity is irrelevant. */
export function sameResultSort(
  left: ResultSort | undefined,
  right: ResultSort | undefined,
): boolean {
  if (!left || !right) return left === right;
  if (left.kind !== right.kind) return false;
  if (left.kind === "random")
    return right.kind === "random" && left.seed === right.seed;
  if (left.kind === "attributes")
    return (
      right.kind === "attributes" &&
      left.rules.length === right.rules.length &&
      left.rules.every(
        (rule, index) =>
          rule.field === right.rules[index].field &&
          rule.direction === right.rules[index].direction,
      )
    );
  return true;
}

/** Source presets are resolved only from the requested seed/profile/generation. */
export function assertSimilarityInventoryIdentity(
  inventory: SimilarityPartsView,
  seed: SimilaritySeed,
  profile: string,
  generation?: string | null,
) {
  if (
    JSON.stringify(inventory.seed) !== JSON.stringify(seed) ||
    inventory.search_profile !== profile
  )
    throw new Error("The comparison sections do not match this source. Retry.");
  if (generation && inventory.artifact_generation !== generation)
    throw new AtlasApiError(409, "The reference collection changed.", {
      code: "artifact_generation_changed",
      message: "The reference collection changed.",
      recoverable_action: "refresh_results",
    });
}
