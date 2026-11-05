import { Alert, Button, Space, Typography } from "antd";
import type { RecordDetailView, RecordSummaryView } from "../../generated/atlas";
import { recordDestination } from "../../app/destinations";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { ErrorNotice } from "../feedback/ErrorNotice";
import type { RecordReferenceHandler } from "./PreparedContent";

type EditionNoticeProps = {
  detail: RecordDetailView;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
};

/** Verified counterpart navigation belongs in the reading flow, above the rules. */
export function RecordEditionNotice({
  detail,
  onReference,
  onReload,
}: EditionNoticeProps) {
  if (detail.selected.embedded_path.length > 0) return null;
  const counterparts = detail.counterparts;
  if (counterparts?.kind === "error" || counterparts?.kind === "unavailable")
    return (
      <ErrorNotice
        compact
        error={counterparts.kind === "error" ? counterparts.error : undefined}
        title="Edition links unavailable"
        description={
          counterparts.kind === "unavailable" ? counterparts.reason : undefined
        }
        action={
          onReload ? <Button onClick={onReload}>Reload record</Button> : undefined
        }
      />
    );
  if (counterparts?.kind !== "content") return null;
  const key = detail.selected.record_key;
  const links = new Map<string, { label: string; record: RecordSummaryView }>();
  for (const pair of counterparts.content.links) {
    const legacy = pair.legacy_record.record_key === key;
    const record = legacy ? pair.remaster_record : pair.legacy_record;
    links.set(record.record_key, {
      label: legacy ? "Remaster version" : "Legacy version",
      record,
    });
  }
  if (!links.size) return null;
  return (
    <Alert
      className="record-edition-notice"
      type="info"
      title={
        <Space orientation="vertical" size={4}>
          {[...links.values()].map(({ label, record }) => (
            <span key={record.record_key}>
              {label}:{" "}
              <Typography.Link
                href={destinationHref(
                  recordDestination({ recordKey: record.record_key }),
                )}
                onClick={(event) => {
                  if (!isPrimaryActivation(event)) return;
                  event.preventDefault();
                  onReference(record.record_key, event.currentTarget);
                }}
              >
                {record.title}
              </Typography.Link>
            </span>
          ))}
        </Space>
      }
    />
  );
}
