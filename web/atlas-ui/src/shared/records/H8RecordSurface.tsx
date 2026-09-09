import { Alert, Button, Card, Descriptions, List, Space, Tag, Typography } from "antd";
import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import type {
  H8FactView,
  JournalPageEntryView,
  JournalSurfaceView,
  RecordSurfaceMetadataView,
  RecordSurfaceView,
  RollTableSurfaceView,
  TableRollView,
  TableResultView,
  TableResultEntryView,
} from "../../generated/atlas";
import { rollTable } from "../../api/atlasApi";
import {
  ATLAS_ROUTE_CHANGE_EVENT,
  navigateToAtlasRoute,
  recordPath,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import { RecordHeader } from "./CreatureRecordSurface";
import { RichBlocks, type ReferenceHandler } from "./RecordRichContent";
import {
  RecordSurfaceIssues,
  RecordSurfaceReferences,
} from "./RecordSurfaceSupplement";

type ReferenceProps = {
  onReference: ReferenceHandler;
  onReferencesOpen?: () => void;
  onReferenceLimit?: (direction: "backlinks" | "outgoing", limit: number) => void;
  references?: NonNullable<RecordSurfaceView["references"]>;
  referencesLoading?: boolean;
};

export function JournalSearchCompactSurface({
  body,
  metadata,
}: {
  body: JournalSurfaceView;
  metadata: RecordSurfaceMetadataView;
}) {
  const pages = known(body.pages);
  return (
    <article className="record-surface record-surface--search-compact">
      <div className="record-surface-search__identity">
        <div className="record-surface-search__heading">
          <h2>{metadata.title}</h2>
        </div>
        <div className="creature-sheet__identity-meta creature-sheet__identity-meta--compact">
          <span>{metadata.kind_label}</span>
        </div>
      </div>
      <Space size="small" wrap>
        <Tag>{pages ? `${pages.length} pages` : factLabel(body.pages)}</Tag>
        {pages?.slice(0, 2).map((entry) => (
          <span key={entryLocator(entry)}>{entryLabel(entry)}</span>
        ))}
      </Space>
    </article>
  );
}

export function JournalDetailSurface({
  body,
  issues,
  metadata,
  onReference,
  onReferenceLimit,
  onReferencesOpen,
  references,
  referencesLoading,
  showTitle,
}: {
  body: JournalSurfaceView;
  issues: RecordSurfaceView["issues"];
  metadata: RecordSurfaceMetadataView;
  showTitle: boolean;
} & ReferenceProps) {
  const pages = known(body.pages) ?? [];
  const locationSearch = useSyncExternalStore(
    subscribeToRouteChanges,
    currentLocationSearch,
    emptyLocationSearch,
  );
  const requested = new URLSearchParams(locationSearch).get("child");
  const selected = pages.find((entry) => entryLocator(entry) === requested) ?? pages[0];
  const recordKey = metadata.record_key;
  return (
    <article className="record-surface creature-sheet">
      <RecordHeader metadata={metadata} showTitle={showTitle} />
      <div className="creature-sheet__mechanics-grid h8-journal__layout">
        <nav
          aria-label="Journal pages"
          className="creature-sheet__mechanics-side h8-journal__navigation"
        >
          <List
            dataSource={pages}
            locale={{ emptyText: `Pages are ${factLabel(body.pages)}.` }}
            renderItem={(entry) => {
              const locator = entryLocator(entry);
              return (
                <List.Item>
                  <Button
                    aria-current={
                      locator === entryLocator(selected) ? "page" : undefined
                    }
                    disabled={!recordKey}
                    onClick={() => {
                      if (recordKey)
                        navigateToAtlasRoute({
                          kind: "record",
                          recordKey,
                          childLocator: locator,
                        });
                    }}
                    type={locator === entryLocator(selected) ? "primary" : "text"}
                  >
                    {entryLabel(entry)}
                  </Button>
                </List.Item>
              );
            }}
            size="small"
          />
        </nav>
        <section
          aria-live="polite"
          className="creature-sheet__mechanics-main h8-journal__content"
        >
          {selected ? (
            <JournalPageDetail entry={selected} onReference={onReference} />
          ) : (
            <Alert message="No journal pages are available." showIcon type="info" />
          )}
        </section>
      </div>
      <RecordSurfaceIssues issues={issues} />
      <RecordSurfaceReferences
        loading={referencesLoading}
        onDisclosureOpen={onReferencesOpen}
        onRequestLimit={onReferenceLimit}
        onReference={onReference}
        recordKey={recordKey}
        references={references}
      />
    </article>
  );
}

function JournalPageDetail({
  entry,
  onReference,
}: {
  entry: JournalPageEntryView;
  onReference: ReferenceHandler;
}) {
  if (entry.entry_type === "unsupported") {
    return (
      <Alert
        description={entry.unsupported.reason}
        message={`Page ${entry.unsupported.source_ordinal + 1} is unavailable`}
        showIcon
        type="warning"
      />
    );
  }
  const { page } = entry;
  const kind = known(page.page_kind);
  const title = known(page.title);
  const titleVisible = title ? known(title.show) : undefined;
  const titleLevel = title ? known(title.level) : undefined;
  const authoredTitle = known(page.name);
  const text = known(page.text);
  const blocks = text ? known(text.content) : undefined;
  return (
    <Card
      title={
        titleVisible === true && titleLevel !== undefined && authoredTitle ? (
          <span aria-level={titleLevel} role="heading">
            {authoredTitle}
          </span>
        ) : undefined
      }
    >
      <Space size="small" wrap>
        <Tag>{kind ?? factLabel(page.page_kind)}</Tag>
        {known(page.sort) !== undefined ? <span>Sort {known(page.sort)}</span> : null}
      </Space>
      {blocks?.length ? (
        <RichBlocks
          blocks={blocks}
          keyPrefix={page.locator}
          onReference={onReference}
        />
      ) : kind === "image" || kind === "pdf" || kind === "video" ? (
        <Alert
          description="Atlas retains the authored media locator and metadata without fetching or rendering it."
          message="Media metadata only"
          showIcon
          type="info"
        />
      ) : (
        <Typography.Paragraph type="secondary">
          Content is {text ? factLabel(text.content) : factLabel(page.text)}.
        </Typography.Paragraph>
      )}
    </Card>
  );
}

export function RollTableSearchCompactSurface({
  body,
  metadata,
}: {
  body: RollTableSurfaceView;
  metadata: RecordSurfaceMetadataView;
}) {
  const results = known(body.results);
  return (
    <article className="record-surface record-surface--search-compact">
      <div className="record-surface-search__identity">
        <div className="record-surface-search__heading">
          <h2>{metadata.title}</h2>
        </div>
        <div className="creature-sheet__identity-meta creature-sheet__identity-meta--compact">
          <span>{metadata.kind_label}</span>
        </div>
      </div>
      <Space size="small" wrap>
        {known(body.formula) ? <Tag>{known(body.formula)}</Tag> : null}
        <span>
          {results ? `${results.length} results` : `Results ${factLabel(body.results)}`}
        </span>
      </Space>
    </article>
  );
}

export function RollTableDetailSurface({
  body,
  issues,
  metadata,
  onReference,
  onReferenceLimit,
  onReferencesOpen,
  references,
  referencesLoading,
  showTitle,
}: {
  body: RollTableSurfaceView;
  issues: RecordSurfaceView["issues"];
  metadata: RecordSurfaceMetadataView;
  showTitle: boolean;
} & ReferenceProps) {
  const results = known(body.results) ?? [];
  const locationSearch = useSyncExternalStore(
    subscribeToRouteChanges,
    currentLocationSearch,
    emptyLocationSearch,
  );
  const selectedLocator = new URLSearchParams(locationSearch).get("child");
  const recordKey = metadata.record_key;
  const roll = useTableRoll(recordKey);
  const selectedResultRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const selected = selectedResultRef.current;
    if (!selectedLocator || !selected) return;
    if (typeof selected.scrollIntoView === "function") {
      selected.scrollIntoView({ block: "nearest" });
    }
    selected.focus({ preventScroll: true });
  }, [selectedLocator]);
  return (
    <article className="record-surface creature-sheet">
      <RecordHeader metadata={metadata} showTitle={showTitle} />
      <Descriptions bordered column={{ xs: 1, sm: 3 }} size="small">
        <Descriptions.Item label="Formula">
          {known(body.formula) ?? factLabel(body.formula)}
        </Descriptions.Item>
        <Descriptions.Item label="Replacement">
          {booleanLabel(body.replacement)}
        </Descriptions.Item>
        <Descriptions.Item label="Display roll">
          {booleanLabel(body.display_roll)}
        </Descriptions.Item>
      </Descriptions>
      <TableRollPanel
        capability={body.roll}
        onReference={onReference}
        recordKey={recordKey}
        request={roll}
      />
      {known(body.description)?.length ? (
        <section>
          <h3>Description</h3>
          <RichBlocks
            blocks={known(body.description) ?? []}
            keyPrefix={`${metadata.record_key ?? "roll-table"}:description`}
            onReference={onReference}
          />
        </section>
      ) : null}
      <section aria-label="Table results">
        <h3>Results</h3>
        <List
          dataSource={results}
          locale={{ emptyText: `Results are ${factLabel(body.results)}.` }}
          renderItem={(entry) => {
            const locator = entryLocator(entry);
            const selected = locator === selectedLocator;
            return (
              <List.Item
                actions={[
                  selected ? (
                    <Tag aria-current="page" color="blue" key="selected-result">
                      Result {resultOrdinal(entry) + 1} selected
                    </Tag>
                  ) : (
                    <Button
                      disabled={!recordKey}
                      href={recordKey ? recordPath(recordKey, locator) : undefined}
                      key="open-result"
                      onClick={(event) => {
                        if (!recordKey || !shouldHandleAtlasRouteClick(event)) return;
                        event.preventDefault();
                        navigateToAtlasRoute({
                          kind: "record",
                          recordKey,
                          childLocator: locator,
                        });
                      }}
                      type="link"
                    >
                      Open result {resultOrdinal(entry) + 1}
                    </Button>
                  ),
                ]}
                className={selected ? "ant-list-item-selected" : undefined}
              >
                <div
                  aria-label={
                    selected ? `Selected result ${resultOrdinal(entry) + 1}` : undefined
                  }
                  ref={selected ? selectedResultRef : undefined}
                  role={selected ? "region" : undefined}
                  tabIndex={selected ? -1 : undefined}
                >
                  <TableResultRow entry={entry} onReference={onReference} />
                </div>
              </List.Item>
            );
          }}
        />
      </section>
      <RecordSurfaceIssues issues={issues} />
      <RecordSurfaceReferences
        loading={referencesLoading}
        onDisclosureOpen={onReferencesOpen}
        onRequestLimit={onReferenceLimit}
        onReference={onReference}
        recordKey={metadata.record_key}
        references={references}
      />
    </article>
  );
}

