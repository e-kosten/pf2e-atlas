import { Alert, Empty, Skeleton } from "antd";
import type { RecordDetailView } from "../../generated/atlas";
import { RecordSurface } from "./RecordSurface";

type RecordDetailPaneError = Error | { message: string } | null | undefined;

export function RecordDetailPane({
  detail,
  emptyMessage,
  errors = [],
  loading,
  loadingMessage,
  onReference,
  showTitle = true,
  stale = false,
}: {
  detail: RecordDetailView | undefined;
  emptyMessage?: string;
  errors?: RecordDetailPaneError[];
  loading: boolean;
  loadingMessage?: string;
  onReference: (recordKey: string) => void;
  showTitle?: boolean;
  stale?: boolean;
}) {
  const visibleErrors = errors.filter((error): error is Error | { message: string } =>
    Boolean(error),
  );
  return (
    <section aria-busy={loading || stale} className="detail-panel">
      {loading ? (
        <div aria-label={loadingMessage ?? "Loading record"} className="detail-state">
          <Skeleton active paragraph={{ rows: 8 }} title />
        </div>
      ) : detail ? (
        <RecordSurface
          surface={detail.surface}
          onReference={onReference}
          showTitle={showTitle}
        />
      ) : (
        <Empty
          className="detail-state"
          description={emptyMessage ?? "Select a result to inspect it."}
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        />
      )}
      {stale && detail && (
        <Alert
          className="detail-state__stale"
          message="Refreshing this record…"
          showIcon
          type="info"
        />
      )}
      {visibleErrors.map((error, index) => (
        <Alert
          className="detail-state__error"
          description={error.message}
          key={index}
          message="Unable to load this record"
          showIcon
          type="error"
        />
      ))}
    </section>
  );
}
