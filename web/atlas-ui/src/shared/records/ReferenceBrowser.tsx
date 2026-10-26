import { Empty, Spin } from "antd";
import { useState } from "react";
import type { ReactNode } from "react";
import type {
  RelationshipClauseSummaryView,
  SearchResultRow,
} from "../../generated/atlas";
import { type BrowsePositions } from "./useBrowsePositions";
import {
  rootNavigation,
  type ReadingRequest,
  type ReferenceReading,
} from "./useReferenceReading";
import { BrowseReader } from "./BrowseReader";
import { ReferenceResult } from "./ReferenceResult";
export function ReferenceBrowser({
  rows,
  loading,
  selectedRecordKey,
  onSelect,
  reading,
  resultsHeader,
  readerActions,
  readerStatus,
  onReaderClose,
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
  onSelect: (row: SearchResultRow, request: ReadingRequest) => void;
  reading: ReferenceReading;
  resultsHeader?: ReactNode;
  readerActions?: ReactNode;
  readerStatus?: ReactNode;
  onReaderClose?: () => void;
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
      readerStatus={readerStatus}
      onReaderClose={onReaderClose}
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
            onSelect(row, { navigation: rootNavigation(row.record.record_key) });
          }}
          onMatch={(navigation) => {
            onSelect(row, {
              navigation: rootNavigation(navigation.record_key),
              match: navigation,
            });
          }}
        />
      ))}
    </BrowseReader>
  );
}
