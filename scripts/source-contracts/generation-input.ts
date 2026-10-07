import { readFile } from 'node:fs/promises';
import path from 'node:path';
import type { GraphField, GraphNode, SourceIdentity } from './contracts.js';

export interface GenerationRoot {
  name: string; declaration: string; fields: GraphField[]; deferred: string[]; module: string; family?: string; sourceRef?: string; valueRef?: string;
}
export interface GenerationInput {
  source: SourceIdentity; selection: GenerationRoot[]; nodes: GraphNode[]; openTraitArrays?: string[];
}
export interface GenerationManifest {
  format: 'atlas-source-generation/v1'; source: SourceIdentity;
  modules: { name: string; file: string }[]; openTraitArrays?: string[];
}
interface ModuleSnapshot {
  format: 'atlas-source-generation-module/v1';
  roots: Omit<GenerationRoot, 'module'>[]; nodes: GraphNode[];
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

/** Resolve the whole selection, then partition nodes once in base-before-refinement root order. */
export function snapshotFiles(input: GenerationInput): Record<string, string> {
  validateInput(input);
  const nodes = new Map(input.nodes.map(node => [node.id, node]));
  const owners = new Map<string, string>();
  const modules = new Map<string, ModuleSnapshot>();
  for (const { module, ...root } of input.selection) {
    if (!modules.has(module)) modules.set(module, { format: 'atlas-source-generation-module/v1', roots: [], nodes: [] });
    modules.get(module)!.roots.push({ name: root.name, declaration: root.declaration, fields: root.fields, deferred: root.deferred,
      ...(root.family ? { family: root.family } : {}), ...(root.sourceRef ? {sourceRef: root.sourceRef} : {}), ...(root.valueRef ? {valueRef: root.valueRef} : {}) });
    const visit = (id: string) => {
      if (owners.has(id)) return;
      const node = nodes.get(id);
      if (!node) throw new Error(`Missing generation node: ${id}`);
      owners.set(id, module);
      nodeReferences(node).forEach(visit);
    };
    root.fields.forEach(field => visit(field.ref));
    if(root.sourceRef) visit(root.sourceRef);
    if(root.valueRef) visit(root.valueRef);
  }
  if (owners.size !== nodes.size) throw new Error('Unreachable nodes in generation input');
  for (const node of [...input.nodes].sort((a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : 0))
    modules.get(owners.get(node.id)!)!.nodes.push(node);
  const manifest: GenerationManifest = { format: 'atlas-source-generation/v1', source: input.source,
    modules: [...modules.keys()].map(name => ({ name, file: `${name}.json` })),
    ...(input.openTraitArrays?.length ? { openTraitArrays: input.openTraitArrays } : {}) };
  const json = (value: unknown) => JSON.stringify(value, null, 2) + '\n';
  return { 'manifest.json': json(manifest), ...Object.fromEntries([...modules].map(([name, value]) => [`${name}.json`, json(value)])) };
}

export async function loadGenerationInput(manifestFile: string): Promise<GenerationInput> {
  const manifest = JSON.parse(await readFile(manifestFile, 'utf8')) as GenerationManifest;
  if (manifest.format !== 'atlas-source-generation/v1' || !manifest.modules.length) throw new Error('Invalid generation manifest');
  const input: GenerationInput = { source: manifest.source, selection: [], nodes: [],
    ...(manifest.openTraitArrays?.length ? { openTraitArrays: manifest.openTraitArrays } : {}) };
  const names = new Set<string>();
  for (const module of manifest.modules) {
    validateModule(module.name);
    if (names.has(module.name) || module.file !== `${module.name}.json`) throw new Error(`Invalid or duplicate snapshot module: ${module.name}`);
    names.add(module.name);
    const snapshot = JSON.parse(await readFile(path.join(path.dirname(manifestFile), module.file), 'utf8')) as ModuleSnapshot;
    if (snapshot.format !== 'atlas-source-generation-module/v1') throw new Error(`Invalid module snapshot: ${module.file}`);
    input.selection.push(...snapshot.roots.map(root => ({ ...root, module: module.name })));
    input.nodes.push(...snapshot.nodes);
  }
  input.nodes.sort((a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : 0);
  validateInput(input);
  return input;
}
