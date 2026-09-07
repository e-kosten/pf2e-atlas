import { RightOutlined } from "@ant-design/icons";
import { Alert, Button, Collapse, Empty, Space, Spin, Tag, Typography } from "antd";
import type React from "react";
import type { RecordSurfaceView } from "../../generated/atlas";
import { navigateToAtlasRoute } from "../../app/routes";
import { formatSlug } from "./recordFormatting";

type RecordSurfaceIssue = NonNullable<RecordSurfaceView["issues"]>[number];
type RecordSurfaceIssuePlacement = RecordSurfaceIssue["placement"];
type RecordSurfaceReferences = NonNullable<RecordSurfaceView["references"]>;
type RecordSurfaceReferenceSection = RecordSurfaceReferences["outgoing"];
type AvailableReferenceSection = Extract<
  RecordSurfaceReferenceSection,
  { state: "available" }
>;
type RecordSurfaceReferenceEdge = AvailableReferenceSection["edges"][number];
type RecordSurfaceReferenceRecord = AvailableReferenceSection["records"][number];

export function RecordSurfaceIssues({
  issues,
}: {
  issues: RecordSurfaceIssue[] | undefined;
}) {
  if (!issues?.length) return null;
  const groups = new Map<string, RecordSurfaceIssue[]>();
  issues.forEach((issue, index) => {
    const key = issue.subject ? JSON.stringify(issue.subject) : `issue:${index}`;
    groups.set(key, [...(groups.get(key) ?? []), issue]);
  });
  return (
    <section aria-label="Data issues" className="record-surface-issues">
      <h3>Data issues</h3>
      <div className="record-surface-issues__list">
        {Array.from(groups, ([key, facts]) => (
          <Alert
            key={key}
            message={facts.length === 1 ? facts[0].message : facts[0].subject?.label}
            description={
              facts.length > 3 ? (
                <Collapse
                  expandIcon={disclosureExpandIcon}
                  ghost
                  size="small"
                  items={[
                    {
                      key: "facts",
                      label: `${facts.length} affected facts`,
                      children: <IssueFacts facts={facts} />,
                    },
                  ]}
                />
              ) : (
                <IssueFacts facts={facts} />
              )
            }
            showIcon
            type="warning"
          />
        ))}
      </div>
    </section>
  );
}

function IssueFacts({ facts }: { facts: RecordSurfaceIssue[] }) {
  return (
    <ul className="record-surface-issues__facts">
      {facts.map((issue, index) => (
        <li
          key={issue.fact_id ?? index}
          data-issue-code={issue.code}
          data-fact-id={issue.fact_id}
        >
          <IssueContext issue={issue} />
          {facts.length > 1 ? <p>{issue.message}</p> : null}
        </li>
      ))}
    </ul>
  );
}

function IssueContext({ issue }: { issue: RecordSurfaceIssue }) {
  const placement = issuePlacementLabel(issue.placement);
  const target = issue.subject?.target;
  return (
    <Space size="small" wrap>
      {issue.subject ? (
        target?.target_type === "activity" ? (
          <Typography.Link href={`#hazard-activity-${target.occurrence_id}`}>
            {issue.subject.label}
          </Typography.Link>
        ) : (
          <span>{issue.subject.label}</span>
        )
      ) : (
        <span>{placement}</span>
      )}
      {issue.fact_label ? <span>· {issue.fact_label}</span> : null}
    </Space>
  );
}

export function RecordSurfaceReferences({
  loading = false,
  onDisclosureOpen,
  onRequestLimit,
  onReference,
  references,
  recordKey,
}: {
  recordKey?: string;
  loading?: boolean;
  onDisclosureOpen?: () => void;
  onRequestLimit?: (direction: "backlinks" | "outgoing", limit: number) => void;
  onReference: (recordKey: string) => void;
  references: RecordSurfaceReferences | undefined;
}) {
  if (!references && !onDisclosureOpen) return null;
  return (
    <Collapse
      className="record-surface__secondary record-surface-references"
      expandIcon={disclosureExpandIcon}
      ghost
      items={[
        {
          key: "record-references",
          label: "References",
          children: loading ? (
            <div className="record-surface-references__loading">
              <Spin size="small" />
              <span>Loading linked records…</span>
            </div>
          ) : (
            <div className="record-surface-references__directions">
              <ReferenceDirection
                recordKey={recordKey}
                direction="outgoing"
                onRequestLimit={onRequestLimit}
                onReference={onReference}
                section={references?.outgoing}
                title="References"
              />
              <ReferenceDirection
                recordKey={recordKey}
                direction="backlinks"
                onRequestLimit={onRequestLimit}
                onReference={onReference}
                section={references?.backlinks}
                title="Referenced by"
              />
            </div>
          ),
        },
      ]}
      onChange={(activeKeys) => {
        const keys = Array.isArray(activeKeys) ? activeKeys : [activeKeys];
        if (keys.includes("record-references")) onDisclosureOpen?.();
      }}
      size="small"
    />
  );
}

