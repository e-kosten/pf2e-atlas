import { Button, Popover, Radio, Space, Typography, theme } from "antd";
import { CloseOutlined, DownOutlined } from "@ant-design/icons";
import { useRef, useState, type KeyboardEvent } from "react";
import {
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import type {
  RelationshipConstraintView,
  RelationshipDirectionView,
} from "../../generated/atlas";
import { recordReadingIdentity } from "../../shared/records/useRetainedRecord";
import type { SearchWorkspaceState } from "./useSearchWorkspace";
import { sameRelationshipConstraint } from "./relationshipConstraints";

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
  const [feedback, setFeedback] = useState<string | null>(null);
  const scopeTrigger = useRef<HTMLButtonElement>(null);
  const constraints = workspace.search.relationships ?? [];
  const summary = workspace.resultPage?.relationships?.find(
    (candidate) =>
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
  const direction = constraint.direction === "incoming" ? "Links to" : "Linked from";
  const title = summary
    ? summary.title +
      (constraint.selection.embedded_path.length ? ` · ${summary.record.title}` : "")
    : workspace.errorMessage
      ? "Record unavailable"
      : "Loading record…";
  function closeScope() {
    setScopeOpen(false);
    setFeedback(null);
    scopeTrigger.current?.focus({ preventScroll: true });
  }
  function update(next: RelationshipConstraintView) {
    if (
      constraints.some(
        (candidate, candidateIndex) =>
          candidateIndex !== index && sameRelationshipConstraint(candidate, next),
      )
    ) {
      setFeedback("This link filter is already added.");
      return;
    }
    workspace.setSearch({
      ...workspace.search,
      relationships: constraints.map((candidate, candidateIndex) =>
        candidateIndex === index ? next : candidate,
      ),
    });
    closeScope();
  }
  function scopeKeyDown(event: KeyboardEvent<HTMLElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      closeScope();
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
        styles={{ root: { maxWidth: "calc(100vw - 24px)" } }}
        open={scopeOpen}
        onOpenChange={(open) => {
          setScopeOpen(open);
          setFeedback(null);
        }}
        title={
          <Radio.Group
            aria-label="Relationship direction"
            buttonStyle="solid"
            value={constraint.direction}
            onChange={(event) => {
              update({
                ...constraint,
                direction: event.target.value as RelationshipDirectionView,
              });
            }}
          >
            <Radio.Button onKeyDown={scopeKeyDown} value="incoming">
              Links to
            </Radio.Button>
            <Radio.Button onKeyDown={scopeKeyDown} value="outgoing">
              Linked from
            </Radio.Button>
          </Radio.Group>
        }
        content={
          <div className="relationship-filter__scope">
            {summary && (
              <Typography.Link
                style={{ display: "block", marginBottom: token.marginSM }}
                href={atlasRoutePath(route)}
                onKeyDown={scopeKeyDown}
                onClick={(event) => {
                  if (!shouldHandleAtlasRouteClick(event)) return;
                  event.preventDefault();
                  setScopeOpen(false);
                  navigateToAtlasRoute(route);
                }}
              >
                Open {summary.title}
              </Typography.Link>
            )}
            <Typography.Paragraph
              strong
              style={{ marginTop: 0, marginBottom: token.marginXS }}
            >
              {constraint.direction === "incoming"
                ? "Look for links in matching records"
                : "Look for links in this entry"}
            </Typography.Paragraph>
            <Radio.Group
              aria-label="Relationship scope"
              value={constraint.include_contents}
              onChange={(event) => {
                update({
                  ...constraint,
                  include_contents: event.target.value,
                });
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
            {feedback && <Typography.Text role="status">{feedback}</Typography.Text>}
          </div>
        }
      >
        <Button
          ref={scopeTrigger}
          color="default"
          variant="filled"
          icon={<DownOutlined aria-hidden="true" />}
          iconPlacement="end"
          styles={{
            root: {
              maxWidth: "100%",
              minWidth: 0,
            },
          }}
          onKeyDown={scopeKeyDown}
          aria-label={`${direction} ${title}; ${constraint.include_contents ? "Whole entry" : "Main entry only"}`}
          aria-expanded={scopeOpen}
        >
          <span className="relationship-filter__label">
            {direction} {title}
          </span>
        </Button>
      </Popover>
      <Button
        type="text"
        icon={<CloseOutlined />}
        styles={{ root: { flexShrink: 0 } }}
        aria-label="Remove relationship filter"
        onClick={() => {
          closeScope();
          workspace.setSearch({
            ...workspace.search,
            relationships: constraints.filter(
              (_, candidateIndex) => candidateIndex !== index,
            ),
          });
        }}
      />
    </div>
  );
}
