import { Button, Tabs } from "antd";
import { queryOptions, useQueries, type UseQueryResult } from "@tanstack/react-query";
import { searchPage } from "../../api/atlasApi";
import { searchDestination } from "../../app/destinations";
import type { DefinitionDetailView, SearchResultsPage } from "../../generated/atlas";
import { ErrorNotice } from "../feedback/ErrorNotice";
import { DEFAULT_SEARCH_STATE, searchStateQueryString } from "../filters/searchState";
import {
  isArtifactGenerationChanged,
  referenceQueryMeta,
} from "../queries/referenceQueries";
import { useSearchProfile } from "../search/SearchProfileContext";
import { SearchProfileNotice } from "../search/SearchProfileNotice";
import { assertSearchIdentity } from "../search/searchIdentity";
import { LoadingStatus } from "../ui/LoadingStatus";
import { ExploreSearchAction } from "./ExploreSearchAction";
import type { RecordReferenceHandler } from "./PreparedContent";
import { SearchPreviewRows } from "./SearchPreviewRows";

type SearchAction = DefinitionDetailView["related_filters"][number];

/** Standalone reading follows the trait page's bounded association previews. */
export function DefinitionAssociations({
  actions,
  generation,
  onReference,
  onReload,
}: {
  actions: SearchAction[];
  generation: string;
  onReference: RecordReferenceHandler;
  onReload: () => void;
}) {
  const profile = useSearchProfile();
  const availability = useQueries({
    queries: actions.map((action) =>
      associationPreviewQueryOptions(action, generation, profile.id, profile.loading),
    ),
  });
  const visible = actions
    .map((action, index) => ({ action, query: availability[index], index }))
    .filter(
      ({ query }) =>
        query.error ||
        query.isFetching ||
        !query.data ||
        query.data.page.total !== 0n ||
        query.data.rows.length > 0,
    );
  if (!visible.length) return null;
  return (
    <section className="record-explore" aria-label="Related records">
      <Tabs
        animated={false}
        destroyOnHidden
        items={visible.map(({ action, query, index }) => ({
          key: JSON.stringify([index, action.predicate, generation]),
          label: action.label,
          children: (
            <DefinitionAssociatedRecords
              action={action}
              query={query}
              onReference={onReference}
              onReload={onReload}
            />
          ),
        }))}
      />
    </section>
  );
}

function DefinitionAssociatedRecords({
  action,
  query,
  onReference,
  onReload,
}: {
  action: SearchAction;
  query: UseQueryResult<SearchResultsPage, Error>;
  onReference: RecordReferenceHandler;
  onReload: () => void;
}) {
  const profile = useSearchProfile();
  const changed = isArtifactGenerationChanged(query.error);
  const data = changed ? undefined : query.data;
  const route = searchDestination({
    query: searchStateQueryString(
      { ...DEFAULT_SEARCH_STATE, filter: action.predicate },
      profile.id ?? profile.pendingUrlProfile,
    ),
  });
  return (
    <section aria-label={action.label} aria-busy={query.isFetching}>
      <ExploreSearchAction
        route={route}
        disabled={changed}
        summary={
          data
            ? `${data.rows.length} of ${data.page.total.toString()} ${data.page.total === 1n ? "entry" : "entries"}`
            : undefined
        }
      >
        Search records
      </ExploreSearchAction>
      {profile.error && <SearchProfileNotice />}
      {query.error && (
        <ErrorNotice
          error={query.error ?? "The reference collection changed."}
          type={changed ? "warning" : "error"}
          title={
            changed ? "Reference collection changed" : "Unable to load these records"
          }
          description={data ? "Previously loaded records remain below." : undefined}
          action={
            <Button onClick={() => (changed ? onReload() : void query.refetch())}>
              {changed ? "Reload definition" : "Retry"}
            </Button>
          }
        />
      )}
      {query.isLoading && <LoadingStatus label="Loading records…" />}
      {data && query.isFetching && (
        <div className="detail-refresh">
          <LoadingStatus label="Updating associated records…" />
        </div>
      )}
      {data && data.rows.length > 0 && (
        <SearchPreviewRows
          page={data}
          filter={action.predicate}
          onReference={onReference}
          onReload={onReload}
        />
      )}
    </section>
  );
}

function associationPreviewQueryOptions(
  action: SearchAction,
  generation: string,
  profileId: string | null,
  profileLoading: boolean,
) {
  return queryOptions({
    meta: referenceQueryMeta,
    queryKey: [
      "definition-association-preview",
      action.predicate,
      generation,
      profileId,
    ],
    staleTime: Infinity,
    retryOnMount: false,
    enabled: !profileLoading && profileId !== null,
    queryFn: async ({ signal }) => {
      const page = await searchPage(
        {
          search_profile: profileId ?? undefined,
          criteria: { kind: "list_records", filter: action.predicate },
          page: { number: 1, size: 5 },
          expected_artifact_generation: generation,
        },
        signal,
      );
      assertSearchIdentity(page, profileId, generation);
      return page;
    },
  });
}
