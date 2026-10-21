import { Button, Empty, Popover, Spin, theme } from "antd";
import { useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import { atlasRoutePath, shouldHandleAtlasRouteClick } from "../../app/routes";
import type {
  RelationshipConstraintSummaryView,
  RecordNavigationView,
  SearchResultRow,
} from "../../generated/atlas";
import { type BrowsePositions } from "./useBrowsePositions";
import { rootNavigation, type ReferenceReading } from "./useReferenceReading";

import { BrowseReader } from "./BrowseReader";
import { RelationshipEvidence } from "./RelationshipEvidence";
import { RecordBrowseSummary } from "./RecordBrowseSummary";

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
  relationships,
  generation,
  onRefreshResults,
}: {
  rows: SearchResultRow[];
  loading: boolean;
  selectedRecordKey: string | null;
  onSelect: (row: SearchResultRow) => void;
  reading: ReferenceReading;
  resultsHeader?: ReactNode;
  readerActions?: ReactNode;
  emptyMessage?: string | null;
  positions?: BrowsePositions;
  disabled?: boolean;
  relationships?: RelationshipConstraintSummaryView[];
  generation?: string;
  onRefreshResults?: () => Promise<void>;
}) {
  const [openMatchKey, setOpenMatchKey] = useState<string | null>(null);
  return (
    <BrowseReader
      reading={reading}
      selectedRecordKey={selectedRecordKey}
      resultsHeader={resultsHeader}
      resultsLabel="Search results"
      resultsIdentity={rows}
      resultsLoading={loading}
      readerActions={readerActions}
      positions={positions}
      onResultsScroll={() => setOpenMatchKey(null)}
    >
      {loading && !rows.length ? (
        <div className="reference-loading" role="status" aria-label="Searching…">
          <Spin />
          <span>Searching…</span>
        </div>
      ) : !rows.length && emptyMessage ? (
        <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description={emptyMessage} />
      ) : null}
      {rows.map((row) => (
        <ReferenceResult
          key={row.record.record_key}
          row={row}
          relationships={relationships}
          generation={generation}
          onRefreshResults={onRefreshResults}
          selected={row.record.record_key === selectedRecordKey}
          matchesOpen={openMatchKey === row.record.record_key}
          onMatchesOpenChange={(open) =>
            setOpenMatchKey(open ? row.record.record_key : null)
          }
          disabled={disabled}
          onPreview={() => {
            onSelect(row);
            reading.select({ navigation: rootNavigation(row.record.record_key) });
          }}
          onMatch={(navigation) => {
            onSelect(row);
            reading.select({
              navigation: rootNavigation(navigation.record_key),
              match: navigation,
            });
          }}
        />
      ))}
    </BrowseReader>
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
  relationships,
  generation,
  onRefreshResults,
}: {
  row: SearchResultRow;
  relationships?: RelationshipConstraintSummaryView[];
  generation?: string;
  onRefreshResults?: () => Promise<void>;
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
      match.navigation.embedded_path.length ||
      (match.navigation.field && match.navigation.field !== "identity") ||
      match.navigation.passage,
  );
  const relationshipEvidence = generation
    ? (row.relationship_evidence ?? []).filter(
        (evidence) => relationships?.[evidence.constraint_index],
      )
    : [];
  const matchCount =
    BigInt(matches.length) +
    relationshipEvidence.reduce((count, evidence) => count + evidence.page.total, 0n);
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
  return (
    <div
      className="reference-result"
      style={{
        background: selected ? token.colorPrimaryBg : undefined,
        borderInlineStart: `3px solid ${selected ? token.colorPrimaryBorder : "transparent"}`,
      }}
    >
      <RecordBrowseSummary
        record={record}
        selected={selected}
        disabled={disabled}
        onPreview={onPreview}
      />
      {matchCount > 0n && (
        <Popover
          open={matchesOpen}
          onOpenChange={setMatchesOpen}
          trigger="click"
          placement="bottomLeft"
          destroyOnHidden
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
              {generation &&
                relationshipEvidence.map((evidence) => {
                  const summary = relationships![evidence.constraint_index];
                  return (
                    <RelationshipEvidence
                      key={JSON.stringify([
                        generation,
                        summary.constraint,
                        row.record.record_key,
                      ])}
                      evidence={evidence}
                      summary={summary}
                      resultKey={row.record.record_key}
                      generation={generation}
                      onRefresh={onRefreshResults}
                      onNavigate={(navigation, anchor) => {
                        setMatchesOpen(false);
                        onMatch(navigation, trigger.current ?? anchor);
                      }}
                    />
                  );
                })}
            </div>
          }
        >
          <Button
            ref={trigger}
            className="reference-result__matches"
            color="default"
            variant="text"
            size="small"
            aria-expanded={matchesOpen}
            aria-label={`${matchCount.toString()} matches in ${record.title}`}
            styles={{
              root: {
                color: token.colorTextDescription,
                outlineColor: token.colorText,
              },
            }}
            disabled={disabled}
          >
            Matches
          </Button>
        </Popover>
      )}
    </div>
  );
}
