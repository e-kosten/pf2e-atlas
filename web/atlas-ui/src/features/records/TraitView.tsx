import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { traitDestination, searchDestination } from "../../app/destinations";
import type { TraitPageInput } from "../../app/destinations";
import { navigateDestination } from "../../app/navigation";
import { canonicalRouteSearch } from "../../app/routeSearch";
import { parseSearchParams } from "../../app/urlState";
import { recordDestination } from "../../app/destinations";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { Alert, Button, Empty, Tabs, Typography } from "antd";
import { useQuery, useQueries, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { AtlasApiError, getTraitRecords, searchPage } from "../../api/atlasApi";
import type {
  TraitAssociationKind,
  TraitDetailView,
  TraitRecordsRequest,
  TraitRecordsPage,
} from "../../generated/atlas";
import type { RecordReferenceHandler } from "../../shared/records/PreparedContent";
import { ReferenceResult } from "../../shared/records/ReferenceResult";
import { ExploreSearchAction } from "../../shared/records/ExploreSearchAction";
import { RecordReferenceDrawer } from "../../shared/records/RecordReferenceDrawer";
import { useRecordPreview } from "../../shared/records/useRecordPreview";
import { AddToRecordButton } from "../record-actions/AddToRecordButton";
import { DocumentPagePane } from "../../shared/records/DocumentPagePane";
import { DocumentIdentityHeader } from "../../shared/records/DocumentIdentityHeader";
import { useTraitDetail } from "../../shared/records/useTraitDetail";
import { PreparedContentBody } from "../../shared/records/PreparedContent";
import { useSearchProfile } from "../../shared/search/SearchProfileContext";
import {
  preferredSourceSections,
  similarityFromPreferences,
} from "../../shared/search/similarityState";
import { SearchPreviewRows } from "../../shared/records/SearchPreviewRows";
import {
  DEFAULT_SEARCH_STATE,
  searchStateQueryString,
  decodeResultSort,
  encodeResultSort,
  effectiveResultSort,
  type SearchFormState,
} from "../../shared/filters/searchState";
import {
  assertSearchIdentity,
  assertSimilarityIdentity,
} from "../../shared/search/searchIdentity";
import {
  isArtifactGenerationChanged,
  retireImmutableReferenceQueries,
} from "../../shared/queries/immutableReferenceQueries";
export function TraitView({ route }: { route: TraitPageInput }) {
  const query = useTraitDetail(route.traitKey);
  const preview = useRecordPreview();
  usePageTitle(query.data?.label ?? "Trait");
  const client = useQueryClient();
  const reload = async () => {
    await retireImmutableReferenceQueries(client);
    await query.refetch();
  };
  return (
    <>
      <main className="record-view">
        <DocumentPagePane>
          <section className="detail-panel" aria-busy={query.isFetching}>
            {query.data && query.isFetching && (
              <div className="detail-refresh" aria-label="Updating trait">
                <LoadingStatus label="Updating trait…" />
              </div>
            )}
            {query.error && (
              <ErrorNotice
                error={query.error}
                title={
                  query.data
                    ? "Unable to refresh this trait"
                    : "Unable to load this trait"
                }
                description={
                  query.data ? "Previously loaded content remains below." : undefined
                }
                action={
                  <Button
                    onClick={() => void query.refetch()}
                    aria-busy={query.isFetching}
                  >
                    Retry
                  </Button>
                }
              />
            )}
            {query.data ? (
              <article className="record-reference">
                <DocumentIdentityHeader
                  title={query.data.label}
                  classification="Trait"
                />
                {query.data.content ? (
                  <PreparedContentBody
                    body={query.data.content.body}
                    references={query.data.content.references}
                    onReference={preview.open}
                  />
                ) : (
                  <Alert
                    type="info"
                    title="No definition is available in this reference collection."
                  />
                )}
                <TraitAssociations
                  key={JSON.stringify([
                    query.data.trait_key,
                    query.data.artifact_generation,
                  ])}
                  detail={query.data}
                  route={route}
                  onReload={() => void reload()}
                  onReference={preview.open}
                />
              </article>
            ) : query.isLoading ? (
              <div className="detail-empty">
                <LoadingStatus label="Loading trait…" />
              </div>
            ) : null}
          </section>
        </DocumentPagePane>
      </main>
      {preview.recordKey && (
        <RecordReferenceDrawer
          actions={
            <AddToRecordButton
              recordKey={preview.recordKey}
              record={preview.detail?.record}
            />
          }
          anchor={preview.anchor}
          readingHistory={preview.readingHistory}
          requestedNavigation={preview.selection}
          landing={preview.landing}
          detail={preview.detail}
          loading={preview.loading}
          error={preview.error}
          onRetry={preview.retry}
          onClose={preview.close}
          onReference={preview.open}
          onRankChange={preview.changeRank}
          onOpenFullPage={() => {
            navigateDestination(
              recordDestination({
                recordKey: preview.recordKey!,
                selection: preview.selection,
                landing: preview.landing,
              }),
            );
            preview.close();
          }}
        />
      )}
    </>
  );
}
function TraitAssociations({
  detail,
  route,
  onReload,
  onReference,
}: {
  detail: TraitDetailView;
  route: TraitPageInput;
  onReload: () => void;
  onReference: RecordReferenceHandler;
}) {
  const [state, setState] = useState(() =>
    traitAssociationState(route.query, detail.trait_key),
  );
  const profile = useSearchProfile();
  const availability = useQueries({
    queries: (["members", "references"] as const).map((kind) => {
      const request: TraitRecordsRequest = {
        trait_key: detail.trait_key,
        kind,
        page: { number: 1, size: 5 },
        expected_artifact_generation: detail.artifact_generation,
        search_profile: profile.id,
        ranking: "score",
      };
      return {
        queryKey: ["trait-records", request],
        queryFn: ({ signal }: { signal: AbortSignal }) =>
          getTraitRecords(request, signal),
        staleTime: Infinity,
        retryOnMount: false,
      };
    }),
  });
  const previousQuery = useRef(route.query);
  useEffect(() => {
    if (previousQuery.current === route.query) return;
    previousQuery.current = route.query;
    setState(traitAssociationState(route.query, detail.trait_key));
  }, [route.query, detail.trait_key]);
  function update(kind: TraitAssociationKind, search = state.search) {
    const params = new URLSearchParams();
    const explicitProfile = canonicalRouteSearch(
      "trait",
      parseSearchParams(route.query ?? ""),
    ).search_profile;
    if (typeof explicitProfile === "string")
      params.set("search_profile", explicitProfile);
    if (kind !== "members") params.set("tab", kind);
    encodeResultSort(params, search.sort);
    const query = params.size ? "?" + params.toString() : undefined;
    previousQuery.current = query;
    setState({ kind, search });
    navigateDestination(traitDestination({ ...route, query }));
  }
  const items = [
    { key: "members", label: "With this trait" },
    { key: "references", label: "References" },
    { key: "similar", label: "Similar" },
  ].filter(
    (item, index) =>
      index === 2 ||
      !availability[index].data ||
      availability[index].data.rows.length > 0,
  );
  if (availability.some((query) => query.isPending))
    return <LoadingStatus label="Loading connections…" />;
  return (
    <section className="record-explore" aria-label="Explore trait">
      <Tabs
        activeKey={
          items.some((item) => item.key === state.kind) ? state.kind : items[0].key
        }
        animated={false}
        destroyOnHidden
        onChange={(value) => update(value as TraitAssociationKind)}
        items={items.map((item) => ({
          ...item,
          children: (
            <TraitRecords
              key={JSON.stringify([item.key, detail.artifact_generation])}
              detail={detail}
              kind={item.key as TraitAssociationKind}
              search={state.search}
              onReload={onReload}
              onReference={onReference}
            />
          ),
        }))}
      />
    </section>
  );
}

function traitAssociationState(
  query: string | undefined,
  traitKey: string,
): {
  kind: TraitAssociationKind;
  search: SearchFormState;
} {
  const params = new URLSearchParams(query);
  const tab = params.get("tab");
  const { sort, sortError } = decodeResultSort(params, "search", true);
  return {
    kind: tab === "similar" || tab === "references" ? tab : "members",
    search: {
      ...DEFAULT_SEARCH_STATE,
      similarSeed: traitKey,
      sort,
      filterError: sortError,
    },
  };
}
type TraitRecordsProps = {
  detail: TraitDetailView;
  onReference: RecordReferenceHandler;
  kind: TraitAssociationKind;
  onReload: () => void;
  search: SearchFormState;
};
function TraitRecords(props: TraitRecordsProps) {
  return props.kind === "similar" ? (
    <TraitSimilarRecords {...props} />
  ) : (
    <TraitAssociatedRecords {...props} kind={props.kind} />
  );
}

function TraitSimilarRecords({
  detail,
  search,
  onReference,
  onReload,
}: TraitRecordsProps) {
  const profile = useSearchProfile();
  const similarity = similarityFromPreferences(
    profile.preferences.traitSimilarity,
    true,
  );
  const sourceSections =
    similarity.method === "baseline"
      ? undefined
      : preferredSourceSections(profile.preferences, "trait");
  const searchContent = profile.preferences.searchContent;
  const request = {
    criteria: {
      kind: "similar_traits" as const,
      seed_trait_key: detail.trait_key,
      filter: null,
      similarity,
      source_sections: sourceSections,
      search_content: searchContent,
    },
    search_profile: profile.id ?? undefined,
    expected_artifact_generation: detail.artifact_generation,
    page: { number: 1, size: 5 },
    sort: effectiveResultSort(search),
  };
  const query = useQuery({
    queryKey: ["trait-similar-preview", request],
    enabled: profile.id !== null && profile.storedVectors && !search.filterError,
    staleTime: Infinity,
    queryFn: async ({ signal }) => {
      const page = await searchPage(request, signal);
      assertSearchIdentity(page, profile.id, detail.artifact_generation, request.sort);
      assertSimilarityIdentity(page, similarity, sourceSections, searchContent);
      return page;
    },
  });
  const changed = isArtifactGenerationChanged(query.error);
  const unavailable =
    query.error instanceof AtlasApiError &&
    query.error.appError?.code === "vector_readiness_required";
  const data = changed || unavailable ? undefined : query.data;
  const route = searchDestination({
    query: searchStateQueryString(
      {
        ...search,
        query: "",
        similarSeed: detail.trait_key,
        similarity,
        searchContent,
        sourceSections,
        traitReferenceSeed: null,
      },
      profile.id ?? profile.pendingUrlProfile,
    ),
  });
  return (
    <section aria-label="Similar records" aria-busy={query.isFetching}>
      <ExploreSearchAction route={route} summary="Top matches">
        Search similar records
      </ExploreSearchAction>
      {search.filterError && (
        <ErrorNotice error={search.filterError} title={search.filterError} />
      )}
      {query.error && (
        <ErrorNotice
          error={query.error}
          type={unavailable ? "info" : "error"}
          title={
            changed
              ? "Reference collection changed"
              : unavailable
                ? "Similar unavailable for this trait with the selected search model"
                : "Unable to load similar records"
          }
          action={
            <Button onClick={() => (changed ? onReload() : void query.refetch())}>
              {changed ? "Reload trait" : "Retry"}
            </Button>
          }
        />
      )}
      {!profile.storedVectors && (
        <Typography.Paragraph type="secondary">
          Similar records require stored vectors for the selected model.
        </Typography.Paragraph>
      )}
      {query.isFetching && <LoadingStatus label="Loading similar records…" />}
      {data &&
        (data.rows.length ? (
          <SearchPreviewRows
            page={data}
            onReference={onReference}
            onReload={onReload}
          />
        ) : (
          <Empty description="No similar records" />
        ))}
    </section>
  );
}

function TraitAssociatedRecords({
  detail,
  kind,
  onReload,
  search,
  onReference,
}: Omit<TraitRecordsProps, "kind"> & { kind: "members" | "references" }) {
  const profile = useSearchProfile();
  const request: TraitRecordsRequest = {
    trait_key: detail.trait_key,
    kind,
    page: { number: 1, size: 5 },
    expected_artifact_generation: detail.artifact_generation,
    search_profile: profile.id,
    ranking: "score",
  };
  const query = useQuery<TraitRecordsPage>({
    queryKey: ["trait-records", request],
    staleTime: Infinity,
    retryOnMount: false,
    queryFn: ({ signal }) => getTraitRecords(request, signal),
  });
  const changed = isArtifactGenerationChanged(query.error);
  const data = changed ? undefined : query.data;
  const memberSearch = searchDestination({
    query: searchStateQueryString(
      {
        ...DEFAULT_SEARCH_STATE,
        filter: {
          kind: "set_match",
          field: "traits",
          op: "includes_any",
          values: [detail.trait_key.slice("trait:".length)],
        },
      },
      profile.id,
    ),
  });
  const fullSearch =
    kind === "members"
      ? memberSearch
      : searchDestination({
          query: searchStateQueryString(
            {
              ...search,
              query: "",
              similarSeed: null,
              traitReferenceSeed: detail.trait_key,
              sort: undefined,
            },
            profile.id,
          ),
        });
  return (
    <section
      aria-label={kind === "members" ? "Records with this trait" : "Trait references"}
      aria-busy={query.isFetching}
    >
      <ExploreSearchAction
        route={fullSearch}
        summary={
          data
            ? `${data.rows.length} of ${data.page.total.toString()} ${data.page.total === 1n ? "entry" : "entries"}`
            : undefined
        }
      >
        {kind === "members"
          ? "Search records with this trait"
          : "Search trait references"}
      </ExploreSearchAction>
      {query.error && (
        <ErrorNotice
          error={query.error}
          type={changed ? "warning" : "error"}
          title={
            changed ? "Reference collection changed" : "Unable to load these records"
          }
          description={data ? "Previously loaded records remain below." : undefined}
          action={
            <Button onClick={() => (changed ? onReload() : void query.refetch())}>
              {changed ? "Reload trait" : "Retry"}
            </Button>
          }
        />
      )}
      {data && query.isFetching && (
        <div className="detail-refresh" aria-label="Updating associated records">
          <LoadingStatus label="Updating associated records…" />
        </div>
      )}
      {query.isLoading && <LoadingStatus label="Loading records…" />}
      {data && (
        <>
          {data.coverage && !data.coverage.exhaustive && (
            <Typography.Paragraph type="secondary">Top matches</Typography.Paragraph>
          )}
          {!data.rows.length ? (
            <Empty
              description={
                kind === "members"
                  ? "No records carry this trait"
                  : "No source-backed references"
              }
            />
          ) : (
            <div className="record-explore__results" role="list">
              {data.rows.map((row, index) => (
                <div role="listitem" key={JSON.stringify([row.navigation, index])}>
                  <ReferenceResult
                    row={{ record: row.record, matches: [], relationship_evidence: [] }}
                    selected={false}
                    disabled={false}
                    navigation={row.navigation}
                    context={[
                      kind === "members" &&
                      row.source_label &&
                      row.navigation.embedded_path.length
                        ? `Trait on ${row.source_label}`
                        : row.source_label,
                      row.reference_direction === "incoming"
                        ? "References this trait"
                        : row.reference_direction === "outgoing"
                          ? "Referenced in definition"
                          : null,
                    ]
                      .filter(Boolean)
                      .join(" · ")}
                    onPreview={(anchor) =>
                      onReference(row.navigation.record_key, anchor, row.navigation)
                    }
                  />
                </div>
              ))}
            </div>
          )}
        </>
      )}
    </section>
  );
}
