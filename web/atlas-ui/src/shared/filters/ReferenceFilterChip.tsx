import { CloseOutlined } from "@ant-design/icons";
import { Button, Tag } from "antd";
import type { RelationshipConstraint } from "../../generated/atlas";
import { useRecordDetail } from "../records/useRecordDetail";

export function ReferenceFilterChip({
  relationship,
  onRemove,
}: {
  relationship: RelationshipConstraint;
  onRemove: () => void;
}) {
  const detail = useRecordDetail(relationship.record_key);
  const metadata = detail.data?.surface.metadata;
  const label = detail.isLoading
    ? "Loading record…"
    : detail.isError
      ? "Record unavailable"
      : metadata?.record_key === relationship.record_key
        ? metadata.title
        : "Record not found";
  return (
    <Tag>
      {relationship.direction === "incoming" ? "References" : "Referenced by"}: {label}
      <Button
        type="text"
        size="small"
        aria-label="Remove reference filter"
        icon={<CloseOutlined />}
        onClick={onRemove}
      />
    </Tag>
  );
}
