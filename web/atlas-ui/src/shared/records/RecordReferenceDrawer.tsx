import { destinationHref, navigateDestination } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { usePreviewReadingPosition } from "./usePreviewReadingPosition";
import { ConfigProvider, Drawer } from "antd";
import { useId, useRef, useState } from "react";
import { RecordDetailPane } from "./RecordDetailPane";
import { RecordPreviewActions } from "./RecordPreviewActions";
import type {
  RecordPreviewAnchor,
  RecordPreviewContentProps,
} from "./recordPreviewTypes";
import { usePreviewContentFocus } from "./usePreviewContentFocus";
import { RecordPresentationIdentity } from "./RecordPresentation";
import type { RecordNavigationView } from "../../generated/atlas";
import { SourceSectionSelectionContext } from "./SourceSectionSelectionContext";
const getPopupContainer = (trigger?: HTMLElement) =>
  trigger?.closest<HTMLElement>(".record-preview-drawer-root") ?? document.body;
export function RecordReferenceDrawer({
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
  onOpenFullPage,
  onReference,
  onRankChange,
  fullPageHref,
  landing,
  readingHistory,
  anchor,
  visibleOwnerRecordKey,
}: {
  anchor: RecordPreviewAnchor | null;
} & RecordPreviewContentProps) {
  const headingId = useId();
  const heading = useRef<HTMLHeadingElement>(null);
  const [closing, setClosing] = useState(false);
  const open = !closing;
  const close = () => setClosing(true);
  const href =
    fullPageHref ??
    (detail
      ? destinationHref(
          recordDestination({
            recordKey: detail.selected.record_key,
            selection: detail.selected,
            landing,
          }),
        )
      : undefined);
  const openFullPage = () => {
    if (landing && detail)
      navigateDestination(
        recordDestination({
          recordKey: detail.selected.record_key,
          selection: detail.selected,
          landing,
        }),
      );
    else onOpenFullPage();
  };
  const readingRef = usePreviewReadingPosition(detail, readingHistory, open, landing);
  const contentFocus = usePreviewContentFocus(open, readingRef.ref);
  const headerFocus = usePreviewContentFocus(open, heading);
  const returnToOwner = (
    parent: string,
    target: RecordNavigationView,
    trigger: HTMLElement,
  ) =>
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
  return (
    <Drawer
      aria-labelledby={headingId}
      afterOpenChange={(open) => {
        if (open) {
          const target = heading.current;
          const active = target?.ownerDocument.activeElement;
          if (target && (active === anchor || active?.contains(target)))
            target.focus({ preventScroll: true });
        } else {
          onClose();
          if (anchor?.isConnected) anchor.focus({ preventScroll: true });
        }
      }}
      focusable={{ focusTriggerAfterClose: false }}
      push={false}
      className="record-preview-drawer"
      drawerRender={(panel) => (
        <SourceSectionSelectionContext value={null}>
          <ConfigProvider getPopupContainer={getPopupContainer}>{panel}</ConfigProvider>
        </SourceSectionSelectionContext>
      )}
      closable={false}
      open={open}
      placement="right"
      title={
        <div
          onFocusCapture={headerFocus.onFocusCapture}
          onBlurCapture={headerFocus.onBlurCapture}
        >
          <RecordPresentationIdentity
            stacked
            headingRef={heading}
            headingId={headingId}
            presentation={detail?.presentation}
            root={detail?.record}
            selection={detail?.selected}
            relationships={detail?.relationships}
            onReference={onReference}
            onRankChange={onRankChange}
            onOwnerReturn={returnToOwner}
            visibleOwnerRecordKey={visibleOwnerRecordKey}
            showOwnerReturn={context !== "encounter"}
            actions={(secondaryActions) => (
              <RecordPreviewActions
                actions={recordActions}
                secondaryActions={secondaryActions}
                readingHistory={readingHistory}
                onClose={close}
                onOpenFullPage={openFullPage}
                fullPageHref={href}
              />
            )}
          />
        </div>
      }
      size="min(760px, 100vw)"
      styles={{
        // Keep anchored tooltips inside the overlay viewport during its slide.
        // A body-level tooltip can widen mobile Chromium and rewrap the reader.
        // Its masked root must also allow pointer events for interactive popups.
        root: { overflow: "hidden", pointerEvents: "auto" },
        title: { minWidth: 0 },
        header: {
          flex: "none",
          alignItems: "flex-start",
          padding: "var(--record-drawer-inset)",
        },
        body: { padding: "var(--record-drawer-inset)" },
      }}
      classNames={{
        root: "record-preview-drawer-root",
        header: "record-preview-drawer__header",
        body: "record-preview-drawer__body",
      }}
      onClose={close}
    >
      <section {...contentFocus} aria-label="Reference content">
        <RecordDetailPane
          identityInHost
          visibleOwnerRecordKey={visibleOwnerRecordKey}
          showRelated={false}
          showOwnerReturn={context !== "encounter"}
          detail={detail}
          loading={loading}
          error={error}
          onRetry={onRetry}
          onReload={onReload}
          reloadLabel={reloadLabel}
          retryLabel={retryLabel}
          onBack={close}
          backLabel="Close preview"
          onReference={onReference}
          onRankChange={onRankChange}
          landing={landing}
          onLandingReady={readingRef.onLandingReady}
          onOwnerReturn={returnToOwner}
        />
      </section>
    </Drawer>
  );
}
