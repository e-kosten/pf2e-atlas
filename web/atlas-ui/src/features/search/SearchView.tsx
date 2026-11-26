import { useEffect, useMemo, useRef } from "react";
import { useAtlasRouter } from "../../app/router";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { type HistoryState } from "@tanstack/react-router";
import {
  FILTER_URL_RESET_NOTICE,
  useFilterUrlRecovery,
} from "../../shared/filters/useFilterUrlRecovery";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { Alert, Button } from "antd";
import { AddToRecordButton } from "../record-actions/AddToRecordButton";
import {
  FilterControls,
  type AdvancedFiltersHandle,
} from "../../shared/filters/FilterPanel";
import { ReferenceBrowser } from "../../shared/records/ReferenceBrowser";
import { ResultPaneHeader } from "./ResultPaneHeader";
import type { SearchWorkspaceState } from "./useSearchWorkspace";
import { appError } from "../../shared/feedback/errorMessages";
import {
  SimilarityFailure,
  type ComparisonControl,
} from "../../shared/search/SimilarityFailure";
import { searchSeed, sameSimilaritySeed } from "../../shared/search/similaritySource";
import {
  SearchSettings,
  type SearchSettingsHandle,
} from "../../shared/search/SearchSettings";
import { SearchProfileNotice } from "../../shared/search/SearchProfileNotice";
import { effectiveMatchDisplay } from "../../shared/search/searchPreferences";
import { useSearchProfile } from "../../shared/search/SearchProfileContext";
import {
  hasExecutableSearch,
  INVALID_SIMILARITY_SOURCE,
} from "../../shared/filters/searchState";
import { ReferenceHelpHost } from "../../shared/records/ReferenceHelpHost";

