import { Alert, Button, Spin } from "antd";
import type { ActorActivityView, OwnedReadingEntryView } from "../../generated/atlas";
import { ActorActivity } from "./ActorReference";
import { ContentFields } from "./RecordPresentation";
import type { RecordReferenceHandler } from "./PreparedContent";

/** Authored prose alongside supplied current activity facts; no detail fallback. */
export function ActivityContentPane({
  activity,
  entry,
  error,
  loading,
  onRetry,
  onReference,
}: {
  activity?: ActorActivityView;
  entry?: OwnedReadingEntryView;
  error?: Error | null;
  loading: boolean;
  onRetry?: () => void;
  onReference: RecordReferenceHandler;
}) {
  return (
    <section className="record-reference record-reference--compact" aria-busy={loading}>
      {activity && (
        <ActorActivity
          activity={activity}
          onReference={onReference}
          inline
          permalink={false}
        />
      )}
      {error && (
        <Alert
          type="error"
          title="Activity content unavailable"
          description={error.message}
          action={
            onRetry && (
              <Button size="small" onClick={onRetry}>
                Retry
              </Button>
            )
          }
        />
      )}
      {entry && (
        <ContentFields
          fields={entry.content}
          relationships={entry.relationships}
          onReference={onReference}
        />
      )}
      {loading && (
        <div role="status">
          <Spin size="small" /> Loading activity content…
        </div>
      )}
    </section>
  );
}
