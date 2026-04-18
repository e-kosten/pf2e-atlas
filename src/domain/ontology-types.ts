import type { SearchFilters } from "./search-types.js";

export type OntologyDomainId = "derivedTags" | "catalogCategories" | "searchSemantics";

export type OntologyLineTone =
  | "default"
  | "heading"
  | "section"
  | "dim"
  | "accent"
  | "success"
  | "warning"
  | "danger"
  | "selected";

export interface OntologyTextLine {
  text: string;
  tone?: OntologyLineTone;
  indent?: number;
  noWrap?: boolean;
}

export interface OntologyNodeQuery {
  kind: "listRecords" | "lookup" | "search";
  label?: string;
  filters: SearchFilters;
}

export type OntologySelectionState = "any" | "all" | "exclude";

export interface OntologyNodeSelection {
  field: string;
  fieldLabel: string;
  value: string;
  allowedStates: OntologySelectionState[];
}

export type OntologyChildPresentation =
  | { mode: "flat" }
  | {
    mode: "grouped";
    groupBy: string;
    render: "inline" | "navigable" | "auto";
    autoInlineMaxGroups?: number;
    autoInlineMaxChildren?: number;
  };

export interface OntologyNode {
  id: string;
  kind: string;
  label: string;
  shortLabel?: string;
  description?: string;
  filterText: string;
  listLabel?: string;
  detailTitle?: string;
  detailLines: OntologyTextLine[];
  children?: OntologyNode[];
  loadChildren?: () => OntologyNode[];
  childPresentation?: OntologyChildPresentation;
  groupValues?: Record<string, string>;
  query?: OntologyNodeQuery;
  selection?: OntologyNodeSelection;
}

export interface OntologyDomainSummary {
  id: OntologyDomainId;
  label: string;
  description: string;
}

export interface OntologyDomainModel extends OntologyDomainSummary {
  rootNodes: OntologyNode[];
}
