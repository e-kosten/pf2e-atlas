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
import { currentAtlasRoute } from "../../app/routes";
type RecordDetailPaneError = Error | { message: string } | null | undefined;
export function RecordDetailPane({
  detail,
  emptyMessage = "Select a result to inspect it.",
  error,
  loading,
  loadingMessage = "Loading record…",
  onRetry,
  onReload = onRetry,
  reloadLabel = "Reload record",
  retryLabel = "Retry",
  onBack,
  backLabel = "Back to search",
  onReference,
  compact = false,
  showRelated = true,
  landing,
  onOwnerReturn,
}: {
  detail: RecordDetailView | undefined;
  emptyMessage?: string;
  error?: RecordDetailPaneError;
  loading: boolean;
  loadingMessage?: string;
  onRetry?: () => void;
  onReload?: () => void;
  reloadLabel?: string;
  retryLabel?: string;
  onBack?: () => void;
  backLabel?: string;
  onReference: RecordReferenceHandler;
  compact?: boolean;
  showRelated?: boolean;
  landing?: import("../../generated/atlas").RecordNavigationView;
  onOwnerReturn?: (
    parent: string,
    landing: import("../../generated/atlas").RecordNavigationView,
    anchor: HTMLElement,
  ) => void;
}) {
  const code =
    error && "appError" in error ? (error as AtlasApiError).appError?.code : undefined;
  const notFound = code === "record_not_found";
  const route = currentAtlasRoute();
  const routeLanding = "landing" in route ? route.landing : undefined;
  const readingLanding =
    landing ??
    (routeLanding?.record_key === detail?.record.record_key ? routeLanding : undefined);
  return (
    <section
      className={`detail-panel${compact ? " detail-panel--compact" : ""}`}
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
                    {retryLabel}
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
            relationships={detail.relationships}
            reading={{
              generation: detail.artifact_generation,
              parent: {
                record_key: detail.record.record_key,
                owners: [],
                field: null,
                passage: null,
                source_fingerprint: null,
              },
              landing: readingLanding,
              reloadParent: onReload,
              reloadParentLabel: reloadLabel,
            }}
            onReference={onReference}
            onOwnerReturn={onOwnerReturn}
          />
          {showRelated && (
            <RelatedRecords
              key={JSON.stringify(detail.selected)}
              detail={detail}
              onReference={onReference}
              onReload={onReload}
              reloadLabel={reloadLabel}
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
