import { recordReadingIdentity } from "./useRetainedRecord";
import { Button, Space, Typography } from "antd";
import { ArrowLeft, ExternalLink } from "lucide-react";
import { useCallback, useEffect, useLayoutEffect, useRef, type ReactNode } from "react";
import {
  ATLAS_ROUTE_CHANGE_EVENT,
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import { useRecordPreview } from "./useRecordPreview";
import { RecordPreviewPopover } from "./RecordPreviewPopover";
import { RecordDetailPane } from "./RecordDetailPane";
import { useBrowsePositions, type BrowsePositions } from "./useBrowsePositions";
import { activityAnchor } from "./ownedNavigation";
import { rootNavigation, type ReferenceReading } from "./useReferenceReading";
import { usePreviewContentFocus } from "./usePreviewContentFocus";
import { landReadingTarget } from "./landReadingTarget";

export function ReferenceReader({
  reading,
  selectedRecordKey = null,
  readerActions,
  positions,
  onResults,
  active = true,
}: {
  reading: ReferenceReading;
  selectedRecordKey?: string | null;
  readerActions?: ReactNode;
  positions?: BrowsePositions;
  onResults?: () => void;
  active?: boolean;
}) {
  const preview = useRecordPreview();
  const localPositions = useBrowsePositions();
  const saved = positions ?? localPositions;
  const readerRef = useRef<HTMLElement>(null);
  const contentFocus = usePreviewContentFocus(active, readerRef);
  const backRef = useRef<HTMLButtonElement>(null);
  const address = JSON.stringify([
    recordReadingIdentity(
      reading.request?.navigation.record_key ?? null,
      reading.request?.navigation,
    ),
    reading.request?.match,
    reading.request?.landing,
  ]);
  const activeRef = useRef(active);
  const readyLanding = useRef<ReferenceReading["request"] | undefined>(undefined);
  useLayoutEffect(() => {
    activeRef.current = active;
  }, [active]);
  const previousVisit = useRef<
    | {
        request: ReferenceReading["request"];
        restoringHistory: boolean;
      }
    | undefined
  >(undefined);
  const restoredRequest = useRef<ReferenceReading["request"] | undefined>(undefined);
  const pendingPosition = useRef<
    | {
        request: ReferenceReading["request"];
        historical: boolean;
        scroll: number;
        focusHref: string | undefined;
      }
    | undefined
  >(undefined);
  const rememberPosition = useCallback(
    (anchor?: HTMLElement) => {
      // A loading DOM can clamp its scroll position before a landed historical
      // document is ready. Preserve the captured bookmark through that interval.
      if (!active || pendingPosition.current?.historical) return;
      const scroll = readerRef.current?.scrollTop ?? 0;
      saved.saveReader(address, scroll);
      if (pendingPosition.current) pendingPosition.current.scroll = scroll;
      if (anchor) {
        const href = anchor.closest("a")?.getAttribute("href");
        saved.saveReaderFocus(address, href);
        if (pendingPosition.current && href) pendingPosition.current.focusHref = href;
      }
    },
    [active, address, saved],
  );
  useLayoutEffect(() => {
    const snapshot = () => rememberPosition();
    window.addEventListener(ATLAS_ROUTE_CHANGE_EVENT, snapshot);
    window.addEventListener("popstate", snapshot);
    return () => {
      window.removeEventListener(ATLAS_ROUTE_CHANGE_EVENT, snapshot);
      window.removeEventListener("popstate", snapshot);
    };
  }, [rememberPosition]);
  useLayoutEffect(() => {
    if (
      previousVisit.current?.request === reading.request &&
      previousVisit.current.restoringHistory === reading.restoringHistory
    )
      return;
    previousVisit.current = {
      request: reading.request,
      restoringHistory: reading.restoringHistory,
    };
    // Capture before the renderer's landing effects or queued native scroll
    // events can replace this address's saved position.
    pendingPosition.current = {
      request: reading.request,
      historical: reading.restoringHistory,
      scroll: saved.readReader(address),
      focusHref: saved.readReaderFocus(address),
    };
    restoredRequest.current = undefined;
    readyLanding.current = undefined;
    if (readerRef.current) readerRef.current.scrollTop = pendingPosition.current.scroll;
  }, [reading.request, reading.restoringHistory, address, saved]);
  const restorePosition = useCallback(() => {
    const position = pendingPosition.current;
    const container = readerRef.current;
    if (!activeRef.current || !container || position?.request !== reading.request)
      return;
    container.scrollTop = position.scroll;
    if (position.focusHref)
      Array.from(container.querySelectorAll<HTMLAnchorElement>("a[href]"))
        .find((link) => link.getAttribute("href") === position.focusHref)
        ?.focus({ preventScroll: true });
    // Retain the bookmark for repeated renderer effects in StrictMode. Normal
    // user scrolls can update it again once the historical content is ready.
    position.historical = false;
    restoredRequest.current = reading.request;
  }, [reading.request]);
  const onLandingReady = useCallback(
    (target?: HTMLElement) => {
      if (previousVisit.current?.request !== reading.request) return;
      readyLanding.current = reading.request;
      if (reading.restoringHistory) restorePosition();
      else {
        if (activeRef.current && target && readerRef.current)
          landReadingTarget(target, readerRef.current);
        restoredRequest.current = reading.request;
      }
    },
    [reading.request, reading.restoringHistory, restorePosition],
  );
  useEffect(() => {
    if (
      reading.detail &&
      (!reading.landing || readyLanding.current === reading.request) &&
      restoredRequest.current !== reading.request
    )
      restorePosition();
    // Landed documents restore from onLandingReady, after an immediate activity
    // or asynchronously fetched owned ability has rendered the exact target.
  }, [active, reading.detail, reading.landing, reading.request, restorePosition]);
  const hasResults = onResults !== undefined;
  const hasRequest = reading.request !== null;
  useEffect(() => {
    if (hasResults && active && hasRequest)
      backRef.current?.focus({ preventScroll: true });
  }, [hasResults, active, hasRequest]);
  useEffect(() => {
    if (
      !reading.detail ||
      reading.detail.selected.owners.length ||
      !reading.landing?.owners.length ||
      !readerRef.current
    )
      return;
    const container = readerRef.current;
    let marked: HTMLElement | null = null;
    let timer: ReturnType<typeof setTimeout>;
    const mark = () => {
      const node = Array.from(
        container.querySelectorAll<HTMLElement>("[data-atlas-activity-anchor]"),
      ).find(
        (element) =>
          element.dataset.atlasActivityAnchor === activityAnchor(reading.landing!),
      );
      if (!node) return;
      marked = node.matches("h2,h3,h4,h5")
        ? node
        : (node.querySelector<HTMLElement>(
            "h2,h3,h4,h5, :scope > a, :scope > button",
          ) ?? node);
      marked.classList.add("reference-landing-highlight");
      timer = setTimeout(
        () => marked?.classList.remove("reference-landing-highlight"),
        2800,
      );
      observer.disconnect();
    };
    const observer = new MutationObserver(mark);
    observer.observe(container, { childList: true, subtree: true });
    mark();
    return () => {
      observer.disconnect();
      clearTimeout(timer);
      marked?.classList.remove("reference-landing-highlight");
    };
  }, [reading.detail, reading.landing]);
  const detail = reading.detail;
  const fullNavigation = detail?.selected ?? reading.request?.navigation;
  const fullRoute = fullNavigation
    ? {
        kind: "record" as const,
        recordKey: fullNavigation.record_key,
        selection:
          fullNavigation.owners.length ||
          fullNavigation.field ||
          fullNavigation.passage ||
          fullNavigation.spell_rank ||
          fullNavigation.source_fingerprint
            ? fullNavigation
            : undefined,
        landing: reading.landing ?? reading.request?.landing,
      }
    : null;
  return (
    <section className="reference-reader" aria-label="Record preview">
      <div className="reference-reader__header">
        {onResults ? (
          <Button ref={backRef} icon={<ArrowLeft size={16} />} onClick={onResults}>
            Results
          </Button>
        ) : (
          <Typography.Text type="secondary">Preview</Typography.Text>
        )}
        <Space wrap>
          {reading.canGoBack && (
            <Button
              onClick={() => {
                rememberPosition();
                reading.back();
              }}
              icon={<ArrowLeft size={14} />}
            >
              Back
            </Button>
          )}
          {selectedRecordKey &&
            reading.request &&
            (recordReadingIdentity(
              reading.request.navigation.record_key,
              reading.request.navigation,
            ) !== recordReadingIdentity(selectedRecordKey) ||
              reading.request.match ||
              reading.request.landing) && (
              <Button
                onClick={() => {
                  rememberPosition();
                  reading.select({ navigation: rootNavigation(selectedRecordKey) });
                }}
              >
                Return to selection
              </Button>
            )}
          {readerActions}
          {fullRoute && (
            <Button
              type="text"
              aria-label="Open full page"
              title="Open full page"
              icon={<ExternalLink size={16} />}
              href={atlasRoutePath(fullRoute)}
              onClick={(event) => {
                if (!shouldHandleAtlasRouteClick(event)) return;
                event.preventDefault();
                navigateToAtlasRoute(fullRoute);
              }}
            />
          )}
        </Space>
      </div>
      <section
        className="reference-reader__scroll"
        {...contentFocus}
        onScroll={() => rememberPosition()}
      >
        <RecordDetailPane
          detail={detail}
          loading={reading.loading}
          error={reading.error}
          onRetry={reading.retry}
          landing={reading.landing}
          onLandingReady={onLandingReady}
          onReference={(recordKey, anchor, selection) => {
            rememberPosition(anchor);
            preview.open(recordKey, anchor, selection);
          }}
          onRankChange={(spell_rank) => {
            if (detail) {
              rememberPosition();
              reading.changeRank({ ...detail.selected, spell_rank });
            }
          }}
          onOwnerReturn={(recordKey, landing, anchor) => {
            rememberPosition(anchor);
            reading.inspect({ navigation: rootNavigation(recordKey), landing });
          }}
        />
      </section>
      {preview.recordKey && (
        <RecordPreviewPopover
          anchor={preview.anchor}
          visibleOwnerRecordKey={
            detail?.selected.owners.length === 0 ? detail.record.record_key : undefined
          }
          detail={preview.detail}
          loading={preview.loading}
          error={preview.error}
          onRetry={preview.retry}
          onClose={preview.close}
          onReference={preview.open}
          onRankChange={preview.changeRank}
          readingHistory={preview.readingHistory}
          landing={preview.landing}
          onOpenFullPage={() => {
            navigateToAtlasRoute({
              kind: "record",
              recordKey: preview.recordKey!,
              selection: preview.selection,
              landing: preview.landing,
            });
            preview.close();
          }}
        />
      )}
    </section>
  );
}
