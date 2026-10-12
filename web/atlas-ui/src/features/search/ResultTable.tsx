import { Table, Tag, Typography } from "antd";
import {
  atlasRoutePath,
  navigateToAtlasRoute,
  recordPath,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import type { ColumnsType } from "antd/es/table";
import type { ResultWindowRow } from "../../generated/atlas";
import { handleResultKeyboard, useActiveResultScroll } from "./resultKeyboard";
import type { SearchWorkspaceState } from "./useSearchWorkspace";

const RESULT_COLUMNS: ColumnsType<ResultWindowRow> = [
  {
    title: "Name",
    dataIndex: ["record", "title"],
    key: "title",
    width: 260,
    render: (_value, row) => (
      <span className="result-title">
        <Typography.Link
          strong
          href={recordPath(row.record.record_key)}
          onClick={(event) => {
            event.stopPropagation();
            if (!shouldHandleAtlasRouteClick(event)) return;
            event.preventDefault();
            navigateToAtlasRoute({ kind: "record", recordKey: row.record.record_key });
          }}
        >
          {row.record.title}
        </Typography.Link>
      </span>
    ),
  },
  {
    title: "Kind",
    dataIndex: ["record", "kind_label"],
    key: "kind",
    render: (_value, row) => row.record.kind_label || row.record.kind,
    width: 100,
  },
  {
    title: "Level",
    dataIndex: ["record", "level_label"],
    key: "level",
    render: (_value, row) => row.record.level_label ?? "",
    width: 70,
  },
  {
    title: "Traits",
    dataIndex: ["record", "traits"],
    key: "traits",
    width: 170,
    render: (_value, row) => <TraitTags row={row} />,
  },
  {
    title: "Source",
    key: "source",
    render: (_value, row) => row.record.publication ?? row.record.pack ?? "",
    width: 140,
  },
  {
    title: "Match",
    key: "match",
    width: 220,
    render: (_value, row) => (
      <div className="result-matches">
        {row.matches.map((match, i) => (
          <Typography.Link
            className="result-match-link"
            href={atlasRoutePath({
              kind: "record",
              recordKey: match.navigation.record_key,
              selection: match.navigation,
            })}
            key={i}
            onClick={(e) => {
              e.stopPropagation();
              if (!shouldHandleAtlasRouteClick(e)) return;
              e.preventDefault();
              navigateToAtlasRoute({
                kind: "record",
                recordKey: match.navigation.record_key,
                selection: match.navigation,
              });
            }}
          >
            {match.label || match.navigation.field || match.lane}
          </Typography.Link>
        ))}
      </div>
    ),
  },
];

export function ResultTable({ workspace }: { workspace: SearchWorkspaceState }) {
  const scrollRef = useActiveResultScroll<HTMLDivElement>(workspace.activeResultKey);
  const rows = workspace.resultPage?.rows ?? [];
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
          className="result-table"
          columns={RESULT_COLUMNS}
          dataSource={rows}
          loading={workspace.resultsLoading}
          locale={{ emptyText: "No results" }}
          onRow={(row) => ({
            "aria-selected": row.record.record_key === workspace.selectedRecordKey,
            "data-active-result":
              row.record.record_key === workspace.activeResultKey ? "true" : undefined,
            onClick: (event) => {
              if (!(event.target as Element).closest("button,input,label,a"))
                workspace.selectRecord(row.record.record_key);
            },
            onFocus: () => workspace.focusResult(row.record.record_key),
            onKeyDown: (event) => {
              if (event.target !== event.currentTarget) return;
              if (event.key === " ") {
                event.preventDefault();
                workspace.selectRecord(row.record.record_key);
                return;
              }
              handleResultKeyboard(event, workspace);
            },
            onMouseEnter: () => workspace.focusResult(row.record.record_key),
            tabIndex: 0,
          })}
          pagination={false}
          rowClassName={(row) =>
            row.record.record_key === workspace.selectedRecordKey
              ? "result-row result-row--selected"
              : "result-row"
          }
          rowKey={(row) => row.record.record_key}
          tableLayout="fixed"
          scroll={{ x: 1000 }}
          size="middle"
        />
      </div>
    </section>
  );
}

function TraitTags({ row }: { row: ResultWindowRow }) {
  const traits = row.record.traits ?? [];
  if (traits.length === 0) {
    return "";
  }
  const visibleTraits = traits.slice(0, 4);
  const hiddenCount = traits.length - visibleTraits.length;
  return (
    <span className="result-traits">
      {visibleTraits.map((trait) => (
        <Tag style={{ marginInlineEnd: 0 }} key={`${trait.kind}-${trait.value}`}>
          {trait.label}
        </Tag>
      ))}
      {hiddenCount > 0 && (
        <Typography.Text type="secondary">+{hiddenCount}</Typography.Text>
      )}
    </span>
  );
}
