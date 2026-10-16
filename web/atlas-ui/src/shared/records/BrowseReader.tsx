import { Grid, Splitter } from "antd";
import { useLayoutEffect, useRef, useState, type ReactNode } from "react";
import {
  ATLAS_ROUTE_CHANGE_EVENT,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import { ReferenceReader } from "./ReferenceReader";
import { useBrowsePositions, type BrowsePositions } from "./useBrowsePositions";
import { type ReferenceReading } from "./useReferenceReading";

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
  positions?: BrowsePositions;
  onResultsScroll?: () => void;
}) {
  const narrow = !Grid.useBreakpoint().lg;
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
    window.addEventListener(ATLAS_ROUTE_CHANGE_EVENT, snapshot);
    window.addEventListener("popstate", snapshot);
    return () => {
      window.removeEventListener(ATLAS_ROUTE_CHANGE_EVENT, snapshot);
      window.removeEventListener("popstate", snapshot);
    };
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
        if (shouldHandleAtlasRouteClick(event)) rememberOpener(event.target);
      }}
      onKeyDownCapture={(event) => {
        if (
          (event.key === " " || event.key === "Enter") &&
          shouldHandleAtlasRouteClick(event)
        )
          rememberOpener(event.target);
      }}
    >
      {resultsHeader && (
        <div className="reference-results__header">{resultsHeader}</div>
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
      positions={saved}
      onResults={narrow ? closeReader : undefined}
      active={!narrow || readerOpen}
    />
  );
  return (
    <div
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
