import { Alert, Button, Pagination, Space, Typography } from "antd";
import { cloneElement, isValidElement, useState, type AriaAttributes } from "react";
import { useQuery } from "@tanstack/react-query";
import { AtlasApiError, getRelationshipOccurrences } from "../../api/atlasApi";
import {
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import type {
  RecordNavigationView,
  RelationshipConstraintSummaryView,
  RelationshipEvidenceView,
  RelationshipOccurrencesPage,
} from "../../generated/atlas";
import { recordReadingIdentity } from "./useRetainedRecord";
import { isArtifactGenerationChanged } from "../queries/immutableReferenceQueries";

export function RelationshipEvidence({
  evidence,
  summary,
  resultKey,
  generation,
  onRefresh,
  onNavigate,
}: {
  evidence: RelationshipEvidenceView;
  summary: RelationshipConstraintSummaryView;
  resultKey: string;
  generation: string;
  onRefresh?: () => Promise<void>;
  onNavigate?: (selection: RecordNavigationView, anchor: HTMLElement) => void;
}) {
  return (
    <EvidencePage
      key={JSON.stringify([generation, summary.constraint, resultKey])}
      evidence={evidence}
      summary={summary}
      resultKey={resultKey}
      generation={generation}
      onRefresh={onRefresh}
      onNavigate={onNavigate}
    />
  );
}

function EvidencePage({
  evidence,
  summary,
  resultKey,
  generation,
  onRefresh,
  onNavigate,
}: Parameters<typeof RelationshipEvidence>[0]) {
  const [page, setPage] = useState(1);
  const query = useQuery<RelationshipOccurrencesPage>({
    queryKey: [
      "relationship-occurrences",
      generation,
      summary.constraint,
      resultKey,
      page,
      evidence.page.size,
    ],
    enabled: page !== 1,
    staleTime: Infinity,
    placeholderData: (previous) => previous,
    queryFn: async ({ signal }) => {
      const response = await getRelationshipOccurrences(
        {
          constraint: summary.constraint,
          result_record_key: resultKey,
          page: { number: page, size: evidence.page.size },
          expected_artifact_generation: generation,
        },
        signal,
      );
      signal.throwIfAborted();
      if (response.artifact_generation !== generation)
        throw new AtlasApiError(409, "Reference collection changed", {
          code: "artifact_generation_changed",
          message: "Reference collection changed",
          recoverable_action: "refresh_results",
        });
      const actual = response.constraint.constraint;
      const expected = summary.constraint;
      if (
        response.result_record.record_key !== resultKey ||
        actual.direction !== expected.direction ||
        actual.include_contents !== expected.include_contents ||
        recordReadingIdentity(actual.selection.record_key, actual.selection) !==
          recordReadingIdentity(expected.selection.record_key, expected.selection)
      )
        throw new Error("Unable to verify reference links response");
      return response;
    },
  });
  const changed = isArtifactGenerationChanged(query.error);
  const displayed = page === 1 ? evidence : (query.data ?? evidence);
  return (
    <div className="relationship-evidence__body" aria-busy={query.isFetching}>
      {query.error && (
        <Alert
          type="error"
          title="Unable to update links"
          description="Previously loaded links remain below."
          action={
            <Button
              size="small"
              onClick={() => (changed ? void onRefresh?.() : void query.refetch())}
              disabled={changed && !onRefresh}
            >
              {changed ? "Refresh results" : "Retry"}
            </Button>
          }
        />
      )}
      <ul className="relationship-evidence__list">
        {displayed.occurrences.map((occurrence, index) => (
          <li key={JSON.stringify([occurrence.source, occurrence.ordinal, index])}>
            <Space wrap>
              <EvidenceLink
                selection={occurrence.source_navigation}
                title={occurrence.source_title ?? "Referring record"}
                onNavigate={onNavigate}
              />
              <Typography.Text type="secondary">→</Typography.Text>
              {occurrence.target ? (
                <EvidenceLink
                  selection={occurrence.target}
                  title={occurrence.target_title ?? "Referenced record"}
                  onNavigate={onNavigate}
                />
              ) : (
                <Typography.Text type="secondary">
                  Reference unavailable
                </Typography.Text>
              )}
            </Space>
          </li>
        ))}
      </ul>
      {evidence.page.total > BigInt(evidence.page.size) && (
        <Pagination
          size="small"
          simple
          current={page}
          pageSize={evidence.page.size}
          total={Number(evidence.page.total)}
          showSizeChanger={false}
          itemRender={(_number, type, element) =>
            (type === "prev" || type === "next") &&
            isValidElement<AriaAttributes>(element)
              ? cloneElement(element, {
                  "aria-label":
                    type === "prev" ? "Previous links page" : "Next links page",
                })
              : element
          }
          disabled={changed || query.isFetching}
          onChange={setPage}
        />
      )}
      <Typography.Text type="secondary">
        {displayed.occurrences.length} of {displayed.page.total.toString()} links
      </Typography.Text>
    </div>
  );
}

function EvidenceLink({
  selection,
  title,
  onNavigate,
}: {
  selection: RecordNavigationView;
  title: string;
  onNavigate?: (selection: RecordNavigationView, anchor: HTMLElement) => void;
}) {
  const route = { kind: "record", recordKey: selection.record_key, selection } as const;
  return (
    <Typography.Link
      href={atlasRoutePath(route)}
      onClick={(event) => {
        if (!shouldHandleAtlasRouteClick(event)) return;
        event.preventDefault();
        if (onNavigate) onNavigate(selection, event.currentTarget);
        else navigateToAtlasRoute(route);
      }}
    >
      {title}
    </Typography.Link>
  );
}
