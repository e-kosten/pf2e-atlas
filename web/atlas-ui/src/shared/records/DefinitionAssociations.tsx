import { Button, Empty, Tabs } from "antd";
import { useQuery } from "@tanstack/react-query";
import { searchPage } from "../../api/atlasApi";
import { searchDestination } from "../../app/destinations";
import type { ReferenceFilterActionView } from "../../generated/atlas";
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

type SearchAction = Extract<ReferenceFilterActionView, { kind: "search" }>;

/** Standalone reading follows the trait page's bounded association previews. */
export function DefinitionAssociations({
  actions,
  generation,
  onReference,
  onReload,
}: {
  actions: ReferenceFilterActionView[];
  generation: string;
  onReference: RecordReferenceHandler;
  onReload: () => void;
}) {
  const searches = actions.filter(
    (action): action is SearchAction => action.kind === "search",
  );
  if (!searches.length) return null;
  return (
    <section className="record-explore" aria-label="Related records">
      <Tabs
        animated={false}
        destroyOnHidden
        items={searches.map((action, index) => ({
          key: JSON.stringify([index, action.predicate, generation]),
          label: action.label,
          children: (
            <DefinitionAssociatedRecords
              action={action}
              generation={generation}
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
  generation,
  onReference,
  onReload,
}: {
  action: SearchAction;
  generation: string;
  onReference: RecordReferenceHandler;
  onReload: () => void;
}) {
  const profile = useSearchProfile();
  const staleAction = action.artifact_generation !== generation;
  const query = useQuery({
    meta: referenceQueryMeta,
    queryKey: [
      "definition-association-preview",
      action.predicate,
      generation,
      profile.id,
    ],
    staleTime: Infinity,
    retryOnMount: false,
    enabled: !staleAction && !profile.loading && profile.id !== null,
    queryFn: async ({ signal }) => {
      const page = await searchPage(
        {
          search_profile: profile.id ?? undefined,
          criteria: { kind: "list_records", filter: action.predicate },
          page: { number: 1, size: 5 },
          expected_artifact_generation: generation,
        },
        signal,
      );
      assertSearchIdentity(page, profile.id, generation);
      return page;
    },
  });
  const changed = staleAction || isArtifactGenerationChanged(query.error);
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
      {(query.error || staleAction) && (
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
      {data &&
        (data.rows.length ? (
          <SearchPreviewRows
            page={data}
            filter={action.predicate}
            onReference={onReference}
            onReload={onReload}
          />
        ) : (
          <Empty
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            description="No matching records"
          />
        ))}
    </section>
  );
}
