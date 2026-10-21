import { Button, Popover, Radio, Space, Tag, Typography, theme } from "antd";
import { CloseOutlined, DownOutlined } from "@ant-design/icons";
import { useRef, useState, type KeyboardEvent } from "react";
import {
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import type { RelationshipConstraintView } from "../../generated/atlas";
import { recordReadingIdentity } from "../../shared/records/useRetainedRecord";
import type { SearchWorkspaceState } from "./useSearchWorkspace";

export function RelationshipFilters({
  workspace,
}: {
  workspace: SearchWorkspaceState;
}) {
  return (workspace.search.relationships ?? []).map((constraint, index) => (
    <RelationshipFilter
      key={index}
      constraint={constraint}
      index={index}
      workspace={workspace}
    />
  ));
}

function RelationshipFilter({
  constraint,
  index,
  workspace,
}: {
  constraint: RelationshipConstraintView;
  index: number;
  workspace: SearchWorkspaceState;
}) {
  const { token } = theme.useToken();
  const [scopeOpen, setScopeOpen] = useState(false);
  const scopeTrigger = useRef<HTMLButtonElement>(null);
  const constraints = workspace.search.relationships ?? [];
  const summary = workspace.resultPage?.relationships?.find(
    (candidate) =>
      candidate.constraint.direction === constraint.direction &&
      recordReadingIdentity(
        candidate.constraint.selection.record_key,
        candidate.constraint.selection,
      ) ===
        recordReadingIdentity(constraint.selection.record_key, constraint.selection),
  );
  const route = {
    kind: "record",
    recordKey: constraint.selection.record_key,
    selection: constraint.selection,
  } as const;
  function closeScope() {
    setScopeOpen(false);
    scopeTrigger.current?.focus({ preventScroll: true });
  }
  function scopeKeyDown(event: KeyboardEvent<HTMLElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      closeScope();
    }
  }
  return (
    <Space
      wrap
      className="relationship-filter"
      role="group"
      aria-label="Relationship filter"
    >
      <Tag>{constraint.direction === "incoming" ? "Links to" : "Linked from"}</Tag>
      {summary ? (
        <Typography.Link
          href={atlasRoutePath(route)}
          onClick={(event) => {
            if (!shouldHandleAtlasRouteClick(event)) return;
            event.preventDefault();
            navigateToAtlasRoute(route);
          }}
        >
          {summary.title}
        </Typography.Link>
      ) : (
        <Typography.Text type="secondary">
          {workspace.errorMessage ? "Record unavailable" : "Loading record…"}
        </Typography.Text>
      )}
      {summary && constraint.selection.embedded_path.length > 0 && (
        <Typography.Text type="secondary">{summary.record.title}</Typography.Text>
      )}
      <Popover
        trigger="click"
        placement="bottom"
        arrow={false}
        styles={{ root: { maxWidth: "calc(100vw - 24px)" } }}
        open={scopeOpen}
        onOpenChange={setScopeOpen}
        title={
          constraint.direction === "incoming"
            ? "Look for links in matching records"
            : "Look for links in this entry"
        }
        content={
          <div className="relationship-filter__scope">
            <Radio.Group
              aria-label="Relationship scope"
              value={constraint.include_contents}
              onChange={(event) => {
                workspace.setSearch({
                  ...workspace.search,
                  relationships: constraints.map((candidate, candidateIndex) =>
                    candidateIndex === index
                      ? { ...candidate, include_contents: event.target.value }
                      : candidate,
                  ),
                });
                closeScope();
              }}
            >
              <Space orientation="vertical">
                <Radio onKeyDown={scopeKeyDown} value={false}>
                  Main entry only
                </Radio>
                <Radio onKeyDown={scopeKeyDown} value={true}>
                  Whole entry
                </Radio>
              </Space>
            </Radio.Group>
            <Typography.Paragraph type="secondary">
              {constraint.direction === "incoming"
                ? "Whole entry also checks each matching record’s abilities, spells, equipment, and other nested documents."
                : "Whole entry also checks this entry’s abilities, spells, equipment, and other nested documents."}
            </Typography.Paragraph>
          </div>
        }
      >
        <Button
          ref={scopeTrigger}
          type="text"
          size="small"
          icon={<DownOutlined aria-hidden="true" />}
          iconPlacement="end"
          styles={{
            root: {
              color: token.colorTextSecondary,
              fontSize: token.fontSizeSM,
            },
          }}
          onKeyDown={scopeKeyDown}
          aria-label="Relationship scope"
          aria-expanded={scopeOpen}
        >
          {constraint.include_contents ? "Whole entry" : "Main entry only"}
        </Button>
      </Popover>
      <Button
        type="text"
        icon={<CloseOutlined />}
        aria-label="Remove relationship filter"
        onClick={() =>
          workspace.setSearch({
            ...workspace.search,
            relationships: constraints.filter(
              (_, candidateIndex) => candidateIndex !== index,
            ),
          })
        }
      />
    </Space>
  );
}
