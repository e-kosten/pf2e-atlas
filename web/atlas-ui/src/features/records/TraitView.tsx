import { usePageTitle } from "../../shared/browser/usePageTitle";
import { Alert, Button, Empty, Pagination, Spin, Tabs, Typography } from "antd";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { AtlasApiError, getTraitRecords } from "../../api/atlasApi";
import {
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
  type AtlasRoute,
} from "../../app/routes";
import type {
  TraitAssociationKind,
  TraitDetailView,
  TraitRecordsRequest,
  TraitRecordsPage,
} from "../../generated/atlas";
import { DocumentPagePane } from "../../shared/records/DocumentPagePane";
import { DocumentIdentityHeader } from "../../shared/records/DocumentIdentityHeader";
import { useTraitDetail } from "../../shared/records/useTraitDetail";
import { PreparedContentBody } from "../../shared/records/PreparedContent";
import { useSearchProfile } from "../../shared/search/SearchProfileContext";
import { ResultSortSelect } from "../../shared/search/ResultSortSelect";
import {
  DEFAULT_SEARCH_STATE,
  decodeResultSort,
  effectiveResultSort,
  type SearchFormState,
} from "../../shared/filters/searchState";
import {
  assertSearchIdentity,
  sameSearchPageContext,
} from "../../shared/search/searchIdentity";
import {
  isArtifactGenerationChanged,
  retireImmutableReferenceQueries,
} from "../../shared/queries/immutableReferenceQueries";

