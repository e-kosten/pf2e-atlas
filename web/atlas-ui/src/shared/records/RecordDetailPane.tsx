import { Alert, Empty, Spin } from "antd";
import type { RecordDetailView } from "../../generated/atlas";
import { RecordSurface } from "./RecordSurface";
import type { RecordReferenceHandler } from "./PreparedContent";
type RecordDetailPaneError = Error | { message: string } | null | undefined;
export function RecordDetailPane({
  detail,
  emptyMessage = "Select a result to inspect it.",
  errors = [],
  loading,
  loadingMessage = "Loading record...",
  onReference,
}: {
  detail: RecordDetailView | undefined;
  emptyMessage?: string;
  errors?: RecordDetailPaneError[];
  loading: boolean;
  loadingMessage?: string;
  onReference: RecordReferenceHandler;
}) {
  return (
    <section className="detail-panel">
      {loading ? (
        <Spin tip={loadingMessage}>
          <div className="detail-empty">{loadingMessage}</div>
        </Spin>
      ) : detail ? (
        <RecordSurface
          surface={detail.surface}
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
