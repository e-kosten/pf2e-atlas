import type { ExtractionSummary, TypeGraph } from '../contracts.js';
import { nodeReferences, type GenerationInput } from './generation-input.js';

/** The trial's four-field selection; emitter and module layout are family-neutral. */
export function selectEquipmentInput(graph: TypeGraph, summary: ExtractionSummary): GenerationInput {
  if (!graph.complete || !summary.complete) throw new Error('Complete declaration extraction is required');
  const nodes = new Map(graph.nodes.map(node => [node.id, node]));
  if (nodes.size !== graph.nodes.length) throw new Error('Duplicate graph node identities');
  const visited = new Set<string>();
  const visit = (id: string) => {
    if (visited.has(id)) return;
    const node = nodes.get(id);
    if (!node) throw new Error(`Missing selected node: ${id}`);
    visited.add(id);
    nodeReferences(node).forEach(visit);
  };
  const selected = ['equipped', 'hp', 'price', 'usage'];
  const selection = [
    { declaration: 'src/module/item/physical/data.ts#PhysicalSystemSource', name: 'PhysicalEquipmentFields', module: 'physical' },
    { declaration: 'src/module/item/equipment/data.ts#EquipmentSystemSource', name: 'EquipmentFields', module: 'items/equipment' },
  ].map(root => {
    const node = nodes.get(root.declaration);
    if (!node || node.kind !== 'object') throw new Error(`Missing object declaration: ${root.declaration}`);
    const fields = selected.map(name => {
      const field = node.fields.find(field => field.name === name);
      if (!field) throw new Error(`Missing selected field: ${root.declaration}.${name}`);
      visit(field.ref);
      return field;
    });
    return { ...root, fields, deferred: node.fields.filter(field => !selected.includes(field.name)).map(field => field.name) };
  });
  return { source: summary.source, selection, nodes: graph.nodes.filter(node => visited.has(node.id)).sort((a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : 0) };
}
