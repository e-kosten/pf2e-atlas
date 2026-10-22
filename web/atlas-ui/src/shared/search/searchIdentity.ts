import { AtlasApiError } from "../../api/atlasApi";
import type {
  ResultSort,
  SearchResultsRequest,
  TraitRecordsRequest,
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

/** Never display retrieval evidence under a different model or collection. */
export function assertSearchIdentity(
  response: { search_profile?: string; artifact_generation: string; sort?: ResultSort },
  profile: string | null | undefined,
  generation?: string | null,
  sort?: ResultSort,
) {
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
