import type { TypeGraph } from '../contracts.js';

/** Apply the existing open identifier policy to every Actor/Item trait array. */
export function documentTraitArrays(graph: TypeGraph): string[] {
  const nodes = new Map(graph.nodes.map(node => [node.id,node]));
  const at = (ref: string, fields: string[], seen = new Set<string>()): string[] => {
    const key = JSON.stringify([ref,fields]);
    if (seen.has(key)) return [];
    const node = nodes.get(ref); if (!node) throw new Error(`Missing trait policy node: ${ref}`);
    const next = new Set(seen).add(key);
    if (node.kind === 'union') return node.members.flatMap(member=>at(member,fields,next));
    // `ItemTraits<never>` deliberately has no trait identifiers. Keep its empty
    // array constraint rather than turning it into an open vocabulary.
    if (!fields.length) return node.kind === 'array' && nodes.get(node.element)?.id !== 'primitive:never' ? [ref] : [];
    if (node.kind !== 'object' && node.kind !== 'intersection') return [];
    const field = node.fields.find(field=>field.name === fields[0] && !field.forbidden);
    return field ? at(field.ref,fields.slice(1),next) : [];
  };
  return [...new Set(graph.roots.filter(root=>['Actor','Item'].includes(root.documentKind ?? ''))
    .flatMap(root=>at(root.ref!,['system','traits','value'])))].sort();
}
