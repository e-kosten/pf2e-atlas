import { usePreviewReadingPosition } from "./usePreviewReadingPosition";
import { Drawer } from "antd";
import { useRef } from "react";
import { RecordDetailPane } from "./RecordDetailPane";
import { RecordPreviewActions } from "./RecordPreviewActions";
import type { RecordPreviewContentProps } from "./recordPreviewTypes";
import { usePreviewContentFocus } from "./usePreviewContentFocus";
import { ActionCost } from "./ActionCost";

export function RecordPreviewDrawer({
  context = "reference",
  actions: recordActions,
  detail,
  loading,
  error,
  onRetry,
  onReload,
  reloadLabel,
  retryLabel,
  onClose,
  onAfterClose,
  onOpenFullPage,
  onReference,
  onRankChange,
  open,
  fullPageHref,
  landing,
  readingHistory,
  onOwnerReturn,
  visibleOwnerRecordKey,
}: {
  onAfterClose: () => void;
  open: boolean;
} & RecordPreviewContentProps) {
  const actions = useRef<HTMLDivElement>(null);
  const readingRef = usePreviewReadingPosition(
    detail,
    readingHistory,
    "drawer",
    open,
    landing,
  );
  const contentFocus = usePreviewContentFocus(open, readingRef.ref);
  return (
    <Drawer
      afterOpenChange={(open) => {
        if (open)
          actions.current?.querySelector<HTMLElement>("a[href],button")?.focus();
        else onAfterClose();
      }}
      focusable={{ focusTriggerAfterClose: false }}
      className="record-preview-drawer"
      closable={{ placement: "end" }}
      open={open}
      placement="right"
      title={
        <>
          {detail?.presentation.identity.title ?? "Loading record…"}{" "}
          {detail?.presentation.body.kind === "activity" && (
            <ActionCost fact={detail.presentation.body.value.action_cost} />
          )}
        </>
      }
      size="min(560px, 100vw)"
      classNames={{
        header: "record-preview-drawer__header",
        body: "record-preview-drawer__body",
      }}
      extra={
        <div ref={actions}>
          <RecordPreviewActions
            actions={recordActions}
            showClose={false}
            readingHistory={readingHistory}
            onClose={onClose}
            onOpenFullPage={onOpenFullPage}
            fullPageHref={fullPageHref}
          />
        </div>
      }
      onClose={onClose}
    >
      <section {...contentFocus} aria-label="Reference content">
        <RecordDetailPane
          hideTitle
          visibleOwnerRecordKey={visibleOwnerRecordKey}
          showRelated={context !== "encounter"}
          showOwnerReturn={context !== "encounter"}
          detail={detail}
          loading={loading}
          error={error}
          onRetry={onRetry}
          onReload={onReload}
          reloadLabel={reloadLabel}
          retryLabel={retryLabel}
          onBack={onClose}
          backLabel="Close preview"
          onReference={onReference}
          onRankChange={onRankChange}
          landing={landing}
          onLandingReady={readingRef.onLandingReady}
          onOwnerReturn={onOwnerReturn}
        />
      </section>
    </Drawer>
  );
}
