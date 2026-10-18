import { Alert, Button, Space, Table, Typography } from "antd";
import { useInfiniteQuery } from "@tanstack/react-query";
import { useCallback, useEffect, useRef, useState } from "react";
import { atlasRoutePath, shouldHandleAtlasRouteClick } from "../../app/routes";
import { getOwnedReading } from "../../api/atlasApi";
import type {
  OwnedReadingEntryView,
  OwnedReadingSelection,
  RecordNavigationView,
  RollTablePresentationView,
  TableRollView,
} from "../../generated/atlas";
import { PreparedContent, type RecordReferenceHandler } from "./PreparedContent";
import type { OwnedReadingContext } from "./OwnedAbilityReading";
import { TableRollControl } from "./TableRollControl";
import "../../styles/ui/roll-table.css";

function address(navigation: RecordNavigationView) {
  return JSON.stringify(navigation.owners);
}

export function RollTableReading({
  table,
  selection,
  context,
  onReference,
}: {
  table: RollTablePresentationView;
  selection: RecordNavigationView;
  context: OwnedReadingContext;
  onReference: RecordReferenceHandler;
}) {
  const [result, setResult] = useState<TableRollView>();
  const [highlight, setHighlight] = useState<string[]>([]);
  const reading = useInfiniteQuery({
    queryKey: [
      "owned-reading",
      context.generation,
      selection.record_key,
      "table-results",
    ],
    initialPageParam: { kind: "table_results", cursor: null } as OwnedReadingSelection,
    queryFn: ({ pageParam, signal }) =>
      getOwnedReading(
        {
          parent: context.parent,
          artifact_generation: context.generation,
          selection: pageParam,
        },
        signal,
      ),
    getNextPageParam: (last) =>
      last.next
        ? ({ kind: "table_results", cursor: last.next } as OwnedReadingSelection)
        : undefined,
    staleTime: Infinity,
    retry: false,
  });
  const highlightTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  useEffect(() => () => window.clearTimeout(highlightTimer.current), []);
  const receiveResult = useCallback((next: TableRollView | undefined) => {
    setResult(next);
    setHighlight(next?.matches.map((match) => address(match.selected)) ?? []);
    window.clearTimeout(highlightTimer.current);
    if (next) highlightTimer.current = window.setTimeout(() => setHighlight([]), 2400);
  }, []);
  const entries = reading.data?.pages.flatMap((page) => page.entries) ?? [];
  const rows = table.results.value ?? [];
  const matches = result?.matches ?? [];
  const data = entries.map((entry) => ({
    entry,
    row: rows.find((row) => address(row.link.navigation) === address(entry.navigation)),
  }));
  return (
    <Space orientation="vertical" size="middle" style={{ width: "100%" }}>
      <TableRollControl
        selection={{
          record_key: selection.record_key,
          owners: selection.owners,
          fields: [],
          passage: null,
          source_fingerprint: selection.source_fingerprint,
        }}
        artifactGeneration={context.generation}
        formula={table.formula.value}
        onResult={receiveResult}
      />
      {matches.map((match) => (
        <section key={address(match.selected)} className="roll-table-result-summary">
          <strong>{match.presentation.identity.title}</strong>
          <TableResultContent
            entry={{
              navigation: match.selected,
              content: match.presentation.content,
              relationships: match.relationships,
            }}
            onReference={onReference}
          />
        </section>
      ))}
      <Table
        dataSource={data}
        rowKey={(row) => address(row.entry.navigation)}
        pagination={false}
        size="small"
        loading={{ spinning: reading.isFetching && entries.length === 0, delay: 300 }}
        rowClassName={(row) =>
          highlight.includes(address(row.entry.navigation))
            ? "roll-table-result--highlight"
            : ""
        }
        columns={[
          {
            title: "Range",
            width: 90,
            render: (_, { row }) => {
              const range = row?.range.value;
              return range
                ? range[0] === range[1]
                  ? String(range[0])
                  : `${range[0]}–${range[1]}`
                : "Unavailable";
            },
          },
          {
            title: "Result",
            render: (_, { entry }) => (
              <TableResultContent entry={entry} onReference={onReference} />
            ),
          },
        ]}
      />
      {reading.isError && (
        <Alert
          type="error"
          title="Couldn’t load table results."
          action={<Button onClick={() => void reading.refetch()}>Retry</Button>}
        />
      )}
      {reading.hasNextPage && (
        <Button
          loading={reading.isFetchingNextPage}
          onClick={() => void reading.fetchNextPage()}
        >
          Load more results
        </Button>
      )}
    </Space>
  );
}

function TableResultContent({
  entry,
  onReference,
}: {
  entry: OwnedReadingEntryView;
  onReference: RecordReferenceHandler;
}) {
  const links = entry.relationships.filter(
    (link) => link.kind === "TableResult" && link.target,
  );
  const label = simpleResultLabel(entry);
  if (links.length === 1 && label) {
    const target = links[0].target!;
    return (
      <Typography.Link
        href={atlasRoutePath({
          kind: "record",
          recordKey: target.record_key,
          selection: target,
        })}
        onClick={(event) => {
          if (!shouldHandleAtlasRouteClick(event)) return;
          event.preventDefault();
          onReference(target.record_key, event.currentTarget, target);
        }}
      >
        {label}
      </Typography.Link>
    );
  }
  return (
    <>
      {entry.content.map((content, index) => (
        <PreparedContent
          key={index}
          content={content}
          relationships={entry.relationships}
          onReference={onReference}
        />
      ))}
      {links.map((link, index) => {
        const target = link.target!;
        return (
          <Button
            key={index}
            type="link"
            onClick={(event) =>
              onReference(target.record_key, event.currentTarget, target)
            }
          >
            Open record
          </Button>
        );
      })}
    </>
  );
}

// Plain table labels become the reference link itself. Rich prose keeps its existing renderer.
function simpleResultLabel(entry: OwnedReadingEntryView): string | undefined {
  if (entry.content.length !== 1) return undefined;
  const body = entry.content[0].body;
  if (body.kind === "plain") return body.text.trim() || undefined;
  if (body.kind !== "html" || body.controls.length > 0) return undefined;
  const parsed = new DOMParser().parseFromString(body.html, "text/html").body;
  if (
    Array.from(parsed.querySelectorAll("*")).some(
      (element) => element.tagName !== "P" && element.tagName !== "SPAN",
    )
  )
    return undefined;
  const label = parsed.textContent?.trim();
  return label && label.length <= 160 ? label : undefined;
}
