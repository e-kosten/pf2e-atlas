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
  quietEmpty = false,
  enabled = true,
}: {
  request: RemasterRelationshipsRequest;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
  quietEmpty?: boolean;
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
          "The reference collection changed. Refresh results.",
          {
            code: "artifact_generation_changed",
            message: "The reference collection changed. Refresh results.",
            recoverable_action: "refresh_results",
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
            {isArtifactGenerationChanged(query.error) ? "Refresh results" : "Retry"}
          </Button>
        }
      />
    );
  const data = query.data;
  if (!data) return null;
  if (data.relationships.kind === "empty" && !data.source_remaster_context)
    return quietEmpty ? null : <p>No remaster information found for this entry.</p>;
  return (
    <section
      className="remaster-query-context"
      aria-label="Source statements about this term or entry"
    >
      {data.subject.kind === "term" && (
        <p className="remaster-information__context">{data.subject.scope_label}</p>
      )}
      <RemasterInformation
        compact={data.subject.kind === "term"}
        category={data.relationships}
        onReference={onReference}
        onReload={() => void query.refetch()}
      />
      {data.subject.kind === "term" && data.relationships.kind === "content" && (
        <details className="remaster-information__evidence">
          <summary>View source statements</summary>
          <RemasterInformation
            category={data.relationships}
            onReference={onReference}
          />
        </details>
      )}
      <SourceRemasterInformation
        category={data.source_remaster_context}
        onReference={onReference}
        onReload={() => void query.refetch()}
      />
    </section>
  );
}
