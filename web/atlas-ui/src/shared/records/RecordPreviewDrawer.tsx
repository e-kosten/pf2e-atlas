import { Drawer } from "antd";
import { useRef } from "react";
import { RecordDetailPane } from "./RecordDetailPane";
import { RecordPreviewActions } from "./RecordPreviewActions";
import type { RecordPreviewContentProps } from "./recordPreviewTypes";

export function RecordPreviewDrawer({
  detail,
  loading,
  error,
  onRetry,
  onClose,
  onAfterClose,
  onOpenFullPage,
  onReference,
  open,
  content,
  fullPageHref,
  landing,
  onOwnerReturn,
}: {
  onAfterClose: () => void;
  open: boolean;
} & RecordPreviewContentProps) {
  const actions = useRef<HTMLDivElement>(null);
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
      title="Reference"
      size="min(560px, calc(100vw - 24px))"
      classNames={{
        header: "record-preview-drawer__header",
        body: "record-preview-drawer__body",
      }}
      extra={
        <div ref={actions}>
          <RecordPreviewActions
            onClose={onClose}
            onOpenFullPage={onOpenFullPage}
            fullPageHref={fullPageHref}
          />
        </div>
      }
      onClose={onClose}
    >
      <section aria-label="Reference content">
        {content ?? (
          <RecordDetailPane
            compact
            detail={detail}
            loading={loading}
            error={error}
            onRetry={onRetry}
            onBack={onClose}
            backLabel="Close preview"
            onReference={onReference}
            landing={landing}
            onOwnerReturn={onOwnerReturn}
          />
        )}
      </section>
    </Drawer>
  );
}
