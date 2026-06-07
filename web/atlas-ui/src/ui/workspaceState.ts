import { decodeSearchState, type SearchFormState } from "../state/searchState";

export type WorkspaceInteractionState = {
  search: SearchFormState;
  selectedRecordKey: string | null;
  focusedResultKey: string | null;
  pageNumber: number;
  activeSearch: SearchFormState;
};

export type WorkspaceInteractionEvent =
  | {
      type: "url.restored";
      search: SearchFormState;
      selectedRecordKey: string | null;
    }
  | { type: "search.changed"; search: SearchFormState }
  | { type: "search.executionCommitted"; search: SearchFormState }
  | { type: "resultPage.changed"; pageNumber: number }
  | { type: "result.focused"; recordKey: string | null }
  | { type: "record.selected"; recordKey: string | null };

export function initialWorkspaceInteractionState(): WorkspaceInteractionState {
  const search = decodeSearchState(
    new URLSearchParams(window.location.search).get("s"),
  );
  return {
    search,
    selectedRecordKey: recordKeyFromPath(window.location.pathname),
    focusedResultKey: null,
    pageNumber: 1,
    activeSearch: search,
  };
}

export function workspaceInteractionReducer(
  state: WorkspaceInteractionState,
  event: WorkspaceInteractionEvent,
): WorkspaceInteractionState {
  switch (event.type) {
    case "url.restored":
      return {
        ...state,
        search: event.search,
        activeSearch: event.search,
        selectedRecordKey: event.selectedRecordKey,
        focusedResultKey: null,
        pageNumber: 1,
      };
    case "search.changed":
      return {
        ...state,
        search: event.search,
      };
    case "search.executionCommitted":
      if (event.search === state.activeSearch) {
        return state;
      }
      return {
        ...state,
        activeSearch: event.search,
        pageNumber: 1,
        focusedResultKey: null,
      };
    case "resultPage.changed":
      return {
        ...state,
        pageNumber: event.pageNumber,
        focusedResultKey: null,
      };
    case "result.focused":
      return {
        ...state,
        focusedResultKey: event.recordKey,
      };
    case "record.selected":
      return {
        ...state,
        selectedRecordKey: event.recordKey,
      };
  }
}

export function recordKeyFromPath(pathname: string): string | null {
  const match = pathname.match(/^\/records\/(.+)$/);
  return match ? decodeURIComponent(match[1]) : null;
}
