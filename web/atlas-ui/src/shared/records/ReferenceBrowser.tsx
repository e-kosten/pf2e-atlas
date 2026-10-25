import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { Button, Empty, Popover, Spin } from "antd";
import { useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import type {
  RelationshipClauseSummaryView,
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
  relationshipClauses,
  filter,
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
  relationshipClauses?: RelationshipClauseSummaryView[];
  filter?: import("../../generated/atlas").QueryPredicate | null;
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
          relationshipClauses={relationshipClauses}
          filter={filter}
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
  relationshipClauses,
  filter,
  generation,
  onRefreshResults,
}: {
  row: SearchResultRow;
  relationshipClauses?: RelationshipClauseSummaryView[];
  filter?: import("../../generated/atlas").QueryPredicate | null;
  generation?: string;
  onRefreshResults?: () => Promise<void>;
  matchesOpen: boolean;
  onMatchesOpenChange: (open: boolean) => void;
  selected: boolean;
  disabled: boolean;
  onPreview: (anchor: HTMLElement) => void;
  onMatch: (navigation: RecordNavigationView, anchor: HTMLElement) => void;
}) {
  const setMatchesOpen = onMatchesOpenChange;
  const trigger = useRef<HTMLButtonElement>(null);
  const matches = row.matches.filter(
    (match) =>
      match.navigation.embedded_path.length ||
      (match.navigation.field && match.navigation.field !== "identity") ||
      match.navigation.passage,
  );
  const relationshipEvidence =
    generation && filter
      ? (row.relationship_evidence ?? []).filter((evidence) =>
          relationshipClauses?.some(
            (clause) => clause.clause_id === evidence.clause_id,
          ),
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
    <div className={`reference-result${selected ? " reference-result--selected" : ""}`}>
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
                  href={destinationHref(
                    recordDestination({
                      recordKey: match.navigation.record_key,
                      selection: match.navigation,
                    }),
                  )}
                  onClick={(event) => {
                    if (!isPrimaryActivation(event)) return;
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
                  const summary = relationshipClauses!.find(
                    (clause) => clause.clause_id === evidence.clause_id,
                  )!;
                  return (
                    <RelationshipEvidence
                      key={JSON.stringify([
                        generation,
                        summary.relationship,
                        row.record.record_key,
                      ])}
                      evidence={evidence}
                      filter={filter!}
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
                color: "var(--muted)",
                outlineColor: "var(--text)",
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