function ReferenceDirection({
  recordKey,
  direction,
  onRequestLimit,
  onReference,
  section,
  title,
}: {
  recordKey?: string;
  direction: "backlinks" | "outgoing";
  onRequestLimit?: (direction: "backlinks" | "outgoing", limit: number) => void;
  onReference: (recordKey: string) => void;
  section: RecordSurfaceReferenceSection | undefined;
  title: string;
}) {
  if (!section || section.state === "not_requested") {
    return (
      <section aria-label={title}>
        <h4>{title}</h4>
        <Empty
          description={
            direction === "backlinks"
              ? "Open this disclosure to request backlinks."
              : "Linked records were not requested."
          }
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        />
      </section>
    );
  }
  if (section.state === "unavailable") {
    return (
      <section aria-label={title}>
        <h4>{title}</h4>
        <Alert
          description={`Requested limit ${section.requested_limit}`}
          message={section.message}
          action={
            onRequestLimit ? (
              <Button
                size="small"
                onClick={() => onRequestLimit(direction, section.requested_limit)}
              >
                Retry
              </Button>
            ) : undefined
          }
          showIcon
          type="warning"
        />
      </section>
    );
  }
  const shown = section.records.length;
  const summary = `Showing ${shown} of ${section.total_records} ${pluralize(section.total_records, "record")}`;
  const capped = section.truncated && section.next_limit === undefined;
  return (
    <section aria-label={title}>
      <div className="record-surface-references__heading">
        <h4>{title}</h4>
        <Space size="small" wrap>
          <span>{summary}</span>
          <span>
            {section.total_edges} {pluralize(section.total_edges, "reference")}
          </span>
          {section.next_limit !== undefined && onRequestLimit ? (
            <Button
              onClick={() => onRequestLimit(direction, section.next_limit as number)}
              size="small"
              type="link"
            >
              View more
            </Button>
          ) : null}
        </Space>
      </div>
      {capped ? (
        <p className="record-surface-references__cap">
          Atlas currently exposes up to {section.requested_limit} linked records.{" "}
        </p>
      ) : null}
      {recordKey && section.total_records > 0 ? (
        <Button
          type="link"
          onClick={() =>
            navigateToAtlasRoute({
              kind: "search",
              selectedRecordKey: null,
              relationship: {
                direction: direction === "backlinks" ? "incoming" : "outgoing",
                record_key: recordKey,
              },
            })
          }
        >
          {direction === "backlinks"
            ? "See all referencing records"
            : "See all referenced records"}
        </Button>
      ) : null}
      {direction === "backlinks" && section.records.length ? (
        <p>Records that reference this record.</p>
      ) : null}
      {section.records.length ? (
        <ul className="record-surface-references__list">
          {section.records.map((record) => (
            <ReferenceRecord
              direction={direction}
              edges={section.edges}
              key={record.record_key}
              onReference={onReference}
              record={record}
            />
          ))}
        </ul>
      ) : (
        <p>No {title.toLowerCase()}.</p>
      )}
    </section>
  );
}

function ReferenceRecord({
  direction,
  edges,
  onReference,
  record,
}: {
  direction: "backlinks" | "outgoing";
  edges: RecordSurfaceReferenceEdge[];
  onReference: (recordKey: string) => void;
  record: RecordSurfaceReferenceRecord;
}) {
  const matchingEdges = edges.filter((edge) =>
    direction === "outgoing"
      ? edge.to_record_key === record.record_key
      : edge.from_record_key === record.record_key,
  );
  return (
    <li>
      <div className="record-surface-references__record">
        <Button onClick={() => onReference(record.record_key)} type="link">
          {record.title}
        </Button>
        <Tag>{formatSlug(record.kind)}</Tag>
      </div>
      {matchingEdges.length ? (
        <ul className="record-surface-references__evidence">
          {matchingEdges.map((edge, index) => (
            <li key={index}>{edge.display_text ?? edge.reference_text}</li>
          ))}
        </ul>
      ) : null}
    </li>
  );
}

function issuePlacementLabel(placement: RecordSurfaceIssuePlacement) {
  switch (placement) {
    case "record":
      return "Record";
    case "classification":
      return "Classification";
    case "defenses":
      return "Defenses";
    case "activity":
      return "Activity";
    case "casting":
      return "Casting";
    case "targeting":
      return "Targeting";
    case "damage":
      return "Damage";
    case "duration":
      return "Duration";
    case "heightening":
      return "Heightening";
    case "ritual":
      return "Ritual";
    case "rules":
      return "Rules";
    case "forms":
      return "Forms";
    case "references":
      return "References";
  }
}

function pluralize(value: number, singular: string) {
  return value === 1 ? singular : `${singular}s`;
}

const disclosureExpandIcon: NonNullable<
  React.ComponentProps<typeof Collapse>["expandIcon"]
> = ({ isActive }) => <RightOutlined aria-hidden="true" rotate={isActive ? 90 : 0} />;
