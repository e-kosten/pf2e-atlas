import { MantineFilters } from "./mantine/MantineFilters";
import { MantineResults } from "./mantine/MantineResults";
import { RecordPresentation } from "./recordPresentation";
import { ResultPaneHeader } from "./ResultPaneHeader";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";
import { WorkspaceLayout } from "./WorkspaceLayout";

export function MantinePrototype({ workspace }: { workspace: AtlasWorkspaceState }) {
  return (
    <WorkspaceLayout
      variant="mantine"
      filter={<MantineFilters workspace={workspace} />}
      results={<MantineResults workspace={workspace} />}
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
