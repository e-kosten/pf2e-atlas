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

export function SearchMatchContext({
  row,
  query,
  clauses,
  filter,
  generation,
  onRefresh,
}: {
  row: SearchResultRow;
  query?: string;
  clauses?: RelationshipClauseSummaryView[];
  filter?: QueryPredicate | null;
  generation?: string;
  onRefresh?: () => Promise<void>;
}) {
  const { token } = theme.useToken();
  const onNavigate = useContext(ReaderMatchNavigation);
  const sections = query?.trim() ? (row.matching_sections ?? []) : [];
  const evidence =
    generation && filter
      ? (row.relationship_evidence ?? []).filter((item) =>
          clauses?.some((clause) => clause.clause_id === item.clause_id),
        )
      : [];
  if (!sections.length && !evidence.length) return null;
  return (
    <div className="reader-match-context">
      {!!sections.length && (
        <section aria-label="Search matches">
          <Typography.Title level={3} style={{ fontSize: token.fontSizeLG, margin: 0 }}>
            Search matches for “{query}”
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
                  <Typography.Paragraph type="secondary" style={{ margin: 0 }}>
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