export function SearchView({
  workspace,
  historyState = {},
}: {
  workspace: SearchWorkspaceState;
  historyState?: Pick<
    HistoryState,
    "atlasSearchReset" | "atlasFilterResetNotice" | "atlasComparisonControl"
  >;
}) {
  const atlasRouter = useAtlasRouter();
  const selected = workspace.reading.detail?.record;
  const tabTitle =
    selected?.record_key === workspace.selectedRecordKey ? selected?.title : undefined;
  usePageTitle("Search", tabTitle);
  useFilterUrlRecovery(workspace.search, workspace.recoverFilterUrl);
  const reset = {
    token: historyState.atlasSearchReset,
    notice: historyState.atlasFilterResetNotice,
  };
  const profile = useSearchProfile();
  const hasCriteria = hasExecutableSearch(workspace.search);
  const failed = Boolean(
    workspace.resultsFailed ||
    workspace.errorMessage ||
    workspace.search.filterError ||
    profile.error,
  );
  const loading =
    !failed &&
    (workspace.resultsLoading ||
      workspace.resultsRefreshing ||
      (hasCriteria && profile.loading));
  const seed = useMemo(
    () =>
      workspace.search.filterError === INVALID_SIMILARITY_SOURCE
        ? null
        : searchSeed(workspace.search),
    [workspace.search],
  );
  const settings = useRef<SearchSettingsHandle>(null);
  const filters = useRef<AdvancedFiltersHandle>(null);
  function openControl(control: ComparisonControl) {
    if (control === "filters") filters.current?.open();
    else settings.current?.open(control);
  }
  const request = historyState.atlasComparisonControl;
  useEffect(() => {
    if (!request || !sameSimilaritySeed(request.seed, seed)) return;
    openControl(request.open);
    void atlasRouter.navigate({
      to: ".",
      search: true,
      replace: true,
      resetScroll: false,
      state: (state) => ({ ...state, atlasComparisonControl: undefined }),
    });
  }, [request, seed, atlasRouter]);
  const page = workspace.resultPage;
  const mode = page?.mode;
  const acknowledged =
    page &&
    mode?.kind === "similar_records" &&
    sameSimilaritySeed(
      mode.source?.seed ?? { kind: "record", record_key: mode.seed.record_key },
      seed,
    )
      ? {
          title: mode.source?.title ?? mode.seed.title,
          parentTitle: mode.seed.title,
        }
      : undefined;
  const rankingCode = appError(workspace.resultsError)?.code;
  const source = workspace.similaritySource;
  const identityCode = appError(source?.error)?.code;
  const missing =
    identityCode === "record_not_found" || rankingCode === "record_not_found";
  const stale =
    identityCode === "similarity_source_stale" ||
    rankingCode === "similarity_source_stale";
  const seedTitle =
    !missing && !stale
      ? (source?.data?.source.title ?? acknowledged?.title)
      : undefined;
  const similarTitle =
    seedTitle ??
    (missing
      ? "Record unavailable"
      : stale
        ? "Source changed"
        : source?.error
          ? "Couldn’t load source name"
          : "Loading record…");
  const resultsFailure = workspace.errorMessage && (
    <ErrorNotice
      error={workspace.resultsError ?? workspace.errorMessage}
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
  );
  return (
    <ReferenceHelpHost generation={workspace.filterEditor?.artifact_generation}>
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
          ref={filters}
          filterState={workspace}
          searchInputError={workspace.searchInputError}
          layout="toolbar"
          collapseToolbar
          similarSearch={
            seed
              ? {
                  title: similarTitle,
                  seed,
                  onSearch: workspace.refresh,
                  onClear: () =>
                    workspace.setSearch({
                      ...workspace.search,
                      query: "",
                      similarSeed: null,
                      similarLocation: undefined,
                      mode: "browse",
                    }),
                }
              : undefined
          }
          searchRowEnd={
            <SearchSettings
              ref={settings}
              search={workspace.search}
              seedTitle={seedTitle}
              seed={seed}
              generation={
                workspace.referenceGeneration ??
                workspace.resultPage?.artifact_generation
              }
              disabled={profile.keywordSearch && Boolean(workspace.search.similarSeed)}
              onApply={(next, model) =>
                workspace.applySettings
                  ? workspace.applySettings(next, model)
                  : workspace.setSearch(next)
              }
            />
          }
        />
        <div className="search-workspace__profile">
          <SearchProfileNotice
            similar={Boolean(workspace.search.similarSeed)}
            showKeyword
            rerunSemantic={workspace.rerunSemantic}
          />
        </div>
        {seed && source?.error && !seedTitle && !missing && !stale && (
          <ErrorNotice
            error={source.error}
            title="Unable to load comparison source"
            action={
              <Button onClick={() => void source.refetch()}>Retry source lookup</Button>
            }
          />
        )}
        {seed && (workspace.resultsError || ((missing || stale) && source?.error)) ? (
          <SimilarityFailure
            seed={seed}
            search={workspace.search}
            error={
              identityCode === "record_not_found" ||
              identityCode === "similarity_source_stale"
                ? source?.error
                : workspace.resultsError
            }
            generation={
              workspace.referenceGeneration ?? source?.data?.artifact_generation
            }
            parentTitle={source?.data?.source.parent_title ?? acknowledged?.parentTitle}
            onRecover={(next, open) => {
              if (open) openControl(open);
              else workspace.setSearch(next);
            }}
            fallback={resultsFailure}
          />
        ) : (
          resultsFailure
        )}
        <ReferenceBrowser
          clearBesideSelection
          searchCards
          previewOnEnter
          showReaderPrompt={Boolean(workspace.resultPage?.rows.length)}
          emptyMessage={
            failed
              ? null
              : hasCriteria
                ? workspace.resultPage
                  ? "No matching records."
                  : null
                : "Search or choose filters."
          }
          rows={workspace.resultPage?.rows ?? []}
          showMatchContext={!workspace.evidencePending}
          matchDisplay={effectiveMatchDisplay(
            profile.preferences.matchDisplay,
            mode?.kind === "similar_records",
          )}
          showSmallerWeights={profile.preferences.showSmallerWeights}
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
          onSelect={(row, request) =>
            workspace.selectRecord(row.record.record_key, request)
          }
          onClearSelection={() => workspace.selectRecord(null)}
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
    </ReferenceHelpHost>
  );
}
