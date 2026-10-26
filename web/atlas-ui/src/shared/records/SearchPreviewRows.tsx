import { useState } from "react";
import type { QueryPredicate, SearchResultsPage } from "../../generated/atlas";
import type { RecordReferenceHandler } from "./PreparedContent";
import { ReferenceResult } from "./ReferenceResult";

export function SearchPreviewRows({
  page,
  filter,
  onReference,
  onReload,
}: {
  page: SearchResultsPage;
  filter?: QueryPredicate;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
}) {
  const [open, setOpen] = useState<string | null>(null);
  return (
    <div className="record-explore__results" role="list">
      {page.rows.map((row) => (
        <div role="listitem" key={row.record.record_key}>
          <ReferenceResult
            row={row}
            selected={false}
            disabled={false}
            relationshipClauses={page.relationship_clauses}
            filter={filter}
            generation={page.artifact_generation}
            onRefreshResults={async () => {
              onReload?.();
            }}
            matchesOpen={open === row.record.record_key}
            onMatchesOpenChange={(show) => setOpen(show ? row.record.record_key : null)}
            onPreview={(anchor) => onReference(row.record.record_key, anchor)}
            onMatch={(selection, anchor) =>
              onReference(selection.record_key, anchor, selection)
            }
          />
        </div>
      ))}
    </div>
  );
}
