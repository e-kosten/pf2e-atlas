import { Alert, Button, Collapse, Spin } from "antd";
import { ErrorNotice } from "../feedback/ErrorNotice";
import type { DefinitionDetailView } from "../../generated/atlas";
import { DocumentIdentityHeader } from "./DocumentIdentityHeader";
import { PreparedContentBody, type RecordReferenceHandler } from "./PreparedContent";
import { ReferenceHelpLink } from "./ReferenceHelp";
import { ReferenceFilterActions } from "../filters/ReferenceFilterActions";

export function DefinitionDetailPane({
  detail,
  loading,
  error,
  onRetry,
  onReference,
  identityInHost = false,
}: {
  detail?: DefinitionDetailView;
  loading: boolean;
  error?: Error | null;
  onRetry?: () => void;
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
          description={detail ? "Previously loaded content remains below." : undefined}
          action={onRetry && <Button onClick={onRetry}>Retry</Button>}
        />
      )}
      {detail ? (
        <article className="record-reference" data-selection-search>
          {!identityInHost && (
            <DocumentIdentityHeader
              title={detail.label}
              classification="Definition"
              attribution={<DefinitionAttribution detail={detail} />}
            />
          )}
          <ReferenceFilterActions
            actions={detail.help.related_filters}
            generation={detail.artifact_generation}
          />
          {detail.content ? (
            <PreparedContentBody
              body={detail.content.body}
              references={detail.content.references}
              sectionRendering={detail.content.section_rendering}
              onReference={onReference}
            />
          ) : (
            <Alert
              type="info"
              title={definitionAvailabilityLabel(detail.help.availability)}
            />
          )}
          {detail.references.length > 0 && (
            <section className="record-reference__content">
              <h3>References</h3>
              <ul>
                {detail.references.map((reference, index) => (
                  <li key={JSON.stringify([reference.occurrence_path, index])}>
                    <ReferenceHelpLink
                      help={{
                        definition_key:
                          reference.destination.kind === "definition"
                            ? reference.destination.key
                            : null,
                        availability: "available",
                        destination: reference.destination,
                        related_filters: [],
                      }}
                    >
                      {reference.label ?? "Referenced entry"}
                    </ReferenceHelpLink>
                  </li>
                ))}
              </ul>
            </section>
          )}
          {detail.references_truncated && (
            <p className="muted">
              Additional references are available through Advanced Filters.
            </p>
          )}
          <DefinitionNotices detail={detail} />
        </article>
      ) : loading ? (
        <Spin delay={500} description="Loading definition…">
          <div className="detail-empty" />
        </Spin>
      ) : null}
    </section>
  );
}

export function definitionAvailabilityLabel(
  availability: DefinitionDetailView["help"]["availability"],
) {
  switch (availability) {
    case "setup_required":
      return "Run Atlas setup to prepare this definition.";
    case "label_only":
      return "This entry supplies a name without a definition.";
    case "withdrawn":
      return "This definition is no longer supplied.";
    case "missing_target":
      return "The referenced entry is unavailable in this collection.";
    default:
      return "No definition is available in this reference collection.";
  }
}

export function DefinitionAttribution({ detail }: { detail: DefinitionDetailView }) {
  const attribution = detail.attribution;
  if (!attribution) return null;
  const publications = attribution.publications
    .map((publication) =>
      [publication.title, publication.page == null ? null : `p. ${publication.page}`]
        .filter(Boolean)
        .join(", "),
    )
    .filter(Boolean);
  return (
    <div className="muted record-reference__source">
      <p>
        Source:{" "}
        {publications.length ? publications.join("; ") : attribution.source_package}
      </p>
      {attribution.content_locale &&
        attribution.content_locale !== attribution.indexing_locale && (
          <p>Content language: {attribution.content_locale}</p>
        )}
    </div>
  );
}

function DefinitionNotices({ detail }: { detail: DefinitionDetailView }) {
  const attribution = detail.attribution;
  if (!attribution && !detail.notices.length) return null;
  return (
    <Collapse
      items={[
        {
          key: "sources",
          label: "Source and notices",
          children: (
            <>
              {attribution && (
                <dl>
                  <dt>Source package</dt>
                  <dd>{attribution.source_package}</dd>
                  {attribution.source_repository && (
                    <>
                      <dt>Repository</dt>
                      <dd>{attribution.source_repository}</dd>
                    </>
                  )}
                  {attribution.source_revision && (
                    <>
                      <dt>Revision</dt>
                      <dd>{attribution.source_revision}</dd>
                    </>
                  )}
                  {attribution.source_file && (
                    <>
                      <dt>Source file</dt>
                      <dd>{attribution.source_file}</dd>
                    </>
                  )}
                  {attribution.source_locations.map((location, index) => (
                    <div key={index}>
                      <dt>Declaration</dt>
                      <dd>
                        {location.file}:{location.line}:{location.column}
                      </dd>
                    </div>
                  ))}
                  {[
                    ...attribution.body_grants.map((grant) => ({
                      ...grant,
                      area: "Content",
                    })),
                    ...attribution.label_grants.map((grant) => ({
                      ...grant,
                      area: "Names",
                    })),
                  ].map((grant, index) => (
                    <div key={index}>
                      <dt>{grant.area}</dt>
                      <dd>
                        {grant.license} ·{" "}
                        {grant.scope === "functional_content"
                          ? "Functional content"
                          : "Descriptive references"}
                      </dd>
                    </div>
                  ))}
                </dl>
              )}
              {detail.notices.map((notice) => (
                <section key={notice.id}>
                  <h4>{notice.id}</h4>
                  {notice.source_url && (
                    <p>
                      <a href={notice.source_url}>Published notice</a>
                    </p>
                  )}
                  <div style={{ whiteSpace: "pre-wrap", overflowWrap: "anywhere" }}>
                    {notice.text}
                  </div>
                </section>
              ))}
            </>
          ),
        },
      ]}
    />
  );
}
