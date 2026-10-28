import { DEFAULT_MATCH_DISPLAY, type MatchDisplay } from "../search/searchPreferences";
import { useContext } from "react";
import { ReaderMatchNavigation } from "./ReaderMatchNavigation";
import { Typography, theme } from "antd";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import type {
  QueryPredicate,
  RelationshipClauseSummaryView,
  SearchResultRow,
} from "../../generated/atlas";
import { RelationshipEvidence } from "./RelationshipEvidence";
import { SimilarityExplanation } from "./SimilarityExplanation";

export function SearchMatchContext({
  row,
  query,
  clauses,
  filter,
  generation,
  onRefresh,
  matchDisplay = DEFAULT_MATCH_DISPLAY,
  showSmallerWeights = false,
}: {
  row: SearchResultRow;
  query?: string;
  clauses?: RelationshipClauseSummaryView[];
  filter?: QueryPredicate | null;
  generation?: string;
  onRefresh?: () => Promise<void>;
  matchDisplay?: MatchDisplay;
  showSmallerWeights?: boolean;
}) {
  const { token } = theme.useToken();
  const onNavigate = useContext(ReaderMatchNavigation);
  if (matchDisplay === "hidden") return null;
  const similarity = matchDisplay === "detailed" ? row.similarity : undefined;
  const sections = !similarity || query?.trim() ? (row.matching_sections ?? []) : [];
  const evidence =
    generation && filter
      ? (row.relationship_evidence ?? []).filter((item) =>
          clauses?.some((clause) => clause.clause_id === item.clause_id),
        )
      : [];
  if (!sections.length && !evidence.length && !similarity) return null;
  return (
    <div className="reader-match-context">
      {similarity && (
        <section aria-label="Similarity comparison">
          <Typography.Title level={3} style={{ fontSize: token.fontSizeLG, margin: 0 }}>
            Similarity comparison
          </Typography.Title>
          <SimilarityExplanation
            evidence={similarity}
            showSmallerWeights={showSmallerWeights}
            onNavigate={onNavigate ?? undefined}
          />
        </section>
      )}
      {!!sections.length && (
        <section aria-label="Search matches">
          <Typography.Title level={3} style={{ fontSize: token.fontSizeLG, margin: 0 }}>
            {query?.trim() ? `Search matches for “${query}”` : "Search matches"}
          </Typography.Title>
          <ul className="reader-match-context__sections">
            {sections.map((section) => (
              <li key={JSON.stringify(section.navigation)}>
                <Typography.Link
                  href={destinationHref(
                    recordDestination({
                      recordKey: section.navigation.record_key,
                      selection: section.navigation,
                    }),
                  )}
                  onClick={(event) => {
                    if (!onNavigate || !isPrimaryActivation(event)) return;
                    event.preventDefault();
                    onNavigate(section.navigation, event.currentTarget);
                  }}
                >
                  {section.label}
                </Typography.Link>
                {section.snippet && (
                  <Typography.Paragraph
                    type="secondary"
                    ellipsis={{ rows: 2 }}
                    style={{ margin: 0 }}
                  >
                    {section.snippet}
                  </Typography.Paragraph>
                )}
              </li>
            ))}
          </ul>
        </section>
      )}
      {!!evidence.length && (
        <section aria-label="Matching reference links">
          <Typography.Title level={3} style={{ fontSize: token.fontSizeLG, margin: 0 }}>
            Reference links
          </Typography.Title>
          {evidence.map((item) => (
            <RelationshipEvidence
              key={item.clause_id}
              evidence={item}
              summary={clauses!.find((clause) => clause.clause_id === item.clause_id)!}
              filter={filter!}
              generation={generation!}
              resultKey={row.record.record_key}
              onRefresh={onRefresh}
              onNavigate={onNavigate ?? undefined}
            />
          ))}
        </section>
      )}
    </div>
  );
}
