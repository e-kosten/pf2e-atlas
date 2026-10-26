import {
  destinationHref,
  isPrimaryActivation,
  navigateDestination,
} from "../../app/navigation";
import { searchDestination } from "../../app/destinations";
import { Alert, Button, Empty, Space, Spin } from "antd";
import type { ReactNode } from "react";
import type { AtlasApiError } from "../../api/atlasApi";
import type { RecordDetailView } from "../../generated/atlas";
import { RecordPresentation, RecordPresentationIdentity } from "./RecordPresentation";
import { RecordReadingWorkspace } from "./RecordReadingWorkspace";
import { recordReadingIdentity } from "./useRetainedRecord";
import type { RecordReferenceHandler } from "./PreparedContent";
import { RelatedRecords } from "./RelatedRecords";
type RecordDetailPaneError =
  | Error
  | {
      message: string;
    }
  | null
  | undefined;
export function RecordDetailPane({
  detail,
  emptyMessage = "Select a record",
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
  onRankChange,
  compact = false,
  showRelated = true,
  landing,
  onLandingReady,
  onOwnerReturn,
  visibleOwnerRecordKey,
  identityInHost = false,
  showOwnerReturn = true,
  sectionLanding,
  identityActions,
  readingKey,
}: {
  sectionLanding?: string;
  identityActions?: ReactNode;
  readingKey?: string;
  detail: RecordDetailView | undefined;
  visibleOwnerRecordKey?: string;
  identityInHost?: boolean;
  showOwnerReturn?: boolean;
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
  onRankChange?: (rank: number | undefined) => void;
  compact?: boolean;
  showRelated?: boolean;
  landing?: import("../../generated/atlas").RecordNavigationView;
  onLandingReady?: (target?: HTMLElement) => void;
  onOwnerReturn?: (
    parent: string,
    landing: import("../../generated/atlas").RecordNavigationView,
    anchor: HTMLElement,
  ) => void;
}) {
  const code =
    error && "appError" in error ? (error as AtlasApiError).appError?.code : undefined;
  const notFound = code === "record_not_found";
  return (
    <section
      className={`detail-panel${compact ? " detail-panel--compact" : ""}`}
      aria-busy={loading}
    >
      <RecordReadingContent
        key={
          readingKey ??
          JSON.stringify([
            recordReadingIdentity(detail?.record.record_key ?? null, detail?.selected),
            detail?.artifact_generation,
          ])
        }
        enabled={showRelated && (Boolean(detail) || Boolean(identityActions))}
        identity={
          !identityInHost
            ? (exploreAction) => (
                <RecordPresentationIdentity
                  presentation={detail?.presentation}
                  root={detail?.record}
                  selection={detail?.selected}
                  relationships={detail?.relationships}
                  onReference={onReference}
                  onRankChange={onRankChange}
                  onOwnerReturn={onOwnerReturn}
                  visibleOwnerRecordKey={visibleOwnerRecordKey}
                  showOwnerReturn={showOwnerReturn}
                  actions={
                    <Space wrap size={4}>
                      {detail && exploreAction}
                      {identityActions}
                    </Space>
                  }
                />
              )
            : undefined
        }
        status={
          <>
            {detail && loading && (
              <div className="detail-refresh" aria-label="Updating record">
                <Spin size="small" delay={500} />
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
                          href={destinationHref(
                            searchDestination({ selectedRecordKey: null }),
                          )}
                          onClick={(event) => {
                            if (!isPrimaryActivation(event)) return;
                            event.preventDefault();
                            navigateDestination(
                              searchDestination({
                                selectedRecordKey: null,
                              }),
                            );
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
          </>
        }
        explore={
          detail && (
            <RelatedRecords
              detail={detail}
              onReference={onReference}
              onReload={onReload}
              reloadLabel={reloadLabel}
            />
          )
        }
      >
        {detail ? (
          <>
            <RecordPresentation
              sectionLanding={sectionLanding}
              identityInHost={identityInHost || showRelated}
              showOwnerReturn={showOwnerReturn}
              visibleOwnerRecordKey={visibleOwnerRecordKey}
              presentation={detail.presentation}
              root={detail.record}
              selection={detail.selected}
              compact={compact}
              relationships={detail.relationships}
              reading={{
                generation: detail.artifact_generation,
                parent: {
                  record_key: detail.record.record_key,
                  embedded_path: [],
                  field: null,
                  passage: null,
                  source_fingerprint: null,
                },
                landing,
                onLandingReady,
                reloadParent: onReload,
                reloadParentLabel: reloadLabel,
              }}
              onReference={onReference}
              onRankChange={onRankChange}
              onOwnerReturn={onOwnerReturn}
            />
          </>
        ) : loading ? (
          <Spin delay={500} description={loadingMessage}>
            <div className="detail-empty" role="status" aria-label={loadingMessage} />
          </Spin>
        ) : !error ? (
          <div className="detail-empty">
            <Empty description={emptyMessage} />
          </div>
        ) : null}
      </RecordReadingContent>
    </section>
  );
}

function RecordReadingContent({
  enabled,
  explore,
  children,
  identity,
  status,
}: {
  enabled: boolean;
  explore: ReactNode;
  children: ReactNode;
  identity?: (action: ReactNode) => ReactNode;
  status?: ReactNode;
}) {
  return enabled ? (
    <RecordReadingWorkspace explore={explore} identity={identity} status={status}>
      {children}
    </RecordReadingWorkspace>
  ) : (
    <>
      {status}
      {children}
    </>
  );
}
