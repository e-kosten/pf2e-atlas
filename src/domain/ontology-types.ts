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
  readonly text: string;
  readonly tone?: OntologyLineTone;
  readonly indent?: number;
  readonly noWrap?: boolean;
}

export interface OntologyNodeQuery {
  readonly kind: "listRecords" | "lookup" | "search";
  readonly label?: string;
  readonly filters: Readonly<SearchFilters>;
}

export type OntologySelectionState = "any" | "all" | "exclude";

export interface OntologyNodeSelection {
  readonly field: string;
  readonly fieldLabel: string;
  readonly value: string;
  readonly allowedStates: readonly OntologySelectionState[];
}

export type OntologyChildPresentation =
  | { readonly mode: "flat" }
  | {
      readonly mode: "grouped";
      readonly groupBy: string;
      readonly render: "inline" | "navigable" | "auto";
      readonly autoInlineMaxGroups?: number;
      readonly autoInlineMaxChildren?: number;
    };

export interface OntologyNode {
  readonly id: string;
  readonly kind: string;
  readonly label: string;
  readonly shortLabel?: string;
  readonly description?: string;
  readonly filterText: string;
  readonly listLabel?: string;
  readonly detailTitle?: string;
  readonly detailLines: readonly OntologyTextLine[];
  readonly children?: readonly OntologyNode[];
  readonly loadChildren?: () => readonly OntologyNode[];
  readonly childPresentation?: OntologyChildPresentation;
  readonly groupValues?: Readonly<Record<string, string>>;
  readonly query?: OntologyNodeQuery;
  readonly selection?: OntologyNodeSelection;
}

export interface OntologyDomainSummary {
  readonly id: OntologyDomainId;
  readonly label: string;
  readonly description: string;
}

export interface OntologyDomainModel extends OntologyDomainSummary {
  readonly rootNodes: readonly OntologyNode[];
}
