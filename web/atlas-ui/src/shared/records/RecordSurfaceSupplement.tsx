import { RightOutlined } from "@ant-design/icons";
import { Alert, Button, Collapse, Empty, Space, Spin, Tag } from "antd";
import type React from "react";
import type { RecordSurfaceView } from "../../generated/atlas";
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
  return (
    <section aria-label="Data issues" className="record-surface-issues">
      <h3>Data issues</h3>
      <div className="record-surface-issues__list">
        {issues.map((issue, index) => (
          <Alert
            description={issuePlacementLabel(issue.placement)}
            key={`${issue.code}:${issue.placement}:${index}`}
            message={issue.message}
            showIcon
            type="warning"
          />
        ))}
      </div>
    </section>
  );
}

export function RecordSurfaceReferences({
  loading = false,
  onDisclosureOpen,
  onReference,
  references,
}: {
  loading?: boolean;
  onDisclosureOpen?: () => void;
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
                direction="outgoing"
                onReference={onReference}
                section={references?.outgoing}
                title="References"
              />
              <ReferenceDirection
                direction="backlinks"
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
  direction,
  onReference,
  section,
  title,
}: {
  direction: "backlinks" | "outgoing";
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
          showIcon
          type="warning"
        />
      </section>
    );
  }
  const summary = `${section.total_records} ${pluralize(section.total_records, "record")} · ${section.total_edges} ${pluralize(section.total_edges, "reference")}`;
  return (
    <section aria-label={title}>
      <div className="record-surface-references__heading">
        <h4>{title}</h4>
        <Space size="small" wrap>
          <span>{summary}</span>
          <span>Limit {section.requested_limit}</span>
          {section.truncated ? <Tag>More available</Tag> : null}
        </Space>
      </div>
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
        <Empty
          description={`No ${title.toLowerCase()}.`}
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        />
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
