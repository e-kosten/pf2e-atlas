import { ChevronLeft, ChevronRight } from "lucide-react";
import { Button, Select } from "antd";
import { useMemo, useState } from "react";
import { totalPages } from "./pageMetrics";
import type { SearchWorkspaceState } from "./useSearchWorkspace";
import { ResultSortControl } from "../../shared/search/ResultSortControl";

export function ResultPaneHeader({ workspace }: { workspace: SearchWorkspaceState }) {
  const page = workspace.resultPage?.page;
  const similar =
    workspace.resultPage?.mode.kind === "similar_records" ||
    workspace.resultPage?.mode.kind === "similar_traits";
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
            ? similar
              ? "Top matches"
              : workspace.resultPage?.coverage?.exhaustive === false
                ? `Top ${page.total.toLocaleString()} matches`
                : `${page.total.toLocaleString()} ${page.total === 1n ? "record" : "records"}`
            : "Results"}
      </span>
      <div className="pager pager--compact">
        <ResultSortControl
          search={workspace.search}
          setSearch={workspace.setSearch}
          disabled={workspace.resultsRecovering}
        />
        {page && page.total > 0n && (
          <>
            <Select
              aria-label="Records per page"
              size="small"
              style={{ width: 128 }}
              disabled={workspace.resultsPaused || workspace.resultsRecovering}
              value={workspace.search.pageSize}
              options={[10, 25, 50, 100].map((value) => ({
                value,
                label: value + " / page",
              }))}
              onChange={(pageSize) =>
                workspace.setSearch({ ...workspace.search, pageSize })
              }
            />
            <div className="pager__navigation" role="group" aria-label="Result pages">
              <Button
                aria-label="Previous page"
                size="small"
                disabled={
                  workspace.pageNumber <= 1 ||
                  workspace.resultsRefreshing ||
                  workspace.resultsPaused ||
                  workspace.resultsRecovering
                }
                icon={<ChevronLeft size={15} />}
                onClick={() => workspace.setPageNumber(workspace.pageNumber - 1)}
                type="text"
              />
              <PageJump
                key={displayedPageNumber}
                disabled={
                  workspace.resultsRefreshing ||
                  workspace.resultsLoading ||
                  Boolean(workspace.resultsPaused) ||
                  Boolean(workspace.resultsRecovering)
                }
                displayedPageNumber={displayedPageNumber}
                onPageChange={workspace.setPageNumber}
                pageCount={pageCount}
                pageCountNumber={pageCountNumber}
                requestedPageNumber={workspace.pageNumber}
              />
              <Button
                aria-label="Next page"
                size="small"
                disabled={
                  !page?.has_more ||
                  workspace.resultsRefreshing ||
                  workspace.resultsPaused ||
                  workspace.resultsRecovering
                }
                icon={<ChevronRight size={15} />}
                onClick={() =>
                  workspace.setPageNumber(page?.next_page ?? workspace.pageNumber + 1)
                }
                type="text"
              />
            </div>
          </>
        )}
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
