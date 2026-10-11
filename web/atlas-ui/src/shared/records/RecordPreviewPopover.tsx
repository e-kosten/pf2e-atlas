import { Grid, Popover } from "antd";
import { useLayoutEffect, useRef, useState } from "react";
import { RecordDetailPane } from "./RecordDetailPane";
import { RecordPreviewActions } from "./RecordPreviewActions";
import { RecordPreviewDrawer } from "./RecordPreviewDrawer";
import type {
  RecordPreviewAnchor,
  RecordPreviewContentProps,
} from "./recordPreviewTypes";
type RecordPreviewPopoverProps = RecordPreviewContentProps & {
  anchor: RecordPreviewAnchor | null;
};
export function RecordPreviewPopover({
  anchor,
  detail,
  loading,
  error,
  onRetry,
  onClose,
  onOpenFullPage,
  onReference,
}: RecordPreviewPopoverProps) {
  const screens = Grid.useBreakpoint();
  const narrow = Boolean(
    !screens.xl && (screens.xs || screens.sm || screens.md || screens.lg),
  );
  const [closingDrawer, setClosingDrawer] = useState(false);
  const popover = useRef<import("antd").TooltipRef>(null);
  const [bounds, setBounds] = useState<DOMRect>();
  useLayoutEffect(() => {
    const update = () => {
      setBounds(anchor?.getBoundingClientRect());
    };
    update();
    window.addEventListener("resize", update);
    window.addEventListener("scroll", update, true);
    return () => {
      window.removeEventListener("resize", update);
      window.removeEventListener("scroll", update, true);
    };
  }, [anchor]);
  useLayoutEffect(() => {
    popover.current?.forceAlign();
  }, [bounds]);
  if (narrow || closingDrawer) {
    return (
      <RecordPreviewDrawer
        detail={detail}
        loading={loading}
        error={error}
        onRetry={onRetry}
        onClose={() => setClosingDrawer(true)}
        onAfterClose={onClose}
        onOpenFullPage={onOpenFullPage}
        onReference={onReference}
        open={!closingDrawer}
      />
    );
  }
  return (
    <Popover
      ref={popover}
      afterOpenChange={(open) => {
        if (open)
          popover.current?.popupElement
            .querySelector<HTMLButtonElement>("button")
            ?.focus();
      }}
      open
      placement="right"
      trigger="click"
      onOpenChange={(open) => {
        // A modal owns dismissal while it is the active overlay, including portals
        // opened by prepared content. Ant keeps the preview positioned and handles Escape.
        if (
          !open &&
          !document.activeElement?.closest('[role="dialog"][aria-modal="true"]')
        )
          onClose();
      }}
      title={
        <div className="record-preview-popover__header">
          <span>Reference</span>
          <RecordPreviewActions onClose={onClose} onOpenFullPage={onOpenFullPage} />
        </div>
      }
      content={
        <section
          aria-label="Reference preview"
          className="record-preview-popover__body"
          role="dialog"
        >
          <RecordDetailPane
            compact
            detail={detail}
            loading={loading}
            error={error}
            onRetry={onRetry}
            onBack={onClose}
            backLabel="Close preview"
            onReference={onReference}
          />
        </section>
      }
    >
      <span
        aria-hidden
        className="record-preview-anchor"
        style={{
          position: "fixed",
          left: bounds?.left || 0,
          top: bounds?.top || 0,
          width: bounds?.width || 1,
          height: bounds?.height || 1,
          pointerEvents: "none",
        }}
      />
    </Popover>
  );
}
