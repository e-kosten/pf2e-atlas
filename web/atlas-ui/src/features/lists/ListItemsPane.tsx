import { Alert, Empty, Spin, theme } from "antd";
import { Trash2 } from "lucide-react";
import type { SavedListItemView } from "../../generated/atlas";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { RecordBrowseSummary } from "../../shared/records/RecordBrowseSummary";

export function ListItemsPane({
  error,
  items,
  loading,
  onRemove,
  onSelect,
  removingKey,
  selectedRecordKey,
  filtered = false,
  unavailable = false,
}: {
  error?: Error | null;
  items: SavedListItemView[];
  loading: boolean;
  onRemove: (recordKey: string) => void | Promise<unknown>;
  onSelect: (recordKey: string) => void;
  removingKey: string | null;
  selectedRecordKey: string | null;
  filtered?: boolean;
  unavailable?: boolean;
}) {
  const { token } = theme.useToken();
  return (
    <section aria-label="List items">
      {error && (
        <Alert
          type="error"
          showIcon
          title="Unable to remove record"
          description={`${error.message} Use the record’s Remove button to try again.`}
        />
      )}
      {loading && items.length > 0 && (
        <div className="detail-status" role="status">
          <Spin size="small" /> Updating saved records…
        </div>
      )}
      {loading && !items.length ? (
        <Spin className="reference-loading" aria-label="Loading saved records" />
      ) : !items.length ? (
        <Empty
          image={Empty.PRESENTED_IMAGE_SIMPLE}
          description={
            unavailable
              ? "Saved records unavailable"
              : filtered
                ? "No saved records match these filters"
                : "No saved records. Add records from search."
          }
        />
      ) : null}
      {items.map((item) => {
        const selected = item.record_key === selectedRecordKey;
        const title = item.record?.title ?? item.snapshot.title;
        return (
          <div
            key={item.record_key}
            className="reference-result list-member"
            style={{
              background: selected ? token.colorPrimaryBg : undefined,
              borderInlineStart: `3px solid ${selected ? token.colorPrimaryBorder : "transparent"}`,
            }}
          >
            {item.status !== "unresolved" && item.record ? (
              <RecordBrowseSummary
                record={item.record}
                selected={selected}
                onPreview={() => onSelect(item.record_key)}
              />
            ) : (
              <div className="list-member__unresolved" aria-disabled="true">
                <strong>{title}</strong>
                <span>{item.snapshot.kind}</span>
                <span>Record unavailable in this reference collection</span>
              </div>
            )}
            <DangerActionButton
              className="list-member__remove"
              type="text"
              size="small"
              aria-label={`Remove ${title}`}
              title="Remove"
              icon={<Trash2 size={15} />}
              disabled={removingKey !== null}
              loading={removingKey === item.record_key}
              confirmTitle={`Remove ${title}?`}
              confirmContent="This removes the record from this list."
              confirmOkText="Remove"
              onConfirm={() => onRemove(item.record_key)}
            />
          </div>
        );
      })}
    </section>
  );
}