function subscribeToRouteChanges(onStoreChange: () => void) {
  window.addEventListener("popstate", onStoreChange);
  window.addEventListener(ATLAS_ROUTE_CHANGE_EVENT, onStoreChange);
  return () => {
    window.removeEventListener("popstate", onStoreChange);
    window.removeEventListener(ATLAS_ROUTE_CHANGE_EVENT, onStoreChange);
  };
}

function currentLocationSearch() {
  return window.location.search;
}

function emptyLocationSearch() {
  return "";
}

function TableResultRow({
  entry,
  onReference,
}: {
  entry: TableResultEntryView;
  onReference: ReferenceHandler;
}) {
  if (entry.entry_type === "unsupported") {
    return (
      <Alert
        description={entry.unsupported.reason}
        message={`Result ${entry.unsupported.source_ordinal + 1} is unavailable`}
        showIcon
        type="warning"
      />
    );
  }
  return <TableResultValue onReference={onReference} result={entry.result} />;
}

function TableResultValue({
  onReference,
  result,
}: {
  onReference: ReferenceHandler;
  result: TableResultView;
}) {
  const range = known(result.range);
  const blocks = known(result.text);
  return (
    <div>
      <Space size="small" wrap>
        <Tag>{range ? `${range.first}–${range.last}` : factLabel(result.range)}</Tag>
        <Tag>{known(result.result_kind) ?? factLabel(result.result_kind)}</Tag>
        {known(result.weight) ? <span>Weight {known(result.weight)}</span> : null}
        <span>Drawn: {booleanLabel(result.drawn)}</span>
      </Space>
      {blocks?.length ? (
        <RichBlocks
          blocks={blocks}
          keyPrefix={result.locator}
          onReference={onReference}
        />
      ) : null}
      {known(result.collection) ? (
        <Typography.Text type="secondary">
          {known(result.collection)} ·{" "}
          {known(result.document_id) ?? "unresolved document"} (navigation unavailable)
        </Typography.Text>
      ) : null}
    </div>
  );
}

