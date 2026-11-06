import { Fragment } from "react";
import { Button, Typography } from "antd";
import { recordDestination } from "../../app/destinations";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import type {
  RelatedCategoryView,
  RemasterRelationshipsView,
  RemasterRelationshipView,
  RemasterSourceContextView,
  RecordNavigationView,
} from "../../generated/atlas";
import { ErrorNotice } from "../feedback/ErrorNotice";
import type { RecordReferenceHandler } from "./PreparedContent";

function RemasterRecordLink({
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

function Relationship({
  relationship,
  onReference,
  referenceAction,
}: {
  relationship: RemasterRelationshipView;
  onReference: RecordReferenceHandler;
  referenceAction?: (navigation: RecordNavigationView) => React.ReactNode;
}) {
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
            More relationship information is available.
          </span>
        )}
      </p>
      {relationship.explanation_links.map((link, index) => (
        <div key={index}>
          <RemasterRecordLink navigation={link.navigation} onReference={onReference}>
            {link.label}
          </RemasterRecordLink>
        </div>
      ))}
    </div>
  );
}

/** Browser reading uses only product facts and useful explanation destinations. */
export function RemasterInformation({
  category,
  onReference,
  onReload,
  referenceAction,
}: {
  category: RelatedCategoryView<RemasterRelationshipsView>;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
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
  return (
    <section
      className="remaster-information"
      aria-label="Legacy and remaster information"
    >
      {category.content.relationships.map((relationship) => (
        <Relationship
          key={relationship.id}
          relationship={relationship}
          onReference={onReference}
          referenceAction={referenceAction}
        />
      ))}
      {category.content.truncated && (
        <p className="remaster-information__context">
          More remaster information is available.
        </p>
      )}
    </section>
  );
}

export function SourceRemasterInformation({
  category,
  onReference,
  onReload,
  referenceAction,
}: {
  category?: RelatedCategoryView<RemasterSourceContextView> | null;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
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
        referenceAction={referenceAction}
      />
    </section>
  );
}
