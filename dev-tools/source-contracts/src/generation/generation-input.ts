import type { GraphField, GraphNode, SourceIdentity, TypeGraph } from '../contracts.js';

export interface GenerationRoot {
  name: string; declaration: string; fields: GraphField[]; deferred: string[]; module: string; family?: string; sourceRef?: string; valueRef?: string;
  documentKind?: string; ruleKey?: string;
}
export interface GenerationInput {
  source: SourceIdentity; selection: GenerationRoot[]; nodes: GraphNode[]; openTraitArrays?: string[];
  portfolio?: { typescript: string; schemaRoots: TypeGraph['roots']; families: NonNullable<TypeGraph['portfolio']>['families'] };
}
export function validateModule(name: string): void {
  if (!/^[a-z][a-z0-9_]*(\/[a-z][a-z0-9_]*)*$/.test(name)
    || name.split('/').some(part => ['mod', 'self', 'super', 'crate', 'type', 'fn', 'use', 'struct', 'enum', 'pub', 'impl', 'trait', 'as', 'in', 'where', 'match', 'ref', 'const', 'static', 'let', 'loop', 'move', 'async', 'await', 'dyn', 'if', 'else', 'return', 'break', 'continue', 'mut', 'unsafe', 'extern', 'for', 'while'].includes(part)))
    throw new Error(`Invalid Rust module path: ${name}`);
}
export function validateInput(input: GenerationInput): void {
  const nodes = new Set(input.nodes.map(node => node.id));
  if (nodes.size !== input.nodes.length) throw new Error('Duplicate generation node identities');
  if (!input.selection.length) throw new Error('No generation roots');
  if (new Set(input.openTraitArrays).size !== (input.openTraitArrays?.length ?? 0)) throw new Error('Duplicate open trait array policies');
  for (const ref of input.openTraitArrays ?? []) if (!nodes.has(ref)) throw new Error(`Missing policy node: ${ref}`);
  const names = new Set<string>();
  const closedModules = new Set<string>();
  let activeModule: string | undefined;
  for (const root of input.selection) {
    validateModule(root.module);
    if(root.module !== activeModule) {
      if(closedModules.has(root.module)) throw new Error('Generation roots must group each module before refinements');
      if(activeModule) closedModules.add(activeModule);
      activeModule = root.module;
    }
    if (!/^[A-Z][A-Za-z0-9]*$/.test(root.name) || names.has(root.name)) throw new Error(`Invalid or duplicate root name: ${root.name}`);
    if(root.valueRef && (root.fields.length || root.family || root.deferred.length || !nodes.has(root.valueRef)))
      throw new Error(`Invalid value root: ${root.name}`);
    if ((root.documentKind || root.ruleKey) && (!root.valueRef || Boolean(root.documentKind) === Boolean(root.ruleKey)
      || root.ruleKey && !/^[A-Z][A-Za-z0-9]*$/.test(root.ruleKey))) throw new Error(`Invalid portfolio root: ${root.name}`);
    names.add(root.name);
  }
}
export function nodeReferences(node: GraphNode): string[] {
  return [...('fields' in node ? node.fields?.map(field => field.ref) ?? [] : []),
    ...('members' in node ? node.members : []), ...(node.kind === 'array' ? [node.element] : []),
    ...(node.kind === 'tuple' ? node.elements.map(element => element.ref) : []),
    ...('indexSignatures' in node ? node.indexSignatures?.map(index => index.value) ?? [] : []),
    ...(node.kind==='array' && node.serialization ? [node.serialization.rawRef] : []),
    ...('fields' in node ? node.fields?.flatMap(field=>field.serialization?.basis==='predicate-constructor-input'?[field.serialization.declaredRef]:[]) ?? [] : [])];
}
