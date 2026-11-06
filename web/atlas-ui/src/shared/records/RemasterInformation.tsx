import { Button, Typography } from "antd";
import { recordDestination } from "../../app/destinations";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import type {
  RelatedCategoryView,
  RemasterCitationView,
  RemasterDeclarationView,
  RemasterEndpointView,
  RemasterRelationshipsView,
  RemasterSideView,
  RemasterSourceContextView,
  RecordNavigationView,
} from "../../generated/atlas";
import { ErrorNotice } from "../feedback/ErrorNotice";
import type { RecordReferenceHandler } from "./PreparedContent";

export function RemasterRecordLink({
  navigation,
  children,
  onReference,
}: {
  navigation: RecordNavigationView;
  children: React.ReactNode;
  onReference: RecordReferenceHandler;
}) {
  return (
    <Typography.Link
      href={destinationHref(
        recordDestination({ recordKey: navigation.record_key, selection: navigation }),
      )}
      onClick={(event) => {
        if (!isPrimaryActivation(event)) return;
        event.preventDefault();
        onReference(navigation.record_key, event.currentTarget, navigation);
      }}
    >
      {children}
    </Typography.Link>
  );
}

export function RemasterEndpoint({
  endpoint,
  onReference,
}: {
  endpoint: RemasterEndpointView;
  onReference: RecordReferenceHandler;
}) {
  const resolution = endpoint.resolution;
  return (
    <div className="remaster-endpoint">
      {resolution.kind === "current" ? (
        endpoint.label === resolution.record.title ? (
          <RemasterRecordLink
            navigation={resolution.navigation}
            onReference={onReference}
          >
            {endpoint.label}
          </RemasterRecordLink>
        ) : (
          <>
            <span>{endpoint.label}</span>
            <div>
              <RemasterRecordLink
                navigation={resolution.navigation}
                onReference={onReference}
              >
                Open current entry: {resolution.record.title}
              </RemasterRecordLink>
            </div>
          </>
        )
      ) : (
        <span>{endpoint.label}</span>
      )}
      {endpoint.context && (
        <span className="remaster-information__context"> · {endpoint.context}</span>
      )}
      {(resolution.kind === "absent" ||
        resolution.kind === "unresolved" ||
        resolution.kind === "term" ||
        resolution.kind === "ambiguous") &&
        resolution.message && (
          <p className="remaster-information__context">{resolution.message}</p>
        )}
      {resolution.kind === "ambiguous" && (
        <ul>
          {resolution.candidates.map((candidate) => (
            <li key={JSON.stringify(candidate.navigation)}>
              <RemasterRecordLink
                navigation={candidate.navigation}
                onReference={onReference}
              >
                {candidate.record.title}
              </RemasterRecordLink>
              <span className="remaster-information__context">
                {" "}
                ·{" "}
                {[candidate.record.kind_label, candidate.record.publication]
                  .filter(Boolean)
                  .join(" · ")}
              </span>
            </li>
          ))}
          {resolution.truncated && <li>More possible matches exist.</li>}
        </ul>
      )}
    </div>
  );
}

export function RemasterCitation({
  citation,
  onReference,
}: {
  citation: RemasterCitationView;
  onReference: RecordReferenceHandler;
}) {
  const destination = citation.destination;
  return (
    <div className="remaster-citation">
      <small className="remaster-information__context">
        {citation.origin === "atlas_reviewed"
          ? "Atlas-reviewed correspondence"
          : "Foundry source"}
      </small>
      <div>
        {destination.kind === "internal" ? (
          <RemasterRecordLink
            navigation={destination.navigation}
            onReference={onReference}
          >
            {citation.attribution}
          </RemasterRecordLink>
        ) : destination.kind === "external" ? (
          <Typography.Link
            href={destination.url}
            target="_blank"
            rel="noopener noreferrer"
          >
            {citation.attribution}
          </Typography.Link>
        ) : (
          citation.attribution
        )}
      </div>
      <p>
        {citation.statement}
        {citation.shortened && (
          <span className="remaster-information__context"> (excerpt shortened)</span>
        )}
      </p>
      <small className="remaster-information__context">
        Revision: {citation.revision}
      </small>
      {destination.kind === "attribution" && destination.reason && (
        <p className="remaster-information__context">{destination.reason}</p>
      )}
    </div>
  );
}

