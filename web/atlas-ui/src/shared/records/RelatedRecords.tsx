import { AtlasModal as Modal } from "../ui/overlays/AtlasModal";
import { ErrorNotice } from "../feedback/ErrorNotice";
import { navigateDestination } from "../../app/navigation";
import { recordDestination, searchDestination } from "../../app/destinations";
import { Button, Empty, Tabs, Typography } from "antd";
import { LoadingStatus } from "../ui/LoadingStatus";
import { useState } from "react";
import { SearchPreviewRows } from "./SearchPreviewRows";
import { ReferenceResult } from "./ReferenceResult";
import { useQuery } from "@tanstack/react-query";
import DOMPurify from "dompurify";
import {
  AtlasApiError,
  getRelatedRecords,
  getSimilarityParts,
  searchPage,
} from "../../api/atlasApi";
import { useSearchProfile } from "../search/SearchProfileContext";
import {
  hasCoreSourcePreference,
  preferredSourceSections,
  similarityFromPreferences,
} from "../search/similarityState";
import { SearchProfileNotice } from "../search/SearchProfileNotice";
import {
  assertSearchIdentity,
  assertSimilarityIdentity,
  assertSimilarityInventoryIdentity,
} from "../search/searchIdentity";
import { currentSearchLocation } from "../../app/searchLocation";
import {
  DEFAULT_SEARCH_STATE,
  decodeSearchStateFromParams,
  searchStateQueryString,
} from "../filters/searchState";
import { linkLeaf } from "../filters/linkPredicates";
import type { RecordDetailView, RecordSummaryView } from "../../generated/atlas";
import type { RecordReferenceHandler } from "./PreparedContent";
import { RecordDetailPane } from "./RecordDetailPane";
import { useRecordDetail } from "./useRecordDetail";
import { isArtifactGenerationChanged } from "../queries/immutableReferenceQueries";
import { relationshipSearchRoute } from "./relationshipSearch";
import { ExploreSearchAction } from "./ExploreSearchAction";
import { useRecordExploreLocation } from "./recordExploreLocation";

type ExploreProps = {
  detail: RecordDetailView;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
  reloadLabel?: string;
};

export function RelatedRecords(props: ExploreProps) {
  const incoming = useReferencePreview(props.detail, true);
  const outgoing = useReferencePreview(props.detail, false);
  const related = useRelated(props.detail);
  const { tab: active, setTab: setActive } = useRecordExploreLocation();
  const graph = related.data?.references;
  const otherReferences =
    graph?.kind === "content" &&
    graph.content.outgoing.occurrences.some((entry) => !entry.target);
  const hasIncoming =
    !!incoming.query.error ||
    !incoming.query.data ||
    incoming.query.data.rows.length > 0;
  const hasOutgoing =
    !!outgoing.query.error ||
    !outgoing.query.data ||
    outgoing.query.data.rows.length > 0 ||
    otherReferences ||
    !!related.error ||
    graph?.kind === "error" ||
    graph?.kind === "unavailable";
  const versions =
    related.data &&
    [related.data.counterparts, related.data.variants].some(
      (category) => category.kind !== "empty",
    );
  const items = [
    ...(hasIncoming
      ? [
          {
            key: "incoming",
            label: "Referenced by",
            children: <ReferencePreview {...props} incoming />,
          },
        ]
      : []),
    ...(hasOutgoing
      ? [
          {
            key: "outgoing",
            label: "References",
            children: <ReferencePreview {...props} incoming={false} />,
          },
        ]
      : []),
    { key: "similar", label: "Similar", children: <SimilarPreview {...props} /> },
    ...(props.detail.selected.embedded_path.length === 0 && (versions || related.error)
      ? [{ key: "versions", label: "Versions", children: <Versions {...props} /> }]
      : []),
  ];
  return (
    <section className="record-explore" aria-label="Explore record">
      <Tabs
        activeKey={items.some((item) => item.key === active) ? active : items[0].key}
        onChange={setActive}
        animated={false}
        destroyOnHidden
        items={items}
      />
    </section>
  );
}

function QueryError({
  error,
  retry,
  onReload,
  reloadLabel = "Reload record",
}: {
  error: Error;
  retry: () => void;
  onReload?: () => void;
  reloadLabel?: string;
}) {
  const reload =
    isArtifactGenerationChanged(error) ||
    (error instanceof AtlasApiError &&
      error.appError?.recoverable_action === "reload_record");
  return (
    <ErrorNotice
      error={error}
      title="Unable to load connections"
      action={
        <Button
          size="small"
          onClick={reload ? onReload : retry}
          disabled={reload && !onReload}
        >
          {reload ? reloadLabel : "Retry"}
        </Button>
      }
    />
  );
}

