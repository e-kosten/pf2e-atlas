import { Alert } from "antd";
import type { TraitDetailView } from "../../generated/atlas";
import { PreparedContentBody, type RecordReferenceHandler } from "./PreparedContent";

/** The same source-backed definition appears on trait pages and in references. */
export function TraitDefinition({
  detail,
  onReference,
}: {
  detail: TraitDetailView;
  onReference: RecordReferenceHandler;
}) {
  return detail.content ? (
    <PreparedContentBody
      body={detail.content.body}
      references={detail.content.references}
      onReference={onReference}
    />
  ) : (
    <Alert
      type="info"
      title="No definition is available in this reference collection."
    />
  );
}
