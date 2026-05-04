import type { SearchRequest } from "../../domain/search-request-types.js";
import type { EntityPageTarget } from "../../app/ontology/entity-page.js";
import type {
  OntologyChildPresentation,
  OntologyDomainModel,
  OntologyNode,
  OntologyNodeQuery,
  OntologyTextLine,
} from "../../domain/ontology-types.js";
import type {
  DerivedTagTerminalActionTargetOption,
  DerivedTagTerminalActionTargetState,
} from "../action-target.js";
import type {
  DerivedTagTerminalLine,
  DerivedTagTerminalPointerEvent,
  DerivedTagTerminalTone,
  DerivedTagTerminalTwoPaneLayoutMode,
} from "../framework/types.js";
import type { PageDocumentInteractionState } from "../page-document/interaction.js";
import type {
  PageDocumentModel,
  PageDocumentSectionModel,
  PageDocumentTargetNode,
} from "../page-document/model.js";
import type { RouteTransitionStatus } from "../route-transition-status.js";
import type { TerminalListDetailNotification } from "../list-detail-presentation.js";

export type FilterExplorerModeKind = "inspect-and-open" | "compose";
export type FilterExplorerLineTone = DerivedTagTerminalTone;
export type FilterExplorerDomainId = string;
export type FilterExplorerActivationStyle = "open" | "toggle" | "edit" | "none";

export type FilterExplorerModeSwitchOption<TMode extends string = string> = {
  readonly value: TMode;
  readonly label: string;
  readonly description: string;
};

type FilterExplorerModeChangeHandler<TMode extends string> = {
  bivarianceHack(mode: TMode): void;
}["bivarianceHack"];

export type FilterExplorerDiscoveryState<TMode extends string = string> = {
  readonly mode: TMode;
  readonly modes: readonly FilterExplorerModeSwitchOption<TMode>[];
  readonly pendingMode?: TMode;
  readonly isRefreshing?: boolean;
  readonly onModeChange?: FilterExplorerModeChangeHandler<TMode>;
};

export type FilterExplorerTextLine = OntologyTextLine;

export type FilterExplorerQueryTarget = {
  readonly label?: string;
  readonly request: Readonly<SearchRequest>;
};

export const FILTER_EXPLORER_LAUNCH_INTENT = {
  EDITOR: "editor",
  RESULTS: "results",
} as const;

export type FilterExplorerLaunchIntent =
  (typeof FILTER_EXPLORER_LAUNCH_INTENT)[keyof typeof FILTER_EXPLORER_LAUNCH_INTENT];

export type FilterExplorerQueryOpenIntent = {
  readonly query: FilterExplorerQueryTarget;
  readonly launchIntent: FilterExplorerLaunchIntent;
};

export type FilterExplorerStateBadge =
  | { readonly kind: "include" }
  | { readonly kind: "exclude" }
  | { readonly kind: "off" }
  | {
      readonly kind: "custom";
      readonly text: string;
      readonly tone?: DerivedTagTerminalTone;
    };

export type FilterExplorerTargetPresentation = {
  readonly activationStyle: FilterExplorerActivationStyle;
  readonly stateBadge?: FilterExplorerStateBadge;
  readonly suffixText?: string;
  readonly tone?: DerivedTagTerminalTone;
};

export type FilterExplorerDescribeNodeArgs = {
  readonly node: FilterExplorerNode | undefined;
  readonly target?: FilterExplorerComposeTarget;
  readonly isFocused: boolean;
  readonly controller?: FilterExplorerControllerContext;
};

export type FilterExplorerHostAdapter = {
  readonly resolveTarget?: (node: FilterExplorerNode | undefined) => FilterExplorerComposeTarget | undefined;
  readonly resolvePageDocument?: (node: FilterExplorerNode | undefined) => PageDocumentModel | null | undefined;
  readonly describeNode: (args: FilterExplorerDescribeNodeArgs) => FilterExplorerTargetPresentation | undefined;
  readonly getDraft?: () => FilterExplorerComposeDraft;
  readonly activateTarget?: (args: {
    readonly target: FilterExplorerComposeTarget;
    readonly controller: FilterExplorerControllerContext;
    readonly reason: "open" | "cycle";
  }) => boolean;
  readonly activatePageTarget?: (args: {
    readonly target: EntityPageTarget;
    readonly controller: FilterExplorerControllerContext;
  }) => boolean | void;
  readonly selectionPresentation?: {
    readonly detailTitle?: string;
    readonly emptySelectionText?: string;
    readonly focusedClauseTitle?: string;
    readonly selectionTitle?: string;
  };
};

export type FilterExplorerSelectTargetOutcome = {
  readonly kind: "selectTarget";
  readonly activationStyle: FilterExplorerActivationStyle;
  readonly result: FilterExplorerInspectResult;
  readonly queryIntent: FilterExplorerQueryOpenIntent;
};

export type FilterExplorerOutcome =
  | { readonly kind: "back" }
  | { readonly kind: "exitRoot" }
  | { readonly kind: "cancel" }
  | FilterExplorerSelectTargetOutcome;

export type FilterExplorerChildPresentation = OntologyChildPresentation;