function useReferencePreview(detail: RecordDetailView, incoming: boolean) {
  const profile = useSearchProfile();
  const filter = linkLeaf(
    {
      direction: incoming ? "incoming" : "outgoing",
      selection: detail.selected,
      include_contents: detail.selected.embedded_path.length === 0,
    },
    "record-explore-reference",
  );
  const query = useQuery({
    queryKey: [
      "record-reference-preview",
      filter,
      detail.artifact_generation,
      profile.id,
    ],
    staleTime: Infinity,
    enabled: !profile.loading && profile.id !== null,
    queryFn: async ({ signal }) => {
      const page = await searchPage(
        {
          search_profile: profile.id ?? undefined,
          criteria: { kind: "list_records", filter },
          page: { number: 1, size: 5 },
          expected_artifact_generation: detail.artifact_generation,
        },
        signal,
      );
      assertSearchIdentity(page, profile.id, detail.artifact_generation);
      return page;
    },
  });
  return { query, filter };
}

function ReferencePreview({
  detail,
  incoming,
  onReference,
  onReload,
  reloadLabel,
}: ExploreProps & { incoming: boolean }) {
  const { query, filter } = useReferencePreview(detail, incoming);
  const profile = useSearchProfile();
  const data = isArtifactGenerationChanged(query.error) ? undefined : query.data;
  return (
    <div className="record-explore__task">
      <ExploreSearchAction
        route={relationshipSearchRoute(filter)}
        summary={
          data
            ? `${data.rows.length} of ${data.page.total.toString()} ${data.page.total === 1n ? "record" : "records"}`
            : undefined
        }
      >
        {incoming ? "Search referring records" : "Search referenced records"}
      </ExploreSearchAction>
      {profile.error && <SearchProfileNotice />}
      {query.isPending && !profile.error && (
        <LoadingStatus label="Loading references…" />
      )}
      {query.error && (
        <QueryError
          error={query.error}
          retry={() => void query.refetch()}
          onReload={onReload}
          reloadLabel={reloadLabel}
        />
      )}
      {data &&
        (data.rows.length ? (
          <SearchPreviewRows
            page={data}
            filter={filter}
            onReference={onReference}
            onReload={onReload}
          />
        ) : (
          <Empty
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            description={
              incoming ? "No records reference this entry." : "No linked records found."
            }
          />
        ))}
      {!incoming && (
        <OtherReferences
          detail={detail}
          onReference={onReference}
          onReload={onReload}
          reloadLabel={reloadLabel}
        />
      )}
    </div>
  );
}

function useRelated(detail: RecordDetailView) {
  const query = useQuery({
    queryKey: ["related", detail.selected, detail.artifact_generation],
    staleTime: Infinity,
    retryOnMount: false,
    queryFn: ({ signal }) =>
      getRelatedRecords(
        { selection: detail.selected, artifact_generation: detail.artifact_generation },
        signal,
      ),
  });
  return {
    ...query,
    data: isArtifactGenerationChanged(query.error) ? undefined : query.data,
  };
}

