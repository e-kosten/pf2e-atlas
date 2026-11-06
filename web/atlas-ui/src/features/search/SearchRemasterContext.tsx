import { useReferencePreview } from "../../shared/records/ReferencePreviewContext";
import { RemasterQueryInformation } from "../../shared/records/RemasterQueryInformation";

/** This source context belongs to the committed text, independently of record filters/counts. */
export function SearchRemasterContext({
  query,
  generation,
  onReload,
}: {
  query: string;
  generation?: string;
  onReload: () => void;
}) {
  const preview = useReferencePreview();
  if (!preview || !query.trim() || [...query].length > 256) return null;
  return (
    <RemasterQueryInformation
      request={{
        subject: { kind: "term", term: query },
        expected_artifact_generation: generation,
        limit: 20,
      }}
      onReference={preview.open}
      onReload={onReload}
      quietEmpty
    />
  );
}
