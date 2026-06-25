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
      filter={<FilterPanel workspace={workspace} />}
      results={<ResultTable workspace={workspace} />}
      resultsHeaderActions={<ResultPaneHeader workspace={workspace} />}
      selectedRecordKey={workspace.selectedRecordKey}
      detailHeaderActions={
        workspace.selectedRecordKey ? (
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
          loading={workspace.detailLoading}
          onReference={workspace.selectRecord}
        />
      }
    />
  );
}
