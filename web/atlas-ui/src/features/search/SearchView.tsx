import { Alert, Button, Space, Typography } from "antd";
import { AddToListButton } from "../lists/AddToListButton";
import { FilterControls } from "../../shared/filters/FilterPanel";
import { ReferenceBrowser } from "../../shared/records/ReferenceBrowser";
import { ResultPaneHeader } from "./ResultPaneHeader";
import type { SearchWorkspaceState } from "./useSearchWorkspace";
import { SearchModelSelect } from "./SearchModelSelect";
import { SearchProfileNotice } from "../../shared/search/SearchProfileNotice";
import { useSearchProfile } from "../../shared/search/SearchProfileContext";
import { hasExecutableSearch } from "../../shared/filters/searchState";

export function SearchView({ workspace }: { workspace: SearchWorkspaceState }) {
  const profile = useSearchProfile();
  const hasCriteria = hasExecutableSearch(workspace.search);
  const failed = Boolean(
    workspace.errorMessage || workspace.search.filterError || profile.error,
  );
  const loading =
    !failed &&
    (workspace.resultsLoading ||
      workspace.resultsRefreshing ||
      (hasCriteria && profile.loading));
  const page = workspace.resultPage?.page;
  const showHeader = page && page.total > 0n;
  return (
    <main className="search-workspace">
      <FilterControls
        filterState={workspace}
        layout="toolbar"
        searchRowEnd={<SearchModelSelect />}
      >
        {workspace.search.similarSeed && (
          <Space>
            <Typography.Text>
              {workspace.resultPage?.mode.kind === "similar_records"
                ? `Similar to ${workspace.resultPage.mode.seed.title}`
                : "Similar records"}
            </Typography.Text>
            <Button
              aria-label="Clear similar criterion"
              size="small"
              onClick={() =>
                workspace.setSearch({
                  ...workspace.search,
                  similarSeed: null,
                  mode: "browse",
                })
              }
            >
              Clear
            </Button>
          </Space>
        )}
      </FilterControls>
      <SearchProfileNotice />
      {workspace.errorMessage && (
        <Alert
          type="error"
          showIcon
          title="Unable to update results"
          description={workspace.errorMessage}
          action={
            <Button
              loading={workspace.resultsRecovering}
              onClick={() =>
                workspace.resultsPaused
                  ? void workspace.refreshResults()
                  : workspace.refresh()
              }
            >
              {workspace.resultsPaused ? "Refresh results" : "Retry"}
            </Button>
          }
        />
      )}
      <ReferenceBrowser
        rows={workspace.resultPage?.rows ?? []}
        loading={loading}
        emptyMessage={
          failed
            ? null
            : hasCriteria
              ? "No matching records."
              : "Search or choose filters."
        }
        selectedRecordKey={workspace.selectedRecordKey}
        onSelect={(row) => workspace.selectRecord(row.record.record_key)}
        reading={workspace.reading}
        positions={workspace.positions}
        resultsHeader={
          showHeader ? <ResultPaneHeader workspace={workspace} /> : undefined
        }
        readerActions={
          workspace.reading.detail && (
            <AddToListButton recordKey={workspace.reading.detail.record.record_key} />
          )
        }
      />
    </main>
  );
}