export type FilterExplorerNodeSelection = {
  readonly field: string;
  readonly fieldLabel: string;
  readonly value: string;
  readonly allowedOperators: readonly FilterExplorerDiscreteClauseOperator[];
};

export type FilterExplorerNode = Omit<OntologyNode, "children" | "loadChildren" | "query" | "selection"> & {
  readonly children?: readonly FilterExplorerNode[];
  readonly loadChildren?: () => readonly FilterExplorerNode[];
  readonly query?: OntologyNodeQuery;
};

export type FilterExplorerModel = Omit<OntologyDomainModel, "rootNodes"> & {
  readonly id: FilterExplorerDomainId;
  readonly rootNodes: readonly FilterExplorerNode[];
};

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
  pageDocument?: PageDocumentModel | null;
  pageInteractionState: PageDocumentInteractionState;
  focusedPageSection?: PageDocumentSectionModel | null;
  selectedPageTarget?: PageDocumentTargetNode | null;
  detailInteractionState: FilterExplorerDetailPageInteractionState;
  detailTargetActionId?: "open" | "preview" | null;
};

export type FilterExplorerScalarOperator = "eq" | "neq" | "gt" | "gte" | "lt" | "lte" | "between";
export type FilterExplorerDiscreteClauseOperator = "include" | "exclude";

export type FilterExplorerDiscreteClause = {
  field: string;
  value: string;
  operator: FilterExplorerDiscreteClauseOperator;
};

export type FilterExplorerDiscreteComposeTarget = {
  kind?: "discrete";
  field: string;
  fieldLabel: string;
  value: string;
  valueLabel?: string;
  allowedOperators: readonly FilterExplorerDiscreteClauseOperator[];
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
  discreteClauses: FilterExplorerDiscreteClause[];
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
  launchIntent: FilterExplorerLaunchIntent;
};

export type FilterExplorerDetailPageInteractionState =
  | { kind: "none" }
  | {
      kind: "section";
      canEnterTargets: boolean;
    }
  | {
      kind: "target";
    };

export type FilterExplorerInspectAndOpenMode = {
  kind: "inspect-and-open";
  onEditScalarTarget?: (
    request: FilterExplorerScalarEditRequest,
  ) => Promise<FilterExplorerScalarClause | null | undefined> | FilterExplorerScalarClause | null | undefined;
  defaultListRecordLaunchIntent?: FilterExplorerLaunchIntent;
};

export type FilterExplorerComposeMode = {
  kind: "compose";
  draft?: FilterExplorerComposeDraft;
  initialDraft?: FilterExplorerComposeDraft;
  onDraftChange?: (draft: FilterExplorerComposeDraft) => void;
  onEditScalarTarget?: (
    request: FilterExplorerScalarEditRequest,
  ) => Promise<FilterExplorerScalarClause | null | undefined> | FilterExplorerScalarClause | null | undefined;
  detailTitle?: string;
  emptySelectionText?: string;
  focusedClauseTitle?: string;
  stagedClausesTitle?: string;
};

export type FilterExplorerMode = FilterExplorerInspectAndOpenMode | FilterExplorerComposeMode;

export type FilterExplorerOptions = {
  model: FilterExplorerModel;
  initialSnapshot?: FilterExplorerBrowserSnapshot;
  rootDepth?: number;
  exitAtRootDepth?: boolean;
  host: FilterExplorerHostAdapter;
  mode: FilterExplorerMode;
  discovery?: FilterExplorerDiscoveryState;
  onOutcome: (outcome: FilterExplorerOutcome, snapshot: FilterExplorerBrowserSnapshot) => void;
  title?: string;
  transitionStatus?: RouteTransitionStatus | null;
};

export type FilterExplorerControllerContext = {
  model: FilterExplorerModel;
  host: FilterExplorerHostAdapter;
  mode: FilterExplorerMode;
  screenTitle: string;
  browser: FilterExplorerBrowserContext;
  draft: FilterExplorerComposeDraft;
  discreteClauses: FilterExplorerDiscreteClause[];
  selectedTarget?: FilterExplorerComposeTarget;
  selectedDiscreteClause?: FilterExplorerDiscreteClause;
  selectedScalarClause?: FilterExplorerScalarClause;
  selectedInspectResult?: FilterExplorerInspectResult;
  discovery?: FilterExplorerDiscoveryState;
  actionEntries: readonly FilterExplorerActionEntry[];
  actionTargetState: DerivedTagTerminalActionTargetState;
  onDetailPointerEvent?: (event: DerivedTagTerminalPointerEvent) => boolean | void;
  notification?: TerminalListDetailNotification | null;
  transitionStatus?: RouteTransitionStatus | null;
};

export type FilterExplorerActionEntryId = `setMode:${string}` | "selectTarget:default" | "selectTarget:query";

export type FilterExplorerActionEntry =
  DerivedTagTerminalActionTargetOption<FilterExplorerActionEntryId> & {
    readonly action:
      | {
          readonly kind: "setMode";
          readonly mode: string;
        }
      | {
          readonly kind: "selectTarget";
          readonly selection: "default" | "query";
        };
  };
