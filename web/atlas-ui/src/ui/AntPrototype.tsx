import { AntFilters } from "./ant/AntFilters";
import { AntResults } from "./ant/AntResults";
import { RecordPresentation } from "./recordPresentation";
import { ResultPaneHeader } from "./ResultPaneHeader";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";
import { WorkspaceLayout } from "./WorkspaceLayout";

type AntPrototypeProps = {
  workspace: AtlasWorkspaceState;
};

export function AntPrototype({ workspace }: AntPrototypeProps) {
  return (
    <WorkspaceLayout
      variant="ant"
      filter={<AntFilters workspace={workspace} />}
      results={<AntResults workspace={workspace} />}
      resultsHeaderActions={<ResultPaneHeader workspace={workspace} />}
      selectedRecordKey={workspace.selectedRecordKey}
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
