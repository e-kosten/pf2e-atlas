import {
  Alert,
  Button,
  Collapse,
  Modal,
  Select,
  Spin,
  Space,
  Tabs,
  Typography,
} from "antd";
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useSearchProfile } from "../search/SearchProfileContext";
import { SearchProfileNotice } from "../search/SearchProfileNotice";
import { assertSearchIdentity } from "../search/searchIdentity";
import DOMPurify from "dompurify";
import { AtlasApiError, getRelatedRecords, searchPage } from "../../api/atlasApi";
import {
  atlasRoutePath,
  currentAtlasRoute,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import type {
  GraphSectionView,
  RecordDetailView,
  RecordNavigationView,
  RecordSummaryView,
} from "../../generated/atlas";
import {
  DEFAULT_SEARCH_STATE,
  decodeSearchStateFromParams,
  searchStateQueryString,
} from "../filters/searchState";
import type { RecordReferenceHandler } from "./PreparedContent";
import { RecordDetailPane } from "./RecordDetailPane";
import { useRecordDetail } from "./useRecordDetail";
import { isArtifactGenerationChanged } from "../queries/immutableReferenceQueries";

import { linkLeaf } from "../filters/linkPredicates";
import { relationshipSearchRoute } from "./relationshipSearch";

export function RelatedRecords({
  detail,
  onReference,
  onReload,
  reloadLabel = "Reload record",
}: {
  detail: RecordDetailView;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
  reloadLabel?: string;
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
            children: (
              <RelatedTabs
                key={JSON.stringify([detail.selected, detail.artifact_generation])}
                detail={detail}
                onReference={onReference}
                onReload={onReload}
                reloadLabel={reloadLabel}
              />
            ),
          },
        ]}
      />
    </section>
  );
}

