import { Empty } from "antd";
import { LoadingStatus } from "../ui/LoadingStatus";
import type { ReactNode } from "react";
import type {
  RelationshipClauseSummaryView,
  RecordSummaryView,
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
import { SearchMatchContext } from "./SearchMatchContext";
export function ReferenceBrowser({
  rows,
  loading,
  selectedRecordKey,
  onSelect,
  reading,
  resultsHeader,
  recordActions,
  readerStatus,
  onReaderClose,
  emptyMessage = "No matching records",
  positions,
  disabled = false,
  relationshipClauses,
  filter,
  generation,
  onRefreshResults,
  query,
}: {
  rows: SearchResultRow[];
  query?: string;
  loading: boolean;
  selectedRecordKey: string | null;
  onSelect: (row: SearchResultRow, request: ReadingRequest) => void;
  reading: ReferenceReading;
  resultsHeader?: ReactNode;
  recordActions?: (record: RecordSummaryView) => ReactNode;
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
  const selected = rows.find((row) => row.record.record_key === selectedRecordKey);
  const detail = reading.detail;
  const showContext =
    selected &&
    detail &&
    detail.record.record_key === selected.record.record_key &&
    detail.artifact_generation === generation &&
    !detail.selected.embedded_path.length &&
    !detail.selected.field &&
    !detail.selected.passage;
  return (
    <BrowseReader
      reading={reading}
      selectedRecordKey={selectedRecordKey}
      resultsHeader={resultsHeader}
      resultsLabel="Search results"
      resultsIdentity={rows}
      resultsLoading={loading}
      recordActions={recordActions}
      readerStatus={readerStatus}
      onReaderClose={onReaderClose}
      positions={positions}
      readerContext={
        showContext ? (
          <SearchMatchContext
            row={selected}
            query={query}
            clauses={relationshipClauses}
            filter={filter}
            generation={generation}
            onRefresh={onRefreshResults}
          />
        ) : undefined
      }
    >
      {loading && !rows.length ? (
        <LoadingStatus className="reference-loading" label="Searching…" />
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
          disabled={disabled}
          onPreview={() => {
            onSelect(row, { navigation: rootNavigation(row.record.record_key) });
          }}
        />
      ))}
    </BrowseReader>
  );
}
