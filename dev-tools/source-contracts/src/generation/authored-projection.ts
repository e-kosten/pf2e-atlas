import type { GraphField, GraphNode, TypeGraph } from '../contracts.js';

export type FieldReplacements = Map<string, Map<string, Partial<GraphField>>>;

/** Rewrite value edges without changing declaration/serialization provenance. */
export function mapValueReferences(node: GraphNode, map: (ref: string) => string): GraphNode {
  const result = structuredClone(node);
  if ('fields' in result) result.fields?.forEach(field => { field.ref = map(field.ref); });
  if ('members' in result) result.members = result.members.map(map);
  if (result.kind === 'array') result.element = map(result.element);
  if (result.kind === 'tuple') result.elements.forEach(element => { element.ref = map(element.ref); });
  if ('indexSignatures' in result) result.indexSignatures?.forEach(index => { index.value = map(index.value); });
  return result;
}

/** Copy affected ancestors, including cycles; retain the original declaration graph. */
export function projectFields(graph: TypeGraph, replacements: FieldReplacements, suffix = 'authored-input') {
  const originals = [...graph.nodes];
  const affected = new Set(replacements.keys());
  let changed = true;
  while (changed) {
    changed = false;
    for (const node of originals) {
      if (affected.has(node.id)) continue;
      mapValueReferences(node, ref => {
        if (affected.has(ref)) { affected.add(node.id); changed = true; }
        return ref;
      });
    }
  }
  const authoredId = (id: string) => affected.has(id)
    ? `${id}#${suffix}${replacements.has(id) ? ':' + [...replacements.get(id)!.keys()].sort().join(',') : ''}` : id;
  const ids = new Set(originals.map(node => node.id));
  for (const node of originals.filter(node => affected.has(node.id))) {
    const copy = mapValueReferences(node, authoredId);
    copy.id = authoredId(node.id);
    if (ids.has(copy.id)) throw new Error(`Authored projection identity collision: ${copy.id}`);
    if ('fields' in copy) copy.fields?.forEach(field => { Object.assign(field, replacements.get(node.id)?.get(field.name)); });
    graph.nodes.push(copy);
  }
  graph.roots.forEach(root => { if (root.ref) root.ref = authoredId(root.ref); });
  return authoredId;
}
