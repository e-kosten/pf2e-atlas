import type { ExtractionSummary, TypeGraph } from '../contracts.js';
import { authoredDocumentInputs } from './document-inputs.js';
import { documentTraitArrays } from './document-traits.js';
import { nodeReferences, validateInput, type GenerationInput, type GenerationRoot } from './generation-input.js';
import { selectSourceInput } from './predicate-selection.js';

const snake = (name: string) => name.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase();

/** Select the complete authored portfolio, retaining callable component slices. */
export function selectPortfolioInput(schema: TypeGraph, summary: ExtractionSummary): GenerationInput {
  if (!schema.complete || !summary.complete || schema.source?.source_digest !== summary.source.source_digest
    || schema.typescript !== summary.typescript_version || !schema.portfolio)
    throw new Error('Portfolio generation requires complete extraction with matching source and compiler identity');
  const components = selectSourceInput(schema, summary);
  const openTraitArrays = [...new Set([...(components.openTraitArrays ?? []), ...documentTraitArrays(schema)])].sort();
  const { graph } = authoredDocumentInputs(schema, openTraitArrays);
  const nodes = new Map(graph.nodes.map(node => [node.id, node]));
  const groups = new Map<string, GenerationRoot[]>();
  const add = (root: GenerationRoot) => {
    if (!groups.has(root.module)) groups.set(root.module, []);
    groups.get(root.module)!.push(root);
  };
  components.selection.forEach(add);
  const valueRoot = (name: string, ref: string, module: string, sourceRef?: string, metadata: Partial<GenerationRoot> = {}) =>
    add({ name, declaration: sourceRef ?? ref, valueRef: ref, module, fields: [], deferred: [],
      ...(sourceRef ? { sourceRef } : {}), ...metadata });
  for (const [ref, module] of [
    ['src/module/item/base/data/system.ts#ItemSystemSource', 'items/common'],
    ['src/module/item/physical/data.ts#PhysicalSystemSource', 'physical'],
    ['src/module/actor/data/base.ts#ActorSystemSource', 'actors/common'],
    ['src/module/actor/data/base.ts#PrototypeTokenSourcePF2e', 'actors/common'],
    ['src/module/actor/creature/data.ts#CreatureSystemSource', 'actors/creature'],
    ['src/module/rules/rule-element/data.ts#RuleElementSource', 'rules/common'],
  ]) {
    const candidates = graph.nodes.filter(node => node.id === ref || node.id.startsWith(`${ref}#authored`));
    const node = candidates.at(-1);
    if (!node?.name) throw new Error(`Missing common source declaration: ${ref}`);
    valueRoot(node.name, node.id, module, ref);
  }
  // Family roots reserve their own modules even when first reached from another
  // family's embedded source. Every nested occurrence reuses that same owner.
  for (const kind of ['Item', 'Actor']) {
    const root = graph.roots.find(root => root.documentKind === kind);
    const union = root?.ref && nodes.get(root.ref);
    const original = schema.roots.find(candidate => candidate.documentKind === kind)!;
    const originalUnion = nodes.get(original.ref!);
    if (!root || !union || union.kind !== 'union') throw new Error(`Missing complete ${kind} family union`);
    const families: string[] = [];
    for (const ref of union.members) {
      const node = nodes.get(ref);
      if (!node || !('fields' in node) || !node.fields || !node.name) throw new Error(`Missing named ${kind} family: ${ref}`);
      const tag = nodes.get(node.fields.find(field => field.name === 'type')?.ref ?? '');
      if (tag?.kind !== 'literal' || typeof tag.value !== 'string') throw new Error(`Missing family discriminator: ${ref}`);
      families.push(tag.value);
      if (originalUnion?.kind !== 'union') throw new Error(`Missing schema ${kind} union`);
      const sourceRefs = originalUnion.members.filter(ref => nodes.get(ref)?.name === node.name);
      if (sourceRefs.length !== 1) throw new Error(`Ambiguous schema family: ${node.name}`);
      const module = `${kind === 'Item' ? 'items' : 'actors'}/families/${snake(tag.value)}`;
      const originalSource = nodes.get(sourceRefs[0])!;
      const systemRef = node.fields.find(field => field.name === 'system')?.ref;
      const system = systemRef ? nodes.get(systemRef) : undefined;
      const schemaSystem = 'fields' in originalSource && originalSource.fields?.find(field => field.name === 'system')?.ref;
      if (!system?.name || !schemaSystem) throw new Error(`Missing family system: ${node.name}`);
      valueRoot(system.name, system.id, module, schemaSystem);
      valueRoot(node.name, ref, module, sourceRefs[0]);
    }
    const registered = schema.portfolio.families.find(family => family.documentKind === kind)?.registered;
    if (!registered || new Set(families).size !== families.length || JSON.stringify([...families].sort()) !== JSON.stringify([...registered].sort()))
      throw new Error(`Registered ${kind} families do not match selected source families`);
    valueRoot(root.name, root.ref!, kind === 'Item' ? 'items/source' : 'actors/source', original.ref!, { documentKind: kind });
  }
  for (const root of graph.roots.filter(root => !['Actor', 'Item'].includes(root.documentKind ?? ''))) {
    const original = schema.roots.find(candidate => candidate.name === root.name && candidate.file === root.file)!;
    if (!root.ref || !original?.ref || !(root.ruleKey || root.documentKind)) throw new Error(`Unclassified portfolio root: ${root.name}`);
    valueRoot(root.ruleKey ? `${root.ruleKey}Rule` : root.name, root.ref,
      root.ruleKey ? `rules/${snake(root.ruleKey)}` : `documents/${snake(root.documentKind!)}_source`, original.ref,
      root.ruleKey ? { ruleKey: root.ruleKey } : { documentKind: root.documentKind });
  }
  if (graph.roots.length !== schema.portfolio.documentKinds.length + schema.portfolio.ruleKeys.length
    || new Set(graph.roots.map(root => root.ruleKey ?? root.documentKind)).size !== graph.roots.length
    || JSON.stringify(graph.roots.flatMap(root => root.ruleKey ? [root.ruleKey] : []).sort()) !== JSON.stringify([...schema.portfolio.ruleKeys].sort())
    || JSON.stringify(graph.roots.flatMap(root => root.documentKind ? [root.documentKind] : []).sort()) !== JSON.stringify([...schema.portfolio.documentKinds].sort()))
    throw new Error('Portfolio roots do not match registered document kinds and rule keys');
  const selection = [...groups.values()].flat();
  const visited = new Set<string>();
  const visit = (ref: string) => {
    if (visited.has(ref)) return;
    const node = nodes.get(ref); if (!node) throw new Error(`Missing portfolio node: ${ref}`);
    visited.add(ref); nodeReferences(node).forEach(visit);
  };
  selection.forEach(root => { root.fields.forEach(field => visit(field.ref)); if (root.sourceRef) visit(root.sourceRef); if (root.valueRef) visit(root.valueRef); });
  const input: GenerationInput = { source: summary.source, selection, openTraitArrays,
    portfolio: { typescript: schema.typescript, schemaRoots: schema.roots, families: schema.portfolio.families },
    nodes: [...nodes.values()].filter(node => visited.has(node.id)).sort((a, b) => a.id.localeCompare(b.id)) };
  validateInput(input);
  return input;
}
