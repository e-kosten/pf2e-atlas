import { chipLinkClauses } from "./filterControls";
import { linkScopeCopy } from "./linkScopeCopy";
import { useReferencePreview } from "../records/ReferencePreviewContext";
import {
  destinationHref,
  isPrimaryActivation,
  navigateDestination,
} from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { Button, Popover, Radio, Space, Tooltip, Typography } from "antd";
import { CloseOutlined, DownOutlined } from "@ant-design/icons";
import { useRef, useState } from "react";
import type { QueryPredicate, QueryRelationship } from "../../generated/atlas";
export type LinkLeaf = Extract<
  QueryPredicate,
  {
    kind: "relationship";
  }
>;
export function linkClauses(filter: QueryPredicate | null): LinkLeaf[] {
  if (!filter) return [];
  if (filter.kind === "relationship") return [filter];
  if (filter.kind === "all_of" || filter.kind === "any_of")
    return filter.children.flatMap(linkClauses);
  if (filter.kind === "not" || filter.kind === "exists")
    return linkClauses(filter.predicate);
  return [];
}
export function LinkFilterControls({
  filter,
  title,
  onEdit,
  onRemove,
  limit,
}: {
  filter: QueryPredicate | null;
  title: (leaf: LinkLeaf) => string;
  onEdit: (leaf: LinkLeaf, relationship?: QueryRelationship) => string | null;
  onRemove: (leaf: LinkLeaf) => void;
  limit?: number;
}) {
  const clauses = chipLinkClauses(filter);
  const linkCount = linkClauses(filter).length;
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
  onEdit: (leaf: LinkLeaf, relationship?: QueryRelationship) => string | null;
  onRemove: (leaf: LinkLeaf) => void;
}) {
  const reference = useReferencePreview();
  const [open, setOpen] = useState(false);
  const [feedback, setFeedback] = useState<string | null>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const relationship = leaf.relationship;
  const direction = relationship.direction === "incoming" ? "Links to" : "Linked from";
  function close() {
    setOpen(false);
    setFeedback(null);
    trigger.current?.focus({ preventScroll: true });
  }
  function update(next: QueryRelationship) {
    const error = onEdit(leaf, next);
    if (error) setFeedback(error);
    else close();
  }
  function keyDown(event: React.KeyboardEvent<HTMLElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      close();
    }
  }
  return (
    <Space.Compact
      className="relationship-filter applied-filter-chip-group"
      size="small"
      role="group"
      aria-label="Relationship filter"
    >
      <Popover
        destroyOnHidden
        trigger="click"
        placement="bottom"
        arrow={false}
        open={open}
        onOpenChange={(value) => {
          setOpen(value);
          setFeedback(null);
        }}
        styles={{ root: { maxWidth: "calc(100vw - 24px)" } }}
        title={
          <Radio.Group
            aria-label="Relationship direction"
            value={relationship.direction}
            onChange={(event) =>
              update({ ...relationship, direction: event.target.value })
            }
          >
            <Radio.Button onKeyDown={keyDown} value="incoming">
              Links to
            </Radio.Button>
            <Radio.Button onKeyDown={keyDown} value="outgoing">
              Linked from
            </Radio.Button>
          </Radio.Group>
        }
        content={
          <div className="relationship-filter__scope">
            <Typography.Link
              href={destinationHref(
                recordDestination({
                  recordKey: relationship.selection.record_key,
                  selection: relationship.selection,
                }),
              )}
              onKeyDown={keyDown}
              onClick={(event) => {
                if (!isPrimaryActivation(event)) return;
                event.preventDefault();
                if (reference) {
                  close();
                  reference.open(
                    relationship.selection.record_key,
                    trigger.current ?? event.currentTarget,
                    relationship.selection,
                  );
                  return;
                }
                navigateDestination(
                  recordDestination({
                    recordKey: relationship.selection.record_key,
                    selection: relationship.selection,
                  }),
                );
              }}
            >
              Open {title}
            </Typography.Link>
            <Typography.Paragraph strong>
              {relationship.direction === "incoming"
                ? "Look for links in matching records"
                : "Look for links in this entry"}
            </Typography.Paragraph>
            <Radio.Group
              aria-label="Relationship scope"
              value={relationship.include_contents}
              onChange={(event) =>
                update({ ...relationship, include_contents: event.target.value })
              }
            >
              <Space orientation="vertical">
                <Radio onKeyDown={keyDown} value={false}>
                  Main entry only
                </Radio>
                <Radio onKeyDown={keyDown} value={true}>
                  Whole entry
                </Radio>
              </Space>
            </Radio.Group>
            <Typography.Paragraph type="secondary">
              <strong>Whole entry:</strong>{" "}
              {linkScopeCopy[relationship.direction].whole}
            </Typography.Paragraph>
            <Button
              onClick={() => {
                close();
                onEdit(leaf);
              }}
            >
              Change record
            </Button>
            {feedback && <Typography.Text role="status">{feedback}</Typography.Text>}
          </div>
        }
      >
        <Button
          ref={trigger}
          className="relationship-filter__edit applied-filter-chip"
          title={`${direction} ${title}`}
          color="default"
          variant="filled"
          icon={<DownOutlined aria-hidden="true" />}
          iconPlacement="end"
          onKeyDown={keyDown}
          aria-expanded={open}
          aria-label={`${direction} ${title}; ${relationship.include_contents ? "Whole entry" : "Main entry only"}`}
        >
          <span className="relationship-filter__label">
            {direction} {title}
          </span>
        </Button>
      </Popover>
      <Tooltip title="Remove relationship filter" trigger={["hover", "focus"]}>
        <Button
          className="applied-filter-chip"
          color="default"
          variant="filled"
          icon={<CloseOutlined />}
          aria-label="Remove relationship filter"
          onClick={() => {
            close();
            onRemove(leaf);
          }}
        />
      </Tooltip>
    </Space.Compact>
  );
}
