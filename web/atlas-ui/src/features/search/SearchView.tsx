import { atlasRouter } from "../../app/router";
import { useRouterState } from "@tanstack/react-router";
import {
  FILTER_URL_RESET_NOTICE,
  useFilterUrlRecovery,
} from "../../shared/filters/useFilterUrlRecovery";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { Alert, Button, Select, Tag } from "antd";
import { AddToRecordButton } from "../record-actions/AddToRecordButton";
import { FilterControls } from "../../shared/filters/FilterPanel";
import { ReferenceBrowser } from "../../shared/records/ReferenceBrowser";
import { ResultPaneHeader } from "./ResultPaneHeader";
import type { SearchWorkspaceState } from "./useSearchWorkspace";
import { SearchModelSelect } from "./SearchModelSelect";
import { SearchProfileNotice } from "../../shared/search/SearchProfileNotice";
import { useSearchProfile } from "../../shared/search/SearchProfileContext";
import {
  hasExecutableSearch,
  UNKNOWN_SEARCH_RANKING,
} from "../../shared/filters/searchState";

export function SearchView({ workspace }: { workspace: SearchWorkspaceState }) {
  usePageTitle("Search");
  useFilterUrlRecovery(workspace.search, workspace.recoverFilterUrl);
  const reset = useRouterState({
    select: (state) => ({
      token: state.location.state.atlasSearchReset,
      notice: state.location.state.atlasFilterResetNotice,
    }),
  });
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
  const similarTitle =
    mode?.kind === "similar_records" &&
    mode.seed.record_key === workspace.search.similarSeed
      ? mode.seed.title
      : mode?.kind === "similar_traits" &&
          mode.trait_key === workspace.search.similarSeed
        ? mode.title
        : workspace.errorMessage
          ? "Record unavailable"
          : "Loading record…";
  return (
    <main className="search-workspace">
      {reset.notice && (
        <Alert
          type="info"
          showIcon
          title={FILTER_URL_RESET_NOTICE}
          closable={{
            onClose: () =>
              void atlasRouter.navigate({
                to: ".",
                search: true,
                replace: true,
                resetScroll: false,
                state: (state) => ({ ...state, atlasFilterResetNotice: false }),
              }),
          }}
        />
      )}
      <FilterControls
        key={reset.token ?? "search"}
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
            <SearchModelSelect />
            <Select
              disabled={Boolean(workspace.search.similarSeed)}
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
        query={
          workspace.resultPage?.mode.kind === "text_search"
            ? workspace.resultPage.mode.query
            : undefined
        }
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
        recordActions={(record) => (
          <AddToRecordButton
            key={record.record_key}
            recordKey={record.record_key}
            record={record}
          />
        )}
      />
    </main>
  );
}
