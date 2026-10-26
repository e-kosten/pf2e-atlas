import { useFilterUrlRecovery } from "../../shared/filters/useFilterUrlRecovery";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { Alert, Button, Tag } from "antd";
import { AddToListButton } from "../lists/AddToListButton";
import { FilterControls } from "../../shared/filters/FilterPanel";
import { ReferenceBrowser } from "../../shared/records/ReferenceBrowser";
import { ResultPaneHeader } from "./ResultPaneHeader";
import type { SearchWorkspaceState } from "./useSearchWorkspace";
import { SearchSettings } from "../../shared/search/SearchSettings";
import { SearchProfileNotice } from "../../shared/search/SearchProfileNotice";
import { useSearchProfile } from "../../shared/search/SearchProfileContext";
import { hasExecutableSearch } from "../../shared/filters/searchState";

export function SearchView({ workspace }: { workspace: SearchWorkspaceState }) {
  usePageTitle("Search");
  useFilterUrlRecovery(workspace.search, workspace.recoverFilterUrl);
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
  const mode = workspace.resultPage?.mode;
  const seedTitle =
    mode?.kind === "similar_records" &&
    mode.seed.record_key === workspace.search.similarSeed
      ? mode.seed.title
      : mode?.kind === "similar_traits" &&
          mode.trait_key === workspace.search.similarSeed
        ? mode.title
        : undefined;
  const similarTitle =
    seedTitle ?? (workspace.errorMessage ? "Record unavailable" : "Loading record…");
  return (
    <main className="search-workspace">
      <FilterControls
        filterState={workspace}
        layout="toolbar"
        similarSearch={
          workspace.search.similarSeed
            ? {
                title: similarTitle,
                onSearch: workspace.refresh,
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
            {workspace.search.traitReferenceSeed && (
              <Tag
                closable
                onClose={() =>
                  workspace.setSearch({ ...workspace.search, traitReferenceSeed: null })
                }
              >
                Trait references:{" "}
                {mode?.kind === "trait_references" &&
                mode.trait_key === workspace.search.traitReferenceSeed
                  ? mode.title
                  : "Loading trait…"}
              </Tag>
            )}
            <SearchSettings
              search={workspace.search}
              seedTitle={seedTitle}
              seed={
                workspace.search.similarSeed
                  ? workspace.search.similarSeed.startsWith("trait:")
                    ? { kind: "trait", trait_key: workspace.search.similarSeed }
                    : { kind: "record", record_key: workspace.search.similarSeed }
                  : null
              }
              generation={workspace.resultPage?.artifact_generation}
              onApply={(next, model) =>
                workspace.applySettings
                  ? workspace.applySettings(next, model)
                  : workspace.setSearch(next)
              }
            />
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
        relationshipClauses={workspace.resultPage?.relationship_clauses}
        filter={workspace.resultFilter}
        generation={workspace.resultPage?.artifact_generation}
        onRefreshResults={workspace.refreshResults}
        loading={loading}
        selectedRecordKey={workspace.selectedRecordKey}
        onSelect={(row, request) => {
          workspace.selectRecord(row.record.record_key);
          workspace.reading.select(request);
        }}
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
