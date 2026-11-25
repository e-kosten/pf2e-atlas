import { Button, Pagination, type InputRef } from "antd";
import { useQuery } from "@tanstack/react-query";
import { useLayoutEffect, useRef, useState } from "react";
import { searchPage } from "../../api/atlasApi";
import type { SearchResultRow, SearchResultsPage } from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { SearchEntry } from "../../shared/filters/SearchEntry";
import type { SearchSubmission } from "../../shared/filters/filterControls";
import {
  isArtifactGenerationChanged,
  referenceQueryMeta,
} from "../../shared/queries/referenceQueries";
import { ReferenceBrowser } from "../../shared/records/ReferenceBrowser";
import { ReferenceSelectionStatus } from "../../shared/records/ReferenceSelectionStatus";
import { usePreparedReferenceSelection } from "../../shared/records/usePreparedReferenceSelection";
import { useReferenceReading } from "../../shared/records/useReferenceReading";
import { useSearchProfile } from "../../shared/search/SearchProfileContext";
import { searchExecution } from "../../shared/search/searchExecution";

type RuleCandidate = { key: string; title: string; generation: string };
const pageSize = 10;
const emptyRows: SearchResultRow[] = [];

export function AbilityRulePicker({
  label,
  query,
  onSearch,
  onBack,
  onLink,
  disabled = false,
}: {
  label: string;
  query: string;
  onSearch: (query: string) => void;
  onBack: () => void;
  onLink: (record: RuleCandidate) => void;
  disabled?: boolean;
}) {
  const profile = useSearchProfile();
  const execution = searchExecution(
    profile.error ? { ...profile, keywordSearch: true } : profile,
    "text_search",
  );
  const { ranking, searchContent } = profile.preferences;
  const text = query.trim();
  const searchIdentity = [
    text,
    execution.profile,
    execution.mode,
    ranking,
    searchContent,
  ];
  const pagingKey = JSON.stringify(searchIdentity);
  const firstPage = {
    key: pagingKey,
    page: 1,
    generation: null as string | null,
    epoch: 0,
  };
  const [paging, setPaging] = useState(firstPage);
  if (paging.key !== pagingKey) setPaging(firstPage);
  const pageNumber = paging.page;
  const [submission, setSubmission] = useState<SearchSubmission>();
  const input = useRef<InputRef>(null);
  useLayoutEffect(() => {
    input.current?.focus({ preventScroll: true });
  }, []);
  const result = useQuery({
    queryKey: [
      "pc-ability-search",
      ...searchIdentity,
      pageNumber,
      paging.generation,
      paging.epoch,
    ],
    queryFn: ({ signal }) =>
      searchPage(
        {
          criteria: {
            kind: "text_search",
            mode: execution.mode,
            query: text,
            search_content: searchContent,
          },
          page: { number: pageNumber, size: pageSize },
          expected_artifact_generation: paging.generation,
          search_profile: execution.profile,
          ranking,
        },
        signal,
      ),
    enabled: Boolean(text && execution.ready),
    retry: false,
    meta: referenceQueryMeta,
  });
  function restartSearch() {
    setPaging({ ...firstPage, epoch: paging.epoch + 1 });
  }
  if (
    paging.key === pagingKey &&
    paging.generation &&
    isArtifactGenerationChanged(result.error)
  )
    restartSearch();
  const loading = Boolean(text && (!execution.ready || result.isFetching));
  const page = text && execution.ready ? result.data : undefined;
  return (
    <section className="ability-rule-picker" aria-label="Ability search">
      <header className="ability-rule-picker__header">
        <Button onClick={onBack}>Cancel</Button>
        <h2>Find rule for {label}</h2>
        <SearchEntry
          inputRef={input}
          label="Search Atlas rules"
          placeholder="Rule name"
          query={query}
          acceptedSubmission={submission}
          disabled={disabled}
          onSearch={(value, accepted) => {
            const next = value.trim();
            setSubmission(accepted);
            if (next === text && next) restartSearch();
            onSearch(next);
          }}
        />
      </header>
      {result.error && (
        <ErrorNotice
          error={result.error}
          title="Could not search Atlas"
          description={
            page?.rows.length ? "The previous results are still shown." : undefined
          }
          action={
            <Button
              loading={result.isFetching ? { delay: 500 } : false}
              disabled={disabled || result.isFetching}
              onClick={() => void result.refetch()}
            >
              Retry
            </Button>
          }
        />
      )}
      <AbilityRuleResults
        key={JSON.stringify([pagingKey, pageNumber, paging.epoch])}
        page={page}
        pageNumber={pageNumber}
        onPage={(number) => {
          if (page)
            setPaging({
              ...paging,
              page: number,
              generation: page.artifact_generation,
            });
        }}
        loading={loading}
        preparing={!execution.ready}
        failed={Boolean(result.error)}
        hasQuery={Boolean(text)}
        onRefresh={restartSearch}
        onLink={onLink}
        disabled={disabled}
      />
    </section>
  );
}

