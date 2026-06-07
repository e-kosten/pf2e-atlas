import { MantineFilters } from "./mantine/MantineFilters";
import { MantineResults } from "./mantine/MantineResults";
import { RecordPresentation } from "./recordPresentation";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";

export function MantinePrototype({
  workspace,
}: {
  workspace: AtlasWorkspaceState;
}) {
  return (
    <main className="workspace-grid workspace-grid--mantine">
      <MantineFilters workspace={workspace} />
      <MantineResults workspace={workspace} />
      <section className="detail-panel">
        <RecordPresentation
          detail={workspace.recordDetail}
          loading={workspace.detailLoading}
          onReference={workspace.selectRecord}
        />
      </section>
    </main>
  );
}
