import { ChevronLeft, ChevronRight } from "lucide-react";
import { Button, Select } from "antd";
import { useRef, useState } from "react";
import { totalPages } from "./pageMetrics";
import type { SearchWorkspaceState } from "./useSearchWorkspace";
import { ResultSortControl } from "../../shared/search/ResultSortControl";

export function searchResultsCaption(workspace: SearchWorkspaceState) {
  const { resultPage, earlyExactMatches } = workspace;
  const page = resultPage?.page;
  if (earlyExactMatches && earlyExactMatches.total > earlyExactMatches.matches.length)
    return `${earlyExactMatches.matches.length} of ${earlyExactMatches.total} exact matches`;
  if (!page) return "Results";
  if (resultPage.mode.kind === "similar_records") return "Top matches";
  if (resultPage.coverage?.exhaustive === false)
    return `Top ${page.total.toLocaleString()} matches`;
  return `${page.total.toLocaleString()} ${page.total === 1 ? "record" : "records"}`;
}

export function ResultPaneHeader({
  workspace,
  pendingLabel,
}: {
  workspace: SearchWorkspaceState;
  pendingLabel?: string;
}) {
  const page = workspace.resultPage?.page;
  const pageCount = page ? totalPages(page) : 0n;
  const pageCountNumber =
    pageCount > BigInt(Number.MAX_SAFE_INTEGER)
      ? Number.MAX_SAFE_INTEGER
      : Number(pageCount);
  const pagerPending =
    workspace.resultsPending === "page" ||
    workspace.resultsPending === "page-size" ||
    workspace.resultsPending === "sort";
  const pagerDisabled = workspace.resultsPaused || workspace.resultsRecovering;

  return (
    <div className="result-pane-header search-results-toolbar">
      <span className="search-results-toolbar__caption">
        {searchResultsCaption(workspace)}
      </span>
      {pendingLabel && (
        <span
          className="search-results-toolbar__status"
          data-retained-page={pagerPending || undefined}
          aria-hidden="true"
        >
          {pendingLabel}
        </span>
      )}
      {page && page.total > 0 && (
        <div className="pager__navigation" role="group" aria-label="Result pages">
          <Button
            aria-label="Previous page"
            aria-disabled={pagerPending || page.number <= 1 || undefined}
            tabIndex={page.number <= 1 ? -1 : undefined}
            size="small"
            disabled={pagerDisabled}
            icon={<ChevronLeft size={15} />}
            onClick={() => {
              if (!pagerPending && page.number > 1)
                workspace.setPageNumber(page.number - 1);
            }}
            type="text"
          />
          <PageJump
            disabled={Boolean(pagerDisabled)}
            pending={pagerPending}
            displayedPageNumber={page.number}
            onPageChange={workspace.setPageNumber}
            pageCount={pageCount}
            pageCountNumber={pageCountNumber}
          />
          <Button
            aria-label="Next page"
            aria-disabled={pagerPending || !page.has_more || undefined}
            tabIndex={!page.has_more ? -1 : undefined}
            size="small"
            disabled={pagerDisabled}
            icon={<ChevronRight size={15} />}
            onClick={() => {
              if (!pagerPending && page.has_more)
                workspace.setPageNumber(page.next_page ?? page.number + 1);
            }}
            type="text"
          />
        </div>
      )}
      <div className="pager pager--compact">
        <ResultSortControl
          search={workspace.search}
          setSearch={workspace.setSearch}
          disabled={workspace.resultsRecovering}
        />
        {page && page.total > 0 && (
          <Select
            aria-label="Records per page"
            aria-disabled={pagerPending || undefined}
            open={pagerPending ? false : undefined}
            size="small"
            style={{ width: 128 }}
            disabled={pagerDisabled}
            value={page.size}
            options={[10, 25, 50, 100].map((value) => ({
              value,
              label: value + " / page",
            }))}
            onChange={(pageSize) => {
              if (!pagerPending) workspace.setSearch({ ...workspace.search, pageSize });
            }}
          />
        )}
      </div>
    </div>
  );
}

type PageJumpProps = {
  disabled: boolean;
  pending: boolean;
  displayedPageNumber: number;
  pageCount: bigint;
  pageCountNumber: number;
  onPageChange: (page: number) => void;
};

function PageJump({
  disabled,
  pending,
  displayedPageNumber,
  pageCount,
  pageCountNumber,
  onPageChange,
}: PageJumpProps) {
  const [draft, setDraft] = useState<{ page: number; value: string }>();
  const cancelingBlur = useRef(false);
  if (draft && (pending || displayedPageNumber !== draft.page)) setDraft(undefined);

  function commitPageDraft() {
    if (disabled || pending || !draft) return;
    setDraft(undefined);
    const parsedPage = Number.parseInt(draft.value, 10);
    if (!Number.isFinite(parsedPage) || parsedPage < 1) return;
    const nextPage = Math.min(pageCountNumber, parsedPage);
    if (nextPage !== displayedPageNumber) onPageChange(nextPage);
  }

  return (
    <label className="page-jump">
      <span className="sr-only">Current page</span>
      <input
        aria-label="Current page"
        className="inline-editable"
        disabled={disabled}
        readOnly={pending}
        inputMode="numeric"
        max={pageCountNumber}
        min={1}
        onBlur={() => {
          if (!cancelingBlur.current) commitPageDraft();
        }}
        onChange={(event) =>
          setDraft({ page: displayedPageNumber, value: event.target.value })
        }
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            commitPageDraft();
          }
          if (event.key === "Escape") {
            event.preventDefault();
            setDraft(undefined);
            cancelingBlur.current = true;
            event.currentTarget.blur();
            cancelingBlur.current = false;
          }
        }}
        pattern="[0-9]*"
        type="text"
        value={draft?.value ?? displayedPageNumber}
      />
      <span>/ {pageCount.toLocaleString()}</span>
    </label>
  );
}
