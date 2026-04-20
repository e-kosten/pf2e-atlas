import type { SearchFilters } from "../../domain/search-types.js";
import type {
  DerivedTagTerminalLine,
  DerivedTagTerminalPolicySelection,
  DerivedTagTerminalPolicyState,
  DerivedTagTerminalTone,
  DerivedTagTerminalTwoPaneLayoutMode,
} from "../framework/types.js";

export type FilterExplorerPolicyState = DerivedTagTerminalPolicyState;
export type FilterExplorerSelection = DerivedTagTerminalPolicySelection<string>;
export type FilterExplorerSelectionMap = Record<string, FilterExplorerSelection>;
export type FilterExplorerModeKind = "inspect-and-open" | "compose";
export type FilterExplorerLineTone = DerivedTagTerminalTone;
export type FilterExplorerDomainId = string;

export interface FilterExplorerTextLine {
  readonly text: string;
  readonly tone?: FilterExplorerLineTone;
  readonly indent?: number;
  readonly noWrap?: boolean;
}

export type FilterExplorerQueryTarget = {
  readonly kind: "listRecords" | "lookup" | "search";
  readonly label?: string;
  readonly filters: Readonly<SearchFilters>;
  readonly openInResults?: boolean;
};

export type FilterExplorerChildPresentation =
  | { readonly mode: "flat" }
  | {
      readonly mode: "grouped";
      readonly groupBy: string;
      readonly render: "inline" | "navigable" | "auto";
      readonly autoInlineMaxGroups?: number;
      readonly autoInlineMaxChildren?: number;
    };

export type FilterExplorerNodeSelection = {
  readonly field: string;
  readonly fieldLabel: string;
  readonly value: string;
  readonly allowedStates: readonly FilterExplorerPolicyState[];
};

export interface FilterExplorerNode {
  readonly id: string;
  readonly kind: string;
  readonly label: string;
  readonly shortLabel?: string;
  readonly description?: string;
  readonly filterText: string;
  readonly listLabel?: string;
  readonly detailTitle?: string;
  readonly detailLines: readonly FilterExplorerTextLine[];
  readonly children?: readonly FilterExplorerNode[];
  readonly loadChildren?: () => readonly FilterExplorerNode[];
  readonly childPresentation?: FilterExplorerChildPresentation;
  readonly groupValues?: Readonly<Record<string, string>>;
  readonly query?: FilterExplorerQueryTarget;
  readonly selection?: FilterExplorerNodeSelection;
}

export interface FilterExplorerModel {
  readonly id: FilterExplorerDomainId;
  readonly label: string;
  readonly description: string;
  readonly rootNodes: readonly FilterExplorerNode[];
}

export type FilterExplorerBrowserState = {
  depth: number;
  selectedNodeIds: string[];
  filter: string;
  detailScroll: number;
};

export type FilterExplorerBrowserUiState = {
  activePane: "list" | "detail";
  browserState: FilterExplorerBrowserState;
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode;
  searchInput: string;
  searchMode: boolean;
};

export type FilterExplorerBrowserSnapshot = FilterExplorerBrowserUiState;

export type FilterExplorerBrowserSelection = {
  ancestors: FilterExplorerNode[];
  currentNodes: readonly FilterExplorerNode[];
  currentNode?: FilterExplorerNode;
  currentParent?: FilterExplorerNode;
};

export type FilterExplorerBrowserContext = {
  state: FilterExplorerBrowserUiState;
  effectiveState: FilterExplorerBrowserState;
  selection: FilterExplorerBrowserSelection;
  currentNode?: FilterExplorerNode;
  currentNodeHasChildren: boolean;
  breadcrumb: string;
  bodyHeight: number;
  detailWidth: number;
  detailLines: DerivedTagTerminalLine[];
  visibleDetailLines: DerivedTagTerminalLine[];
  detailTitle: string;
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode;
  maxDetailScroll: number;
  detailJumpSize: number;
  detailPageSize: number;
  selectionJumpSize: number;
  searchIndicator: string;
};

export type FilterExplorerScalarOperator = "eq" | "neq" | "gte" | "lte" | "between";

