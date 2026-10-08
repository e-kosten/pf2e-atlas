import type { GraphField, GraphNode, TypeGraph } from '../contracts.js';

/** Describe an object diff, not a complete value or an executable merge. Arrays replace. */
export function objectPatch(graph: TypeGraph, root: string): string {
  const nodes = new Map(graph.nodes.map(node => [node.id, node]));
  const memo = new Map<string, string>();
  const add = (node: GraphNode) => {
    if (nodes.has(node.id)) throw new Error(`Object patch identity collision: ${node.id}`);
    nodes.set(node.id, node); graph.nodes.push(node);
  };
  const union = (id: string, refs: string[]) => {
    const members = [...new Set(refs.flatMap(ref => {
      const node = nodes.get(ref);
      return node?.kind === 'union' ? node.members : [ref];
    }))];
    if (members.length === 1) return members[0];
    add({ id, kind: 'union', members }); return id;
  };
  const visit = (ref: string): string => {
    if (memo.has(ref)) return memo.get(ref)!;
    const node = nodes.get(ref);
    if (!node) throw new Error(`Missing object patch node: ${ref}`);
    if (node.kind === 'unsupported' || node.kind === 'unresolved') throw new Error(`Unsupported object patch node: ${ref}`);
    // Atomic values and array/tuple elements retain their complete shapes.
    if (!['object', 'intersection', 'union'].includes(node.kind)) return ref;
    const id = `${ref}#object-patch`;
    memo.set(ref, id);
    if (node.kind === 'union') {
      const members = node.members.map(ref => nodes.get(ref)!);
      if (members.some(member => !member)) throw new Error(`Missing object patch union member: ${ref}`);
      if (members.some(member => member.kind === 'intersection' && member.impossible))
        throw new Error(`Impossible object patch union member: ${ref}`);
      const objects = members.filter(member => member.kind === 'object' || member.kind === 'intersection');
      if (!objects.length) { memo.set(ref, ref); return ref; }
      // A diff can omit an arm's discriminator. One partial object admits overlapping
      // patches without pretending it identifies a complete union arm before merging.
      const objectId = objects.length === members.length ? id : `${id}:object`;
      const names = [...new Set(objects.flatMap(object => 'fields' in object ? object.fields!.map(field => field.name) : []))].sort();
      const fields: GraphField[] = names.map(name => {
        const candidates = objects.flatMap(object => 'fields' in object ? object.fields!.filter(field => field.name === name) : []);
        const allowed = candidates.filter(field => !field.forbidden);
        const base = structuredClone(allowed[0] ?? candidates[0]);
        return { ...base, optional: true, forbidden: !allowed.length,
          ref: union(`${objectId}:${name}`, (allowed.length ? allowed : candidates).map(field => visit(field.ref))),
          nullable: allowed.some(field => field.nullable), undefinedAllowed: allowed.some(field => field.undefinedAllowed),
          declaredAt: candidates.flatMap(field => field.declaredAt) };
      });
      const indexes = objects.flatMap(object => 'indexSignatures' in object ? object.indexSignatures ?? [] : []);
      if (indexes.length && new Set(indexes.map(index => index.key)).size !== 1)
        throw new Error(`Incompatible object patch index keys: ${ref}`);
      add({ id: objectId, kind: 'object', fields,
        indexSignatures: indexes.length ? [{ ...indexes[0], value: union(`${objectId}:index`, indexes.map(index => visit(index.value))) }] : [] });
      if (objectId !== id) add({ id, kind: 'union', members: [objectId, ...members.filter(member => member.kind !== 'object' && member.kind !== 'intersection').map(member => visit(member.id))] });
      return id;
    }
    if (node.kind !== 'object' && node.kind !== 'intersection') throw new Error(`Unexpected object patch: ${ref}`);
    if (node.kind === 'intersection' && node.impossible) throw new Error(`Impossible object patch: ${ref}`);
    add({ id, name: node.name ? `Patch${node.name}` : undefined, declaredAt: structuredClone(node.declaredAt), typeArguments: structuredClone(node.typeArguments),
      kind: 'object', fields: node.fields.map(field => ({ ...structuredClone(field), optional: true, ref: visit(field.ref) })),
      indexSignatures: node.indexSignatures.map(index => ({ ...index, value: visit(index.value) })) });
    return id;
  };
  return visit(root);
}
