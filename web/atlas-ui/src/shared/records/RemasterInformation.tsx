import { Fragment, useState } from "react";
import { Button, Typography } from "antd";
import { recordDestination } from "../../app/destinations";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import type {
  RelatedCategoryView,
  RemasterCitationView,
  RemasterDeclarationView,
  RemasterEndpointView,
  RemasterRelationshipsView,
  RemasterRelationshipView,
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
  showMessage = true,
}: {
  endpoint: RemasterEndpointView;
  onReference: RecordReferenceHandler;
  showMessage?: boolean;
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
      {showMessage &&
        (resolution.kind === "absent" ||
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

function sourceOriginLabel(origin: RemasterCitationView["origin"]) {
  return origin === "atlas_reviewed" ? "Relationship added by Atlas" : "Foundry source";
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
        {sourceOriginLabel(citation.origin)}
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

function endpointMessage(endpoint: RemasterEndpointView): string | null {
  const resolution = endpoint.resolution;
  return resolution.kind === "absent" ||
    resolution.kind === "unresolved" ||
    resolution.kind === "ambiguous" ||
    resolution.kind === "term"
    ? (resolution.message ?? null)
    : null;
}

/** Compare exact display forms only; qualifiers and case differences stay visible. */
function repeatsVisibleLabels(text: string, labels: Set<string>): boolean {
  if (labels.has(text)) return true;
  const entries = [...labels];
  if (!entries.length) return false;
  const forms = [entries.join(", "), entries.join(" · ")];
  if (entries.length > 1) {
    const last = entries[entries.length - 1];
    const preceding = entries.slice(0, -1).join(", ");
    forms.push(`${preceding} and ${last}`, `${preceding}, and ${last}`);
  }
  return forms.some(
    (form) => text === form || text === `${form}.` || text === `${form} .`,
  );
}

/** Each canonical group keeps its own entries and exact authored wording. */
function SideGroup({
  group,
  onReference,
}: {
  group: RemasterSideView;
  onReference: RecordReferenceHandler;
}) {
  const labels = new Set(group.endpoints.map((endpoint) => endpoint.label));
  const terms = [...new Set(group.terms)].filter((term) => !labels.has(term));
  for (const term of terms) labels.add(term);
  const first = group.endpoints[0];
  const firstMessage = first ? endpointMessage(first) : null;
  const sharedMessage =
    firstMessage &&
    group.endpoints.length > 1 &&
    group.endpoints.every(
      (endpoint) =>
        endpoint.resolution.kind === first.resolution.kind &&
        endpointMessage(endpoint) === firstMessage,
    )
      ? firstMessage
      : null;
  const showCaption = group.text && !repeatsVisibleLabels(group.text, labels);
  return (
    <div className="remaster-side__group">
      {showCaption && <p>{group.text}</p>}
      {group.endpoints.map((endpoint, index) => (
        <RemasterEndpoint
          key={index}
          endpoint={endpoint}
          onReference={onReference}
          showMessage={!sharedMessage}
        />
      ))}
      {sharedMessage && (
        <p className="remaster-information__context">{sharedMessage}</p>
      )}
      {terms.length > 0 && (
        <p className="remaster-information__context">{terms.join(" · ")}</p>
      )}
      {group.truncated && (
        <p className="remaster-information__context">
          More entries exist in this source group.
        </p>
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
        <SideGroup key={index} group={group} onReference={onReference} />
      ))}
    </div>
  );
}

/** Supporting declarations remain evidence; they never become extra product rows. */
function SourceDeclaration({
  declaration,
  onReference,
}: {
  declaration: RemasterDeclarationView;
  onReference: RecordReferenceHandler;
}) {
  return (
    <div className="remaster-source-declaration">
      {declaration.origins
        .filter(
          (origin) =>
            !declaration.citations.some((citation) => citation.origin === origin),
        )
        .map((origin) => (
          <p key={origin} className="remaster-information__context">
            {sourceOriginLabel(origin)}
          </p>
        ))}
      {declaration.citations.map((citation, index) => (
        <RemasterCitation key={index} citation={citation} onReference={onReference} />
      ))}
      <div className="remaster-sides">
        <Side label="Before" groups={declaration.before} onReference={onReference} />
        <Side label="After" groups={declaration.after} onReference={onReference} />
      </div>
      {declaration.citations_truncated && (
        <p className="remaster-information__context">More supporting sources exist.</p>
      )}
      {declaration.truncated && (
        <p className="remaster-information__context">
          Additional details exist in this source statement.
        </p>
      )}
    </div>
  );
}

function Relationship({
  relationship,
  declarations,
  compact,
  onReference,
  referenceAction,
}: {
  relationship: RemasterRelationshipView;
  declarations: RemasterDeclarationView[];
  compact: boolean;
  onReference: RecordReferenceHandler;
  referenceAction?: (navigation: RecordNavigationView) => React.ReactNode;
}) {
  const [sourcesOpen, setSourcesOpen] = useState(false);
  return (
    <div className="remaster-relationship">
      <p className="remaster-relationship__line">
        <strong>
          {relationship.label}
          {relationship.parts.length > 0 ? ":" : ""}
        </strong>
        {relationship.parts.length > 0 && " "}
        {relationship.parts.map((part, index) =>
          part.kind === "text" ? (
            <span key={index}>{part.text}</span>
          ) : (
            <Fragment key={index}>
              <RemasterRecordLink
                navigation={part.navigation}
                onReference={onReference}
              >
                {part.label}
              </RemasterRecordLink>
              {referenceAction?.(part.navigation)}
            </Fragment>
          ),
        )}
        {relationship.truncated && (
          <span className="remaster-information__context">
            {" "}
            More relationship information exists in the sources.
          </span>
        )}
      </p>
      {!compact && (
        <details
          className="remaster-information__evidence"
          onToggle={(event) => setSourcesOpen(event.currentTarget.open)}
        >
          <summary>Sources</summary>
          {sourcesOpen &&
            relationship.declaration_ids.map((id) => {
              const declaration = declarations.find((item) => item.id === id);
              return declaration ? (
                <SourceDeclaration
                  key={id}
                  declaration={declaration}
                  onReference={onReference}
                />
              ) : (
                <p key={id} className="remaster-information__context">
                  Some source information is unavailable.
                </p>
              );
            })}
        </details>
      )}
    </div>
  );
}

export function RemasterInformation({
  category,
  onReference,
  onReload,
  compact = false,
  referenceAction,
}: {
  category: RelatedCategoryView<RemasterRelationshipsView>;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
  compact?: boolean;
  referenceAction?: (navigation: RecordNavigationView) => React.ReactNode;
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
      {content.relationships.map((relationship) => (
        <Relationship
          key={relationship.id}
          relationship={relationship}
          declarations={content.declarations}
          compact={compact}
          onReference={onReference}
          referenceAction={referenceAction}
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
  referenceAction,
}: {
  category?: RelatedCategoryView<RemasterSourceContextView> | null;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
  compact?: boolean;
  referenceAction?: (navigation: RecordNavigationView) => React.ReactNode;
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
  if (category.content.relationships.kind === "empty") return null;
  const source = category.content.source;
  const resolution = source.resolution;
  return (
    <section
      className="remaster-source-context"
      aria-label="Source-entry remaster information"
    >
      <div className="remaster-source-context__heading">
        {resolution.kind === "current" && source.label === resolution.record.title ? (
          <RemasterRecordLink
            navigation={resolution.navigation}
            onReference={onReference}
          >
            {category.content.heading}
          </RemasterRecordLink>
        ) : (
          <span>{category.content.heading}</span>
        )}
        {resolution.kind === "current" && source.label !== resolution.record.title && (
          <>
            {" "}
            ·{" "}
            <RemasterRecordLink
              navigation={resolution.navigation}
              onReference={onReference}
            >
              Open current entry: {resolution.record.title}
            </RemasterRecordLink>
          </>
        )}
      </div>
      <RemasterInformation
        category={category.content.relationships}
        onReference={onReference}
        onReload={onReload}
        compact={compact}
        referenceAction={referenceAction}
      />
      {resolution.kind !== "current" && endpointMessage(source) && (
        <p className="remaster-information__context">{endpointMessage(source)}</p>
      )}
    </section>
  );
}
