import { Button, Checkbox, Space, Tag, Typography } from "antd";
import { CloseOutlined } from "@ant-design/icons";
import {
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import { recordReadingIdentity } from "../../shared/records/useRetainedRecord";
import type { SearchWorkspaceState } from "./useSearchWorkspace";

export function RelationshipFilters({
  workspace,
}: {
  workspace: SearchWorkspaceState;
}) {
  const constraints = workspace.search.relationships ?? [];
  return constraints.map((constraint, index) => {
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
    return (
      <Space
        wrap
        className="relationship-filter"
        role="group"
        aria-label="Relationship filter"
        key={index}
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
        <Checkbox
          checked={constraint.include_contents}
          onChange={(event) =>
            workspace.setSearch({
              ...workspace.search,
              relationships: constraints.map((candidate, candidateIndex) =>
                candidateIndex === index
                  ? { ...candidate, include_contents: event.target.checked }
                  : candidate,
              ),
            })
          }
        >
          Include contents
        </Checkbox>
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
  });
}
