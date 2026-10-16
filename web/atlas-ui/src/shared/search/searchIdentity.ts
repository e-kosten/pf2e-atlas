import { AtlasApiError } from "../../api/atlasApi";

/** Never display retrieval evidence under a different model or collection. */
export function assertSearchIdentity(
  response: { search_profile: string; artifact_generation: string },
  profile: string | null | undefined,
  generation?: string | null,
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
}