function AbilityRuleResults({
  page,
  pageNumber,
  onPage,
  loading,
  preparing,
  failed,
  hasQuery,
  onRefresh,
  onLink,
  disabled,
}: {
  page?: SearchResultsPage;
  pageNumber: number;
  onPage: (page: number) => void;
  loading: boolean;
  preparing: boolean;
  failed: boolean;
  hasQuery: boolean;
  onRefresh: () => void;
  onLink: (record: RuleCandidate) => void;
  disabled: boolean;
}) {
  const generation = page?.artifact_generation;
  const [selectionKey, setSelectionKey] = useState<{
    key: string;
    generation: string;
  } | null>(null);
  const reading = useReferenceReading(null, true, 0, generation);
  const rows = page?.rows ?? emptyRows;
  const selected = selectionKey?.generation === generation ? selectionKey?.key : null;
  const candidate = rows.find((row) => row.record.record_key === selected);
  const selection = usePreparedReferenceSelection(
    reading,
    Boolean(generation) && !disabled && !loading && !failed,
    0,
    generation ?? null,
  );
  if (selectionKey && !candidate) {
    setSelectionKey(null);
    reading.select(null);
  }
  const detail = reading.detail;
  const request = reading.request;
  const count = page?.page.total ?? 0;
  const prepared =
    candidate &&
    generation &&
    detail?.artifact_generation === generation &&
    detail.record.record_key === selected &&
    detail.selected.record_key === selected &&
    !detail.selected.embedded_path.length &&
    !detail.selected.field &&
    !detail.selected.passage &&
    request?.navigation.record_key === selected &&
    !request.match &&
    !request.landing &&
    !reading.canGoBack &&
    !reading.error
      ? { key: candidate.record.record_key, title: detail.record.title, generation }
      : undefined;
  return (
    <ReferenceBrowser
      observeWidth
      previewOnEnter
      searchCards
      showRelated={false}
      showMatchContext={false}
      rows={rows}
      loading={loading}
      loadingLabel={preparing ? "Preparing rule search…" : "Searching Atlas…"}
      selectedRecordKey={candidate?.record.record_key ?? null}
      onSelect={(row, request) => {
        if (!generation) return;
        void selection.select({
          request,
          name: row.record.title,
          generation,
          commit: () => setSelectionKey({ key: row.record.record_key, generation }),
        });
      }}
      onReaderClose={selection.cancel}
      reading={reading}
      generation={generation}
      disabled={disabled}
      emptyMessage={
        loading || failed
          ? null
          : !hasQuery
            ? "Search by rule name."
            : page
              ? "No rules found. Try a different name."
              : null
      }
      resultsHeader={
        <div className="result-pane-header">
          <span>
            {count
              ? page?.coverage?.exhaustive === false
                ? `Top ${count.toLocaleString()} ${count === 1 ? "match" : "matches"}`
                : `${count.toLocaleString()} ${count === 1 ? "result" : "results"}`
              : "Results"}
          </span>
          {count > pageSize && (
            <div
              className="pager pager--compact"
              role="group"
              aria-label="Result pages"
            >
              <Pagination
                simple
                size="small"
                current={pageNumber}
                pageSize={pageSize}
                total={count}
                showSizeChanger={false}
                disabled={disabled || loading}
                onChange={onPage}
              />
            </div>
          )}
          <ReferenceSelectionStatus
            selection={selection}
            onRetry={() => {
              if (isArtifactGenerationChanged(selection.current?.error)) {
                selection.cancel();
                onRefresh();
              } else {
                selection.retry();
              }
            }}
          />
        </div>
      }
      readerFooter={
        candidate
          ? {
              expanded: false,
              content: (
                <div className="ability-rule-picker__actions">
                  <Button
                    type="primary"
                    disabled={
                      !prepared ||
                      disabled ||
                      loading ||
                      failed ||
                      reading.loading ||
                      Boolean(selection.current)
                    }
                    onClick={() => {
                      if (prepared && !selection.current) onLink(prepared);
                    }}
                  >
                    Link this rule
                  </Button>
                </div>
              ),
            }
          : undefined
      }
    />
  );
}
