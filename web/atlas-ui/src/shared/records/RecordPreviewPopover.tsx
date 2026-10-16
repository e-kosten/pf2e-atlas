import { usePreviewReadingPosition } from "./usePreviewReadingPosition";
import { Grid, Popover } from "antd";
import { useLayoutEffect, useRef, useState } from "react";
import { RecordDetailPane } from "./RecordDetailPane";
import { RecordPreviewActions } from "./RecordPreviewActions";
import { RecordPreviewDrawer } from "./RecordPreviewDrawer";
import { usePreviewContentFocus } from "./usePreviewContentFocus";
import { atlasRoutePath, navigateToAtlasRoute } from "../../app/routes";
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
  onReload,
  reloadLabel,
  retryLabel,
  onClose,
  onOpenFullPage,
  onReference,
  fullPageHref,
  readingHistory,
  landing,
}: RecordPreviewPopoverProps) {
  const screens = Grid.useBreakpoint();
  const href =
    fullPageHref ??
    (detail
      ? atlasRoutePath({
          kind: "record",
          recordKey: detail.selected.record_key,
          selection: detail.selected,
          landing,
        })
      : undefined);
  const openFullPage = () => {
    if (landing && detail)
      navigateToAtlasRoute({
        kind: "record",
        recordKey: detail.selected.record_key,
        selection: detail.selected,
        landing,
      });
    else onOpenFullPage();
  };
  const returnToOwner = (
    parent: string,
    target: import("../../generated/atlas").RecordNavigationView,
    trigger: HTMLElement,
  ) => {
    onReference(
      parent,
      trigger,
      {
        record_key: parent,
        owners: [],
        field: null,
        passage: null,
        source_fingerprint: null,
      },
      target,
    );
  };
  const narrow = Boolean(
    !screens.xl && (screens.xs || screens.sm || screens.md || screens.lg),
  );
  const [closingDrawer, setClosingDrawer] = useState(false);
  const readingRef = usePreviewReadingPosition(
    detail,
    readingHistory,
    "popover",
    !narrow && !closingDrawer,
  );
  const contentFocus = usePreviewContentFocus(!narrow && !closingDrawer, readingRef);
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
        readingHistory={readingHistory}
        detail={detail}
        loading={loading}
        error={error}
        onRetry={onRetry}
        onReload={onReload}
        reloadLabel={reloadLabel}
        retryLabel={retryLabel}
        onClose={() => setClosingDrawer(true)}
        onAfterClose={onClose}
        onOpenFullPage={openFullPage}
        onReference={onReference}
        landing={landing}
        onOwnerReturn={returnToOwner}
        fullPageHref={href}
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
            .querySelector<HTMLElement>("a[href],button")
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
          <RecordPreviewActions
            readingHistory={readingHistory}
            onClose={onClose}
            onOpenFullPage={openFullPage}
            fullPageHref={href}
          />
        </div>
      }
      content={
        <section
          {...contentFocus}
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
            onReload={onReload}
            reloadLabel={reloadLabel}
            retryLabel={retryLabel}
            onBack={onClose}
            backLabel="Close preview"
            onReference={onReference}
            landing={landing}
            onOwnerReturn={returnToOwner}
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
