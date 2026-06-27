import type { RecordDetailView } from "../../generated/atlas";
import { RecordPresentation } from "./RecordPresentation";
import { RecordSurface } from "./RecordSurface";

type RecordDetailPaneError = Error | { message: string } | null | undefined;

export function RecordDetailPane({
  detail,
  emptyMessage,
  errors = [],
  loading,
  loadingMessage,
  onReference,
}: {
  detail: RecordDetailView | undefined;
  emptyMessage?: string;
  errors?: RecordDetailPaneError[];
  loading: boolean;
  loadingMessage?: string;
  onReference: (recordKey: string, anchorRect?: DOMRect) => void;
}) {
  return (
    <section className="detail-panel">
      {loading ? (
        <RecordPresentation
          detail={detail}
          emptyMessage={emptyMessage}
          loading={loading}
          loadingMessage={loadingMessage}
          onReference={onReference}
        />
      ) : detail?.surface ? (
        <RecordSurface surface={detail.surface} onReference={onReference} />
      ) : (
        <RecordPresentation
          detail={detail}
          emptyMessage={emptyMessage}
          loading={loading}
          loadingMessage={loadingMessage}
          onReference={onReference}
        />
      )}
      {errors.map((error, index) =>
        error ? <InlineError key={index} message={error.message} /> : null,
      )}
    </section>
  );
}

function InlineError({ message }: { message: string }) {
  return <div className="error-banner">{message}</div>;
}
