import { Button, Popover, Radio, Space, Tooltip, Typography, theme } from "antd";
import { CloseOutlined, DownOutlined } from "@ant-design/icons";
import { useRef, useState } from "react";
import type {
  QueryPredicate,
  QueryRelationship,
  FilterEditorFieldView,
} from "../../generated/atlas";
import {
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
export type LinkLeaf = Extract<QueryPredicate, { kind: "relationship" }>;
export function linkClauses(filter: QueryPredicate | null): LinkLeaf[] {
  if (!filter) return [];
  if (filter.kind === "relationship") return [filter];
  if (filter.kind === "all_of" || filter.kind === "any_of")
    return filter.children.flatMap(linkClauses);
  if (filter.kind === "not" || filter.kind === "exists")
    return linkClauses(filter.predicate);
  return [];
}
function simple(filter: QueryPredicate): boolean {
  if (filter.kind === "not" || filter.kind === "any_of") return false;
  if (filter.kind === "all_of") return filter.children.every(simple);
  if (filter.kind === "exists") return simple(filter.predicate);
  return true;
}
export function humanExpression(
  filter: QueryPredicate,
  title: (leaf: LinkLeaf) => string,
  fields: FilterEditorFieldView[],
): string {
  if (filter.kind === "all_of" || filter.kind === "any_of")
    return `(${filter.children.map((child) => humanExpression(child, title, fields)).join(filter.kind === "all_of" ? " AND " : " OR ")})`;
  if (filter.kind === "not")
    return `NOT ${humanExpression(filter.predicate, title, fields)}`;
  if (filter.kind === "relationship")
    return `${filter.relationship.direction === "incoming" ? "Links to" : "Linked from"} ${title(filter)} [${filter.relationship.include_contents ? "Whole entry" : "Main entry only"}]`;
  if (filter.kind === "boolean_constant")
    return filter.value ? "All records" : "No records";
  const field = fields.find(
    (entry) =>
      entry.definition.path ===
      (filter.kind === "exists" ? filter.collection : filter.field),
  );
  const name = field?.label ?? "Unavailable field";
  if (filter.kind === "exists")
    return `${name}: ${humanExpression(filter.predicate, title, fields)}`;
  if (filter.kind === "state_match") return `${name}: ${filter.state}`;
  if (filter.kind === "in") return `${name}: ${filter.values.join(", ")}`;
  const operator =
    field?.operators.find((entry) => entry.operator === filter.op)?.label ?? filter.op;
  return `${name} ${operator} ${filter.kind === "set_match" ? filter.values.join(", ") : filter.value}`;
}
export function LinkFilterControls({
  filter,
  fields,
  title,
  onAdd,
  onEdit,
  onRemove,
  onAdvanced,
  limit,
}: {
  filter: QueryPredicate | null;
  fields: FilterEditorFieldView[];
  title: (leaf: LinkLeaf) => string;
  onAdd: () => void;
  onEdit: (leaf: LinkLeaf, relationship?: QueryRelationship) => string | null;
  onRemove: (leaf: LinkLeaf) => void;
  onAdvanced: () => void;
  limit?: number;
}) {
  const clauses = linkClauses(filter);
  const expression =
    filter && clauses.length ? humanExpression(filter, title, fields) : "";
  const rootOperator = "Grouped filters";
  return (
    <>
      <Button onClick={onAdd}>
        Links <DownOutlined aria-hidden="true" />
      </Button>
      {!!clauses.length &&
        (filter && simple(filter) ? (
          clauses.map((leaf) => (
            <LinkChip
              key={leaf.clause_id}
              leaf={leaf}
              title={title(leaf)}
              onEdit={onEdit}
              onRemove={onRemove}
            />
          ))
        ) : (
          <Tooltip
            title={<span style={{ whiteSpace: "normal" }}>{expression}</span>}
            trigger={["hover", "focus"]}
            styles={{ root: { maxWidth: "min(600px, calc(100vw - 24px))" } }}
          >
            <Button
              className="link-filter-summary"
              onClick={onAdvanced}
              aria-label={`${rootOperator}: ${clauses.length} link conditions. Edit advanced filters.`}
            >
              <span>
                {rootOperator} · {clauses.length} link conditions
              </span>
              <span className="link-filter-summary__expression">{expression}</span>
            </Button>
          </Tooltip>
        ))}
      {limit !== undefined && clauses.length >= limit && (
        <Typography.Text role="status">
          Remove a link condition before adding another.
        </Typography.Text>
      )}
    </>
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
  const { token } = theme.useToken();
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
    <div
      className="relationship-filter"
      style={{ gap: token.marginXXS }}
      role="group"
      aria-label="Relationship filter"
    >
      <Popover
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
              href={atlasRoutePath({
                kind: "record",
                recordKey: relationship.selection.record_key,
                selection: relationship.selection,
              })}
              onKeyDown={keyDown}
              onClick={(event) => {
                if (!shouldHandleAtlasRouteClick(event)) return;
                event.preventDefault();
                navigateToAtlasRoute({
                  kind: "record",
                  recordKey: relationship.selection.record_key,
                  selection: relationship.selection,
                });
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
              {relationship.direction === "incoming"
                ? "Whole entry also checks each matching record’s abilities, spells, equipment, and other nested documents."
                : "Whole entry also checks this entry’s abilities, spells, equipment, and other nested documents."}
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
      <Button
        type="text"
        icon={<CloseOutlined />}
        aria-label="Remove relationship filter"
        onClick={() => {
          close();
          onRemove(leaf);
        }}
      />
    </div>
  );
}
