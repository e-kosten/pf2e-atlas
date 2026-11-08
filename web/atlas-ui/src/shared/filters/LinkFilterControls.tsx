import { chipLinkClauses } from "./filterControls";
import { Button, Space, Tooltip, Typography } from "antd";
import { CloseOutlined } from "@ant-design/icons";
import type { QueryPredicate } from "../../generated/atlas";
import { countLinkConditions, type LinkLeaf } from "./linkPredicates";

export function LinkFilterControls({
  filter,
  title,
  onEdit,
  onRemove,
  limit,
}: {
  filter: QueryPredicate | null;
  title: (leaf: LinkLeaf) => string;
  onEdit: (leaf: LinkLeaf, opener: HTMLElement) => void;
  onRemove: (leaf: LinkLeaf) => void;
  limit?: number;
}) {
  const clauses = chipLinkClauses(filter);
  const linkCount = countLinkConditions(filter);
  const limitReached = linkCount > 0 && limit !== undefined && linkCount >= limit;
  if (!clauses.length && !limitReached) return null;
  return (
    <div
      className="filter-controls__applied-links"
      role="group"
      aria-label="Link filters"
    >
      {clauses.map((leaf) => (
        <LinkChip
          key={leaf.clause_id}
          leaf={leaf}
          title={title(leaf)}
          onEdit={onEdit}
          onRemove={onRemove}
        />
      ))}
      {limitReached && (
        <Typography.Text role="status">
          Remove a link condition before adding another.
        </Typography.Text>
      )}
    </div>
  );
}
function LinkChip({
  leaf,
  title,
  onEdit,
  onRemove,
}: {
  leaf: LinkLeaf;
  title: string;
  onEdit: (leaf: LinkLeaf, opener: HTMLElement) => void;
  onRemove: (leaf: LinkLeaf) => void;
}) {
  const relationship = leaf.relationship;
  const direction = relationship.direction === "incoming" ? "Links to" : "Linked from";
  return (
    <Space.Compact
      className="relationship-filter applied-filter-chip-group"
      size="small"
      role="group"
      aria-label="Relationship filter"
    >
      <Button
        className="relationship-filter__edit applied-filter-chip"
        title={`${direction} ${title}`}
        color="default"
        variant="filled"
        onClick={(event) => onEdit(leaf, event.currentTarget)}
        aria-haspopup="dialog"
        aria-label={`${direction} ${title}; ${relationship.include_contents ? "Whole entry" : "Main entry only"}`}
      >
        <span className="relationship-filter__label">
          {direction} {title}
        </span>
      </Button>
      <Tooltip title="Remove relationship filter" trigger={["hover", "focus"]}>
        <Button
          className="applied-filter-chip"
          color="default"
          variant="filled"
          icon={<CloseOutlined />}
          aria-label="Remove relationship filter"
          onClick={() => onRemove(leaf)}
        />
      </Tooltip>
    </Space.Compact>
  );
}
