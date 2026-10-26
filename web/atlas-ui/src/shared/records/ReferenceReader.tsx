import { ReaderMatchNavigation } from "./ReaderMatchNavigation";
import type { RecordNavigationView } from "../../generated/atlas";
import {
  destinationHref,
  isPrimaryActivation,
  navigateDestination,
} from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { recordLayout } from "./recordLayout";
import { recordReadingIdentity } from "./useRetainedRecord";
import { Button, Space, Typography } from "antd";
import { ArrowLeft, ExternalLink } from "lucide-react";
import { useCallback, useEffect, useLayoutEffect, useRef, type ReactNode } from "react";
import { atlasRouter } from "../../app/router";
import { useRecordPreview } from "./useRecordPreview";
import { RecordReferenceDrawer } from "./RecordReferenceDrawer";
import { RecordDetailPane } from "./RecordDetailPane";
import { useBrowsePositions, type BrowsePositions } from "./useBrowsePositions";
import { activityAnchor } from "./embeddedNavigation";
import { rootNavigation, type ReferenceReading } from "./useReferenceReading";
import { usePreviewContentFocus } from "./usePreviewContentFocus";
import { landReadingTarget } from "./landReadingTarget";
import { BrowsePaneHeader } from "./BrowsePaneHeader";
export function ReferenceReader({
  reading,
  selectedRecordKey = null,
  readerActions,
  readerContext,
  status,
  readerFooter,
  positions,
  onResults,
  active = true,
}: {
  reading: ReferenceReading;
  selectedRecordKey?: string | null;
  readerContext?: ReactNode;
  readerActions?: ReactNode;
  status?: ReactNode;
  readerFooter?: ReactNode;
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
  const matchOpener = useRef<HTMLElement | null>(null);
  const focusMatchTarget = useCallback((target: HTMLElement) => {
    const opener = matchOpener.current;
    const container = readerRef.current;
    if (!opener || !container) return;
    const focused = container.ownerDocument.activeElement;
    if (
      focused === opener ||
      focused === container ||
      focused === container.ownerDocument.body
    ) {
      target.tabIndex = -1;
      target.focus({ preventScroll: true });
    }
    matchOpener.current = null;
  }, []);
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
  const inspect = reading.inspect;
  const navigateToMatch = useCallback(
    (navigation: RecordNavigationView, anchor: HTMLElement) => {
      rememberPosition(anchor);
      matchOpener.current = anchor;
      inspect({ navigation: rootNavigation(navigation.record_key), match: navigation });
    },
    [rememberPosition, inspect],
  );
  useLayoutEffect(() => {
    const snapshot = () => rememberPosition();
    return atlasRouter.subscribe("onBeforeNavigate", snapshot);
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
        if (activeRef.current && target && readerRef.current) {
          landReadingTarget(target, readerRef.current);
          focusMatchTarget(target);
        }
        restoredRequest.current = reading.request;
      }
    },
    [reading.request, reading.restoringHistory, restorePosition, focusMatchTarget],
  );
  useEffect(() => {
    if (
      reading.detail &&
      (!reading.landing || readyLanding.current === reading.request) &&
      restoredRequest.current !== reading.request
    )
      restorePosition();
    // Landed documents restore from onLandingReady, after an immediate activity
    // or asynchronously fetched embedded ability has rendered the exact target.
  }, [active, reading.detail, reading.landing, reading.request, restorePosition]);
  const hasResults = onResults !== undefined;
  const hasRequest = reading.request !== null;
  useEffect(() => {
    if (!hasResults || !active || !hasRequest) return;
    const focused = backRef.current?.ownerDocument.activeElement;
    // Preparing a selection can finish after the user starts editing the Add
    // form. Hidden results/search controls still hand focus to the reader.
    if (
      focused instanceof HTMLElement &&
      focused.matches("input, textarea, [contenteditable=true], [role=combobox]") &&
      focused.getClientRects().length > 0
    )
      return;
    backRef.current?.focus({ preventScroll: true });
  }, [hasResults, active, hasRequest]);
  useEffect(() => {
    if (
      !reading.detail ||
      reading.detail.selected.embedded_path.length ||
      !reading.landing?.embedded_path.length ||
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
  useEffect(() => {
    const detail = reading.detail;
    const target = reading.request?.match;
    if (
      !active ||
      !matchOpener.current ||
      !detail ||
      !target ||
      reading.landing ||
      recordReadingIdentity(detail.record.record_key, detail.selected) !==
        recordReadingIdentity(target.record_key, target)
    )
      return;
    const content =
      readerRef.current?.querySelector<HTMLElement>(".record-reference__content") ??
      readerRef.current?.querySelector<HTMLElement>(".record-workspace__reading");
    if (!content) return;
    content.classList.add("reference-landing-highlight");
    focusMatchTarget(content);
    const timer = setTimeout(
      () => content.classList.remove("reference-landing-highlight"),
      2800,
    );
    return () => {
      clearTimeout(timer);
      content.classList.remove("reference-landing-highlight");
    };
  }, [active, reading.detail, reading.request, reading.landing, focusMatchTarget]);
  const detail = reading.detail;
  const fullNavigation = detail?.selected ?? reading.request?.navigation;
  const fullRoute = fullNavigation
    ? recordDestination({
        recordKey: fullNavigation.record_key,
        selection:
          fullNavigation.embedded_path.length ||
          fullNavigation.field ||
          fullNavigation.passage ||
          fullNavigation.spell_rank ||
          fullNavigation.source_fingerprint
            ? fullNavigation
            : undefined,
        landing: reading.landing ?? reading.request?.landing,
      })
    : null;
  return (
    <section
      className={`reference-reader reference-reader--${recordLayout(detail?.presentation)}`}
      aria-label="Record preview"
    >
      <BrowsePaneHeader className="reference-reader__header">
        <div className="reference-reader__heading">
          {onResults ? (
            <Button ref={backRef} icon={<ArrowLeft size={16} />} onClick={onResults}>
              Results
            </Button>
          ) : (
            <Typography.Text type="secondary">Preview</Typography.Text>
          )}
          {status}
        </div>
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
              href={destinationHref(fullRoute)}
              onClick={(event) => {
                if (!isPrimaryActivation(event)) return;
                event.preventDefault();
                navigateDestination(fullRoute);
              }}
            />
          )}
        </Space>
      </BrowsePaneHeader>
      <section
        className="reference-reader__scroll"
        {...contentFocus}
        onScroll={() => rememberPosition()}
      >
        <ReaderMatchNavigation.Provider value={navigateToMatch}>
          <RecordDetailPane
            detail={detail}
            readingContext={detail && readerContext}
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
        </ReaderMatchNavigation.Provider>
      </section>
      {readerFooter}
      {preview.recordKey && (
        <RecordReferenceDrawer
          anchor={preview.anchor}
          visibleOwnerRecordKey={
            detail?.selected.embedded_path.length === 0
              ? detail.record.record_key
              : undefined
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
            navigateDestination(
              recordDestination({
                recordKey: preview.recordKey!,
                selection: preview.selection,
                landing: preview.landing,
              }),
            );
            preview.close();
          }}
        />
      )}
    </section>
  );
}
