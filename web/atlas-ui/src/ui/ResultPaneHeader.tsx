import { ChevronLeft, ChevronRight } from "lucide-react";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";

export function ResultPaneHeader({ workspace }: { workspace: AtlasWorkspaceState }) {
  const page = workspace.resultPage?.page;
  return (
    <div className="result-pane-header">
      <span>
        {page ? `${page.total.toLocaleString()} records` : "No result window yet"}
      </span>
      <div className="pager pager--compact">
        <button
          aria-label="Previous page"
          className="icon-button icon-button--compact"
          disabled={workspace.pageNumber <= 1}
          onClick={() => workspace.setPageNumber(workspace.pageNumber - 1)}
          type="button"
        >
          <ChevronLeft size={15} />
        </button>
        <span>{page ? `${page.number}` : workspace.pageNumber}</span>
        <button
          aria-label="Next page"
          className="icon-button icon-button--compact"
          disabled={!page?.has_more}
          onClick={() =>
            workspace.setPageNumber(page?.next_page ?? workspace.pageNumber + 1)
          }
          type="button"
        >
          <ChevronRight size={15} />
        </button>
      </div>
    </div>
  );
}
