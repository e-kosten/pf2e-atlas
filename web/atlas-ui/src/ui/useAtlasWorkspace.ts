import { useEffect, useMemo, useState } from "react";
import { useQuery, type UseQueryResult } from "@tanstack/react-query";
import {
  getReadiness,
  getRecordDetail,
  openResultWindow,
  readResultWindowPage,
} from "../api/atlasApi";
import type { RecordDetailView, ResultWindowPage } from "../generated/atlas";
import {
  buildOpenRequest,
  decodeSearchState,
  DEFAULT_SEARCH_STATE,
  encodeSearchState,
  type SearchFormState,
} from "../state/searchState";

export type AtlasWorkspaceState = {
  search: SearchFormState;
  setSearch: (next: SearchFormState) => void;
  selectedRecordKey: string | null;
  selectRecord: (recordKey: string | null) => void;
  pageNumber: number;
  setPageNumber: (page: number) => void;
  resultPage: ResultWindowPage | undefined;
  recordDetail: RecordDetailView | undefined;
  readiness: UseQueryResult<Awaited<ReturnType<typeof getReadiness>>, Error>;
  resultsLoading: boolean;
  detailLoading: boolean;
  errorMessage: string | null;
  refresh: () => void;
};

export function useAtlasWorkspace(): AtlasWorkspaceState {
  const [search, setSearchState] = useState<SearchFormState>(() =>
    decodeSearchState(new URLSearchParams(window.location.search).get("s")),
  );
  const [selectedRecordKey, setSelectedRecordKey] = useState<string | null>(
    () => recordKeyFromPath(window.location.pathname),
  );
  const [pageNumber, setPageNumber] = useState(search.pageSize > 0 ? 1 : 1);
  const [windowId, setWindowId] = useState<bigint | null>(null);
  const searchToken = useMemo(() => encodeSearchState(search), [search]);

  useEffect(() => {
    const onPopState = () => {
      setSearchState(
        decodeSearchState(new URLSearchParams(window.location.search).get("s")),
      );
      setSelectedRecordKey(recordKeyFromPath(window.location.pathname));
    };
    window.addEventListener("popstate", onPopState);
    return () => window.removeEventListener("popstate", onPopState);
  }, []);

  useEffect(() => {
    setWindowId(null);
    setPageNumber(1);
  }, [searchToken]);

  const readiness = useQuery({
    queryKey: ["readiness"],
    queryFn: getReadiness,
  });

  const resultsQuery = useQuery({
    queryKey: ["results", searchToken, pageNumber],
    queryFn: async () => {
      if (windowId === null || pageNumber === 1) {
        const page = await openResultWindow(buildOpenRequest(search, pageNumber));
        setWindowId(page.window_id);
        return page;
      }
      return readResultWindowPage(windowId, {
        page: { number: pageNumber, size: search.pageSize },
      });
    },
  });

  const detailQuery = useQuery({
    queryKey: ["record-detail", selectedRecordKey],
    queryFn: () => getRecordDetail(selectedRecordKey!),
    enabled: selectedRecordKey !== null,
  });

  function setSearch(next: SearchFormState) {
    setSearchState(next);
    const url = selectedRecordKey
      ? `/records/${encodeURIComponent(selectedRecordKey)}?s=${encodeSearchState(next)}`
      : `/?s=${encodeSearchState(next)}`;
    history.replaceState(null, "", url);
  }

  function selectRecord(recordKey: string | null) {
    setSelectedRecordKey(recordKey);
    const url =
      recordKey === null
        ? `/?s=${searchToken}`
        : `/records/${encodeURIComponent(recordKey)}?s=${searchToken}`;
    history.pushState(null, "", url);
  }

  const errorMessage =
    messageFromError(resultsQuery.error) ??
    messageFromError(detailQuery.error) ??
    messageFromError(readiness.error);

  return {
    search,
    setSearch,
    selectedRecordKey,
    selectRecord,
    pageNumber,
    setPageNumber,
    resultPage: resultsQuery.data,
    recordDetail: detailQuery.data,
    readiness,
    resultsLoading: resultsQuery.isLoading || resultsQuery.isFetching,
    detailLoading: detailQuery.isLoading || detailQuery.isFetching,
    errorMessage,
    refresh: () => {
      void readiness.refetch();
      void resultsQuery.refetch();
      void detailQuery.refetch();
    },
  };
}

function recordKeyFromPath(pathname: string): string | null {
  const match = pathname.match(/^\/records\/(.+)$/);
  return match ? decodeURIComponent(match[1]) : null;
}

function messageFromError(error: unknown): string | null {
  if (!error) {
    return null;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

export function resetSearchState(): SearchFormState {
  return DEFAULT_SEARCH_STATE;
}
