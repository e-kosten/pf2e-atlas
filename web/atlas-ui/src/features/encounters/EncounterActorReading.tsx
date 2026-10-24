import type { ComponentProps } from "react";
import { Button, Space, Spin } from "antd";
import { useQuery } from "@tanstack/react-query";
import { getEmbeddedReading } from "../../api/atlasApi";
import { RecordPresentation } from "../../shared/records/RecordPresentation";
import { isArtifactGenerationChanged } from "../../shared/queries/immutableReferenceQueries";
import {
  useReferenceQueryRetirement,
  useRetainedRecord,
} from "../../shared/records/useRetainedRecord";

/** Read authored text separately; never replace effective encounter facts. */
export function EncounterActorReading(
  props: ComponentProps<typeof RecordPresentation>,
) {
  const { reading, presentation, relationships = [] } = props;
  const identity = JSON.stringify([
    reading?.parent,
    reading?.generation,
    "actor_content",
  ]);
  const queryKey = ["embedded-reading", identity];
  const enabled =
    Boolean(reading) &&
    (presentation.body.kind === "creature" || presentation.body.kind === "hazard");
  const query = useQuery({
    queryKey,
    enabled,
    queryFn: ({ signal }) =>
      getEmbeddedReading(
        {
          parent: reading!.parent,
          artifact_generation: reading!.generation,
          selection: { kind: "actor_content" },
        },
        signal,
      ),
    staleTime: Infinity,
    retry: false,
    refetchOnWindowFocus: false,
  });
  const retirement = useReferenceQueryRetirement(queryKey);
  const data = useRetainedRecord(
    JSON.stringify([identity, retirement]),
    query.data,
    query.error,
  );
  const entry = enabled ? data?.entries[0] : undefined;
  const changed = isArtifactGenerationChanged(query.error);
  return (
    <div className="encounter-actor-reading">
      {enabled && query.isFetching && (
        <div className="detail-refresh" aria-label="Loading rules text">
          <Spin size="small" delay={500} />
        </div>
      )}
      {enabled && query.isError && (
        <Space className="encounter-actor-reading__status" role="status" wrap>
          <span>
            {changed
              ? "Reference changed. Reload the encounter."
              : entry
                ? "Could not update rules text. Previous text shown."
                : "Rules text unavailable."}
          </span>
          <Button
            size="small"
            onClick={() => {
              if (changed) {
                if (reading?.reloadParent) reading.reloadParent();
                else window.location.reload();
              } else void query.refetch();
            }}
          >
            {changed ? "Reload encounter" : "Retry"}
          </Button>
        </Space>
      )}
      <RecordPresentation
        {...props}
        presentation={
          entry ? { ...presentation, content: entry.content } : presentation
        }
        relationships={
          entry ? [...relationships, ...entry.relationships] : relationships
        }
      />
    </div>
  );
}
