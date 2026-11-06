import { queryOptions } from "@tanstack/react-query";
import { AtlasApiError, getDefinitionFilterOptions } from "../../api/atlasApi";
import type { QueryDefinitionReferenceTarget } from "../../generated/atlas";
import { referenceQueryMeta } from "../queries/referenceQueries";
import { sameReferenceCondition } from "./linkPredicates";

/** Exact selected metadata uses the same owner for loaded filters and record help. */
export function definitionFilterSelectionQueryOptions(
  target: QueryDefinitionReferenceTarget | undefined,
  generation: string | undefined,
) {
  return queryOptions({
    queryKey: ["definition-filter-selection", generation, target],
    enabled: !!target && !!generation,
    meta: referenceQueryMeta,
    staleTime: Infinity,
    queryFn: async ({ signal }) => {
      const response = await getDefinitionFilterOptions(
        {
          text: null,
          offset: 0,
          limit: 1,
          selected: target ? [target] : [],
          expected_artifact_generation: generation ?? null,
        },
        signal,
      );
      signal.throwIfAborted();
      if (response.artifact_generation !== generation)
        throw new AtlasApiError(409, "Reference collection changed", {
          code: "artifact_generation_changed",
          message: "Reference collection changed",
          recoverable_action: "refresh_results",
        });
      if (
        target &&
        response.selected.some(
          (entry) =>
            !sameReferenceCondition(
              { kind: "definition", reference: { target, include_contents: true } },
              {
                kind: "definition",
                reference: { target: entry.target, include_contents: true },
              },
            ),
        )
      )
        throw new Error("Unable to verify selected definition");
      return response;
    },
  });
}
