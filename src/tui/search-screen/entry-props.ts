import type { SearchRequest } from "../../domain/search-request-types.js";
import type { Pf2eTerminalSearchSession } from "../search/service.js";
import type { RouteTransitionStatus } from "../route-transition-status.js";
import type { SearchScreenOrigin } from "./workflow-types.js";

type SearchScreenCommonProps = {
  transitionStatus?: RouteTransitionStatus | null;
  origin?: SearchScreenOrigin;
  promptForInitialMode?: boolean;
  onBack: () => void;
};

export type SearchScreenEditorEntryProps = SearchScreenCommonProps & {
  entry?: "editor";
  initialRequest?: SearchRequest;
  initialSession?: never;
};

export type SearchScreenResultsEntryProps = SearchScreenCommonProps & {
  entry: "results";
  initialQuery?: never;
  initialSession: Pf2eTerminalSearchSession;
};

export type SearchScreenProps = SearchScreenEditorEntryProps | SearchScreenResultsEntryProps;