export type FilterExplorerDiscreteComposeTarget = {
  kind?: "discrete";
  field: string;
  fieldLabel: string;
  value: string;
  valueLabel?: string;
  allowedStates: readonly FilterExplorerPolicyState[];
};

export type FilterExplorerScalarClause =
  | {
      operator: Exclude<FilterExplorerScalarOperator, "between">;
      value: string | number | boolean;
      valueLabel?: string;
      summaryLabel?: string;
    }
  | {
      operator: "between";
      min: number;
      max: number;
      summaryLabel?: string;
    };

export type FilterExplorerScalarComposeTarget = {
  kind: "scalar";
  key: string;
  fieldLabel: string;
  subjectLabel: string;
  valueType: "number" | "text" | "boolean";
  editorLabel?: string;
};

export type FilterExplorerComposeTarget =
  | FilterExplorerDiscreteComposeTarget
  | FilterExplorerScalarComposeTarget;

export type FilterExplorerScalarClauseMap = Record<string, FilterExplorerScalarClause>;

export type FilterExplorerComposeDraft = {
  selection: FilterExplorerSelectionMap;
  scalarClauses: FilterExplorerScalarClauseMap;
};

export type FilterExplorerScalarEditRequest = {
  target: FilterExplorerScalarComposeTarget;
  currentClause?: FilterExplorerScalarClause;
  draft: FilterExplorerComposeDraft;
};

export type FilterExplorerInspectResult = {
  node: FilterExplorerNode;
  query: FilterExplorerQueryTarget;
  target?: FilterExplorerComposeTarget;
  openIntent: "browse" | "results";
};

export type FilterExplorerInspectAndOpenMode = {
  kind: "inspect-and-open";
  resolveInspectTarget?: (node: FilterExplorerNode | undefined) => FilterExplorerComposeTarget | undefined;
  onEditScalarTarget?: (
    request: FilterExplorerScalarEditRequest,
  ) => Promise<FilterExplorerScalarClause | null | undefined> | FilterExplorerScalarClause | null | undefined;
  onOpenInspectResult?: (
    result: FilterExplorerInspectResult,
    snapshot: FilterExplorerBrowserSnapshot,
  ) => void;
  onOpenQuery?: (query: FilterExplorerQueryTarget, snapshot: FilterExplorerBrowserSnapshot) => void;
  openListRecordQueriesInResults?: boolean;
};

export type FilterExplorerComposeMode = {
  kind: "compose";
  resolveSelectionTarget: (node: FilterExplorerNode | undefined) => FilterExplorerComposeTarget | undefined;
  draft?: FilterExplorerComposeDraft;
  initialDraft?: FilterExplorerComposeDraft;
  onDraftChange?: (draft: FilterExplorerComposeDraft) => void;
  onEditScalarTarget?: (
    request: FilterExplorerScalarEditRequest,
  ) => Promise<FilterExplorerScalarClause | null | undefined> | FilterExplorerScalarClause | null | undefined;
  selection?: FilterExplorerSelectionMap;
  initialSelection?: FilterExplorerSelectionMap;
  onSelectionChange?: (selection: FilterExplorerSelectionMap) => void;
  detailTitle?: string;
  emptySelectionText?: string;
  focusedSelectionTitle?: string;
  selectedSelectionsTitle?: string;
};

export type FilterExplorerMode = FilterExplorerInspectAndOpenMode | FilterExplorerComposeMode;

export type FilterExplorerOptions = {
  model: FilterExplorerModel;
  initialSnapshot?: FilterExplorerBrowserSnapshot;
  rootDepth?: number;
  exitAtRootDepth?: boolean;
  mode: FilterExplorerMode;
  onExit: () => void;
  title?: string;
};

export type FilterExplorerControllerContext = {
  model: FilterExplorerModel;
  mode: FilterExplorerMode;
  screenTitle: string;
  browser: FilterExplorerBrowserContext;
  draft: FilterExplorerComposeDraft;
  selection: FilterExplorerSelectionMap;
  selectedTarget?: FilterExplorerComposeTarget;
  selectedPolicyState?: FilterExplorerPolicyState;
  selectedScalarClause?: FilterExplorerScalarClause;
  selectedInspectResult?: FilterExplorerInspectResult;
};
