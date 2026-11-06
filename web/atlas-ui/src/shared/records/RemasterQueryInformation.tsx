import { useQuery } from "@tanstack/react-query";
import { Button } from "antd";
import { AtlasApiError, getRemasterRelationships } from "../../api/atlasApi";
import type { RemasterRelationshipsRequest } from "../../generated/atlas";
import { ErrorNotice } from "../feedback/ErrorNotice";
import {
  isArtifactGenerationChanged,
  referenceQueryMeta,
} from "../queries/referenceQueries";
import { LoadingStatus } from "../ui/LoadingStatus";
import { RemasterInformation, SourceRemasterInformation } from "./RemasterInformation";
import type { RecordReferenceHandler } from "./PreparedContent";

/** An explicit information request has its own identity and never resolves a record. */
export function RemasterQueryInformation({
  request,
  onReference,
  onReload,
  enabled = true,
}: {
  request: RemasterRelationshipsRequest;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
  enabled?: boolean;
}) {
  const query = useQuery({
    queryKey: ["remaster-information", request],
    enabled,
    meta: referenceQueryMeta,
    staleTime: Infinity,
    retryOnMount: false,
    queryFn: async ({ signal }) => {
      const response = await getRemasterRelationships(request, signal);
      if (
        request.expected_artifact_generation &&
        response.artifact_generation !== request.expected_artifact_generation
      )
        throw new AtlasApiError(
          409,
          "The reference collection changed. Reload information.",
          {
            code: "artifact_generation_changed",
            message: "The reference collection changed. Reload information.",
            recoverable_action: "reload_record",
          },
        );
      return response;
    },
  });
  if (!enabled) return null;
  if (query.isPending) return <LoadingStatus label="Checking remaster information…" />;
  if (query.error)
    return (
      <ErrorNotice
        compact
        error={query.error}
        title="Remaster information could not be checked"
        action={
          <Button
            size="small"
            onClick={
              isArtifactGenerationChanged(query.error) && onReload
                ? onReload
                : () => void query.refetch()
            }
          >
            {isArtifactGenerationChanged(query.error) ? "Reload information" : "Retry"}
          </Button>
        }
      />
    );
  const data = query.data;
  if (!data) return null;
  if (data.relationships.kind === "empty" && !data.source_remaster_context)
    return <p>No remaster information found for this entry.</p>;
  return (
    <section className="remaster-query-context" aria-label="Remaster information">
      <RemasterInformation
        category={data.relationships}
        onReference={onReference}
        onReload={() => void query.refetch()}
      />
      <SourceRemasterInformation
        category={data.source_remaster_context}
        onReference={onReference}
        onReload={() => void query.refetch()}
      />
    </section>
  );
}
