/** Public discovery products. These describe declarations, not ingest admission. */
export interface Location { file: string; line: number; column: number }
export interface SourceIdentity {
  system_version: string | null; source_digest: string; input_file_count: number;
  git_commit: string | null; git_clean: boolean | null;
}
export interface ExtractionSummary {
  format: 'atlas-source-extraction/v1'; source: SourceIdentity;
  typescript_version: string; dependency_lock_digest: string; complete: boolean;
  type_graph_complete: boolean; trait_catalog_complete: boolean;
  products: { type_graph: string; trait_catalog: string };
}
export interface DiscoveryDiagnostic { code: string; message: string; node?: string; location?: Location }
export interface RootSelection {
  file: string; name: string; documentKind?: string; ruleKey?: string;
  schemaConstructor?: { file: string; name: string };
}
export interface RuleArrayInput {
  field: string; arrayRef: string; elementRef: string;
  fieldClass: 'ArrayField' | 'StrictArrayField'; declaredAt: Location[];
}
export interface Family { documentKind: string; registered: string[]; discovered?: string[] }
export interface Portfolio {
  selections: RootSelection[]; families: Family[]; diagnostics: DiscoveryDiagnostic[];
  documentKinds: string[]; ruleKeys: string[]; virtualSource: string;
}
export type FieldSerialization =
  | { basis: 'omitted-function-property'; declaredType: string; declaredOptional: boolean }
  | { basis: 'predicate-constructor-input'; declaredRef: string; declaredType: string; declaredAt: Location[] };
export interface GraphField {
  name: string; ref: string; optional: boolean; nullable: boolean;
  undefinedAllowed: boolean; forbidden: boolean; declaredAt: Location[]; serialization?: FieldSerialization;
}
export interface GraphBase {
  id: string; name?: string; declaredAt?: Location[];
  typeArguments?: { expression: string; declaredAt: Location[] }[];
}
export type GraphShape =
  | { kind: 'union'; members: string[] }
  | { kind: 'intersection'; members: string[]; fields: GraphField[]; indexSignatures: IndexSignature[]; impossible?: boolean }
  | { kind: 'literal'; value: string | number | boolean }
  | { kind: 'primitive'; value: string }
  | { kind: 'unresolved' }
  | { kind: 'unsupported'; reason: string }
  | { kind: 'open'; domain: 'any' | 'unknown' | 'object' | 'non-nullish'; fields?: GraphField[]; indexSignatures?: IndexSignature[] }
  | { kind: 'template'; text: readonly string[]; parameters: string[] }
  | { kind: 'array'; element: string; readonly: boolean;
      serialization?: { basis: 'array-subclass'; rawRef: string; method: 'toObject'; declaredAt: Location[] } }
  | { kind: 'tuple'; elements: { ref: string; optional: boolean; rest: boolean }[]; readonly: boolean }
  | { kind: 'object'; fields: GraphField[]; extends?: string[]; indexSignatures: IndexSignature[] };
export interface IndexSignature { key: string; value: string; readonly: boolean }
export type GraphNode = GraphBase & GraphShape;
export interface ProjectDiagnostic { code: number; category: string; message: string; location?: Location }
export interface TypeGraph {
  format: 'atlas-source-type-graph/v1'; typescript: string; complete: boolean;
  source?: SourceIdentity;
  status: 'complete' | 'incomplete'; roots: (RootSelection & { ref: string | null; arrayInputs?: RuleArrayInput[] })[];
  portfolio?: Pick<Portfolio, 'documentKinds' | 'families' | 'ruleKeys'>;
  nodes: GraphNode[]; diagnostics: DiscoveryDiagnostic[];
  projectDiagnostics: { selected: ProjectDiagnostic[]; unrelated: ProjectDiagnostic[] };
}
export interface CatalogDiagnostic {
  severity: 'error' | 'warning'; code: string; message: string; source: Location | null;
}
export interface FamilyBinding {
  scope: 'actor' | 'item'; family: string;
  basis: 'catalog-key-type' | 'runtime-validTraits' | 'declared-keyof-vocabulary';
  declaration: string | null; source: Location;
}
export interface Catalog {
  name: string; namespace: 'trait' | 'otherTag' | 'rarity'; complete: boolean;
  exposedInConfig: boolean; exportedFromTraits: boolean; source: Location;
  entries: { identifier: string; labelKey: string | null; label: string | null;
    descriptionKey: string | null; description: string | null; source: Location }[];
  familyBindings: FamilyBinding[];
}
export interface TraitCatalog {
  complete: boolean; catalogs: Catalog[];
  descriptions: { identifier: string; key: string | null; text: string | null; source: Location }[];
  diagnostics: CatalogDiagnostic[];
}
