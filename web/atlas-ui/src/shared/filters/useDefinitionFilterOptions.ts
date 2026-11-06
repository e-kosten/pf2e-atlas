import { useQuery } from "@tanstack/react-query";
import { AtlasApiError, getDefinitionFilterOptions } from "../../api/atlasApi";
import type {
  DefinitionFilterOptionsView,
  QueryDefinitionReferenceTarget,
} from "../../generated/atlas";
import { referenceQueryMeta } from "../queries/referenceQueries";
import { sameReferenceCondition } from "./linkPredicates";

export const DEFINITION_PAGE_SIZE = 40;
export const definitionTargetIdentity = (target: QueryDefinitionReferenceTarget) =>
  JSON.stringify(target);

/** Artifact vocabulary is independent of contextual field values and counts. */
export function useDefinitionFilterOptions({
  generation,
  text,
  offset,
  selected,
  enabled,
}: {
  generation?: string;
  text: string;
  offset: number;
  selected: QueryDefinitionReferenceTarget[];
  enabled: boolean;
}) {
  return useQuery<DefinitionFilterOptionsView & { offset: number }>({
    queryKey: ["definition-filter-options", generation, text, offset, selected],
    meta: referenceQueryMeta,
    enabled: enabled && !!generation,
    staleTime: Infinity,
    placeholderData: (previous, previousQuery) =>
      previousQuery?.queryKey[1] === generation && previousQuery?.queryKey[2] === text
        ? previous
        : undefined,
    queryFn: async ({ signal }) => {
      const response = await getDefinitionFilterOptions(
        {
          text: text.trim() || null,
          offset,
          limit: DEFINITION_PAGE_SIZE,
          selected,
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
        response.selected.length !== selected.length ||
        selected.some(
          (target) =>
            response.selected.filter((option) =>
              sameReferenceCondition(
                { kind: "definition", reference: { target, include_contents: true } },
                {
                  kind: "definition",
                  reference: { target: option.target, include_contents: true },
                },
              ),
            ).length !== 1,
        )
      )
        throw new Error("Unable to verify selected definition");
      return { ...response, offset };
    },
  });
}
