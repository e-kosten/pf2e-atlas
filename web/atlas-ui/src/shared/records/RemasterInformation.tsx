import { Info } from "lucide-react";
import { PaneIconLink } from "../ui/actions/PaneAction";
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

export type RemasterReferencePart = Extract<
  RemasterRelationshipView["parts"][number],
  { kind: "reference" }
>;

function Relationship({
  relationship,
  onReference,
  referenceActions,
  showExplanations = true,
}: {
  relationship: RemasterRelationshipView;
  onReference: RecordReferenceHandler;
  referenceActions?: (references: RemasterReferencePart[]) => React.ReactNode;
  showExplanations?: boolean;
}) {
  const actions = referenceActions?.(
    relationship.parts.filter(
      (part): part is RemasterReferencePart => part.kind === "reference",
    ),
  );
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
            <RemasterRecordLink
              key={index}
              navigation={part.navigation}
              onReference={onReference}
            >
              {part.label}
            </RemasterRecordLink>
          ),
        )}
        {showExplanations &&
          relationship.explanation_links.map((link, index) => (
            <PaneIconLink
              key={index}
              size="small"
              label={link.label}
              icon={<Info size={14} aria-hidden="true" />}
              href={destinationHref(
                recordDestination({
                  recordKey: link.navigation.record_key,
                  selection: link.navigation,
                }),
              )}
              onClick={(event) => {
                if (!isPrimaryActivation(event)) return;
                event.preventDefault();
                onReference(
                  link.navigation.record_key,
                  event.currentTarget,
                  link.navigation,
                );
              }}
            />
          ))}
        {relationship.truncated && (
          <span className="remaster-information__context">
            {" "}
            More relationship information is available.
          </span>
        )}
      </p>
      {actions && <div className="remaster-relationship__actions">{actions}</div>}
    </div>
  );
}

/** Browser reading uses only product facts and useful explanation destinations. */
export function RemasterInformation({
  category,
  onReference,
  onReload,
  referenceActions,
  showExplanations = true,
}: {
  category: RelatedCategoryView<RemasterRelationshipsView>;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
  referenceActions?: (references: RemasterReferencePart[]) => React.ReactNode;
  showExplanations?: boolean;
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
          referenceActions={referenceActions}
          showExplanations={showExplanations}
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
  referenceActions,
  showExplanations = true,
}: {
  category?: RelatedCategoryView<RemasterSourceContextView> | null;
  onReference: RecordReferenceHandler;
  onReload?: () => void;
  referenceActions?: (references: RemasterReferencePart[]) => React.ReactNode;
  showExplanations?: boolean;
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
        referenceActions={referenceActions}
        showExplanations={showExplanations}
      />
    </section>
  );
}
