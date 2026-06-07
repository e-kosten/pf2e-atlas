import { ChevronLeft, ChevronRight } from "lucide-react";
import { useMemo, useState } from "react";
import { totalPages } from "./pageMetrics";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";

export function ResultPaneHeader({ workspace }: { workspace: AtlasWorkspaceState }) {
  const page = workspace.resultPage?.page;
  const displayedPageNumber = workspace.resultsRefreshing
    ? workspace.pageNumber
    : (page?.number ?? workspace.pageNumber);
  const pageCount = useMemo(() => (page ? totalPages(page) : 0n), [page]);
  const pageCountNumber =
    pageCount > BigInt(Number.MAX_SAFE_INTEGER)
      ? Number.MAX_SAFE_INTEGER
      : Number(pageCount);

  return (
    <div className="result-pane-header">
      <span>
        {workspace.resultsRefreshing
          ? "Updating results"
          : page
            ? `${page.total.toLocaleString()} records`
            : "No result window yet"}
      </span>
      <div className="pager pager--compact">
        <button
          aria-label="Previous page"
          className="icon-button icon-button--compact"
          disabled={workspace.pageNumber <= 1 || workspace.resultsRefreshing}
          onClick={() => workspace.setPageNumber(workspace.pageNumber - 1)}
          type="button"
        >
          <ChevronLeft size={15} />
        </button>
        <PageJump
          key={displayedPageNumber}
          disabled={workspace.resultsRefreshing || workspace.resultsLoading}
          displayedPageNumber={displayedPageNumber}
          onPageChange={workspace.setPageNumber}
          pageCount={pageCount}
          pageCountNumber={pageCountNumber}
          requestedPageNumber={workspace.pageNumber}
        />
        <button
          aria-label="Next page"
          className="icon-button icon-button--compact"
          disabled={!page?.has_more || workspace.resultsRefreshing}
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

type PageJumpProps = {
  disabled: boolean;
  displayedPageNumber: number;
  requestedPageNumber: number;
  pageCount: bigint;
  pageCountNumber: number;
  onPageChange: (page: number) => void;
};

function PageJump({
  disabled,
  displayedPageNumber,
  requestedPageNumber,
  pageCount,
  pageCountNumber,
  onPageChange,
}: PageJumpProps) {
  const [pageDraft, setPageDraft] = useState(displayedPageNumber.toString());

  function commitPageDraft() {
    const parsedPage = Number.parseInt(pageDraft, 10);
    if (!Number.isFinite(parsedPage) || parsedPage < 1) {
      setPageDraft(displayedPageNumber.toString());
      return;
    }

    const nextPage =
      pageCountNumber > 0
        ? Math.min(pageCountNumber, Math.max(1, parsedPage))
        : parsedPage;
    setPageDraft(nextPage.toString());
    if (nextPage !== requestedPageNumber) {
      onPageChange(nextPage);
    }
  }

  return (
    <label className="page-jump">
      <span className="sr-only">Current page</span>
      <input
        aria-label="Current page"
        disabled={disabled}
        inputMode="numeric"
        max={pageCountNumber > 0 ? pageCountNumber : undefined}
        min={1}
        onBlur={commitPageDraft}
        onChange={(event) => setPageDraft(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.currentTarget.blur();
          }
          if (event.key === "Escape") {
            setPageDraft(displayedPageNumber.toString());
            event.currentTarget.blur();
          }
        }}
        pattern="[0-9]*"
        type="text"
        value={pageDraft}
      />
      <span>/ {pageCount.toLocaleString()}</span>
    </label>
  );
}
