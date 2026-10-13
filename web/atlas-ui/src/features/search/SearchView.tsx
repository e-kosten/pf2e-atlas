import { Alert, Button, Space, Typography } from "antd";
import { AddToListButton } from "../lists/AddToListButton";
import { FilterControls } from "../../shared/filters/FilterPanel";
import { ReferenceBrowser } from "../../shared/records/ReferenceBrowser";
import { ResultPaneHeader } from "./ResultPaneHeader";
import type { SearchWorkspaceState } from "./useSearchWorkspace";

export function SearchView({ workspace }: { workspace: SearchWorkspaceState }) {
  return (
    <main className="search-workspace">
      <FilterControls filterState={workspace} layout="toolbar">
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
        loading={workspace.resultsLoading || workspace.resultsRefreshing}
        selectedRecordKey={workspace.selectedRecordKey}
        onSelect={(row) => workspace.selectRecord(row.record.record_key)}
        reading={workspace.reading}
        positions={workspace.positions}
        resultsHeader={<ResultPaneHeader workspace={workspace} />}
        readerActions={
          workspace.reading.detail && (
            <AddToListButton recordKey={workspace.reading.detail.record.record_key} />
          )
        }
      />
    </main>
  );
}
