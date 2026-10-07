import type { TypeGraph } from './contracts.js';

/** Bounded authored-input projection; the extracted cleaned schema remains unchanged. */
export function authoredRuleInputs(schema: TypeGraph) {
  if (!schema.complete) throw new Error('Authored rule inputs require complete schema extraction');
  const graph = structuredClone(schema);
  const nodes = new Map(graph.nodes.map(node => [node.id, node]));
  const add = (node: TypeGraph['nodes'][number]) => {
    if (nodes.has(node.id)) return;
    nodes.set(node.id, node); graph.nodes.push(node);
  };
  const changes: { rule: string; field: string; schemaRef: string; authoredRef: string; declaredAt: unknown }[] = [];
  for (const root of graph.roots.filter(root => root.ruleKey)) {
    if (!root.arrayInputs) throw new Error(`Re-extract schema field classes for ${root.ruleKey}`);
    const original = nodes.get(root.ref!);
    if (!original || !('fields' in original) || !original.fields) throw new Error(`Expected rule object: ${root.ruleKey}`);
    const fields = structuredClone(original.fields);
    for (const input of root.arrayInputs) {
      const selected = ['selector', 'selectors'].includes(input.field)
        || ['Immunity', 'Resistance', 'Weakness'].includes(root.ruleKey!) && input.field === 'type';
      if (!selected || input.fieldClass === 'StrictArrayField') continue;
      const field = fields.find(field => field.name === input.field);
      const source = field && nodes.get(field.ref);
      const members = source?.kind === 'union' ? source.members : [field?.ref];
      const array = nodes.get(input.arrayRef), element = nodes.get(input.elementRef);
      if (!field || !members.includes(input.arrayRef) || array?.kind !== 'array' || array.element !== input.elementRef
        || element?.kind !== 'primitive' || element.value !== 'string')
        throw new Error(`Authored string-array projection drift: ${root.ruleKey}.${input.field}`);
      const schemaRef = field.ref, authoredRef = `${original.id}#authored:${input.field}`;
      add({ id: authoredRef, kind: 'union', members: [...new Set([...members as string[], input.elementRef])] });
      field.ref = authoredRef;
      changes.push({ rule: root.ruleKey!, field: input.field, schemaRef, authoredRef, declaredAt: input.declaredAt });
    }
    const selected = fields.filter((field, index) => field.ref !== original.fields![index].ref);
    if (selected.length) {
      root.ref = `${original.id}#authored-input:${selected.map(field => field.name).join(',')}`;
      add({ ...original, id: root.ref, fields });
    }
  }
  return { graph, changes };
}