function OtherReferences({ detail, onReload, reloadLabel }: ExploreProps) {
  const related = useRelated(detail);
  const graph = related.data?.references;
  const other =
    graph?.kind === "content"
      ? graph.content.outgoing.occurrences.filter((entry) => !entry.target)
      : [];
  if (related.error)
    return (
      <QueryError
        error={related.error}
        retry={() => void related.refetch()}
        onReload={onReload}
        reloadLabel={reloadLabel}
      />
    );
  if (graph?.kind === "error")
    return (
      <ErrorNotice
        error={graph.error}
        compact
        title="Additional references could not be checked"
      />
    );
  if (!other.length) return null;
  return (
    <div className="record-explore__other">
      <h3>Other references</h3>
      <ul>
        {other.slice(0, 5).map((entry, index) => (
          <li key={index}>
            {entry.url &&
            ["resolved_url", "unverified_url"].includes(entry.status) &&
            DOMPurify.isValidAttribute("a", "href", entry.url) ? (
              <Typography.Link
                href={entry.url}
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
        ))}
      </ul>
    </div>
  );
}

function VersionCard({
  record,
  onReference,
  onCompare,
  context,
}: {
  record: RecordSummaryView;
  onReference: RecordReferenceHandler;
  onCompare: () => void;
  context?: string;
}) {
  return (
    <div role="listitem">
      <ReferenceResult
        row={{ record, matches: [], relationship_evidence: [] }}
        selected={false}
        disabled={false}
        onPreview={(anchor) => onReference(record.record_key, anchor)}
        context={context}
        actions={
          <Button size="small" onClick={onCompare}>
            Compare
          </Button>
        }
      />
    </div>
  );
}

function Versions({ detail, onReference, onReload, reloadLabel }: ExploreProps) {
  const related = useRelated(detail);
  const [comparison, setComparison] = useState<{
    target: string;
    open: boolean;
  } | null>(null);
  const counterparts = related.data?.counterparts;
  const variants = related.data?.variants;
  const pairs = counterparts?.kind === "content" ? counterparts.content.links : [];
  const choices =
    variants?.kind === "content"
      ? variants.content.variants.filter(
          (record) => record.record_key !== detail.record.record_key,
        )
      : [];
  return (
    <div className="record-explore__task">
      {related.isLoading && <LoadingStatus label="Checking versions…" />}
      {related.error && (
        <QueryError
          error={related.error}
          retry={() => void related.refetch()}
          onReload={onReload}
          reloadLabel={reloadLabel}
        />
      )}
      {[counterparts, variants].map((category, index) =>
        category?.kind === "error" ? (
          <ErrorNotice
            error={category.error}
            compact
            key={index}
            title={`${index === 0 ? "Editions" : "Variants"} could not be checked`}
            action={
              <Button size="small" onClick={() => void related.refetch()}>
                Retry
              </Button>
            }
          />
        ) : category?.kind === "unavailable" ? (
          <Typography.Paragraph key={index} type="secondary">
            {category.reason}
          </Typography.Paragraph>
        ) : null,
      )}
      {pairs.length > 0 && (
        <section>
          <h3>Other editions</h3>
          <div className="record-explore__results" role="list">
            {pairs.map((pair) => {
              const legacy =
                pair.remaster_record.record_key === detail.record.record_key;
              const other = legacy ? pair.legacy_record : pair.remaster_record;
              return (
                <VersionCard
                  key={other.record_key}
                  record={other}
                  onReference={onReference}
                  onCompare={() =>
                    setComparison({ target: other.record_key, open: true })
                  }
                  context={legacy ? "Legacy" : "Remaster"}
                />
              );
            })}
          </div>
        </section>
      )}
      {choices.length > 0 && (
        <section>
          <h3>Suggested variants</h3>
          {variants?.kind === "content" && variants.content.ambiguous && (
            <Typography.Paragraph type="secondary">
              These suggestions are uncertain; compare before treating them as variants.
            </Typography.Paragraph>
          )}
          <div className="record-explore__results" role="list">
            {choices.map((record) => (
              <VersionCard
                key={record.record_key}
                record={record}
                onReference={onReference}
                onCompare={() =>
                  setComparison({ target: record.record_key, open: true })
                }
              />
            ))}
          </div>
          {variants?.kind === "content" && variants.content.truncated && (
            <Typography.Paragraph type="secondary">
              More suggested variants may exist.
            </Typography.Paragraph>
          )}
        </section>
      )}
      {counterparts?.kind === "empty" && variants?.kind === "empty" && (
        <Empty
          image={Empty.PRESENTED_IMAGE_SIMPLE}
          description="No other editions or suggested variants found."
        />
      )}
      {comparison && (
        <Comparison
          source={detail.record.record_key}
          target={comparison.target}
          open={comparison.open}
          onClose={() => setComparison({ ...comparison, open: false })}
          onAfterClose={() => setComparison(null)}
        />
      )}
    </div>
  );
}

function SimilarPreview({ detail, onReference, onReload, reloadLabel }: ExploreProps) {
  const seed = detail.record.record_key;
  const profile = useSearchProfile();
  const similarity = similarityFromPreferences(profile.preferences.recordSimilarity);
  const needsFamily =
    similarity.method !== "baseline" && hasCoreSourcePreference(profile.preferences);
  const seedInventory = useQuery({
    queryKey: [
      "similarity-parts",
      { kind: "record", record_key: seed },
      profile.id,
      detail.artifact_generation,
      0,
    ],
    enabled: needsFamily && profile.id !== null && profile.storedVectors,
    staleTime: Infinity,
    queryFn: async ({ signal }) => {
      const source = { kind: "record" as const, record_key: seed };
      const inventory = await getSimilarityParts(
        {
          seed: source,
          search_profile: profile.id ?? undefined,
          expected_artifact_generation: detail.artifact_generation,
        },
        signal,
      );
      assertSimilarityInventoryIdentity(
        inventory,
        source,
        profile.id!,
        detail.artifact_generation,
      );
      return inventory;
    },
  });
  const sourceSections =
    similarity.method === "baseline"
      ? undefined
      : seedInventory.data
        ? preferredSourceSections(profile.preferences, seedInventory.data.core.family)
        : "all";
  const currentSearch =
    currentSearchLocation() !== null
      ? decodeSearchStateFromParams(
          new URLSearchParams(window.location.search),
          "search",
          profile.preferences,
        )
      : null;
  const searchContent =
    currentSearch?.searchContent ?? profile.preferences.searchContent;
  const similar = useQuery({
    queryKey: [
      "similar-preview",
      seed,
      detail.artifact_generation,
      profile.id,
      similarity,
      sourceSections,
      searchContent,
    ],
    enabled:
      profile.id !== null &&
      profile.storedVectors &&
      (!needsFamily || seedInventory.data !== undefined),
    staleTime: Infinity,
    queryFn: async ({ signal }) => {
      const page = await searchPage(
        {
          search_profile: profile.id ?? undefined,
          criteria: {
            kind: "similar_records",
            seed_record_key: seed,
            filter: null,
            similarity,
            source_sections: sourceSections,
            search_content: searchContent,
          },
          page: { number: 1, size: 5 },
          expected_artifact_generation: detail.artifact_generation,
        },
        signal,
      );
      assertSearchIdentity(page, profile.id, detail.artifact_generation, {
        kind: "relevance",
      });
      assertSimilarityIdentity(page, similarity, sourceSections, searchContent);
      return page;
    },
  });
  const data = isArtifactGenerationChanged(similar.error) ? undefined : similar.data;
  const previewError = seedInventory.error ?? similar.error;
  const route = searchDestination({
    selectedRecordKey: null,
    query: searchStateQueryString(
      {
        ...(currentSearch ?? DEFAULT_SEARCH_STATE),
        query: "",
        mode: "similar_records",
        similarSeed: seed,
        similarity,
        searchContent,
        sourceSections,
        similarityGeneration: undefined,
        traitReferenceSeed: null,
      },
      profile.id ?? profile.pendingUrlProfile,
    ),
  });
  return (
    <div className="record-explore__task">
      <ExploreSearchAction
        route={route}
        summary="Top matches"
        disabled={needsFamily && !seedInventory.data}
      >
        Search similar records
      </ExploreSearchAction>
      {!!detail.selected.embedded_path.length && (
        <Typography.Paragraph>
          Similar to parent record: {detail.record.title}
        </Typography.Paragraph>
      )}
      {!profile.storedVectors && <SearchProfileNotice />}
      {similar.isFetching && (
        <div className={similar.data ? "detail-refresh" : undefined}>
          <LoadingStatus label="Loading similar records…" />
        </div>
      )}
      {previewError && (
        <QueryError
          error={previewError}
          retry={() =>
            void (seedInventory.error ? seedInventory.refetch() : similar.refetch())
          }
          onReload={onReload}
          reloadLabel={reloadLabel}
        />
      )}
      {data &&
        (data.rows.length ? (
          <>
            <SearchPreviewRows
              page={data}
              onReference={onReference}
              onReload={onReload}
            />
          </>
        ) : (
          <Empty
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            description="No similar records found."
          />
        ))}
    </div>
  );
}
function Comparison({
  source,
  target,
  open,
  onClose,
  onAfterClose,
}: {
  source: string;
  target: string;
  open: boolean;
  onClose: () => void;
  onAfterClose: () => void;
}) {
  const left = useRecordDetail(source);
  const right = useRecordDetail(target);
  const navigate: RecordReferenceHandler = (recordKey, _anchor, selection) =>
    navigateDestination(
      recordDestination({
        recordKey,
        selection,
      }),
    );
  return (
    <Modal
      title="Variant comparison"
      open={open}
      afterClose={onAfterClose}
      footer={null}
      onCancel={onClose}
      width="min(90vw, calc(2 * var(--document-measure) + 6 * var(--space-section) + var(--panel-gap)))"
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
