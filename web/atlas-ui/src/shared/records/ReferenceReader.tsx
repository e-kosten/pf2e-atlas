import { Button, Space, Typography } from "antd";
import { ArrowLeft, ExternalLink } from "lucide-react";
import { useEffect, useLayoutEffect, useRef, type ReactNode } from "react";
import {
  ATLAS_ROUTE_CHANGE_EVENT,
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import { RecordDetailPane } from "./RecordDetailPane";
import { useBrowsePositions, type BrowsePositions } from "./useBrowsePositions";
import { activityAnchor } from "./ownedNavigation";
import { rootNavigation, type ReferenceReading } from "./useReferenceReading";
import { usePreviewContentFocus } from "./usePreviewContentFocus";

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
  const localPositions = useBrowsePositions();
  const saved = positions ?? localPositions;
  const readerRef = useRef<HTMLElement>(null);
  const contentFocus = usePreviewContentFocus(active, readerRef);
  const backRef = useRef<HTMLButtonElement>(null);
  const address = JSON.stringify(reading.request);
  const previousAddress = useRef(address);
  const restoredAddress = useRef<string | null>(null);
  useLayoutEffect(() => {
    const snapshot = () => {
      if (active && readerRef.current)
        saved.saveReader(previousAddress.current, readerRef.current.scrollTop);
    };
    window.addEventListener(ATLAS_ROUTE_CHANGE_EVENT, snapshot);
    window.addEventListener("popstate", snapshot);
    return () => {
      window.removeEventListener(ATLAS_ROUTE_CHANGE_EVENT, snapshot);
      window.removeEventListener("popstate", snapshot);
    };
  }, [saved, active, readerRef]);
  useLayoutEffect(() => {
    if (previousAddress.current !== address && readerRef.current) {
      // The prior position was captured before navigation, while its DOM still existed.
      readerRef.current.scrollTop = saved.readReader(address);
      previousAddress.current = address;
      restoredAddress.current = null;
    }
  }, [address, saved, readerRef]);
  useEffect(() => {
    const container = readerRef.current;
    if (!reading.detail || !container || !active || restoredAddress.current === address)
      return;
    if (!reading.landing) {
      container.scrollTop = saved.readReader(address);
      const href = saved.readReaderFocus(address);
      if (href)
        Array.from(container.querySelectorAll<HTMLAnchorElement>("a[href]"))
          .find((link) => link.getAttribute("href") === href)
          ?.focus({ preventScroll: true });
    }
    restoredAddress.current = address;
  }, [address, reading.detail, reading.landing, saved, active, readerRef]);
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
                saved.saveReader(address, readerRef.current?.scrollTop ?? 0);
                reading.back();
              }}
              icon={<ArrowLeft size={14} />}
            >
              Back
            </Button>
          )}
          {selectedRecordKey &&
            reading.request &&
            (JSON.stringify(reading.request.navigation) !==
              JSON.stringify(rootNavigation(selectedRecordKey)) ||
              reading.request.match ||
              reading.request.landing) && (
              <Button
                onClick={() => {
                  saved.saveReader(address, readerRef.current?.scrollTop ?? 0);
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
        onScroll={(event) => {
          if (active) saved.saveReader(address, event.currentTarget.scrollTop);
        }}
      >
        <RecordDetailPane
          detail={detail}
          loading={reading.loading}
          error={reading.error}
          onRetry={reading.retry}
          landing={reading.landing}
          onReference={(recordKey, anchor, selection) => {
            saved.saveReader(address, readerRef.current?.scrollTop ?? 0);
            saved.saveReaderFocus(address, anchor?.closest("a")?.getAttribute("href"));
            reading.inspect({ navigation: selection ?? rootNavigation(recordKey) });
          }}
          onOwnerReturn={(recordKey, landing, anchor) => {
            saved.saveReader(address, readerRef.current?.scrollTop ?? 0);
            saved.saveReaderFocus(address, anchor.closest("a")?.getAttribute("href"));
            reading.inspect({ navigation: rootNavigation(recordKey), landing });
          }}
        />
      </section>
    </section>
  );
}
