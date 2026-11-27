import { Button } from "antd";
import { recordDestination } from "../../app/destinations";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import type { ExactSearchView } from "../../generated/atlas";
import { SearchNameLabel } from "../../shared/search/SearchNameLabel";

export function ExactMatches({
  page,
  selectedRecordKey,
  onSelect,
}: {
  page: ExactSearchView;
  selectedRecordKey: string | null;
  onSelect: (key: string, generation: string) => void;
}) {
  const additional = page.total - page.matches.length;
  return (
    <section className="search-exact-matches" aria-label="Exact matches">
      <h2>Exact matches</h2>
      <ul>
        {page.matches.map(({ record, match_kind, matched_text }) => (
          <li key={record.record_key}>
            <Button
              color="default"
              variant="text"
              block
              className="search-exact-matches__link"
              href={destinationHref(
                recordDestination({ recordKey: record.record_key }),
              )}
              aria-current={
                selectedRecordKey === record.record_key ? "true" : undefined
              }
              data-reader-opener
              onClick={(event) => {
                if (!isPrimaryActivation(event)) return;
                event.preventDefault();
                onSelect(record.record_key, page.artifact_generation);
              }}
            >
              <SearchNameLabel
                record={record}
                alias={match_kind === "alias" ? matched_text : undefined}
              />
            </Button>
          </li>
        ))}
      </ul>
      {additional > 0 && (
        <span className="search-name-label__source">
          {additional} more exact {additional === 1 ? "match" : "matches"}
        </span>
      )}
    </section>
  );
}
