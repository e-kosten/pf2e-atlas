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

export type FilterExplorerComposeTarget = {
  field: string;
  fieldLabel: string;
  value: string;
  valueLabel?: string;
  allowedStates: readonly FilterExplorerPolicyState[];
};

export type FilterExplorerInspectAndOpenMode = {
  kind: "inspect-and-open";
  onOpenQuery?: (query: OntologyNodeQuery, snapshot: OntologyBrowserSnapshot) => void;
  openListRecordQueriesInResults?: boolean;
};

export type FilterExplorerComposeMode = {
  kind: "compose";
  resolveSelectionTarget: (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined;
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
  selection: FilterExplorerSelectionMap;
  selectedTarget?: FilterExplorerComposeTarget;
  selectedPolicyState?: FilterExplorerPolicyState;
  selectedQuery?: OntologyNodeQuery;
};
