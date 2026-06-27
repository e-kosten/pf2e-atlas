import type { ResultWindowRow } from "../../generated/atlas";
import { RecordSurface } from "../../shared/records/RecordSurface";
import { handleResultKeyboard, useActiveResultScroll } from "./resultKeyboard";
import type { SearchWorkspaceState } from "./useSearchWorkspace";

export function ResultTable({ workspace }: { workspace: SearchWorkspaceState }) {
  const scrollRef = useActiveResultScroll<HTMLDivElement>(workspace.activeResultKey);
  const rows = workspace.resultPage?.rows ?? [];
  return (
    <section className="results-panel">
      <div
        aria-label="Results"
        aria-busy={workspace.resultsLoading || workspace.resultsRefreshing}
        className="results-scroll results-scroll--focusable"
        onKeyDown={(event) => handleResultKeyboard(event, workspace)}
        ref={scrollRef}
        role="listbox"
        tabIndex={0}
      >
        {workspace.resultsLoading ? (
          <div className="detail-empty">Loading results...</div>
        ) : (
          <div className="result-list">
            {rows.map((row) => (
              <ResultRow
                active={row.record.record_key === workspace.activeResultKey}
                key={row.record.record_key}
                onFocus={() => workspace.focusResult(row.record.record_key)}
                onSelect={() => workspace.selectRecord(row.record.record_key)}
                row={row}
              />
            ))}
          </div>
        )}
      </div>
    </section>
  );
}

function ResultRow({
  active,
  onFocus,
  onSelect,
  row,
}: {
  active: boolean;
  onFocus: () => void;
  onSelect: () => void;
  row: ResultWindowRow;
}) {
  return (
    <div
      aria-selected={active}
      className={["result-rich-row", active ? "result-rich-row--active" : ""]
        .filter(Boolean)
        .join(" ")}
      data-active-result={active ? "true" : undefined}
      onClick={(event) => {
        if (
          event.target instanceof Element &&
          event.target.closest("button,a,input,select,textarea")
        ) {
          return;
        }
        onSelect();
      }}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelect();
        }
      }}
      onMouseEnter={onFocus}
      role="option"
      tabIndex={-1}
    >
      {row.surface ? (
        <RecordSurface
          surface={row.surface}
          onReference={(recordKey) => {
            if (recordKey === row.record.record_key) {
              onSelect();
            }
          }}
        />
      ) : (
        <FallbackRow row={row} />
      )}
    </div>
  );
}

function FallbackRow({ row }: { row: ResultWindowRow }) {
  return (
    <span className="row-link">
      <span>{row.record.title}</span>
      <small>{row.record.record_key}</small>
    </span>
  );
}
