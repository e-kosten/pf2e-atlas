import type { GraphField, GraphNode, Location, TypeGraph } from './contracts.js';

// Only persisted value edges change; declaration and serialization provenance stays original.
function mapValueReferences(node: GraphNode, map: (ref: string) => string): GraphNode {
  const result = structuredClone(node);
  if ('fields' in result) result.fields?.forEach(field => { field.ref = map(field.ref); });
  if ('members' in result) result.members = result.members.map(map);
  if (result.kind === 'array') result.element = map(result.element);
  if (result.kind === 'tuple') result.elements.forEach(element => { element.ref = map(element.ref); });
  if ('indexSignatures' in result) result.indexSignatures?.forEach(index => { index.value = map(index.value); });
  return result;
}
function declarationKey(locations: Location[]): string {
  return JSON.stringify(locations.map(location => [location.file, location.line, location.column]).sort());
}

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
  const replacements = new Map<string, Map<string, string>>();
  const sharedIwr = new Map<string, { schemaRef: string; authoredRef: string }>();
  const strictFields = new Set(graph.roots.flatMap(root => (root.arrayInputs ?? [])
    .filter(input => input.fieldClass === 'StrictArrayField').map(input => JSON.stringify([root.ref, input.field]))));
  const replace = (owner: string, field: string, ref: string) => {
    if (!replacements.has(owner)) replacements.set(owner, new Map());
    replacements.get(owner)!.set(field, ref);
  };
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
      const schemaRef = field.ref, authoredRef = `${schemaRef}#authored-string-array`;
      add({ id: authoredRef, kind: 'union', members: [...new Set([...members as string[], input.elementRef])] });
      replace(original.id, field.name, authoredRef);
      changes.push({ rule: root.ruleKey!, field: input.field, schemaRef, authoredRef, declaredAt: input.declaredAt });
      if (input.field === 'type') {
        if (!field.declaredAt.length || declarationKey(field.declaredAt) !== declarationKey(input.declaredAt))
          throw new Error(`Missing shared IWR declaration provenance: ${root.ruleKey}`);
        const key = declarationKey(field.declaredAt), previous = sharedIwr.get(key);
        if (previous && previous.schemaRef !== schemaRef) throw new Error('Shared IWR type projection drift');
        sharedIwr.set(key, previous ?? { schemaRef, authoredRef });
      }
    }
  }
  const sharedIwrChanges: { ownerRef: string; field: string; schemaRef: string; authoredRef: string; declaredAt: Location[] }[] = [];
  for (const node of schema.nodes) {
    for (const field of 'fields' in node ? node.fields ?? [] : []) {
      if (strictFields.has(JSON.stringify([node.id, field.name]))) continue;
      const policy = field.name === 'type' && sharedIwr.get(declarationKey(field.declaredAt));
      if (!policy) continue;
      if (field.ref !== policy.schemaRef) throw new Error(`Shared IWR type projection drift: ${node.id}`);
      if (!replacements.get(node.id)?.has(field.name))
        sharedIwrChanges.push({ ownerRef: node.id, field: field.name, ...policy, declaredAt: field.declaredAt });
      replace(node.id, field.name, policy.authoredRef);
    }
  }
  // Find all ancestors before copying, so recursive graphs need no provisional nodes.
  const affected = new Set(replacements.keys());
  let changed = true;
  while (changed) {
    changed = false;
    for (const node of schema.nodes) {
      if (affected.has(node.id)) continue;
      mapValueReferences(node, ref => {
        if (affected.has(ref)) { affected.add(node.id); changed = true; }
        return ref;
      });
    }
  }
  const authoredId = (id: string) => affected.has(id)
    ? `${id}#authored-input${replacements.has(id) ? ':' + [...replacements.get(id)!.keys()].sort().join(',') : ''}` : id;
  for (const node of schema.nodes.filter(node => affected.has(node.id))) {
    const copy = mapValueReferences(node, authoredId);
    copy.id = authoredId(node.id);
    if ('fields' in copy) copy.fields?.forEach((field: GraphField) => {
      field.ref = replacements.get(node.id)?.get(field.name) ?? field.ref;
    });
    add(copy);
  }
  graph.roots.forEach(root => { if (root.ref) root.ref = authoredId(root.ref); });
  return { graph, changes, sharedIwrChanges };
}
