import { useReferenceRecordNavigation } from "../../shared/records/ReferenceHelp";
import { ReferenceHelpProvider } from "../../shared/records/ReferenceHelp";
import { LinkNotFound } from "../../app/InvalidUrl";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { searchDestination } from "../../app/destinations";
import type { TraitPageInput } from "../../app/destinations";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { Button } from "antd";
import { useQuery, useQueryClient } from "@tanstack/react-query";
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
                  <TraitMembers
                    key={JSON.stringify([
                      query.data.trait_key,
                      query.data.artifact_generation,
                    ])}
                    detail={query.data}
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
function TraitMembers({
  detail,
  onReload,
  onReference,
}: {
  detail: TraitDetailView;
  onReload: () => void;
  onReference: RecordReferenceHandler;
}) {
  const profile = useSearchProfile();
  const request: TraitRecordsRequest = {
    trait_key: detail.trait_key,
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
  if (query.isPending) return <LoadingStatus label="Loading records…" />;
  if (data?.rows.length === 0) return null;

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
  return (
    <section
      className="record-explore record-explore__task"
      aria-label="Records with this trait"
      aria-busy={query.isFetching}
    >
      <h3>With this trait</h3>
      <ExploreSearchAction
        route={memberSearch}
        summary={
          data
            ? `${data.rows.length} of ${data.page.total.toString()} ${data.page.total === 1 ? "entry" : "entries"}`
            : undefined
        }
      >
        Search records with this trait
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
      {data && (
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
                context={
                  row.source_label && row.navigation.embedded_path.length
                    ? `Trait on ${row.source_label}`
                    : (row.source_label ?? undefined)
                }
                onPreview={(anchor) =>
                  onReference(row.navigation.record_key, anchor, row.navigation)
                }
              />
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
