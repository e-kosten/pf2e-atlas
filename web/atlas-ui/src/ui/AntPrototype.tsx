import { AntFilters } from "./ant/AntFilters";
import { AntResults } from "./ant/AntResults";
import { RecordPresentation } from "./recordPresentation";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";

type AntPrototypeProps = {
  workspace: AtlasWorkspaceState;
};

export function AntPrototype({ workspace }: AntPrototypeProps) {
  return (
    <main className="workspace-grid workspace-grid--ant">
      <AntFilters workspace={workspace} />
      <AntResults workspace={workspace} />
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
