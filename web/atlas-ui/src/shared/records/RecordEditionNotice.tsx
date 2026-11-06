import type { RecordDetailView } from "../../generated/atlas";
import type { RecordReferenceHandler } from "./PreparedContent";
import { RemasterInformation, SourceRemasterInformation } from "./RemasterInformation";

/** Rust projects applicable context for this exact selected document. */
export function RecordEditionNotice({
  detail,
  onReference,
  onReload,
}: {
  detail: RecordDetailView;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
}) {
  const direct = detail.remaster_relationships;
  const source = detail.source_remaster_context;
  if (
    (direct.kind === "empty" ||
      (direct.kind === "content" && !direct.content.relationships.length)) &&
    (!source ||
      source.kind === "empty" ||
      (source.kind === "content" && source.content.relationships.kind === "empty"))
  )
    return null;
  return (
    <div className="record-edition-notice">
      <RemasterInformation
        compact
        category={detail.remaster_relationships}
        onReference={onReference}
        onReload={onReload}
      />
      <SourceRemasterInformation
        compact
        category={detail.source_remaster_context}
        onReference={onReference}
        onReload={onReload}
      />
    </div>
  );
}
