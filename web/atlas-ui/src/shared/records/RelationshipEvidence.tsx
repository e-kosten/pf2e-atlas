import { useReferenceRecordNavigation } from "./ReferenceHelp";
import { referenceQueryMeta } from "../queries/referenceQueries";
import { ErrorNotice } from "../feedback/ErrorNotice";
import {
  destinationHref,
  isPrimaryActivation,
  navigateDestination,
} from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { Button, Pagination, Space, Typography } from "antd";
import { cloneElement, isValidElement, useState, type AriaAttributes } from "react";
import { useQuery } from "@tanstack/react-query";
import { AtlasApiError, getRelationshipOccurrences } from "../../api/atlasApi";
import type {
  RecordNavigationView,
  RelationshipClauseSummaryView,
  RelationshipEvidenceView,
  RelationshipOccurrencesPage,
} from "../../generated/atlas";
import { sameReferenceCondition } from "../filters/linkPredicates";
import { isArtifactGenerationChanged } from "../queries/referenceQueries";
import { ReferenceHelpLink } from "./ReferenceHelp";
export function RelationshipEvidence({
  evidence,
  filter,
  summary,
  resultKey,
  generation,
  onRefresh,
  onNavigate,
}: {
  evidence: RelationshipEvidenceView;
  filter: import("../../generated/atlas").QueryPredicate;
  summary: RelationshipClauseSummaryView;
  resultKey: string;
  generation: string;
  onRefresh?: () => Promise<void>;
  onNavigate?: (selection: RecordNavigationView, anchor: HTMLElement) => void;
}) {
  return (
    <EvidencePage
      key={JSON.stringify([
        generation,
        filter,
        summary.clause_id,
        summary.condition,
        resultKey,
      ])}
      evidence={evidence}
      filter={filter}
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
  filter,
  summary,
  resultKey,
  generation,
  onRefresh,
  onNavigate,
}: Parameters<typeof RelationshipEvidence>[0]) {
  const [page, setPage] = useState(1);
  const query = useQuery<RelationshipOccurrencesPage>({
    meta: referenceQueryMeta,
    queryKey: [
      "relationship-occurrences",
      generation,
      filter,
      summary.clause_id,
      summary.condition,
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
          filter,
          clause_id: summary.clause_id,
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
      if (
        response.result_record.record_key !== resultKey ||
        response.clause.clause_id !== summary.clause_id ||
        !sameReferenceCondition(response.clause.condition, summary.condition)
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
        <ErrorNotice
          error={query.error}
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
        {displayed.occurrences.map((entry, index) => {
          const occurrence = entry.occurrence;
          const source =
            entry.kind === "record"
              ? {
                  kind: "record" as const,
                  navigation: entry.occurrence.source_navigation,
                }
              : entry.occurrence.source_destination;
          const target =
            entry.kind === "record"
              ? entry.destination
              : entry.occurrence.target_destination;
          return (
            <li key={JSON.stringify([entry.kind, occurrence.source, index])}>
              <Space wrap>
                <EvidenceDestination
                  destination={source}
                  title={occurrence.source_title ?? "Referring record"}
                  onNavigate={onNavigate}
                />
                <Typography.Text type="secondary">→</Typography.Text>
                <EvidenceDestination
                  destination={target}
                  definitionKey={
                    summary.condition.kind === "definition" &&
                    summary.condition.reference.target.kind === "definition"
                      ? summary.condition.reference.target.key
                      : undefined
                  }
                  title={
                    occurrence.target_title ??
                    (target ? "Referenced record" : "Reference unavailable")
                  }
                  onNavigate={onNavigate}
                />
              </Space>
            </li>
          );
        })}
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
function EvidenceDestination({
  destination,
  definitionKey,
  title,
  onNavigate,
}: {
  destination: import("../../generated/atlas").ReferenceDestinationView | null;
  definitionKey?: string;
  title: string;
  onNavigate?: (selection: RecordNavigationView, anchor: HTMLElement) => void;
}) {
  if (!destination) return <Typography.Text>{title}</Typography.Text>;
  return destination.kind === "record" && !definitionKey ? (
    <EvidenceLink
      selection={destination.navigation}
      title={title}
      onNavigate={onNavigate}
    />
  ) : (
    <ReferenceHelpLink
      help={{
        definition_key:
          destination.kind === "definition" ? destination.key : (definitionKey ?? null),
        availability: "available",
        destination,
        related_filters: [],
      }}
    >
      {title}
    </ReferenceHelpLink>
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
  const reference = useReferenceRecordNavigation();
  const route = recordDestination({
    recordKey: selection.record_key,
    selection,
  });
  return (
    <Typography.Link
      href={destinationHref(route)}
      onClick={(event) => {
        if (!isPrimaryActivation(event)) return;
        event.preventDefault();
        if (onNavigate) onNavigate(selection, event.currentTarget);
        else if (reference)
          reference(selection.record_key, event.currentTarget, selection);
        else navigateDestination(route);
      }}
    >
      {title}
    </Typography.Link>
  );
}
