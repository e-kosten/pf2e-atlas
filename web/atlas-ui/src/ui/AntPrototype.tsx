import { ExternalLink } from "lucide-react";
import { AntFilters } from "./ant/AntFilters";
import { AntResults } from "./ant/AntResults";
import { AddToListButton } from "./ListViews";
import { RecordPresentation } from "./recordPresentation";
import { ResultPaneHeader } from "./ResultPaneHeader";
import {
  navigateToAtlasRoute,
  recordPath,
  shouldHandleAtlasRouteClick,
} from "./routes";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";
import { WorkspaceLayout } from "./WorkspaceLayout";

type AntPrototypeProps = {
  workspace: AtlasWorkspaceState;
};

export function AntPrototype({ workspace }: AntPrototypeProps) {
  return (
    <WorkspaceLayout
      filter={<AntFilters workspace={workspace} />}
      results={<AntResults workspace={workspace} />}
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
