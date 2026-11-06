import { navigateDestination } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import type { RemasterAliasMatchView } from "../../generated/atlas";
import { RemasterCitation } from "./RemasterInformation";
import { useReferencePreview } from "./ReferencePreviewContext";
import type { RecordReferenceHandler } from "./PreparedContent";

/** Alias witnesses are delivered by retrieval; display-name equality proves nothing. */
export function RemasterAliasEvidence({
  matches,
}: {
  matches: RemasterAliasMatchView[];
}) {
  const preview = useReferencePreview();
  if (!matches.length) return null;
  const onReference: RecordReferenceHandler = (
    recordKey,
    anchor,
    selection,
    landing,
  ) => {
    if (preview) preview.open(recordKey, anchor, selection, landing);
    else navigateDestination(recordDestination({ recordKey, selection, landing }));
  };
  return (
    <div className="remaster-alias-evidence">
      {matches.map((match, index) => (
        <details key={index}>
          <summary>
            {match.lookup_context.label}: “{match.matched_text}”
          </summary>
          {match.lookup_context.supports.map((support, supportIndex) => (
            <div key={supportIndex}>
              <p>{support.kind_labels.join(" · ")}</p>
              {support.context.length > 0 && <p>{support.context.join(" · ")}</p>}
              {support.citations.map((citation, citationIndex) => (
                <RemasterCitation
                  key={citationIndex}
                  citation={citation}
                  onReference={onReference}
                />
              ))}
              {support.citations_truncated && <p>More supporting sources exist.</p>}
            </div>
          ))}
          {match.lookup_context.truncated && <p>More supporting statements exist.</p>}
        </details>
      ))}
    </div>
  );
}
