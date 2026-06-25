import { ExternalLink } from "lucide-react";
import { AddToListButton } from "../lists/AddToListButton";
import { FilterPanel } from "../../shared/filters/FilterPanel";
import { ResultTable } from "./ResultTable";
import { RecordPresentation } from "../../shared/records/RecordPresentation";
import { ResultPaneHeader } from "./ResultPaneHeader";
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
            <a
              aria-label="Open full page"
              className="pane-toggle"
              href={recordPath(workspace.selectedRecordKey)}
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
              title="Open full page"
            >
              <ExternalLink size={16} />
            </a>
          </>
        ) : null
      }
      detail={
        <section className="detail-panel">
          <RecordPresentation
            detail={workspace.recordDetail}
            loading={workspace.detailLoading}
            onReference={workspace.selectRecord}
          />
        </section>
      }
    />
  );
}