export function TraitView({
  route,
}: {
  route: Extract<AtlasRoute, { kind: "trait" }>;
}) {
  const query = useTraitDetail(route.traitKey);
  usePageTitle(query.data?.label ?? "Trait");
  const client = useQueryClient();
  const reload = async () => {
    await retireImmutableReferenceQueries(client);
    await query.refetch();
  };
  return (
    <main className="record-view">
      <DocumentPagePane>
        <section className="detail-panel" aria-busy={query.isFetching}>
          {query.data && query.isFetching && (
            <div className="detail-refresh" aria-label="Updating trait">
              <Spin size="small" delay={500} />
            </div>
          )}
          {query.error && (
            <Alert
              showIcon
              type="error"
              title={
                query.data
                  ? "Unable to refresh this trait"
                  : "Unable to load this trait"
              }
              description={
                <>
                  {query.data && "Previously loaded content remains below. "}
                  {query.error.message}
                </>
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
              <DocumentIdentityHeader title={query.data.label} classification="Trait" />
              {query.data.content ? (
                <PreparedContentBody
                  body={query.data.content.body}
                  references={query.data.content.references}
                  onReference={(recordKey, _anchor, selection) =>
                    navigateToAtlasRoute({ kind: "record", recordKey, selection })
                  }
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
              />
            </article>
          ) : query.isLoading ? (
            <div className="detail-empty">
              <Spin delay={500} description="Loading trait…" />
            </div>
          ) : null}
        </section>
      </DocumentPagePane>
    </main>
  );
}

function TraitAssociations({
  detail,
  route,
  onReload,
}: {
  detail: TraitDetailView;
  route: Extract<AtlasRoute, { kind: "trait" }>;
  onReload: () => void;
}) {
  const [state, setState] = useState(() =>
    traitAssociationState(route.query, detail.trait_key),
  );
  const previousQuery = useRef(route.query);
  useEffect(() => {
    if (previousQuery.current === route.query) return;
    previousQuery.current = route.query;
    setState(traitAssociationState(route.query, detail.trait_key));
  }, [route.query, detail.trait_key]);
  function update(kind: TraitAssociationKind, search = state.search) {
    const params = new URLSearchParams(route.query);
    if (kind === "members") params.delete("tab");
    else params.set("tab", kind);
    if (search.sort) params.set("sort", search.sort);
    else params.delete("sort");
    const query = params.size ? "?" + params.toString() : undefined;
    previousQuery.current = query;
    setState({ kind, search });
    navigateToAtlasRoute({ ...route, query });
  }
  return (
    <Tabs
      activeKey={state.kind}
      onChange={(value) => update(value as TraitAssociationKind)}
      items={[
        { key: "members", label: "Has this trait" },
        { key: "references", label: "References" },
        { key: "similar", label: "Similar" },
      ].map((item) => ({
        ...item,
        children: (
          <TraitRecords
            key={JSON.stringify([item.key, detail.artifact_generation])}
            detail={detail}
            kind={item.key as TraitAssociationKind}
            search={state.search}
            setSearch={(search) => update(state.kind, search)}
            onReload={onReload}
          />
        ),
      }))}
    />
  );
}

function traitAssociationState(
  query: string | undefined,
  traitKey: string,
): { kind: TraitAssociationKind; search: SearchFormState } {
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

function TraitRecords({
  detail,
  kind,
  onReload,
  search,
  setSearch,
}: {
  detail: TraitDetailView;
  kind: TraitAssociationKind;
  onReload: () => void;
  search: SearchFormState;
  setSearch: (search: SearchFormState) => void;
}) {
  const [paging, setPaging] = useState({ sort: search.sort, number: 1 });
  const page = kind === "similar" && paging.sort !== search.sort ? 1 : paging.number;
  const setPage = (number: number) => setPaging({ sort: search.sort, number });
  const profile = useSearchProfile();
  const request: TraitRecordsRequest = {
    trait_key: detail.trait_key,
    kind,
    page: { number: page, size: 20 },
    expected_artifact_generation: detail.artifact_generation,
    search_profile: profile.id,
    ranking: "score",
    ...(kind === "similar" && search.sort ? { sort: search.sort } : {}),
  };
  const query = useQuery<TraitRecordsPage>({
    queryKey: ["trait-records", request],
    placeholderData: (previous, previousQuery) =>
      kind === "similar" &&
      previousQuery &&
      sameSearchPageContext(previousQuery.queryKey[1] as TraitRecordsRequest, request)
        ? previous
        : undefined,
    queryFn: async ({ signal }) => {
      const response = await getTraitRecords(request, signal);
      if (kind === "similar")
        assertSearchIdentity(
          { ...response, search_profile: response.search_profile ?? "" },
          profile.id,
          detail.artifact_generation,
          effectiveResultSort(search),
        );
      return response;
    },
    enabled: kind !== "similar" || (!profile.loading && !search.filterError),
  });
  const changed = isArtifactGenerationChanged(query.error);
  const unavailable =
    kind === "similar" &&
    query.error instanceof AtlasApiError &&
    query.error.appError?.code === "vector_readiness_required";
  const data = changed || unavailable ? undefined : query.data;
  return (
    <section
      aria-label={
        kind === "members"
          ? "Records with this trait"
          : kind === "similar"
            ? "Similar records"
            : "Trait references"
      }
      aria-busy={query.isFetching}
    >
      {kind === "similar" && (
        <div className="result-pane-header">
          <span>
            {query.isPlaceholderData
              ? "Updating results"
              : data?.coverage?.exhaustive === false
                ? "Top matches"
                : "Similar records"}
          </span>
          <ResultSortSelect
            search={search}
            setSearch={(next) => {
              setPage(1);
              setSearch(next);
            }}
          />
        </div>
      )}
      {kind === "similar" && search.filterError && (
        <Alert type="error" title={search.filterError} />
      )}
      {query.error && (
        <Alert
          showIcon
          type={changed ? "warning" : unavailable ? "info" : "error"}
          title={
            changed
              ? "Reference collection changed"
              : unavailable
                ? "Similar unavailable for this trait with the selected search model"
                : "Unable to load these records"
          }
          description={
            unavailable ? undefined : (
              <>
                {data && "Previously loaded records remain below. "}
                {query.error.message}
              </>
            )
          }
          action={
            unavailable ? undefined : (
              <Button onClick={() => (changed ? onReload() : void query.refetch())}>
                {changed ? "Reload trait" : "Retry"}
              </Button>
            )
          }
        />
      )}
      {data && query.isFetching && (
        <div className="detail-refresh" aria-label="Updating associated records">
          <Spin size="small" delay={500} />
        </div>
      )}
      {query.isLoading && <Spin delay={500} />}
      {data && (
        <>
          {kind !== "similar" && data.coverage && !data.coverage.exhaustive && (
            <Typography.Paragraph type="secondary">Top matches</Typography.Paragraph>
          )}
          {!data.rows.length ? (
            <Empty
              description={
                kind === "members"
                  ? "No records carry this trait"
                  : kind === "references"
                    ? "No source-backed references"
                    : "No similar records"
              }
            />
          ) : (
            <ul className="trait-records">
              {data.rows.map((row, index) => {
                const route = {
                  kind: "record" as const,
                  recordKey: row.navigation.record_key,
                  selection: row.navigation,
                };
                return (
                  <li key={JSON.stringify([row.navigation, index])}>
                    <Typography.Link
                      href={atlasRoutePath(route)}
                      onClick={(event) => {
                        if (!shouldHandleAtlasRouteClick(event)) return;
                        event.preventDefault();
                        navigateToAtlasRoute(route);
                      }}
                    >
                      {row.source_label
                        ? `${row.source_label} · ${row.record.title}`
                        : row.record.title}
                    </Typography.Link>
                    {kind === "similar" && row.record.unit_price_label && (
                      <Typography.Text type="secondary">
                        {" "}
                        · {row.record.unit_price_label}
                      </Typography.Text>
                    )}
                    {kind === "similar" &&
                      (data.sort === "level_asc" || data.sort === "level_desc") &&
                      row.record.level_label && (
                        <Typography.Text type="secondary">
                          {" "}
                          · {row.record.level_label}
                        </Typography.Text>
                      )}
                    {row.reference_direction && (
                      <Typography.Text type="secondary">
                        {" "}
                        ·{" "}
                        {row.reference_direction === "incoming"
                          ? "References this trait"
                          : "Referenced in definition"}
                      </Typography.Text>
                    )}
                    {row.navigation.embedded_path.length > 0 && !row.source_label && (
                      <Typography.Text type="secondary">
                        {" "}
                        · Embedded source
                      </Typography.Text>
                    )}
                  </li>
                );
              })}
            </ul>
          )}
          {data.page.total > data.page.size && (
            <Pagination
              current={page}
              pageSize={data.page.size}
              total={Number(data.page.total)}
              showSizeChanger={false}
              onChange={setPage}
            />
          )}
        </>
      )}
    </section>
  );
}
