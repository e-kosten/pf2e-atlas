import { isPrimaryActivation } from "../../app/navigation";
import { Grid, Splitter } from "antd";
import { useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { atlasRouter } from "../../app/router";
import { ReferenceReader } from "./ReferenceReader";
import { useBrowsePositions, type BrowsePositions } from "./useBrowsePositions";
import { type ReferenceReading } from "./useReferenceReading";
import { BrowsePaneHeader } from "./BrowsePaneHeader";
import { useBrowseHeaderAlignment } from "./useBrowseHeaderAlignment";
// The browse host owns layout, scroll and mobile return. Features own rows,
// queries, selection and mutation semantics; no search DTO is required here.
export function BrowseReader({
  children,
  resultsHeader,
  resultsLabel,
  resultsIdentity,
  resultsLoading,
  selectedRecordKey,
  reading,
  readerActions,
  readerStatus,
  onReaderClose,
  readerFooter,
  positions,
  onResultsScroll,
}: {
  children: ReactNode;
  resultsHeader?: ReactNode;
  resultsLabel: string;
  resultsIdentity: unknown;
  resultsLoading: boolean;
  selectedRecordKey: string | null;
  reading: ReferenceReading;
  readerActions?: ReactNode;
  readerStatus?: ReactNode;
  onReaderClose?: () => void;
  readerFooter?: ReactNode;
  positions?: BrowsePositions;
  onResultsScroll?: () => void;
}) {
  const narrow = !Grid.useBreakpoint().lg;
  const root = useRef<HTMLDivElement>(null);
  useBrowseHeaderAlignment(root, !narrow, Boolean(resultsHeader));
  const [closedRequest, setClosedRequest] = useState<ReferenceReading["request"]>();
  const localPositions = useBrowsePositions();
  const saved = positions ?? localPositions;
  const listRef = useRef<HTMLDivElement>(null);
  const beforePreview = useRef<HTMLElement | null>(null);
  const readerOpen = reading.request !== null && closedRequest !== reading.request;
  useLayoutEffect(() => {
    // Capture before route rendering removes scroll content. Native scroll events
    // may still be queued when the user immediately opens the full reference.
    const snapshot = () => {
      if (listRef.current && (!narrow || !readerOpen))
        saved.saveList(listRef.current.scrollTop);
    };
    return atlasRouter.subscribe("onBeforeNavigate", snapshot);
  }, [saved, narrow, readerOpen]);
  useLayoutEffect(() => {
    if (listRef.current && (!narrow || !readerOpen))
      listRef.current.scrollTop = saved.readList();
  }, [resultsIdentity, saved, narrow, readerOpen]);
  function rememberOpener(target: EventTarget) {
    const element =
      target instanceof Element
        ? target.closest<HTMLElement>("[data-record-key],.reference-result__matches")
        : null;
    if (element) {
      beforePreview.current = element;
    }
  }
  function closeReader() {
    onReaderClose?.();
    setClosedRequest(reading.request);
    requestAnimationFrame(() => {
      const trigger = beforePreview.current?.isConnected
        ? beforePreview.current
        : (listRef.current?.querySelector<HTMLElement>('[aria-current="true"]') ??
          listRef.current?.querySelector<HTMLElement>("[data-record-key]"));
      trigger?.focus({ preventScroll: true });
    });
  }
  const results = (
    <section
      className="reference-results"
      aria-label={resultsLabel}
      onClickCapture={(event) => {
        if (isPrimaryActivation(event)) rememberOpener(event.target);
      }}
      onKeyDownCapture={(event) => {
        if ((event.key === " " || event.key === "Enter") && isPrimaryActivation(event))
          rememberOpener(event.target);
      }}
    >
      {resultsHeader && (
        <BrowsePaneHeader className="reference-results__header">
          {resultsHeader}
        </BrowsePaneHeader>
      )}
      <div
        className="reference-results__scroll"
        ref={listRef}
        aria-busy={resultsLoading}
        onScroll={(event) => {
          if (!narrow || !readerOpen) saved.saveList(event.currentTarget.scrollTop);
          onResultsScroll?.();
        }}
      >
        {children}
      </div>
    </section>
  );
  const reader = (
    <ReferenceReader
      reading={reading}
      selectedRecordKey={selectedRecordKey}
      readerActions={readerActions}
      status={readerStatus}
      readerFooter={readerFooter}
      positions={saved}
      onResults={narrow ? closeReader : undefined}
      active={!narrow || readerOpen}
    />
  );
  return (
    <div
      ref={root}
      className="reference-browser"
      data-reading={narrow && readerOpen ? "true" : undefined}
    >
      {narrow ? (
        <>
          <div hidden={readerOpen}>{results}</div>
          <div hidden={!readerOpen}>{reader}</div>
        </>
      ) : (
        <Splitter styles={{ root: { overflow: "clip" }, panel: { overflow: "clip" } }}>
          <Splitter.Panel defaultSize="35%" min="25%" max="55%">
            {results}
          </Splitter.Panel>
          <Splitter.Panel min="40%">{reader}</Splitter.Panel>
        </Splitter>
      )}
    </div>
  );
}
