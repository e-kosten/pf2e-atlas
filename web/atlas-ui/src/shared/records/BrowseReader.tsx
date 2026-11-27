import { useAtlasRouter } from "../../app/router";
import { isPrimaryActivation } from "../../app/navigation";
import { Button, Grid, Splitter } from "antd";
import { X } from "lucide-react";
import { PaneIconButton } from "../ui/actions/PaneAction";
import { useCallback, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { ReferenceReader, type ReaderFooter } from "./ReferenceReader";
import { useBrowsePositions, type BrowsePositions } from "./useBrowsePositions";
import { type ReferenceReading } from "./useReferenceReading";
import type { RecordSummaryView } from "../../generated/atlas";
import { BrowsePaneHeader } from "./BrowsePaneHeader";
// The browse host owns layout, scroll and mobile return. Features own rows,
// queries, selection and mutation semantics; no search DTO is required here.
export function BrowseReader({
  children,
  resultsHeader,
  resultsLeadIn,
  renderResultsPaneHeader,
  resultsLabel,
  resultsIdentity,
  resultsLoading,
  selectedRecordKey,
  reading,
  recordActions,
  readerStatus,
  readerEmptyMessage,
  readerContext,
  onReaderClose,
  onClearSelection,
  clearSelectionDisabled,
  narrowClearPlacement,
  removedSelectionKey,
  readerFooter,
  positions,
  fullWidthUntilSelection = false,
  observeWidth = false,
  showRelated,
  returnLabel,
}: {
  children: ReactNode | ((rowAction: ReactNode) => ReactNode);
  resultsHeader?: ReactNode;
  resultsLeadIn?: ReactNode;
  renderResultsPaneHeader?: (actions: ReactNode) => ReactNode;
  resultsLabel: string;
  resultsIdentity: unknown;
  resultsLoading: boolean;
  selectedRecordKey: string | null;
  reading: ReferenceReading;
  readerContext?: ReactNode;
  recordActions?: (record: RecordSummaryView) => ReactNode;
  readerStatus?: ReactNode;
  readerEmptyMessage?: string | null;
  onReaderClose?: () => void;
  onClearSelection?: () => void;
  clearSelectionDisabled?: boolean;
  narrowClearPlacement?: "row" | "header";
  removedSelectionKey?: string;
  readerFooter?: ReaderFooter;
  positions?: BrowsePositions;
  fullWidthUntilSelection?: boolean;
  observeWidth?: boolean;
  showRelated?: boolean;
  returnLabel?: string;
}) {
  const atlasRouter = useAtlasRouter();
  const paneHeader = Boolean(renderResultsPaneHeader);
  const viewportNarrow = !Grid.useBreakpoint().lg;
  const browserRef = useRef<HTMLDivElement>(null);
  const [hostNarrow, setHostNarrow] = useState<boolean>();
  useLayoutEffect(() => {
    const host = browserRef.current;
    if (!observeWidth || !host) return;
    const measure = () => {
      if (host.clientWidth) setHostNarrow(host.clientWidth < 768);
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(host);
    return () => observer.disconnect();
  }, [observeWidth]);
  const narrow = observeWidth ? (hostNarrow ?? viewportNarrow) : viewportNarrow;
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
  const listOnly = fullWidthUntilSelection && !readerOpen;
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
  }, [saved, narrow, readerOpen, atlasRouter]);
  useLayoutEffect(() => {
    if (listRef.current && (!narrow || !readerOpen))
      listRef.current.scrollTop = saved.readList();
  }, [resultsIdentity, saved, narrow, readerOpen]);
  function rememberOpener(target: EventTarget) {
    const element =
      target instanceof Element
        ? target.closest<HTMLElement>("[data-record-key], [data-reader-opener]")
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
  const restoreResultsFocus = useCallback((removedKey?: string) => {
    requestAnimationFrame(() => {
      const original = beforePreview.current;
      const canFocus = (row: HTMLElement | null) =>
        row?.isConnected && (!removedKey || row.dataset.recordKey !== removedKey);
      const remaining =
        listRef.current?.querySelectorAll<HTMLElement>("[data-record-key]") ?? [];
      const trigger = canFocus(original)
        ? original
        : (Array.from(remaining).find(
            (row) => canFocus(row) && row.getAttribute("aria-current") === "true",
          ) ??
          Array.from(remaining).find(canFocus) ??
          listRef.current?.closest<HTMLElement>(".reference-results"));
      trigger?.focus({ preventScroll: true });
    });
  }, []);
  const handledRemoval = useRef<string | undefined>(undefined);
  useLayoutEffect(() => {
    if (!removedSelectionKey) handledRemoval.current = undefined;
    else if (
      !selectedRecordKey &&
      !readerOpen &&
      handledRemoval.current !== removedSelectionKey
    ) {
      handledRemoval.current = removedSelectionKey;
      restoreResultsFocus(removedSelectionKey);
    }
  }, [readerOpen, removedSelectionKey, restoreResultsFocus, selectedRecordKey]);
  const selectionActionsInIdentity = paneHeader && !narrow;
  function clearSelectionAction(labeled = false) {
    if (
      !onClearSelection ||
      !(selectedRecordKey || reading.request) ||
      (narrow && returnLabel)
    )
      return undefined;
    const clear = () => {
      onClearSelection();
      restoreResultsFocus();
    };
    if (labeled) {
      return (
        <Button size="small" disabled={clearSelectionDisabled} onClick={clear}>
          Clear selection
        </Button>
      );
    }
    return (
      <PaneIconButton
        outlined={selectionActionsInIdentity}
        label="Clear selection"
        icon={<X size={16} aria-hidden="true" />}
        disabled={clearSelectionDisabled}
        onClick={clear}
      />
    );
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
  const resultsOnlyClear = narrow && Boolean(narrowClearPlacement);
  const rowClear = resultsOnlyClear && narrowClearPlacement === "row";
  const resultsAction = rowClear ? null : clearSelectionAction(resultsOnlyClear);
  const rows =
    typeof children === "function"
      ? children(rowClear ? clearSelectionAction(true) : null)
      : children;
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
        (resultsHeader || resultsAction) &&
        toolbar(resultsAction)}
      <div
        className="reference-results__scroll"
        ref={listRef}
        aria-busy={paneHeader || resultsLeadIn ? undefined : resultsLoading}
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
              {renderResultsPaneHeader(narrow ? resultsAction : null)}
            </div>
            {resultsLeadIn}
            <div aria-busy={resultsLoading}>{rows}</div>
          </>
        ) : resultsLeadIn ? (
          <>
            {resultsLeadIn}
            <div aria-busy={resultsLoading}>{rows}</div>
          </>
        ) : (
          rows
        )}
      </div>
    </section>
  );
  const reader = (
    <ReferenceReader
      emptyMessage={readerEmptyMessage}
      reading={reading}
      selectedRecordKey={selectedRecordKey}
      recordActions={recordActions}
      readerContext={readerContext}
      status={readerStatus}
      readerFooter={readerFooter}
      positions={saved}
      onResults={narrow ? closeReader : undefined}
      resultsReturnLabel={returnLabel}
      showRelated={showRelated}
      active={(!narrow || readerOpen) && !listOnly}
      selectionActions={resultsOnlyClear ? undefined : clearSelectionAction()}
      selectionActionsInIdentity={selectionActionsInIdentity}
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
                    resizable={!narrow && !listOnly}
                    style={
                      narrow
                        ? { height: "100%", display: readerOpen ? "none" : "block" }
                        : listOnly
                          ? { flexBasis: "100%" }
                          : undefined
                    }
                  >
                    {results}
                  </Splitter.Panel>
                  <Splitter.Panel
                    min="40%"
                    resizable={!narrow && !listOnly}
                    style={
                      narrow
                        ? { height: "100%", display: readerOpen ? "block" : "none" }
                        : listOnly
                          ? { display: "none" }
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
      ref={browserRef}
      className="reference-browser"
      data-list-only={listOnly || undefined}
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
