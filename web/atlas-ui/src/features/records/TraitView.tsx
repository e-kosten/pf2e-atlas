import { useReferenceRecordNavigation } from "../../shared/records/ReferenceHelp";
import { ReferenceHelpProvider } from "../../shared/records/ReferenceHelp";
import { LinkNotFound } from "../../app/InvalidUrl";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { traitDestination, searchDestination } from "../../app/destinations";
import type { TraitPageInput } from "../../app/destinations";
import { navigateDestination } from "../../app/navigation";
import { canonicalRouteSearch } from "../../app/routeSearch";
import { parseSearchParams } from "../../app/urlState";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { Button, Empty, Tabs, Typography } from "antd";
import { useQuery, useQueries, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { getTraitRecords } from "../../api/atlasApi";
import type {
  TraitDetailView,
  TraitRecordsRequest,
  TraitRecordsPage,
} from "../../generated/atlas";
import type { RecordReferenceHandler } from "../../shared/records/PreparedContent";
import { ReferenceResult } from "../../shared/records/ReferenceResult";
import { ExploreSearchAction } from "../../shared/records/ExploreSearchAction";
import { ReferenceHelpHost } from "../../shared/records/ReferenceHelpHost";
import { DocumentPagePane } from "../../shared/records/DocumentPagePane";
import { DocumentIdentityHeader } from "../../shared/records/DocumentIdentityHeader";
import { useTraitDetail } from "../../shared/records/useTraitDetail";
import { TraitDefinition } from "../../shared/records/TraitDefinition";
import { useSearchProfile } from "../../shared/search/SearchProfileContext";
import {
  DEFAULT_SEARCH_STATE,
  searchStateQueryString,
  decodeResultSort,
  encodeResultSort,
  type SearchFormState,
} from "../../shared/filters/searchState";
import {
  isArtifactGenerationChanged,
  retireReferenceQueries,
} from "../../shared/queries/referenceQueries";
export function TraitView({ route }: { route: TraitPageInput }) {
  return (
    <ReferenceHelpHost>
      <TraitViewContent route={route} />
    </ReferenceHelpHost>
  );
}

function TraitViewContent({ route }: { route: TraitPageInput }) {
  const query = useTraitDetail(route.traitKey);
  const onReference = useReferenceRecordNavigation()!;
  const unavailable = query.data?.content === null;
  usePageTitle(unavailable ? "Link not found" : (query.data?.label ?? "Trait"));
  const client = useQueryClient();
  const reload = async () => {
    await retireReferenceQueries(client);
    await query.refetch();
  };
  if (unavailable) return <LinkNotFound />;
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
              <ReferenceHelpProvider generation={query.data.artifact_generation}>
                <article className="record-reference" data-selection-search>
                  <DocumentIdentityHeader
                    title={query.data.label}
                    classification="Trait"
                  />
                  <TraitDefinition detail={query.data} onReference={onReference} />
                  <TraitAssociations
                    key={JSON.stringify([
                      query.data.trait_key,
                      query.data.artifact_generation,
                    ])}
                    detail={query.data}
                    route={route}
                    onReload={() => void reload()}
                    onReference={onReference}
                  />
                </article>
              </ReferenceHelpProvider>
            ) : query.isLoading ? (
              <div className="detail-empty">
                <LoadingStatus label="Loading trait…" />
              </div>
            ) : null}
          </section>
        </DocumentPagePane>
      </main>
    </>
  );
}
type TraitAssociation = "members" | "references";

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
  const [state, setState] = useState(() => traitAssociationState(route.query));
  const availability = useQueries({
    queries: (["members", "references"] as const).map((kind) => {
      const request: TraitRecordsRequest = {
        trait_key: detail.trait_key,
        kind,
        page: { number: 1, size: 5 },
        expected_artifact_generation: detail.artifact_generation,
      };
      return {
        meta: referenceQueryMeta,
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
    setState(traitAssociationState(route.query));
  }, [route.query]);
  function update(kind: TraitAssociation, search = state.search) {
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
  ] as const;
  const visibleItems = items.filter(
    (_, index) => !availability[index].data || availability[index].data.rows.length > 0,
  );
  if (availability.some((query) => query.isPending))
    return <LoadingStatus label="Loading connections…" />;
  if (!visibleItems.length) return null;
  return (
    <section className="record-explore" aria-label="Explore trait">
      <Tabs
        activeKey={
          visibleItems.some((item) => item.key === state.kind)
            ? state.kind
            : visibleItems[0].key
        }
        animated={false}
        destroyOnHidden
        onChange={(value) => update(value as TraitAssociation)}
        items={visibleItems.map((item) => ({
          ...item,
          children: (
            <TraitAssociatedRecords
              key={JSON.stringify([item.key, detail.artifact_generation])}
              detail={detail}
              kind={item.key}
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

function traitAssociationState(query: string | undefined): {
  kind: TraitAssociation;
  search: SearchFormState;
} {
  const params = new URLSearchParams(query);
  const tab = params.get("tab");
  const { sort, sortError } = decodeResultSort(params, "search", true);
  return {
    kind: tab === "references" ? tab : "members",
    search: {
      ...DEFAULT_SEARCH_STATE,
      sort,
      filterError: sortError,
    },
  };
}
type TraitRecordsProps = {
  detail: TraitDetailView;
  onReference: RecordReferenceHandler;
  kind: TraitAssociation;
  onReload: () => void;
  search: SearchFormState;
};
function TraitAssociatedRecords({
  detail,
  kind,
  onReload,
  search,
  onReference,
}: TraitRecordsProps) {
  const profile = useSearchProfile();
  const request: TraitRecordsRequest = {
    trait_key: detail.trait_key,
    kind,
    page: { number: 1, size: 5 },
    expected_artifact_generation: detail.artifact_generation,
  };
  const query = useQuery<TraitRecordsPage>({
    meta: referenceQueryMeta,
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
                    row={{
                      record: row.record,
                      matches: [],
                      relationship_evidence: [],
                    }}
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