type TableRollRequest =
  | { state: "idle" }
  | { state: "pending"; recordKey: string }
  | { state: "success"; recordKey: string; value: TableRollView }
  | { state: "error"; recordKey: string; message: string };

function useTableRoll(recordKey: string | undefined) {
  const [request, setRequest] = useState<TableRollRequest>({ state: "idle" });
  const active = useRef<{
    controller: AbortController;
    id: number;
    recordKey: string;
  }>();
  const nextId = useRef(0);

  useEffect(() => {
    active.current?.controller.abort();
    active.current = undefined;
    return () => active.current?.controller.abort();
  }, [recordKey]);

  const run = () => {
    if (!recordKey) return;
    if (active.current?.recordKey === recordKey) return;
    active.current?.controller.abort();
    const controller = new AbortController();
    const id = ++nextId.current;
    active.current = { controller, id, recordKey };
    setRequest({ state: "pending", recordKey });
    void rollTable(recordKey, controller.signal)
      .then((value) => {
        if (active.current?.id !== id) return;
        active.current = undefined;
        if (value.table_key !== recordKey) {
          setRequest({
            state: "error",
            recordKey,
            message: "The table-roll response did not match the current record.",
          });
          return;
        }
        setRequest({ state: "success", recordKey, value });
      })
      .catch((error: unknown) => {
        if (active.current?.id !== id) return;
        active.current = undefined;
        if (error instanceof DOMException && error.name === "AbortError") return;
        setRequest({
          state: "error",
          recordKey,
          message: error instanceof Error ? error.message : "Table roll failed.",
        });
      });
  };

  const currentRequest =
    request.state !== "idle" && request.recordKey === recordKey
      ? request
      : ({ state: "idle" } as const);
  return { request: currentRequest, run };
}

