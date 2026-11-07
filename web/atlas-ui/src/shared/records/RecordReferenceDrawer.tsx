import { AtlasDrawer as Drawer } from "../ui/overlays/AtlasDrawer";
import { destinationHref, navigateDestination } from "../../app/navigation";
import { usePreviewReadingPosition } from "./usePreviewReadingPosition";
import { ConfigProvider } from "antd";
import { useId, useRef, useState } from "react";
import { DefinitionDetailPane, DefinitionAttribution } from "./DefinitionDetailPane";
import {
  ReferenceHelpProvider,
  referenceDestination,
  definitionClassification,
} from "./ReferenceHelp";
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
import { DocumentIdentityHeader } from "./DocumentIdentityHeader";
import { readingNavigationForUrl } from "./readingUrlNavigation";
import { isArtifactGenerationChanged } from "../queries/referenceQueries";
const getPopupContainer = (trigger?: HTMLElement) =>
  trigger?.closest<HTMLElement>(".record-preview-drawer-root") ?? document.body;
export function RecordReferenceDrawer({
  destination,
  definition,
  help,
  onHelp,
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
  onNavigateAway,
  restoreAnchorFocus = true,
  onReference,
  onRankChange,
  requestedNavigation,
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
  const unavailable =
    destination.kind === "definition" && (!help || definition?.help === null);
  const retryActionLabel =
    retryLabel ??
    (isArtifactGenerationChanged(error)
      ? destination?.kind === "definition" || help?.definition_key
        ? "Refresh definition"
        : "Refresh record"
      : "Retry");
  const navigation = detail
    ? readingNavigationForUrl(detail.selected, { navigation: requestedNavigation })
    : undefined;
  const typedRoute = referenceDestination(
    destination.kind === "record" && navigation
      ? { ...destination, navigation }
      : destination,
    help?.definition_key,
    landing,
  );
  const href = unavailable ? undefined : destinationHref(typedRoute);
  const openFullPage = () => {
    if (unavailable) return;
    navigateDestination(typedRoute);
    (onNavigateAway ?? onClose)();
  };
  const readingRef = usePreviewReadingPosition(
    definition ?? detail,
    readingHistory,
    open,
    landing,
  );
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
          if (restoreAnchorFocus && anchor?.isConnected)
            anchor.focus({ preventScroll: true });
        }
      }}
      focusable={{ focusTriggerAfterClose: false }}
      push={false}
      className="record-preview-drawer"
      drawerRender={(panel) => (
        <SourceSectionSelectionContext value={null}>
          <ReferenceHelpProvider
            onHelp={onHelp}
            onReference={onReference}
            generation={definition?.artifact_generation ?? detail?.artifact_generation}
          >
            <ConfigProvider getPopupContainer={getPopupContainer}>
              {panel}
            </ConfigProvider>
          </ReferenceHelpProvider>
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
          {destination?.kind === "definition" ? (
            <DocumentIdentityHeader
              stacked
              headingRef={heading}
              headingId={headingId}
              title={definition?.label ?? "Definition"}
              classification={definitionClassification(destination.key)}
              attribution={definition && <DefinitionAttribution detail={definition} />}
              actions={
                <RecordPreviewActions
                  readingHistory={readingHistory}
                  onClose={close}
                  onOpenFullPage={openFullPage}
                  fullPageHref={href}
                  fullPageDisabled={unavailable}
                />
              }
            />
          ) : (
            <RecordPresentationIdentity
              stacked
              headingRef={heading}
              headingId={headingId}
              presentation={detail?.presentation}
              root={detail?.record}
              selection={detail?.selected}
              relationships={detail?.relationships}
              onRankChange={onRankChange}
              onOwnerReturn={returnToOwner}
              visibleOwnerRecordKey={visibleOwnerRecordKey}
              showOwnerReturn={context !== "encounter"}
              actions={(secondaryActions) => (
                <RecordPreviewActions
                  actions={context === "encounter" ? undefined : recordActions}
                  contextActions={context === "encounter" ? recordActions : undefined}
                  secondaryActions={secondaryActions}
                  readingHistory={readingHistory}
                  onClose={close}
                  onOpenFullPage={openFullPage}
                  fullPageHref={href}
                  fullPageDisabled={unavailable}
                />
              )}
            />
          )}
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
        {destination?.kind === "definition" ? (
          <DefinitionDetailPane
            identityInHost
            detail={definition}
            loading={loading}
            error={error}
            onRetry={onRetry}
            retryLabel={retryActionLabel}
            onReference={onReference}
          />
        ) : (
          <>
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
              retryLabel={retryActionLabel}
              onBack={close}
              backLabel="Close preview"
              onReference={onReference}
              onHelp={onHelp}
              onRankChange={onRankChange}
              landing={landing}
              onLandingReady={readingRef.onLandingReady}
              onOwnerReturn={returnToOwner}
            />
          </>
        )}
      </section>
    </Drawer>
  );
}
