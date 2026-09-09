import { Alert, Button, Card, Descriptions, List, Space, Tag, Typography } from "antd";
import type {
  H8FactView,
  JournalPageEntryView,
  JournalSurfaceView,
  RecordSurfaceMetadataView,
  RecordSurfaceView,
  RollTableSurfaceView,
  TableResultEntryView,
} from "../../generated/atlas";
import { navigateToAtlasRoute } from "../../app/routes";
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
  const requested = new URLSearchParams(window.location.search).get("child");
  const selected = pages.find((entry) => entryLocator(entry) === requested) ?? pages[0];
  const recordKey = metadata.record_key;
  return (
    <article className="record-surface creature-sheet">
      <RecordHeader metadata={metadata} showTitle={showTitle} />
      <div className="creature-sheet__mechanics-grid">
        <nav aria-label="Journal pages" className="creature-sheet__mechanics-side">
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
        <section aria-live="polite" className="creature-sheet__mechanics-main">
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
  const selectedLocator = new URLSearchParams(window.location.search).get("child");
  const recordKey = metadata.record_key;
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
                  <Button
                    aria-current={selected ? "page" : undefined}
                    disabled={!recordKey}
                    key="open-result"
                    onClick={() => {
                      if (recordKey)
                        navigateToAtlasRoute({
                          kind: "record",
                          recordKey,
                          childLocator: locator,
                        });
                    }}
                    type={selected ? "primary" : "link"}
                  >
                    Open result {resultOrdinal(entry) + 1}
                  </Button>,
                ]}
                className={selected ? "ant-list-item-selected" : undefined}
              >
                <TableResultRow entry={entry} onReference={onReference} />
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
  const result = entry.result;
  const range = known(result.range);
  const blocks = known(result.text);
  return (
    <div>
      <Space size="small" wrap>
        <Tag>{range ? `${range.first}–${range.last}` : factLabel(result.range)}</Tag>
        <Tag>{known(result.result_kind) ?? factLabel(result.result_kind)}</Tag>
        {known(result.weight) ? <span>Weight {known(result.weight)}</span> : null}
        {known(result.drawn) ? <Tag>Drawn</Tag> : null}
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
          {known(result.document_id) ?? "unresolved document"}
        </Typography.Text>
      ) : null}
    </div>
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
