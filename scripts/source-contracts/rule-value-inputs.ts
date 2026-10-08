import type { GraphField, GraphNode, TypeGraph } from './contracts.js';
import { nodeReferences } from './generation-input.js';

/** Explicit source/prepared differences supported by the pinned rule implementations. */
export function ruleValueInputs(graph: TypeGraph, replace: (owner: string, field: string, patch: Partial<Pick<GraphField, 'ref' | 'optional'>>) => void) {
  const nodes = new Map(graph.nodes.map(node => [node.id, node]));
  const changes: { rule: string; ownerRef: string; field: string; schemaRef: string; authoredRef: string; optional: boolean; declaredAt: GraphField['declaredAt'] }[] = [];
  const apply = (rule: string, name: string, file: string, fieldNames: string[], patch: (field: GraphField) => Partial<Pick<GraphField, 'ref' | 'optional'>>) => {
    const root = graph.roots.find(root => root.ruleKey === rule);
    if (!root) return;
    const reachable = new Set<string>();
    const visit = (ref: string) => {
      if (reachable.has(ref)) return;
      const node = nodes.get(ref);
      if (!node) throw new Error(`Missing rule value node: ${ref}`);
      reachable.add(ref); nodeReferences(node).forEach(visit);
    };
    visit(root.ref!);
    const owners = [...reachable].map(ref => nodes.get(ref)!).filter(node => node.name === name);
    if (owners.length !== 1 || !('fields' in owners[0]) || !owners[0].fields)
      throw new Error(`Authored rule value declaration drift: ${rule}.${name}`);
    const owner = owners[0] as GraphNode & { fields: GraphField[] };
    for (const name of fieldNames) {
      const field = owner.fields.find(field => field.name === name);
      if (!field || field.forbidden || !field.declaredAt.length || field.declaredAt.some(location => location.file !== file))
        throw new Error(`Authored rule value field drift: ${rule}.${owner.name}.${name}`);
      const value = patch(field);
      replace(owner.id, name, value);
      changes.push({ rule, ownerRef: owner.id, field: name, schemaRef: field.ref, authoredRef: value.ref ?? field.ref,
        optional: value.optional ?? field.optional, declaredAt: field.declaredAt });
    }
  };
  for (const name of ['ChoiceSetOwnedItems', 'ChoiceSetAttacks', 'ChoiceSetConfig'])
    apply('ChoiceSet', name, 'src/module/rules/rule-element/choice-set/data.ts', ['predicate'], field => {
      const node = nodes.get(field.ref);
      if (field.optional || node?.kind !== 'array' || node.name !== 'RawPredicate')
        throw new Error(`ChoiceSet predicate declaration drift: ${name}`);
      return { optional: true };
    });
  apply('DamageDice', 'DamageDiceOverride', 'src/module/actor/modifiers.ts', ['damageType', 'dieSize', 'diceNumber'], field => {
    const node = nodes.get(field.ref);
    const members = node?.kind === 'union' ? node.members.map(ref => nodes.get(ref)) : [node];
    const domain = field.name === 'diceNumber' ? 'number' : 'string';
    if (!field.optional || !members.some(node => node?.kind === 'primitive' && node.value === domain
      || domain === 'string' && node?.kind === 'literal' && typeof node.value === 'string')
      || members.some(node => !node || !(node.kind === 'primitive' && [domain, 'undefined'].includes(node.value)
      || domain === 'string' && node.kind === 'literal' && typeof node.value === 'string')))
      throw new Error(`DamageDice override declaration drift: ${field.name}`);
    if (domain === 'string') return { ref: 'primitive:string' };
    const id = `${field.ref}#authored-expression`;
    graph.nodes.push({ id, kind: 'union', members: ['primitive:string', 'primitive:number'] });
    return { ref: id };
  });
  apply('BattleForm', 'BattleFormStrike', 'src/module/rules/rule-element/battle-form/types.ts', ['baseType'], field => {
    const node = nodes.get(field.ref);
    const members = node?.kind === 'union' ? node.members.map(ref => nodes.get(ref)) : [node];
    if (!field.optional || !field.nullable || !members.some(node => node?.kind === 'literal' && typeof node.value === 'string')
      || members.some(node => !node || !(node.kind === 'literal' && typeof node.value === 'string'
      || node.kind === 'primitive' && ['null', 'undefined'].includes(node.value))))
      throw new Error('BattleForm strike baseType declaration drift');
    // The pinned preparation maps this authored value to Strike.baseItem, not its closed baseType field.
    return { ref: 'primitive:string' };
  });
  return changes;
}
