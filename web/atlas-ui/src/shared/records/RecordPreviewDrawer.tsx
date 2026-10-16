import { usePreviewReadingPosition } from "./usePreviewReadingPosition";
import { Drawer } from "antd";
import { useRef } from "react";
import { RecordDetailPane } from "./RecordDetailPane";
import { RecordPreviewActions } from "./RecordPreviewActions";
import type { RecordPreviewContentProps } from "./recordPreviewTypes";
import { usePreviewContentFocus } from "./usePreviewContentFocus";

export function RecordPreviewDrawer({
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
  open,
  fullPageHref,
  landing,
  readingHistory,
  onOwnerReturn,
}: {
  onAfterClose: () => void;
  open: boolean;
} & RecordPreviewContentProps) {
  const actions = useRef<HTMLDivElement>(null);
  const readingRef = usePreviewReadingPosition(detail, readingHistory, "drawer", open);
  const contentFocus = usePreviewContentFocus(open, readingRef);
  return (
    <Drawer
      afterOpenChange={(open) => {
        if (open)
          actions.current?.querySelector<HTMLElement>("a[href],button")?.focus();
        else onAfterClose();
      }}
      focusable={{ focusTriggerAfterClose: false }}
      className="record-preview-drawer"
      closable={false}
      open={open}
      placement="right"
      title={<span className="sr-only">Reference</span>}
      size="min(560px, calc(100vw - 24px))"
      classNames={{
        header: "record-preview-drawer__header",
        body: "record-preview-drawer__body",
      }}
      extra={
        <div ref={actions}>
          <RecordPreviewActions
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
          compact
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
          landing={landing}
          onOwnerReturn={onOwnerReturn}
        />
      </section>
    </Drawer>
  );
}
