import { Typography } from "antd";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import type { RecordGrantsView } from "../../generated/atlas";
import type { RecordReferenceHandler } from "./PreparedContent";
import { ReferenceResult } from "./ReferenceResult";

export function RecordGrants({
  grants,
  onReference,
}: {
  grants: RecordGrantsView;
  onReference: RecordReferenceHandler;
}) {
  return (
    <section className="record-grants" aria-label="Grants">
      <h3>Grants</h3>
      <div role="list">
        {grants.entries.map((entry) => (
          <div key={JSON.stringify(entry.navigation)} role="listitem">
            {entry.record ? (
              <ReferenceResult
                row={{ record: entry.record, matches: [], relationship_evidence: [] }}
                navigation={entry.navigation}
                selected={false}
                disabled={false}
                matchesOpen={false}
                onMatchesOpenChange={() => {}}
                onPreview={(anchor) =>
                  onReference(entry.navigation.record_key, anchor, entry.navigation)
                }
                onMatch={(navigation, anchor) =>
                  onReference(navigation.record_key, anchor, navigation)
                }
              />
            ) : (
              <Typography.Link
                href={destinationHref(
                  recordDestination({
                    recordKey: entry.navigation.record_key,
                    selection: entry.navigation,
                  }),
                )}
                onClick={(event) => {
                  if (!isPrimaryActivation(event)) return;
                  event.preventDefault();
                  onReference(
                    entry.navigation.record_key,
                    event.currentTarget,
                    entry.navigation,
                  );
                }}
              >
                {entry.title}
              </Typography.Link>
            )}
          </div>
        ))}
      </div>
      {grants.unavailable_count > 0 && (
        <p className="muted">
          {grants.unavailable_count === 1
            ? "One granted reference is unavailable."
            : `${grants.unavailable_count} granted references are unavailable.`}
        </p>
      )}
      {grants.truncated && (
        <p className="muted">Some granted references could not be loaded.</p>
      )}
    </section>
  );
}
