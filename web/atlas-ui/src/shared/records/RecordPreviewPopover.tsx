import { usePreviewReadingPosition } from "./usePreviewReadingPosition";
import { Grid, Popover, Spin } from "antd";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
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
  context = "reference",
  actions: recordActions,
  anchor,
  visibleOwnerRecordKey,
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
  onRankChange,
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
        embedded_path: [],
        field: null,
        passage: null,
        source_fingerprint: null,
      },
      target,
    );
  };
  const narrow = screens.md === false;
  const [closingDrawer, setClosingDrawer] = useState(false);
  const readingRef = usePreviewReadingPosition(
    detail,
    readingHistory,
    "popover",
    !narrow && !closingDrawer,
    landing,
  );
  const contentFocus = usePreviewContentFocus(
    !narrow && !closingDrawer,
    readingRef.ref,
  );
  const popover = useRef<import("antd").TooltipRef>(null);
  const [bounds, setBounds] = useState<DOMRect>();
  // Let Ant measure the actual document before the first visible placement.
  // A short loading popup would move when replaced by the full record.
  const waitingForRecord = loading && !detail && !error;
  useEffect(() => {
    // Before the popup exists, its pending request still needs dismissal.
    // Once visible (or on phones), the native overlay owns these interactions.
    if (!waitingForRecord || narrow || closingDrawer) return;
    const dismissOutside = (event: PointerEvent) => {
      if (event.target instanceof Node && !anchor?.contains(event.target)) onClose();
    };
    const dismissEscape = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      onClose();
    };
    document.addEventListener("pointerdown", dismissOutside, true);
    document.addEventListener("keydown", dismissEscape, true);
    return () => {
      document.removeEventListener("pointerdown", dismissOutside, true);
      document.removeEventListener("keydown", dismissEscape, true);
    };
  }, [waitingForRecord, narrow, closingDrawer, anchor, onClose]);
  useLayoutEffect(() => {
    let resizeFrame = 0;
    const update = () => {
      const next = anchor?.getBoundingClientRect();
      setBounds((current) =>
        current?.left === next?.left &&
        current?.top === next?.top &&
        current?.width === next?.width &&
        current?.height === next?.height
          ? current
          : next,
      );
    };
    const resize = () => {
      cancelAnimationFrame(resizeFrame);
      resizeFrame = requestAnimationFrame(() => {
        // Responsive text reflow can move the opener outside its scroll pane.
        // Preserve that reading anchor; Ant still owns popup positioning.
        if (!narrow) anchor?.scrollIntoView({ block: "nearest", inline: "nearest" });
        update();
      });
    };
    update();
    window.addEventListener("resize", resize);
    window.addEventListener("scroll", update, true);
    return () => {
      cancelAnimationFrame(resizeFrame);
      window.removeEventListener("resize", resize);
      window.removeEventListener("scroll", update, true);
    };
  }, [anchor, narrow]);
  useLayoutEffect(() => {
    popover.current?.forceAlign();
  }, [bounds]);
  if (narrow || closingDrawer) {
    return (
      <RecordPreviewDrawer
        actions={recordActions}
        context={context}
        visibleOwnerRecordKey={visibleOwnerRecordKey}
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
        onRankChange={onRankChange}
        landing={landing}
        onOwnerReturn={returnToOwner}
        fullPageHref={href}
        open={!closingDrawer}
      />
    );
  }
  return (
    <Popover
      classNames={{ root: "record-preview-popover" }}
      ref={popover}
      afterOpenChange={(open) => {
        if (open)
          popover.current?.popupElement
            .querySelector<HTMLElement>("a[href],button")
            ?.focus({ preventScroll: true });
      }}
      open={Boolean(bounds && (detail || error))}
      placement="right"
      // Let Ant shift along either axis without retaining its arrow-gap offset
      // beyond the viewport edge when neither side has room for a full record.
      align={{
        offset: [0, 0],
        overflow: { adjustX: true, adjustY: true, shiftX: true, shiftY: true },
      }}
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
      content={
        <section role="dialog" aria-label="Reference preview">
          <div className="record-preview-popover__header">
            <RecordPreviewActions
              actions={recordActions}
              readingHistory={readingHistory}
              onClose={onClose}
              onOpenFullPage={openFullPage}
              fullPageHref={href}
            />
          </div>

          <section {...contentFocus} className="record-preview-popover__body">
            <RecordDetailPane
              compact
              showRelated={false}
              showOwnerReturn={context !== "encounter"}
              visibleOwnerRecordKey={visibleOwnerRecordKey}
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
              onOwnerReturn={returnToOwner}
            />
          </section>
        </section>
      }
    >
      <span
        aria-hidden={waitingForRecord ? undefined : true}
        className="record-preview-anchor"
        style={{
          position: "fixed",
          left: bounds?.left || 0,
          top: bounds?.top || 0,
          width: bounds?.width || 1,
          height: bounds?.height || 1,
          pointerEvents: "none",
        }}
      >
        {waitingForRecord && (
          <span
            className="record-preview-loading"
            role="status"
            aria-label="Loading reference"
          >
            <Spin size="small" delay={500} />
          </span>
        )}
      </span>
    </Popover>
  );
}
