import type { OntologyDomainModel, OntologyNode, OntologyNodeQuery } from "../../domain/ontology-types.js";
import type {
  DerivedTagTerminalPolicySelection,
  DerivedTagTerminalPolicyState,
} from "../framework/types.js";
import type { OntologyExplorerControllerContext } from "../ontology-explorer/controller.js";
import type { OntologyBrowserSnapshot } from "../ontology-explorer/ui.js";

export type FilterExplorerPolicyState = DerivedTagTerminalPolicyState;
export type FilterExplorerSelection = DerivedTagTerminalPolicySelection<string>;
export type FilterExplorerSelectionMap = Record<string, FilterExplorerSelection>;
export type FilterExplorerModeKind = "inspect-and-open" | "compose";

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

export type FilterExplorerInspectAndOpenMode = {
  kind: "inspect-and-open";
  onOpenQuery?: (query: OntologyNodeQuery, snapshot: OntologyBrowserSnapshot) => void;
  openListRecordQueriesInResults?: boolean;
};

export type FilterExplorerComposeMode = {
  kind: "compose";
  resolveSelectionTarget: (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined;
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
  model: OntologyDomainModel;
  initialSnapshot?: OntologyBrowserSnapshot;
  rootDepth?: number;
  exitAtRootDepth?: boolean;
  mode: FilterExplorerMode;
  onExit: () => void;
  title?: string;
};

export type FilterExplorerControllerContext = {
  model: OntologyDomainModel;
  mode: FilterExplorerMode;
  screenTitle: string;
  ontology: OntologyExplorerControllerContext;
  draft: FilterExplorerComposeDraft;
  selection: FilterExplorerSelectionMap;
  selectedTarget?: FilterExplorerComposeTarget;
  selectedPolicyState?: FilterExplorerPolicyState;
  selectedScalarClause?: FilterExplorerScalarClause;
  selectedQuery?: OntologyNodeQuery;
};
