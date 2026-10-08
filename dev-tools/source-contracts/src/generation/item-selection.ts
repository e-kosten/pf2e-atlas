import type { ExtractionSummary, GraphField, GraphNode, TypeGraph } from '../contracts.js';
import { selectEquipmentInput } from './equipment-selection.js';
import { nodeReferences, type GenerationInput, type GenerationRoot } from './generation-input.js';

const pascal = (name: string) => name[0].toUpperCase() + name.slice(1);
/** Select shared Item components across the complete registered family set. */
export function selectItemInput(graph: TypeGraph, summary: ExtractionSummary): GenerationInput {
  const equipment = selectEquipmentInput(graph, summary);
  const nodes = new Map(graph.nodes.map(node => [node.id, node]));
  const lookup = (ref: string): GraphNode => {
    const node = nodes.get(ref); if (!node) throw new Error(`Missing selected node: ${ref}`); return node;
  };
  const object = (ref: string): GraphNode & { fields: GraphField[] } => {
    const node = lookup(ref);
    if (node.kind === 'union') {
      const viable = node.members.filter(ref => { const member = lookup(ref); return !(member.kind === 'intersection' && member.impossible)
        && !(member.kind === 'primitive' && ['null', 'undefined', 'never'].includes(member.value)); });
      if (viable.length === 1) return object(viable[0]);
    }
    if (node.kind !== 'object' && node.kind !== 'intersection') throw new Error(`Selected object has unresolved alternatives: ${ref}`);
    if (node.kind === 'intersection' && node.impossible) throw new Error(`Impossible selected object: ${ref}`);
    return node;
  };
  const field = (node: GraphNode & { fields: GraphField[] }, name: string) => {
    const found = node.fields.find(field => field.name === name);
    if (!found) throw new Error(`Missing selected field: ${node.id}.${name}`); return found;
  };
  const selection: GenerationRoot[] = [];
  const add = (ref: string, name: string, fields?: string[], family?: string) => {
    const node = object(ref); const selected = fields ? fields.map(name => field(node, name)) : node.fields;
    selection.push({ declaration: node.id, name, module: family ? 'items/traits' : 'items/common', fields: selected,
      deferred: node.fields.filter(field => !selected.includes(field)).map(field => field.name), ...(family ? { family } : {}),
      ...(node.id !== ref ? {sourceRef:ref} : {}) });
  };
  add('src/module/item/base/data/system.ts#ItemDescriptionSource', 'ItemDescriptionSource');
  add('src/module/data.ts#PublicationData', 'PublicationData');
  const flags = object('src/module/item/base/data/system.ts#ItemSourceFlagsPF2e');
  selection.push({declaration:flags.id,name:'ItemSourceFlagsPF2e',valueRef:flags.id,
    module:'items/flags',fields:[],deferred:[]});
  const aggregate = lookup(graph.roots.find(root => root.documentKind === 'Item')?.ref ?? '');
  if (aggregate.kind !== 'union') throw new Error('Expected complete Item family union');
  const openTraitArrays = new Set<string>();
  for (const ref of aggregate.members) {
    const source = object(ref); const discriminator = lookup(field(source, 'type').ref);
    if (discriminator.kind !== 'literal' || typeof discriminator.value !== 'string') throw new Error(`Invalid Item discriminator: ${ref}`);
    const family = discriminator.value;
    const system = object(field(source, 'system').ref);
    const traitRef = field(system, 'traits').ref;
    const traits = object(traitRef);
    add(traitRef, `${pascal(family)}TraitsFields`, traits.fields.filter(field => ['value', 'rarity', 'otherTags'].includes(field.name)).map(field => field.name), family);
    const value = traits.fields.find(field => field.name === 'value' && !field.forbidden);
    if (value) { const array = lookup(value.ref); if (array.kind !== 'array') throw new Error(`Expected trait array: ${array.id}`);
      const element = lookup(array.element);
      if (!(element.kind === 'primitive' && element.value === 'never')) openTraitArrays.add(array.id);
    }
  }
  selection.push(...equipment.selection);
  const visited = new Set<string>();
  const visit = (ref: string) => { if (visited.has(ref)) return; visited.add(ref); nodeReferences(lookup(ref)).forEach(visit); };
  selection.forEach(root => {root.fields.forEach(field => visit(field.ref)); if(root.sourceRef) visit(root.sourceRef); if(root.valueRef) visit(root.valueRef);});
  return { source: summary.source, selection, nodes: graph.nodes.filter(node => visited.has(node.id)).sort((a,b) => a.id.localeCompare(b.id)),
    openTraitArrays: [...openTraitArrays].sort() };
}
