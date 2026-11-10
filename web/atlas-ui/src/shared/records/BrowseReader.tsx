import { isPrimaryActivation } from "../../app/navigation";
import { Grid, Splitter } from "antd";
import { X } from "lucide-react";
import { PaneIconButton } from "../ui/actions/PaneAction";
import { useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { atlasRouter } from "../../app/router";
import { ReferenceReader } from "./ReferenceReader";
import { useBrowsePositions, type BrowsePositions } from "./useBrowsePositions";
import { type ReferenceReading } from "./useReferenceReading";
import type { RecordSummaryView } from "../../generated/atlas";
import { BrowsePaneHeader } from "./BrowsePaneHeader";
// The browse host owns layout, scroll and mobile return. Features own rows,
// queries, selection and mutation semantics; no search DTO is required here.
export function BrowseReader({
  children,
  resultsHeader,
  renderResultsPaneHeader,
  resultsLabel,
  resultsIdentity,
  resultsLoading,
  selectedRecordKey,
  reading,
  recordActions,
  readerStatus,
  readerContext,
  onReaderClose,
  onClearSelection,
  clearSelectionDisabled,
  readerFooter,
  positions,
}: {
  children: ReactNode;
  resultsHeader?: ReactNode;
  renderResultsPaneHeader?: (actions: ReactNode) => ReactNode;
  resultsLabel: string;
  resultsIdentity: unknown;
  resultsLoading: boolean;
  selectedRecordKey: string | null;
  reading: ReferenceReading;
  readerContext?: ReactNode;
  recordActions?: (record: RecordSummaryView) => ReactNode;
  readerStatus?: ReactNode;
  onReaderClose?: () => void;
  onClearSelection?: () => void;
  clearSelectionDisabled?: boolean;
  readerFooter?: ReactNode;
  positions?: BrowsePositions;
}) {
  const paneHeader = Boolean(renderResultsPaneHeader);
  const narrow = !Grid.useBreakpoint().lg;
  const [closedRequest, setClosedRequest] = useState<ReferenceReading["request"]>();
  const localPositions = useBrowsePositions();
  const saved = positions ?? localPositions;
  const listRef = useRef<HTMLDivElement>(null);
  // Activity can keep this DOM while suspending its layout effects.
  const layoutActive = useRef(false);
  const navigationPending = useRef(false);
  useLayoutEffect(() => {
    layoutActive.current = true;
    navigationPending.current = false;
    return () => {
      layoutActive.current = false;
    };
  }, []);
  const beforePreview = useRef<HTMLElement | null>(null);
  const readerOpen = reading.request !== null && closedRequest !== reading.request;
  useLayoutEffect(() => {
    // Capture before route rendering removes scroll content. Native scroll events
    // may still be queued when the user immediately opens the full reference.
    const snapshot = () => {
      if (
        layoutActive.current &&
        !navigationPending.current &&
        listRef.current &&
        (!narrow || !readerOpen)
      )
        saved.saveList(listRef.current.scrollTop);
      navigationPending.current = true;
    };
    const before = atlasRouter.subscribe("onBeforeNavigate", snapshot);
    const resolved = atlasRouter.subscribe("onResolved", () => {
      navigationPending.current = false;
    });
    return () => {
      before();
      resolved();
    };
  }, [saved, narrow, readerOpen]);
  useLayoutEffect(() => {
    if (listRef.current && (!narrow || !readerOpen))
      listRef.current.scrollTop = saved.readList();
  }, [resultsIdentity, saved, narrow, readerOpen]);
  function rememberOpener(target: EventTarget) {
    const element =
      target instanceof Element
        ? target.closest<HTMLElement>("[data-record-key]")
        : null;
    if (element) {
      beforePreview.current = element;
    }
  }
  function closeReader() {
    onReaderClose?.();
    setClosedRequest(reading.request);
    restoreResultsFocus();
  }
  function restoreResultsFocus() {
    requestAnimationFrame(() => {
      const trigger = beforePreview.current?.isConnected
        ? beforePreview.current
        : (listRef.current?.querySelector<HTMLElement>('[aria-current="true"]') ??
          listRef.current?.querySelector<HTMLElement>("[data-record-key]") ??
          listRef.current?.closest<HTMLElement>(".reference-results"));
      trigger?.focus({ preventScroll: true });
    });
  }
  function clearSelectionAction() {
    return onClearSelection && (selectedRecordKey || reading.request) ? (
      <PaneIconButton
        label="Clear selection"
        icon={<X size={16} aria-hidden="true" />}
        disabled={clearSelectionDisabled}
        onClick={() => {
          onClearSelection();
          restoreResultsFocus();
        }}
      />
    ) : undefined;
  }
  function toolbar(actions: ReactNode) {
    return (
      <BrowsePaneHeader
        className={
          narrow
            ? "reference-browser__toolbar reference-results__header"
            : "reference-browser__toolbar"
        }
      >
        {resultsHeader}
        <div className="reference-browser__reader-actions">{actions}</div>
      </BrowsePaneHeader>
    );
  }
  const results = (
    <section
      className="reference-results"
      tabIndex={-1}
      aria-label={resultsLabel}
      onClickCapture={(event) => {
        if (isPrimaryActivation(event)) rememberOpener(event.target);
      }}
      onKeyDownCapture={(event) => {
        if ((event.key === " " || event.key === "Enter") && isPrimaryActivation(event))
          rememberOpener(event.target);
      }}
    >
      {!renderResultsPaneHeader &&
        narrow &&
        (resultsHeader || clearSelectionAction()) &&
        toolbar(clearSelectionAction())}
      <div
        className="reference-results__scroll"
        ref={listRef}
        aria-busy={paneHeader ? undefined : resultsLoading}
        onScroll={(event) => {
          if (
            layoutActive.current &&
            !navigationPending.current &&
            (!narrow || !readerOpen)
          )
            saved.saveList(event.currentTarget.scrollTop);
        }}
      >
        {renderResultsPaneHeader ? (
          <>
            <div
              className="reference-results__controls"
              data-narrow={narrow || undefined}
            >
              {renderResultsPaneHeader(narrow ? clearSelectionAction() : null)}
            </div>
            <div aria-busy={resultsLoading}>{children}</div>
          </>
        ) : (
          children
        )}
      </div>
    </section>
  );
  const reader = (
    <ReferenceReader
      reading={reading}
      selectedRecordKey={selectedRecordKey}
      recordActions={recordActions}
      readerContext={readerContext}
      status={readerStatus}
      readerFooter={readerFooter}
      positions={saved}
      onResults={narrow ? closeReader : undefined}
      active={!narrow || readerOpen}
      selectionActions={clearSelectionAction()}
      selectionActionsInIdentity={paneHeader && !narrow}
      renderLayout={
        narrow && !paneHeader
          ? undefined
          : (actions, content, hasReaderActions) => (
              <>
                {!paneHeader && toolbar(actions)}
                <Splitter
                  styles={{
                    root: {
                      flex: 1,
                      minHeight: 0,
                      overflow: "clip",
                      display: narrow ? "block" : undefined,
                    },
                    panel: { overflow: "clip" },
                  }}
                >
                  <Splitter.Panel
                    defaultSize="35%"
                    min="25%"
                    max="55%"
                    resizable={!narrow}
                    style={
                      narrow
                        ? { height: "100%", display: readerOpen ? "none" : "block" }
                        : undefined
                    }
                  >
                    {results}
                  </Splitter.Panel>
                  <Splitter.Panel
                    min="40%"
                    resizable={!narrow}
                    style={
                      narrow
                        ? { height: "100%", display: readerOpen ? "block" : "none" }
                        : undefined
                    }
                  >
                    {paneHeader ? (
                      <div className="reference-reader-column">
                        {hasReaderActions && (
                          <BrowsePaneHeader className="reference-reader__header">
                            {actions}
                          </BrowsePaneHeader>
                        )}
                        {content}
                      </div>
                    ) : (
                      content
                    )}
                  </Splitter.Panel>
                </Splitter>
              </>
            )
      }
    />
  );
  return (
    <div
      className="reference-browser"
      data-reading={narrow && readerOpen ? "true" : undefined}
    >
      {narrow && !paneHeader ? (
        <>
          <div className="reference-browser__pane" hidden={readerOpen}>
            {results}
          </div>
          <div className="reference-browser__pane" hidden={!readerOpen}>
            {reader}
          </div>
        </>
      ) : (
        reader
      )}
    </div>
  );
}
