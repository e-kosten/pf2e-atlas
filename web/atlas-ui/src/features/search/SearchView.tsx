import { usePageTitle } from "../../shared/browser/usePageTitle";
import { Alert, Button, Select } from "antd";
import { AddToListButton } from "../lists/AddToListButton";
import { FilterControls } from "../../shared/filters/FilterPanel";
import { ReferenceBrowser } from "../../shared/records/ReferenceBrowser";
import { ResultPaneHeader } from "./ResultPaneHeader";
import type { SearchWorkspaceState } from "./useSearchWorkspace";
import { RelationshipFilters } from "./RelationshipFilters";
import { SearchModelSelect } from "./SearchModelSelect";
import { SearchProfileNotice } from "../../shared/search/SearchProfileNotice";
import { useSearchProfile } from "../../shared/search/SearchProfileContext";
import {
  hasExecutableSearch,
  UNKNOWN_SEARCH_RANKING,
} from "../../shared/filters/searchState";

export function SearchView({ workspace }: { workspace: SearchWorkspaceState }) {
  usePageTitle("Search");
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
        additionalFilters={<RelationshipFilters workspace={workspace} />}
        similarSearch={
          workspace.search.similarSeed
            ? {
                title:
                  workspace.resultPage?.mode.kind === "similar_records" &&
                  workspace.resultPage.mode.seed.record_key ===
                    workspace.search.similarSeed
                    ? workspace.resultPage.mode.seed.title
                    : workspace.errorMessage
                      ? "Record unavailable"
                      : "Loading record…",
                onClear: () =>
                  workspace.setSearch({
                    ...workspace.search,
                    query: "",
                    similarSeed: null,
                    mode: "browse",
                  }),
              }
            : undefined
        }
        searchRowEnd={
          <>
            <SearchModelSelect />
            {!workspace.search.similarSeed && (
              <Select
                aria-label="Search ranking"
                className="search-model-select"
                value={workspace.search.ranking ?? "score"}
                options={[
                  { value: "rrf", label: "Rank-based" },
                  { value: "score", label: "Score-based" },
                ]}
                onChange={(ranking) =>
                  workspace.setSearch({
                    ...workspace.search,
                    ranking,
                    filterError:
                      workspace.search.filterError === UNKNOWN_SEARCH_RANKING
                        ? null
                        : workspace.search.filterError,
                  })
                }
                popupMatchSelectWidth={false}
              />
            )}
          </>
        }
      />
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
        emptyMessage={
          failed
            ? null
            : hasCriteria
              ? "No matching records."
              : "Search or choose filters."
        }
        rows={workspace.resultPage?.rows ?? []}
        relationships={workspace.resultPage?.relationships}
        generation={workspace.resultPage?.artifact_generation}
        onRefreshResults={workspace.refreshResults}
        loading={loading}
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
