import { Alert, Empty, Spin } from "antd";
import type { RecordDetailView } from "../../generated/atlas";
import { RecordPresentation } from "./RecordPresentation";
import type { RecordReferenceHandler } from "./PreparedContent";
type RecordDetailPaneError = Error | { message: string } | null | undefined;
export function RecordDetailPane({
  detail,
  emptyMessage = "Select a result to inspect it.",
  errors = [],
  loading,
  loadingMessage = "Loading record...",
  onReference,
  compact = false,
}: {
  detail: RecordDetailView | undefined;
  emptyMessage?: string;
  errors?: RecordDetailPaneError[];
  loading: boolean;
  loadingMessage?: string;
  onReference: RecordReferenceHandler;
  compact?: boolean;
}) {
  return (
    <section className="detail-panel">
      {loading ? (
        <Spin tip={loadingMessage}>
          <div className="detail-empty">{loadingMessage}</div>
        </Spin>
      ) : detail ? (
        <RecordPresentation
          presentation={detail.presentation}
          root={detail.record}
          selection={detail.selected}
          compact={compact}
          relationships={detail.relationships}
          onReference={onReference}
        />
      ) : (
        <Empty description={emptyMessage} />
      )}
      {errors.map((e, i) =>
        e ? <Alert key={i} type="error" message={e.message} /> : null,
      )}
    </section>
  );
}