function TableRollPanel({
  capability,
  onReference,
  recordKey,
  request,
}: {
  capability: RollTableSurfaceView["roll"];
  onReference: ReferenceHandler;
  recordKey: string | undefined;
  request: ReturnType<typeof useTableRoll>;
}) {
  if (capability.state === "unavailable") {
    return (
      <Alert
        description={capability.unavailable.message}
        message="Roll unavailable"
        showIcon
        type="warning"
      />
    );
  }
  const response = request.request.state === "success" ? request.request.value : null;
  return (
    <Card className="h8-roll-table__operation" title="Roll table">
      <Space direction="vertical" size="middle" style={{ width: "100%" }}>
        <Space wrap>
          <Tag>{capability.formula}</Tag>
          <Button
            disabled={!recordKey || request.request.state === "pending"}
            loading={request.request.state === "pending"}
            onClick={request.run}
            type="primary"
          >
            Roll
          </Button>
        </Space>
        {request.request.state === "error" ? (
          <Alert message={request.request.message} showIcon type="error" />
        ) : null}
        {response?.state === "unavailable" ? (
          <Alert
            description={response.unavailable.message}
            message="Roll unavailable"
            showIcon
            type="warning"
          />
        ) : null}
        {response?.state === "available" ? (
          <div aria-live="polite">
            <Typography.Title level={4}>
              Rolled {response.total} on {response.formula}
            </Typography.Title>
            {response.outcomes.length ? (
              <List
                dataSource={response.outcomes}
                renderItem={(result) => (
                  <List.Item
                    actions={[
                      <Button
                        disabled={!recordKey}
                        href={
                          recordKey ? recordPath(recordKey, result.locator) : undefined
                        }
                        key="open-rolled-result"
                        onClick={(event) => {
                          if (!recordKey || !shouldHandleAtlasRouteClick(event)) return;
                          event.preventDefault();
                          navigateToAtlasRoute({
                            kind: "record",
                            recordKey,
                            childLocator: result.locator,
                          });
                        }}
                        type="link"
                      >
                        Open rolled result {result.source_ordinal + 1}
                      </Button>,
                    ]}
                  >
                    <TableResultValue onReference={onReference} result={result} />
                  </List.Item>
                )}
                size="small"
              />
            ) : (
              <Typography.Paragraph type="secondary">
                No results match total {response.total}.
              </Typography.Paragraph>
            )}
          </div>
        ) : null}
      </Space>
    </Card>
  );
}

function known<T>(fact: H8FactView<T>): T | undefined {
  return fact.state === "known" ? fact.value : undefined;
}

function factLabel<T>(fact: H8FactView<T>) {
  switch (fact.state) {
    case "known":
      return "available";
    case "missing":
      return "missing";
    case "null":
      return "null";
    case "unsupported":
      return "unsupported";
  }
}

function booleanLabel(fact: H8FactView<boolean>) {
  const value = known(fact);
  return value === undefined ? factLabel(fact) : value ? "Yes" : "No";
}

function entryLocator(entry: JournalPageEntryView | TableResultEntryView) {
  if (entry.entry_type === "unsupported") return entry.unsupported.locator;
  if (entry.entry_type === "page") return entry.page.locator;
  return entry.result.locator;
}

function entryLabel(entry: JournalPageEntryView) {
  return entry.entry_type === "page"
    ? (known(entry.page.name) ?? `Page ${entry.page.source_ordinal + 1}`)
    : `Unavailable page ${entry.unsupported.source_ordinal + 1}`;
}

function resultOrdinal(entry: TableResultEntryView) {
  return entry.entry_type === "result"
    ? entry.result.source_ordinal
    : entry.unsupported.source_ordinal;
}
