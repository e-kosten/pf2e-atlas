import { Alert, Button, Empty, Pagination, Space, Spin, Tabs, Typography } from "antd";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
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
} from "../../generated/atlas";
import { PaneFrame } from "../../shared/layout/PaneLayout";
import { useTraitDetail } from "../../shared/records/useTraitDetail";
import { PreparedContentBody } from "../../shared/records/PreparedContent";
import { useSearchProfile } from "../../shared/search/SearchProfileContext";
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
  const client = useQueryClient();
  const reload = async () => {
    await retireImmutableReferenceQueries(client);
    await query.refetch();
  };
  return (
    <main className="record-view">
      <PaneFrame
        className="record-view__pane"
        label="Trait"
        headerActions={
          <Space>
            <Button
              onClick={() => {
                if (history.length > 1) history.back();
                else navigateToAtlasRoute({ kind: "search", selectedRecordKey: null });
              }}
            >
              Back
            </Button>
            <Button onClick={() => void query.refetch()} aria-busy={query.isFetching}>
              Refresh
            </Button>
          </Space>
        }
      >
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
              <header className="record-reference__header">
                <Typography.Title level={1}>
                  {query.data.label}{" "}
                  <Typography.Text type="secondary">Trait</Typography.Text>
                </Typography.Title>
              </header>
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
                onReload={() => void reload()}
              />
            </article>
          ) : query.isLoading ? (
            <div className="detail-empty">
              <Spin delay={500} description="Loading trait…" />
            </div>
          ) : null}
        </section>
      </PaneFrame>
    </main>
  );
}

function TraitAssociations({
  detail,
  onReload,
}: {
  detail: TraitDetailView;
  onReload: () => void;
}) {
  const [kind, setKind] = useState<TraitAssociationKind>("members");
  return (
    <Tabs
      activeKey={kind}
      onChange={(value) => setKind(value as TraitAssociationKind)}
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
            onReload={onReload}
          />
        ),
      }))}
    />
  );
}

function TraitRecords({
  detail,
  kind,
  onReload,
}: {
  detail: TraitDetailView;
  kind: TraitAssociationKind;
  onReload: () => void;
}) {
  const [page, setPage] = useState(1);
  const profile = useSearchProfile();
  const request: TraitRecordsRequest = {
    trait_key: detail.trait_key,
    kind,
    page: { number: page, size: 20 },
    expected_artifact_generation: detail.artifact_generation,
    search_profile: profile.id,
    ranking: "score",
  };
  const query = useQuery({
    queryKey: ["trait-records", request],
    queryFn: ({ signal }) => getTraitRecords(request, signal),
    enabled: kind !== "similar" || !profile.loading,
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
          {data.coverage && !data.coverage.exhaustive && (
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
