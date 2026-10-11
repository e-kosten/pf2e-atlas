import { Alert, Button, Collapse, Modal, Select, Spin, Tabs, Typography } from "antd";
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import DOMPurify from "dompurify";
import {
  getRecordGraph,
  getRecordRemaster,
  getRecordVariants,
  getSimilarRecords,
} from "../../api/atlasApi";
import {
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import type {
  GraphSectionView,
  RecordDetailView,
  RecordNavigationView,
  RecordSummaryView,
} from "../../generated/atlas";
import type { RecordReferenceHandler } from "./PreparedContent";
import { RecordDetailPane } from "./RecordDetailPane";
import { useRecordDetail } from "./useRecordDetail";

export function RelatedRecords({
  detail,
  onReference,
}: {
  detail: RecordDetailView;
  onReference: RecordReferenceHandler;
}) {
  return (
    <section className="related-records" aria-label="Related records">
      <Collapse
        size="small"
        destroyOnHidden
        items={[
          {
            key: "related",
            label: "Related records",
            children: <RelatedTabs detail={detail} onReference={onReference} />,
          },
        ]}
      />
    </section>
  );
}

function RelatedTabs({
  detail,
  onReference,
}: {
  detail: RecordDetailView;
  onReference: RecordReferenceHandler;
}) {
  const [active, setActive] = useState("references");
  const [comparison, setComparison] = useState<string | null>(null);
  const key = detail.record.record_key;
  const graph = useQuery({
    queryKey: ["graph", detail.selected],
    enabled: active === "references",
    queryFn: () =>
      getRecordGraph(key, detail.selected.owners, detail.selected.source_fingerprint),
  });
  const remaster = useQuery({
    queryKey: ["remaster", key],
    enabled: active === "remaster",
    queryFn: () => getRecordRemaster(key),
  });
  const variants = useQuery({
    queryKey: ["variants", key],
    enabled: active === "variants",
    queryFn: () => getRecordVariants(key),
  });
  const similar = useQuery({
    queryKey: ["similar", key],
    enabled: active === "similar",
    queryFn: () => getSimilarRecords(key),
  });
  const query =
    active === "references"
      ? graph
      : active === "remaster"
        ? remaster
        : active === "variants"
          ? variants
          : similar;
  const variantChoices =
    variants.data?.variants.filter((record) => record.record_key !== key) ?? [];
  const rootScope = detail.selected.owners.length > 0 && active !== "references";

  function link(label: string, selection: RecordNavigationView) {
    return (
      <Typography.Link
        href={atlasRoutePath({
          kind: "record",
          recordKey: selection.record_key,
          selection,
        })}
        onClick={(event) => {
          if (!shouldHandleAtlasRouteClick(event)) return;
          event.preventDefault();
          onReference(selection.record_key, event.currentTarget, selection);
        }}
      >
        {label}
      </Typography.Link>
    );
  }
  function records(rows: RecordSummaryView[], empty: string) {
    return rows.length ? (
      <ul className="related-records__list">
        {rows.map((record) => (
          <li key={record.record_key}>
            {link(record.title, {
              record_key: record.record_key,
              owners: [],
              field: null,
              passage: null,
              source_fingerprint: null,
            })}
          </li>
        ))}
      </ul>
    ) : (
      <Typography.Paragraph type="secondary">{empty}</Typography.Paragraph>
    );
  }
  function references(section: GraphSectionView, incoming: boolean) {
    if (!section.occurrences.length)
      return (
        <Typography.Paragraph type="secondary">
          {incoming
            ? "No references to this record were found."
            : "No references were found here."}
        </Typography.Paragraph>
      );
    return (
      <>
        <ul className="related-records__list">
          {section.occurrences.map((occurrence) => {
            const navigation = incoming
              ? occurrence.source_navigation
              : occurrence.target;
            const summary = section.records.find(
              (record) => record.record_key === navigation?.record_key,
            );
            const label =
              summary?.title ??
              (incoming ? "Open referring record" : "Open referenced record");
            return (
              <li
                key={JSON.stringify([
                  occurrence.source,
                  occurrence.ordinal,
                  occurrence.target,
                ])}
              >
                {navigation ? (
                  link(label, navigation)
                ) : occurrence.url &&
                  ["resolved_url", "unverified_url"].includes(occurrence.status) &&
                  DOMPurify.isValidAttribute("a", "href", occurrence.url) ? (
                  <Typography.Link
                    href={occurrence.url}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    External reference
                  </Typography.Link>
                ) : (
                  <Typography.Text type="secondary">
                    This reference is unavailable in the current collection.
                  </Typography.Text>
                )}
              </li>
            );
          })}
        </ul>
        {section.truncated && (
          <Typography.Paragraph type="secondary">
            Showing a limited set of references; additional references may exist.
          </Typography.Paragraph>
        )}
      </>
    );
  }

  const tabContent = (
    <>
      {rootScope && (
        <Typography.Paragraph type="secondary">
          These suggestions are for the parent record, {detail.record.title}.
        </Typography.Paragraph>
      )}
      {query.isFetching && (
        <div role="status" className="related-records__status">
          <Spin size="small" /> Loading related records…
        </div>
      )}
      {query.error && (
        <Alert
          type="error"
          showIcon
          title="Unable to load related records"
          description={query.error.message}
          action={
            <Button
              size="small"
              loading={query.isFetching}
              onClick={() => {
                void query.refetch();
              }}
            >
              Retry
            </Button>
          }
        />
      )}
      {active === "references" && graph.data && (
        <>
          <h3>Referenced here</h3>
          {references(graph.data.outgoing, false)}
          <h3>References to this record</h3>
          {references(graph.data.backlinks, true)}
        </>
      )}
      {active === "remaster" &&
        remaster.data &&
        (remaster.data.links.length ? (
          <ul className="related-records__list">
            {remaster.data.links.map((pair) => (
              <li
                key={
                  pair.legacy_record.record_key + ":" + pair.remaster_record.record_key
                }
              >
                {records([pair.legacy_record, pair.remaster_record], "")}
              </li>
            ))}
          </ul>
        ) : (
          <Typography.Paragraph type="secondary">
            No verified legacy or remastered counterpart was found.
          </Typography.Paragraph>
        ))}
      {active === "variants" && variants.data && (
        <>
          {variantChoices.length > 0 && (
            <Typography.Paragraph type="secondary">
              {variants.data.ambiguous
                ? "These suggestions are uncertain; compare the records before treating them as variants."
                : "These records share a naming pattern and compatible source facts."}
            </Typography.Paragraph>
          )}
          {records(variantChoices, "No suggested variants were found.")}
          {variantChoices.length > 0 && (
            <Select
              className="related-records__compare"
              aria-label="Compare variant"
              placeholder="Choose a variant to compare…"
              value={comparison}
              showSearch={{ optionFilterProp: "label" }}
              options={variantChoices.map((record) => ({
                value: record.record_key,
                label: record.title,
              }))}
              onChange={setComparison}
            />
          )}
          {variants.data.truncated && (
            <Typography.Paragraph type="secondary">
              Showing a limited set of suggested variants; others may exist.
            </Typography.Paragraph>
          )}
        </>
      )}
      {active === "similar" && similar.data && (
        <>
          {!similar.data.results.coverage.exhaustive && (
            <Typography.Paragraph type="secondary">
              <Collapse
                size="small"
                ghost
                items={[
                  {
                    key: "coverage",
                    label: "About these suggestions",
                    children: "More matches may exist.",
                  },
                ]}
              />
            </Typography.Paragraph>
          )}
          {records(
            similar.data.results.rows.map((row) => row.record),
            "No suggestions found.",
          )}
        </>
      )}
      {!query.isFetching && !query.error && query.data === null && (
        <Typography.Paragraph type="secondary">
          Related records are unavailable for this selection.
        </Typography.Paragraph>
      )}
    </>
  );
  return (
    <div className="related-records__body">
      <Tabs
        activeKey={active}
        onChange={setActive}
        destroyOnHidden
        items={[
          { key: "references", label: "References" },
          { key: "remaster", label: "Remaster" },
          { key: "variants", label: "Variants" },
          { key: "similar", label: "Similar" },
        ].map((tab) => ({ ...tab, children: tab.key === active ? tabContent : null }))}
      />
      {comparison && (
        <Comparison
          source={key}
          target={comparison}
          onClose={() => setComparison(null)}
        />
      )}
    </div>
  );
}

function Comparison({
  source,
  target,
  onClose,
}: {
  source: string;
  target: string;
  onClose: () => void;
}) {
  const left = useRecordDetail(source);
  const right = useRecordDetail(target);
  const navigate: RecordReferenceHandler = (recordKey, _anchor, selection) =>
    navigateToAtlasRoute({ kind: "record", recordKey, selection });
  return (
    <Modal
      title="Variant comparison"
      open
      footer={null}
      onCancel={onClose}
      width="90vw"
    >
      <div className="record-comparison">
        <RecordDetailPane
          detail={left.data}
          loading={left.isLoading || left.isFetching}
          error={left.error}
          onRetry={() => {
            void left.refetch();
          }}
          onReference={navigate}
          showRelated={false}
        />
        <RecordDetailPane
          detail={right.data}
          loading={right.isLoading || right.isFetching}
          error={right.error}
          onRetry={() => {
            void right.refetch();
          }}
          onReference={navigate}
          showRelated={false}
        />
      </div>
    </Modal>
  );
}
