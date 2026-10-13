import {
  Button,
  Empty,
  Grid,
  Popover,
  Space,
  Spin,
  Splitter,
  Tag,
  theme,
  Typography,
} from "antd";
import { ArrowLeft, ExternalLink } from "lucide-react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import {
  ATLAS_ROUTE_CHANGE_EVENT,
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import type { RecordNavigationView, SearchResultRow } from "../../generated/atlas";
import { sentenceCase } from "../ui/labels";
import { RecordDetailPane } from "./RecordDetailPane";
import { useBrowsePositions, type BrowsePositions } from "./useBrowsePositions";
import { activityAnchor } from "./ownedNavigation";
import { rootNavigation, type ReferenceReading } from "./useReferenceReading";

export function ReferenceBrowser({
  rows,
  loading,
  selectedRecordKey,
  onSelect,
  reading,
  resultsHeader,
  readerActions,
  emptyMessage = "No matching records",
  positions,
  disabled = false,
}: {
  rows: SearchResultRow[];
  loading: boolean;
  selectedRecordKey: string | null;
  onSelect: (row: SearchResultRow) => void;
  reading: ReferenceReading;
  resultsHeader?: ReactNode;
  readerActions?: ReactNode;
  emptyMessage?: string;
  positions?: BrowsePositions;
  disabled?: boolean;
}) {
  const narrow = !Grid.useBreakpoint().lg;
  const [openMatchKey, setOpenMatchKey] = useState<string | null>(null);
  const [closedAddress, setClosedAddress] = useState<string | null>(null);
  const localPositions = useBrowsePositions();
  const saved = positions ?? localPositions;
  const listRef = useRef<HTMLDivElement>(null);
  const readerRef = useRef<HTMLDivElement>(null);
  const backRef = useRef<HTMLButtonElement>(null);
  const beforePreview = useRef<HTMLElement | null>(null);
  const address = JSON.stringify(reading.request);
  const previousAddress = useRef(address);
  const readerOpen = reading.request !== null && closedAddress !== address;
  useLayoutEffect(() => {
    // Capture before route rendering removes scroll content. Native scroll events
    // may still be queued when the user immediately opens the full reference.
    const snapshot = () => {
      if (listRef.current && (!narrow || !readerOpen))
        saved.saveList(listRef.current.scrollTop);
      if (readerRef.current && (!narrow || readerOpen))
        saved.saveReader(previousAddress.current, readerRef.current.scrollTop);
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
  }, [rows, saved, narrow, readerOpen]);
  useLayoutEffect(() => {
    if (previousAddress.current !== address && readerRef.current) {
      saved.saveReader(previousAddress.current, readerRef.current.scrollTop);
      readerRef.current.scrollTop = saved.readReader(address);
      previousAddress.current = address;
    }
  }, [address, saved]);
  useEffect(() => {
    if (
      reading.detail &&
      readerRef.current &&
      !reading.landing &&
      (!narrow || readerOpen)
    )
      readerRef.current.scrollTop = saved.readReader(address);
  }, [address, reading.detail, reading.landing, saved, readerOpen, narrow]);
  useEffect(() => {
    if (narrow && readerOpen && reading.request)
      backRef.current?.focus({ preventScroll: true });
  }, [narrow, readerOpen, reading.request]);
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
  function closeReader() {
    setClosedAddress(address);
    requestAnimationFrame(() => {
      const trigger = beforePreview.current?.isConnected
        ? beforePreview.current
        : (listRef.current?.querySelector<HTMLElement>('[aria-current="true"]') ??
          listRef.current?.querySelector<HTMLElement>("[data-record-key]"));
      trigger?.focus({ preventScroll: true });
    });
  }
  const results = (
    <section className="reference-results" aria-label="Search results">
      <div className="reference-results__header">{resultsHeader}</div>
      <div
        className="reference-results__scroll"
        ref={listRef}
        aria-busy={loading}
        onScroll={(event) => {
          if (!narrow || !readerOpen) saved.saveList(event.currentTarget.scrollTop);
          setOpenMatchKey(null);
        }}
      >
        {loading && !rows.length ? (
          <Spin className="reference-loading" />
        ) : !rows.length ? (
          <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description={emptyMessage} />
        ) : null}
        {rows.map((row) => (
          <ReferenceResult
            key={row.record.record_key}
            row={row}
            selected={row.record.record_key === selectedRecordKey}
            matchesOpen={openMatchKey === row.record.record_key}
            onMatchesOpenChange={(open) =>
              setOpenMatchKey(open ? row.record.record_key : null)
            }
            disabled={disabled}
            onPreview={(anchor) => {
              beforePreview.current = anchor;
              onSelect(row);
              reading.inspect({ navigation: rootNavigation(row.record.record_key) });
              setClosedAddress(null);
            }}
            onMatch={(navigation, anchor) => {
              beforePreview.current = anchor;
              onSelect(row);
              reading.inspect({
                navigation: rootNavigation(navigation.record_key),
                match: navigation,
              });
              setClosedAddress(null);
            }}
          />
        ))}
      </div>
    </section>
  );
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
  const reader = (
    <section className="reference-reader" aria-label="Record preview">
      <div className="reference-reader__header">
        {narrow ? (
          <Button ref={backRef} icon={<ArrowLeft size={16} />} onClick={closeReader}>
            Results
          </Button>
        ) : (
          <Typography.Text type="secondary">Preview</Typography.Text>
        )}
        <Space>
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
      <div
        className="reference-reader__scroll"
        ref={readerRef}
        onScroll={(event) => {
          if (!narrow || readerOpen)
            saved.saveReader(address, event.currentTarget.scrollTop);
        }}
      >
        <RecordDetailPane
          detail={detail}
          loading={reading.loading}
          error={reading.error}
          onRetry={reading.retry}
          landing={reading.landing}
          onReference={(recordKey, _anchor, selection) =>
            reading.inspect({ navigation: selection ?? rootNavigation(recordKey) })
          }
          onOwnerReturn={(recordKey, landing) =>
            reading.inspect({ navigation: rootNavigation(recordKey), landing })
          }
        />
      </div>
    </section>
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

function ReferenceResult({
  row,
  selected,
  disabled,
  onPreview,
  onMatch,
  matchesOpen,
  onMatchesOpenChange,
}: {
  row: SearchResultRow;
  matchesOpen: boolean;
  onMatchesOpenChange: (open: boolean) => void;
  selected: boolean;
  disabled: boolean;
  onPreview: (anchor: HTMLElement) => void;
  onMatch: (navigation: RecordNavigationView, anchor: HTMLElement) => void;
}) {
  const { token } = theme.useToken();
  const setMatchesOpen = onMatchesOpenChange;
  const trigger = useRef<HTMLButtonElement>(null);
  const matches = row.matches.filter(
    (match) =>
      match.navigation.owners.length ||
      (match.navigation.field && match.navigation.field !== "identity") ||
      match.navigation.passage,
  );
  useEffect(() => {
    if (!matchesOpen) return;
    const escape = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      setMatchesOpen(false);
      trigger.current?.focus({ preventScroll: true });
    };
    document.addEventListener("keydown", escape, true);
    return () => document.removeEventListener("keydown", escape, true);
  }, [matchesOpen, setMatchesOpen]);
  const record = row.record;
  const classification = [
    record.kind_label,
    record.level_label?.replace(/^(Level|Rank)\s+/, ""),
  ]
    .filter(Boolean)
    .join(" ");
  const rarity =
    record.rarity && record.rarity !== "common" ? sentenceCase(record.rarity) : null;
  return (
    <div
      className="reference-result"
      style={{
        background: selected ? token.colorPrimaryBg : undefined,
        borderInlineStart: `3px solid ${selected ? token.colorPrimaryBorder : "transparent"}`,
      }}
    >
      <Button
        className="reference-result__summary"
        styles={{
          root: {
            height: "auto",
            whiteSpace: "normal",
            display: "grid",
            gridTemplateRows: "subgrid",
            gridTemplateColumns: "subgrid",
            gridRow: "1 / 4",
            gridColumn: "1 / -1",
            alignItems: "stretch",
            gap: 7,
            padding: "14px 16px 10px",
            borderRadius: 0,
            textAlign: "left",
          },
        }}
        data-record-key={record.record_key}
        href={atlasRoutePath({ kind: "record", recordKey: record.record_key })}
        block
        color={selected ? "primary" : "default"}
        variant="text"
        disabled={disabled}
        aria-current={selected ? "true" : undefined}
        onClick={(event) => {
          if (!shouldHandleAtlasRouteClick(event)) return;
          event.preventDefault();
          onPreview(event.currentTarget);
        }}
        onKeyDown={(event) => {
          if (
            event.key === "Enter" &&
            !event.ctrlKey &&
            !event.metaKey &&
            !event.altKey &&
            !event.shiftKey
          ) {
            event.preventDefault();
            navigateToAtlasRoute({ kind: "record", recordKey: record.record_key });
            return;
          }
          if (event.key === " ") {
            event.preventDefault();
            onPreview(event.currentTarget);
          }
          if (["ArrowDown", "ArrowUp", "j", "k"].includes(event.key)) {
            event.preventDefault();
            const links = Array.from(
              event.currentTarget
                .closest(".reference-results")
                ?.querySelectorAll<HTMLElement>("[data-record-key]") ?? [],
            );
            const index = links.indexOf(event.currentTarget);
            links[
              Math.max(
                0,
                Math.min(
                  links.length - 1,
                  index + (["ArrowDown", "j"].includes(event.key) ? 1 : -1),
                ),
              )
            ]?.focus();
          }
        }}
      >
        <div className="reference-result__title">
          <strong>{record.title}</strong>
          <span>{classification}</span>
        </div>
        <div className="reference-result__metadata">
          {rarity && <span>{rarity}</span>}
          <span>{record.publication ?? record.pack}</span>
        </div>
        {!!record.traits.length && (
          <div className="reference-result__traits">
            {record.traits
              .filter((trait) => trait.kind !== "rarity")
              .map((trait) => (
                <Tag key={trait.kind + ":" + trait.value}>
                  {sentenceCase(trait.label)}
                </Tag>
              ))}
          </div>
        )}
      </Button>
      {matches.length > 0 && (
        <Popover
          open={matchesOpen}
          onOpenChange={setMatchesOpen}
          trigger="click"
          placement="bottomLeft"
          content={
            <div
              className="reference-match-list"
              ref={(node) => {
                node
                  ?.querySelector<HTMLAnchorElement>("a")
                  ?.focus({ preventScroll: true });
              }}
            >
              {matches.map((match, index) => (
                <Button
                  key={index}
                  color="default"
                  variant="text"
                  block
                  styles={{
                    root: {
                      height: "auto",
                      whiteSpace: "normal",
                      justifyContent: "flex-start",
                    },
                  }}
                  href={atlasRoutePath({
                    kind: "record",
                    recordKey: match.navigation.record_key,
                    selection: match.navigation,
                  })}
                  onClick={(event) => {
                    if (!shouldHandleAtlasRouteClick(event)) return;
                    event.preventDefault();
                    setMatchesOpen(false);
                    onMatch(match.navigation, trigger.current ?? event.currentTarget);
                  }}
                >
                  {match.label || "Matching passage"}
                </Button>
              ))}
            </div>
          }
        >
          <Button
            ref={trigger}
            className="reference-result__matches"
            color="default"
            variant="link"
            size="small"
            aria-expanded={matchesOpen}
            aria-label={`${matches.length} matches in ${record.title}`}
            styles={{
              root: {
                color: token.colorText,
                outlineColor: token.colorText,
                textDecoration: "underline",
                textUnderlineOffset: 3,
              },
            }}
            disabled={disabled}
          >
            {matches.length} {matches.length === 1 ? "match" : "matches"}
          </Button>
        </Popover>
      )}
    </div>
  );
}