function Side({
  label,
  groups,
  onReference,
}: {
  label: string;
  groups: RemasterSideView[];
  onReference: RecordReferenceHandler;
}) {
  if (!groups.length) return null;
  return (
    <div className="remaster-side">
      <h4>{label}</h4>
      {groups.map((group, index) => (
        <div key={index} className="remaster-side__group">
          {group.text && <p>{group.text}</p>}
          {group.endpoints.map((endpoint, endpointIndex) => (
            <RemasterEndpoint
              key={endpointIndex}
              endpoint={endpoint}
              onReference={onReference}
            />
          ))}
          {group.terms.length > 0 && (
            <p className="remaster-information__context">{group.terms.join(" · ")}</p>
          )}
          {group.truncated && (
            <p className="remaster-information__context">
              More entries exist in this source group.
            </p>
          )}
        </div>
      ))}
    </div>
  );
}

function Declaration({
  declaration,
  onReference,
}: {
  declaration: RemasterDeclarationView;
  onReference: RecordReferenceHandler;
}) {
  return (
    <article className="remaster-declaration">
      <h3>{declaration.heading}</h3>
      {declaration.summary && <p>{declaration.summary}</p>}
      {declaration.authored_status && <p>{declaration.authored_status}</p>}
      {declaration.context.length > 0 && (
        <p className="remaster-information__context">
          {declaration.context.join(" · ")}
        </p>
      )}
      <div className="remaster-sides">
        <Side label="Before" groups={declaration.before} onReference={onReference} />
        <Side label="After" groups={declaration.after} onReference={onReference} />
      </div>
      {declaration.truncated && (
        <p className="remaster-information__context">
          Additional details exist in this source statement.
        </p>
      )}
      <details className="remaster-information__evidence">
        <summary>Sources ({declaration.citations.length})</summary>
        {declaration.citations.map((citation, index) => (
          <RemasterCitation key={index} citation={citation} onReference={onReference} />
        ))}
        {declaration.citations_truncated && <p>More supporting sources exist.</p>}
      </details>
    </article>
  );
}

export function RemasterInformation({
  category,
  onReference,
  onReload,
  compact = false,
}: {
  category: RelatedCategoryView<RemasterRelationshipsView>;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
  compact?: boolean;
}) {
  if (category.kind === "empty") return null;
  if (category.kind === "error")
    return (
      <ErrorNotice
        compact
        title="Remaster information could not be checked"
        error={category.error}
        action={
          onReload && (
            <Button size="small" onClick={onReload}>
              Retry
            </Button>
          )
        }
      />
    );
  if (category.kind === "unavailable")
    return <p className="remaster-information__context">{category.reason}</p>;
  const content = category.content;
  return (
    <section
      className={`remaster-information${compact ? " remaster-information--compact" : ""}`}
      aria-label="Legacy and remaster information"
    >
      {compact
        ? content.notices.map((notice) => (
            <div key={notice.declaration_id} className="remaster-notice">
              <strong>{notice.heading}</strong>
              {notice.description && <p>{notice.description}</p>}
              {notice.links.length > 0 && (
                <ul>
                  {notice.links.map((link, index) => (
                    <li key={index}>
                      <RemasterRecordLink
                        navigation={link.navigation}
                        onReference={onReference}
                      >
                        {link.label}
                      </RemasterRecordLink>
                    </li>
                  ))}
                </ul>
              )}
              {notice.attribution && (
                <small className="remaster-information__context">
                  {notice.attribution}
                </small>
              )}
            </div>
          ))
        : content.declarations.map((declaration) => (
            <Declaration
              key={declaration.id}
              declaration={declaration}
              onReference={onReference}
            />
          ))}
      {content.truncated && (
        <p className="remaster-information__context">
          More remaster information exists in the sources.
        </p>
      )}
    </section>
  );
}

export function SourceRemasterInformation({
  category,
  onReference,
  onReload,
  compact = false,
}: {
  category?: RelatedCategoryView<RemasterSourceContextView> | null;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
  compact?: boolean;
}) {
  if (!category || category.kind === "empty") return null;
  if (category.kind === "error")
    return (
      <ErrorNotice
        compact
        title="Source-entry remaster information could not be checked"
        error={category.error}
        action={
          onReload && (
            <Button size="small" onClick={onReload}>
              Retry
            </Button>
          )
        }
      />
    );
  if (category.kind === "unavailable")
    return <p className="remaster-information__context">{category.reason}</p>;
  return (
    <section
      className="remaster-source-context"
      aria-label="Source-entry remaster information"
    >
      <h3>{category.content.heading}</h3>
      <RemasterEndpoint endpoint={category.content.source} onReference={onReference} />
      <RemasterInformation
        category={category.content.relationships}
        onReference={onReference}
        onReload={onReload}
        compact={compact}
      />
    </section>
  );
}
