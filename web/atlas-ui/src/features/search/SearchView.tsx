import { Button, Input, Tag } from "antd";
import { ExternalLink } from "lucide-react";
import { AddToListButton } from "../lists/AddToListButton";
import { ReferenceFilterChip } from "../../shared/filters/ReferenceFilterChip";
import { FilterPanel } from "../../shared/filters/FilterPanel";
import { ResultTable } from "./ResultTable";
import { RecordDetailPane } from "../../shared/records/RecordDetailPane";
import { ResultPaneHeader } from "./ResultPaneHeader";
import { PaneIconLink } from "../../shared/ui/actions/PaneAction";
import {
  navigateToAtlasRoute,
  recordPath,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import type { SearchWorkspaceState } from "./useSearchWorkspace";
import { WorkspaceLayout } from "../../shared/layout/WorkspaceLayout";

type SearchViewProps = {
  workspace: SearchWorkspaceState;
};

export function SearchView({ workspace }: SearchViewProps) {
  return (
    <WorkspaceLayout
      responsiveSearch
      searchControls={
        <div className="search-workspace__query">
          <Input.Search
            aria-label="Search records"
            value={workspace.search.query}
            onChange={(event) =>
              workspace.setSearch({
                ...workspace.search,
                query: event.target.value,
                mode: event.target.value.trim() ? "text_search" : "browse",
              })
            }
          />
          {workspace.search.relationshipInvalid ? (
            <Tag color="error">
              Invalid reference filter
              <Button
                type="text"
                size="small"
                onClick={() =>
                  workspace.setSearch({
                    ...workspace.search,
                    relationship: undefined,
                    relationshipInvalid: undefined,
                  })
                }
              >
                Remove reference filter
              </Button>
            </Tag>
          ) : null}
          {workspace.search.relationship ? (
            <ReferenceFilterChip
              relationship={workspace.search.relationship}
              onRemove={() =>
                workspace.setSearch({ ...workspace.search, relationship: undefined })
              }
            />
          ) : null}
          {workspace.search.filterClauses.length ? (
            <Tag>{workspace.search.filterClauses.length} active filters</Tag>
          ) : null}
        </div>
      }
      filter={<FilterPanel workspace={workspace} />}
      results={<ResultTable workspace={workspace} />}
      resultsHeaderActions={<ResultPaneHeader workspace={workspace} />}
      selectedRecordKey={workspace.selectedRecordKey}
      detailHeaderActions={
        workspace.selectedRecordKey && workspace.recordDetail ? (
          <>
            <AddToListButton recordKey={workspace.selectedRecordKey} />
            <PaneIconLink
              href={recordPath(workspace.selectedRecordKey)}
              icon={<ExternalLink size={16} />}
              label="Open full page"
              onClick={(event) => {
                if (!shouldHandleAtlasRouteClick(event)) {
                  return;
                }
                event.preventDefault();
                navigateToAtlasRoute({
                  kind: "record",
                  recordKey: workspace.selectedRecordKey!,
                });
              }}
            />
          </>
        ) : null
      }
      detail={
        <RecordDetailPane
          detail={workspace.recordDetail}
          errors={[workspace.detailError]}
          loading={workspace.detailLoading}
          stale={workspace.detailRefreshing}
          onReference={workspace.selectRecord}
        />
      }
    />
  );
}
