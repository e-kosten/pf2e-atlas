import type { GraphField, Location, TypeGraph } from '../contracts.js';
import { ruleValueInputs } from './rule-value-inputs.js';

import { projectFields } from './authored-projection.js';

function declarationKey(locations: Location[]): string {
  return JSON.stringify(locations.map(location => [location.file, location.line, location.column]).sort());
}

/** Bounded authored-input projection; the extracted cleaned schema remains unchanged. */
export function authoredRuleInputs(schema: TypeGraph, openTraitArrays: string[] = []) {
  if (!schema.complete) throw new Error('Authored rule inputs require complete schema extraction');
  const graph = structuredClone(schema);
  const nodes = new Map(graph.nodes.map(node => [node.id, node]));
  const add = (node: TypeGraph['nodes'][number]) => {
    if (nodes.has(node.id)) return;
    nodes.set(node.id, node); graph.nodes.push(node);
  };
  const changes: { rule: string; field: string; schemaRef: string; authoredRef: string; declaredAt: unknown }[] = [];
  const replacements = new Map<string, Map<string, Partial<GraphField>>>();
  const sharedIwr = new Map<string, { schemaRef: string; authoredRef: string }>();
  const strictFields = new Set(graph.roots.flatMap(root => (root.arrayInputs ?? [])
    .filter(input => input.fieldClass === 'StrictArrayField').map(input => JSON.stringify([root.ref, input.field]))));
  const replace = (owner: string, field: string, patch: Partial<GraphField>) => {
    if (!replacements.has(owner)) replacements.set(owner, new Map());
    replacements.get(owner)!.set(field, patch);
  };
  for (const root of graph.roots.filter(root => root.ruleKey)) {
    if (!root.arrayInputs) throw new Error(`Re-extract schema field classes for ${root.ruleKey}`);
    const original = nodes.get(root.ref!);
    if (!original || !('fields' in original) || !original.fields) throw new Error(`Expected rule object: ${root.ruleKey}`);
    const fields = structuredClone(original.fields);
    for (const input of root.arrayInputs) {
      const selected = ['selector', 'selectors'].includes(input.field)
        || ['Immunity', 'Resistance', 'Weakness'].includes(root.ruleKey!) && input.field === 'type'
        || root.ruleKey === 'Strike' && input.field === 'traits';
      if (!selected || input.fieldClass === 'StrictArrayField') continue;
      const field = fields.find(field => field.name === input.field);
      const source = field && nodes.get(field.ref);
      const members = source?.kind === 'union' ? source.members : [field?.ref];
      const array = nodes.get(input.arrayRef), element = nodes.get(input.elementRef);
      const trait = root.ruleKey === 'Strike' && input.field === 'traits';
      const stringVocabulary = (ref: string, seen = new Set<string>()): boolean => {
        if (seen.has(ref)) return false;
        const node = nodes.get(ref), next = new Set(seen).add(ref);
        return node?.kind === 'primitive' && node.value === 'string' || node?.kind === 'literal' && typeof node.value === 'string'
          || node?.kind === 'union' && node.members.length > 0 && node.members.every(ref => stringVocabulary(ref, next));
      };
      if (!field || !members.includes(input.arrayRef) || array?.kind !== 'array' || array.element !== input.elementRef
        || !(trait ? stringVocabulary(input.elementRef) : element?.kind === 'primitive' && element.value === 'string'))
        throw new Error(`Authored string-array projection drift: ${root.ruleKey}.${input.field}`);
      const scalarRef = trait && openTraitArrays.includes(input.arrayRef) ? 'primitive:string' : input.elementRef;
      if (!nodes.has(scalarRef)) add({ id: scalarRef, kind: 'primitive', value: 'string' });
      const schemaRef = field.ref, authoredRef = `${schemaRef}#authored-string-array`;
      add({ id: authoredRef, kind: 'union', members: [...new Set([...members as string[], scalarRef])] });
      replace(original.id, field.name, { ref: authoredRef });
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
      replace(node.id, field.name, { ref: policy.authoredRef });
    }
  }
  const valueChanges = ruleValueInputs(graph, replace);
  projectFields(graph, replacements);
  return { graph, changes, sharedIwrChanges, valueChanges };
}
