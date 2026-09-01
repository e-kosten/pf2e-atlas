import { Table } from "antd";
import type { ColumnsType } from "antd/es/table";
import type { ResultWindowRow } from "../../generated/atlas";
import { RecordSurface } from "../../shared/records/RecordSurface";
import { handleResultKeyboard, useActiveResultScroll } from "./resultKeyboard";
import type { SearchWorkspaceState } from "./useSearchWorkspace";

export function ResultTable({ workspace }: { workspace: SearchWorkspaceState }) {
  const scrollRef = useActiveResultScroll<HTMLDivElement>(workspace.activeResultKey);
  const rows = workspace.resultPage?.rows ?? [];
  const columns: ColumnsType<ResultWindowRow> = [
    {
      key: "record",
      render: (_value, row) => (
        <RecordSurface
          onReference={(recordKey) => workspace.selectRecord(recordKey)}
          surface={row.record.surface}
        />
      ),
    },
  ];
  return (
    <section className="results-panel">
      <div
        aria-label="Results"
        aria-busy={workspace.resultsLoading || workspace.resultsRefreshing}
        className="results-scroll"
        ref={scrollRef}
      >
        <Table<ResultWindowRow>
          aria-label="Results"
          className="result-table result-table--record-surfaces"
          columns={columns}
          dataSource={rows}
          loading={workspace.resultsLoading}
          locale={{ emptyText: "No results" }}
          onRow={(row) => {
            const recordKey = row.record.surface.metadata.record_key;
            return {
              "data-active-result":
                recordKey === workspace.activeResultKey ? "true" : undefined,
              onClick: () => recordKey && workspace.selectRecord(recordKey),
              onFocus: () => recordKey && workspace.focusResult(recordKey),
              onKeyDown: (event) => {
                if (event.key === " ") {
                  event.preventDefault();
                  if (recordKey) {
                    workspace.selectRecord(recordKey);
                  }
                  return;
                }
                handleResultKeyboard(event, workspace);
              },
              onMouseEnter: () => recordKey && workspace.focusResult(recordKey),
              role: "button",
              tabIndex: 0,
            };
          }}
          pagination={false}
          rowClassName={(row) =>
            row.record.surface.metadata.record_key === workspace.activeResultKey
              ? "result-row ant-table-row-selected"
              : "result-row"
          }
          rowKey={(row) =>
            row.record.surface.metadata.record_key ?? row.record.surface.metadata.title
          }
          showHeader={false}
          size="middle"
        />
      </div>
    </section>
  );
}
