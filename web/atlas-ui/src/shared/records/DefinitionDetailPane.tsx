import { Button, Spin } from "antd";
import { ErrorNotice } from "../feedback/ErrorNotice";
import type { DefinitionDetailView } from "../../generated/atlas";
import { DocumentIdentityHeader } from "./DocumentIdentityHeader";
import { PreparedContentBody, type RecordReferenceHandler } from "./PreparedContent";
import { definitionClassification } from "./ReferenceHelp";
import { RecordContextError } from "./recordContextError";

export function DefinitionDetailPane({
  detail,
  loading,
  error,
  onRetry,
  retryLabel = "Retry",
  onReference,
  identityInHost = false,
}: {
  detail?: DefinitionDetailView;
  loading: boolean;
  error?: Error | null;
  onRetry?: () => void;
  retryLabel?: string;
  onReference: RecordReferenceHandler;
  identityInHost?: boolean;
}) {
  return (
    <section className="detail-panel" aria-busy={loading}>
      {detail && loading && (
        <div className="detail-refresh" aria-label="Updating definition">
          <Spin delay={500} size="small" />
        </div>
      )}
      {error && (
        <ErrorNotice
          error={error}
          title={
            detail
              ? "Unable to refresh this definition"
              : "Unable to load this definition"
          }
          description={
            error instanceof RecordContextError
              ? error.explanation
              : detail?.content
                ? "Previously loaded content remains below."
                : undefined
          }
          action={onRetry && <Button onClick={onRetry}>{retryLabel}</Button>}
        />
      )}
      {detail ? (
        <article className="record-reference" data-selection-search>
          {!identityInHost && (
            <DocumentIdentityHeader
              title={detail.label}
              classification={definitionClassification(detail.definition_key)}
              attribution={<DefinitionAttribution detail={detail} />}
            />
          )}
          {detail.content && (
            <PreparedContentBody
              body={detail.content.body}
              references={detail.content.references}
              sectionRendering={detail.content.section_rendering}
              onReference={onReference}
            />
          )}
        </article>
      ) : loading ? (
        <Spin delay={500} description="Loading definition…">
          <div className="detail-empty" />
        </Spin>
      ) : null}
    </section>
  );
}

export function DefinitionAttribution({ detail }: { detail: DefinitionDetailView }) {
  const attribution = detail.attribution;
  if (!attribution) return null;
  const publications = attribution.publications
    .filter((publication) => publication.title?.trim())
    .map((publication) =>
      [publication.title, publication.page == null ? null : `p. ${publication.page}`]
        .filter(Boolean)
        .join(", "),
    )
    .filter(Boolean);
  if (!publications.length) return null;
  return (
    <div className="muted record-reference__source">
      <p>Source: {publications.join("; ")}</p>
    </div>
  );
}
