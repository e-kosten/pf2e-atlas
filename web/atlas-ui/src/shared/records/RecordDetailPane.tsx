import { Alert, Button, Empty, Space, Spin } from "antd";
import type { AtlasApiError } from "../../api/atlasApi";
import {
  navigateToAtlasRoute,
  searchPath,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import type { RecordDetailView } from "../../generated/atlas";
import { RecordPresentation } from "./RecordPresentation";
import type { RecordReferenceHandler } from "./PreparedContent";
import { RelatedRecords } from "./RelatedRecords";
type RecordDetailPaneError = Error | { message: string } | null | undefined;
export function RecordDetailPane({
  detail,
  emptyMessage = "Select a result to inspect it.",
  error,
  loading,
  loadingMessage = "Loading record…",
  onRetry,
  onBack,
  backLabel = "Back to search",
  onReference,
  compact = false,
  showRelated = true,
  inline = false,
}: {
  detail: RecordDetailView | undefined;
  emptyMessage?: string;
  error?: RecordDetailPaneError;
  loading: boolean;
  loadingMessage?: string;
  onRetry?: () => void;
  onBack?: () => void;
  backLabel?: string;
  onReference: RecordReferenceHandler;
  compact?: boolean;
  showRelated?: boolean;
  inline?: boolean;
}) {
  const code =
    error && "appError" in error ? (error as AtlasApiError).appError?.code : undefined;
  const notFound = code === "record_not_found";
  return (
    <section
      className={`detail-panel${compact ? " detail-panel--compact" : ""}${inline ? " detail-panel--inline" : ""}`}
      aria-busy={loading}
    >
      {detail && loading && (
        <div className="detail-status" role="status">
          <Spin size="small" /> Updating record…
        </div>
      )}
      {error && (
        <div className="detail-status">
          <Alert
            showIcon
            type="error"
            title={
              detail
                ? "Unable to refresh this record"
                : notFound
                  ? "Record unavailable"
                  : "Unable to load this record"
            }
            description={
              detail ? (
                <>Previously loaded content remains below. {error.message}</>
              ) : notFound ? (
                "This record is not present in the current reference collection."
              ) : (
                error.message
              )
            }
            action={
              <Space wrap>
                {onRetry && (
                  <Button size="small" onClick={onRetry} loading={loading}>
                    Retry
                  </Button>
                )}
                {onBack ? (
                  <Button size="small" onClick={onBack}>
                    {backLabel}
                  </Button>
                ) : (
                  <Button
                    size="small"
                    href={searchPath()}
                    onClick={(event) => {
                      if (!shouldHandleAtlasRouteClick(event)) return;
                      event.preventDefault();
                      navigateToAtlasRoute({ kind: "search", selectedRecordKey: null });
                    }}
                  >
                    Back to search
                  </Button>
                )}
              </Space>
            }
          />
        </div>
      )}
      {detail ? (
        <>
          <RecordPresentation
            presentation={detail.presentation}
            root={detail.record}
            selection={detail.selected}
            compact={compact}
            hideIdentity={inline}
            hideActivityCost={inline}
            relationships={detail.relationships}
            onReference={onReference}
          />
          {showRelated && (
            <RelatedRecords
              key={JSON.stringify(detail.selected)}
              detail={detail}
              onReference={onReference}
            />
          )}
        </>
      ) : loading ? (
        <Spin description={loadingMessage}>
          <div className="detail-empty" role="status">
            {loadingMessage}
          </div>
        </Spin>
      ) : !error ? (
        <div className="detail-empty">
          <Empty description={emptyMessage} />
        </div>
      ) : null}
    </section>
  );
}