function RelatedTabs({
  detail,
  onReference,
  onReload,
  reloadLabel,
}: {
  detail: RecordDetailView;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
  reloadLabel: string;
}) {
  const [active, setActive] = useState("references");
  const [comparison, setComparison] = useState<string | null>(null);
  const key = detail.record.record_key;
  const related = useQuery({
    queryKey: ["related", detail.selected, detail.artifact_generation],
    staleTime: Infinity,
    queryFn: ({ signal }) =>
      getRelatedRecords(
        {
          selection: detail.selected,
          artifact_generation: detail.artifact_generation,
        },
        signal,
      ),
  });
  const reloadRecord =
    related.error instanceof AtlasApiError &&
    related.error.appError?.recoverable_action === "reload_record";
  const graph =
    related.data?.references.kind === "content"
      ? related.data.references.content
      : undefined;
  const remaster =
    related.data?.counterparts.kind === "content"
      ? related.data.counterparts.content
      : undefined;
  const variants =
    related.data?.variants.kind === "content"
      ? related.data.variants.content
      : undefined;
  const variantChoices =
    variants?.variants.filter((record) => record.record_key !== key) ?? [];

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
              embedded_path: [],
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
  function viewAll(section: GraphSectionView) {
    if (!section.filter) return null;
    const route = relationshipSearchRoute(section.filter);
    return (
      <Typography.Link
        href={atlasRoutePath(route)}
        onClick={(event) => {
          if (!shouldHandleAtlasRouteClick(event)) return;
          event.preventDefault();
          navigateToAtlasRoute(route);
        }}
      >
        View all
      </Typography.Link>
    );
  }
  function references(section: GraphSectionView, incoming: boolean) {
    if (!section.occurrences.length)
      return (
        <Typography.Paragraph type="secondary">
          {incoming
            ? "No references to this record were found."
            : "No references were found here."}{" "}
          {viewAll(section)}
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
        {viewAll(section)}
        {section.truncated && (
          <Typography.Paragraph type="secondary">
            Showing a limited set of references; additional references may exist.
          </Typography.Paragraph>
        )}
      </>
    );
  }

  const items = [
    ...(graph
      ? [
          {
            key: "references",
            label: "References",
            children: (
              <>
                <h3>Referenced here</h3>
                {references(graph.outgoing, false)}
                <h3>References to this record</h3>
                {references(graph.backlinks, true)}
              </>
            ),
          },
        ]
      : []),
    ...(remaster
      ? [
          {
            key: "remaster",
            label: "Remaster",
            children: (
              <ul className="related-records__list">
                {remaster.links.map((pair) => (
                  <li
                    key={
                      pair.legacy_record.record_key +
                      ":" +
                      pair.remaster_record.record_key
                    }
                  >
                    {records([pair.legacy_record, pair.remaster_record], "")}
                  </li>
                ))}
              </ul>
            ),
          },
        ]
      : []),
    ...(variants
      ? [
          {
            key: "variants",
            label: "Variants",
            children: (
              <>
                <Typography.Paragraph type="secondary">
                  {variants.ambiguous
                    ? "These suggestions are uncertain; compare the records before treating them as variants."
                    : "These records share a naming pattern and compatible source facts."}
                </Typography.Paragraph>
                {records(variantChoices, "No suggested variants were found.")}
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
                {variants.truncated && (
                  <Typography.Paragraph type="secondary">
                    Showing a limited set of suggested variants; others may exist.
                  </Typography.Paragraph>
                )}
              </>
            ),
          },
        ]
      : []),
  ];
  const selectedTab = items.some((item) => item.key === active)
    ? active
    : items[0]?.key;
  const failures = related.data
    ? ([
        ["References", related.data.references],
        ["Remaster", related.data.counterparts],
        ["Variants", related.data.variants],
      ] as const)
    : [];
  return (
    <div className="related-records__body">
      <Space wrap className="related-records__actions">
        {(["incoming", "outgoing"] as const).map((direction) => {
          const route = relationshipSearchRoute(
            linkLeaf({
              direction,
              selection: detail.selected,
              include_contents: detail.selected.embedded_path.length === 0,
            }),
          );
          return (
            <Button
              key={direction}
              href={atlasRoutePath(route)}
              onClick={(event) => {
                if (!shouldHandleAtlasRouteClick(event)) return;
                event.preventDefault();
                navigateToAtlasRoute(route);
              }}
            >
              {direction === "incoming"
                ? "Find records linking here"
                : "Browse links from here"}
            </Button>
          );
        })}
      </Space>

      {related.isFetching && (
        <div role="status" className="related-records__status">
          <Spin size="small" /> Loading related records…
        </div>
      )}
      {related.error && (
        <Alert
          type="error"
          showIcon
          title="Unable to load related records"
          description={related.error.message}
          action={
            <Button
              size="small"
              disabled={reloadRecord && !onReload}
              onClick={() => {
                if (reloadRecord) onReload?.();
                else void related.refetch();
              }}
            >
              {reloadRecord ? reloadLabel : "Retry"}
            </Button>
          }
        />
      )}
      {failures.map(([label, category]) =>
        category.kind === "error" ? (
          <Alert
            key={label}
            type="error"
            title={`${label} could not be checked`}
            description={category.error.message}
            action={
              <Button
                size="small"
                onClick={() => {
                  void related.refetch();
                }}
              >
                Retry
              </Button>
            }
          />
        ) : category.kind === "unavailable" &&
          detail.selected.embedded_path.length === 0 ? (
          <Typography.Paragraph key={label} type="secondary">
            {label}: {category.reason}
          </Typography.Paragraph>
        ) : null,
      )}
      {items.length > 0 && (
        <Tabs
          activeKey={selectedTab}
          onChange={setActive}
          destroyOnHidden
          items={items}
        />
      )}
      <Collapse
        size="small"
        ghost
        destroyOnHidden
        items={[
          {
            key: "similar",
            label: detail.selected.embedded_path.length
              ? `Similar to parent record: ${detail.record.title}`
              : "Similar records",
            children: (
              <SimilarPreview
                detail={detail}
                onReference={onReference}
                onReload={onReload}
                reloadLabel={reloadLabel}
              />
            ),
          },
        ]}
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

function SimilarPreview({
  detail,
  onReference,
  onReload,
  reloadLabel,
}: {
  detail: RecordDetailView;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
  reloadLabel: string;
}) {
  const seed = detail.record.record_key;
  const profile = useSearchProfile();
  const similar = useQuery({
    queryKey: ["similar-preview", seed, detail.artifact_generation, profile.id],
    enabled: profile.id !== null && profile.storedVectors,
    staleTime: Infinity,
    queryFn: async ({ signal }) => {
      const page = await searchPage(
        {
          search_profile: profile.id ?? undefined,
          criteria: { kind: "similar_records", seed_record_key: seed, filter: null },
          page: { number: 1, size: 5 },
          expected_artifact_generation: detail.artifact_generation,
        },
        signal,
      );
      assertSearchIdentity(page, profile.id, detail.artifact_generation, {
        kind: "relevance",
      });
      return page;
    },
  });
  const reloadRecord = isArtifactGenerationChanged(similar.error);
  const route = {
    kind: "search",
    selectedRecordKey: null,
    query: searchStateQueryString(
      {
        ...(currentAtlasRoute().kind === "search"
          ? decodeSearchStateFromParams(new URLSearchParams(window.location.search))
          : DEFAULT_SEARCH_STATE),
        query: "",
        mode: "similar_records",
        similarSeed: seed,
      },
      profile.id ?? profile.pendingUrlProfile,
    ),
  } as const;
  return (
    <>
      {!profile.storedVectors && <SearchProfileNotice />}
      {similar.isFetching && (
        <div role="status">
          <Spin size="small" /> Loading similar records…
        </div>
      )}
      {similar.error && (
        <Alert
          type="error"
          title="Unable to load similar records"
          description={similar.error.message}
          action={
            <Button
              size="small"
              disabled={reloadRecord && !onReload}
              onClick={() => {
                if (reloadRecord) onReload?.();
                else void similar.refetch();
              }}
            >
              {reloadRecord ? reloadLabel : "Retry"}
            </Button>
          }
        />
      )}
      {similar.data &&
        (similar.data.rows.length ? (
          <ul className="related-records__list">
            {similar.data.rows.map((row) => (
              <li key={row.record.record_key}>
                <Typography.Link
                  href={atlasRoutePath({
                    kind: "record",
                    recordKey: row.record.record_key,
                  })}
                  onClick={(event) => {
                    if (!shouldHandleAtlasRouteClick(event)) return;
                    event.preventDefault();
                    onReference(row.record.record_key, event.currentTarget);
                  }}
                >
                  {row.record.title}
                </Typography.Link>
              </li>
            ))}
          </ul>
        ) : (
          <Typography.Paragraph type="secondary">
            No similar records found.
          </Typography.Paragraph>
        ))}
      <Typography.Link
        href={atlasRoutePath(route)}
        onClick={(event) => {
          if (!shouldHandleAtlasRouteClick(event)) return;
          event.preventDefault();
          navigateToAtlasRoute(route);
        }}
      >
        See more similar records
      </Typography.Link>
    </>
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
