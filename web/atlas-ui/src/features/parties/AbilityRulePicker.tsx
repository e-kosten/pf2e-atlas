import { Button, Pagination, type InputRef } from "antd";
import { ArrowLeft } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { useLayoutEffect, useRef, useState } from "react";
import { searchPage } from "../../api/atlasApi";
import type { SearchResultRow, SearchResultsPage } from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { SearchEntry } from "../../shared/filters/SearchEntry";
import type { SearchSubmission } from "../../shared/filters/filterControls";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { ReferenceBrowser } from "../../shared/records/ReferenceBrowser";
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
  const [paging, setPaging] = useState({ key: pagingKey, page: 1 });
  const pageNumber = paging.key === pagingKey ? paging.page : 1;
  const [submission, setSubmission] = useState<SearchSubmission>();
  const input = useRef<InputRef>(null);
  useLayoutEffect(() => {
    input.current?.focus({ preventScroll: true });
  }, []);
  const result = useQuery({
    queryKey: ["pc-ability-search", ...searchIdentity, pageNumber],
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
          search_profile: execution.profile,
          ranking,
        },
        signal,
      ),
    enabled: Boolean(text && execution.ready),
    retry: false,
    meta: referenceQueryMeta,
  });
  const loading = Boolean(text && (!execution.ready || result.isFetching));
  const page = text && execution.ready ? result.data : undefined;
  return (
    <section className="ability-rule-picker" aria-label="Ability search">
      <header className="ability-rule-picker__header">
        <Button icon={<ArrowLeft size={16} aria-hidden="true" />} onClick={onBack}>
          Back to abilities
        </Button>
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
            setPaging({ key: pagingKey, page: 1 });
            if (next === text && next && pageNumber === 1 && execution.ready)
              void result.refetch();
            onSearch(next);
          }}
        />
      </header>
      {result.error && (
        <ErrorNotice
          error={result.error}
          title="Could not search Atlas"
          action={
            <Button
              loading={result.isFetching}
              disabled={disabled}
              onClick={() => void result.refetch()}
            >
              Retry
            </Button>
          }
        />
      )}
      <AbilityRuleResults
        key={JSON.stringify([pagingKey, pageNumber, page?.artifact_generation])}
        page={page}
        pageNumber={pageNumber}
        onPage={(page) => setPaging({ key: pagingKey, page })}
        loading={loading}
        preparing={!execution.ready}
        failed={Boolean(result.error)}
        hasQuery={Boolean(text)}
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
  onLink: (record: RuleCandidate) => void;
  disabled: boolean;
}) {
  const generation = page?.artifact_generation;
  const [selected, setSelected] = useState<string | null>(null);
  const reading = useReferenceReading(null, true, 0, generation);
  const rows = page?.rows ?? emptyRows;
  const candidate = rows.find((row) => row.record.record_key === selected);
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
        setSelected(row.record.record_key);
        reading.select(request);
      }}
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
        </div>
      }
      readerFooter={
        prepared
          ? {
              expanded: false,
              content: (
                <div className="ability-rule-picker__actions">
                  <Button
                    type="primary"
                    disabled={disabled || loading || failed || reading.loading}
                    onClick={() => onLink(prepared)}
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
