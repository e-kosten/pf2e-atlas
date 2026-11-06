import { useQuery } from "@tanstack/react-query";
import { AtlasApiError, getDefinitionDetail } from "../../api/atlasApi";
import { referenceQueryMeta } from "../queries/referenceQueries";
import { useReferenceQueryRetirement } from "../queries/useReferenceQueryRetirement";
import { useRetainedRecord } from "./useRetainedRecord";
import type { RecordNavigationView, ReferenceHelpView } from "../../generated/atlas";
import { recordNavigationIdentity } from "./recordNavigationIdentity";

export function useDefinitionDetail(key: string | null, generation?: string | null) {
  const queryKey = ["definition-detail", key, generation ?? null];
  const query = useQuery({
    queryKey,
    meta: referenceQueryMeta,
    enabled: key !== null,
    queryFn: async ({ signal }) => {
      const detail = await getDefinitionDetail(
        { definition_key: key!, expected_artifact_generation: generation ?? null },
        signal,
      );
      signal.throwIfAborted();
      if (generation != null && detail.artifact_generation !== generation)
        throw new AtlasApiError(409, "The reference collection changed.", {
          code: "artifact_generation_changed",
          message: "The reference collection changed.",
          recoverable_action: "refresh_results",
        });
      return detail;
    },
  });
  const retirement = useReferenceQueryRetirement(queryKey);
  const data = useRetainedRecord(
    JSON.stringify([key, generation, retirement]),
    query.data,
    query.error,
  );
  return { ...query, data };
}

/** Actions are recovered only through the current backend's exact destination. */
export function definitionHelpMatchesRecord(
  help: ReferenceHelpView,
  navigation: RecordNavigationView,
  readingContext?: import("../../generated/atlas").RecordReadingContextView,
): boolean {
  return (
    help.destination?.kind === "record" &&
    help.destination.reading_context === readingContext &&
    recordNavigationIdentity(
      help.destination.navigation.record_key,
      help.destination.navigation,
    ) === recordNavigationIdentity(navigation.record_key, navigation)
  );
}
