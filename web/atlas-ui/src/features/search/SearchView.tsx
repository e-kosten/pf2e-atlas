import { Input } from "antd";
import { ExternalLink } from "lucide-react";
import { AddToListButton } from "../lists/AddToListButton";
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
        </div>
      }
      activeFilterCount={
        workspace.search.filterClauses.length +
        Number(
          Boolean(
            workspace.search.relationship || workspace.search.relationshipInvalid,
          ),
        )
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
        workspace.selectedRecordKey ? (
          <RecordDetailPane
            detail={workspace.recordDetail}
            errors={[workspace.detailError]}
            loading={workspace.detailLoading}
            stale={workspace.detailRefreshing}
            onReference={(recordKey, childLocator) => {
              if (childLocator) {
                navigateToAtlasRoute({ kind: "record", recordKey, childLocator });
                return;
              }
              workspace.selectRecord(recordKey);
            }}
          />
        ) : undefined
      }
    />
  );
}
