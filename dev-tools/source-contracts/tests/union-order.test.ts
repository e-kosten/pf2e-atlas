import assert from 'node:assert/strict';
import test from 'node:test';
import type { GraphNode } from '../src/contracts.js';
import type { GenerationInput } from '../src/generation/generation-input.js';
import { generateRustModules } from '../src/generation/source-generation.js';
import { UnionOrder, assertUnionOrderRegistry, type UnionOrderRegistry } from '../src/generation/union-order.js';

const string: GraphNode = { id: 'primitive:string', kind: 'primitive', value: 'string' };
const number: GraphNode = { id: 'primitive:number', kind: 'primitive', value: 'number' };
const harm: GraphNode = { id: 'literal:string:"harm"', kind: 'literal', value: 'harm' };
const heal: GraphNode = { id: 'literal:string:"heal"', kind: 'literal', value: 'heal' };
function input(nodes: GraphNode[], root = 'root', name = 'Cases'): GenerationInput {
  return { source: { source_digest: 'fixture', system_version: null, input_file_count: 1, git_commit: null, git_clean: null },
    nodes, selection: [{ name, declaration: root, valueRef: root, module: 'cases', fields: [], deferred: [] }] };
}
function fonts(): GenerationInput {
  const tuple = (id: string, refs: string[]): GraphNode => ({ id, kind: 'tuple', readonly: false, elements: refs.map(ref => ({ ref, optional: false, rest: false })) });
  return input([string, harm, heal, { id: 'primitive:never', kind: 'primitive', value: 'never' },
    { id: 'empty', kind: 'array', element: 'primitive:never', readonly: false },
    tuple('both', [harm.id, heal.id]), tuple('harm-only', [harm.id]), tuple('heal-only', [heal.id]),
    { id: 'root', name: 'Fonts', kind: 'union', members: ['empty', 'both', 'harm-only', 'heal-only'] }], 'root', 'Fonts');
}
function capture(value: GenerationInput): UnionOrderRegistry {
  let registry: UnionOrderRegistry | undefined;
  generateRustModules(value, { recordUnionOrder: result => { registry = result; } });
  assert.ok(registry); return registry;
}
const bodies = (files: Record<string, string>) => Object.fromEntries(Object.entries(files).map(([name, value]) =>
  [name, value.split('\n').filter(line => !line.trimStart().startsWith('//')).join('\n')]));
function permute(value: GenerationInput): GenerationInput {
  const copy = structuredClone(value);
  const rename = (ref: string) => /^(primitive|literal):/.test(ref) ? ref : `new-compiler:${ref}`;
  copy.nodes.reverse();
  copy.nodes = copy.nodes.map(node => ({ ...node, id: rename(node.id),
    ...('members' in node ? { members: node.members.map(rename).reverse() } : {}),
    ...('fields' in node ? { fields: node.fields?.map(field => ({ ...field, ref: rename(field.ref) })) } : {}),
    ...('indexSignatures' in node ? { indexSignatures: node.indexSignatures?.map(index => ({ ...index, value: rename(index.value) })) } : {}),
    ...(node.kind === 'array' ? { element: rename(node.element) } : {}),
    ...(node.kind === 'tuple' ? { elements: node.elements.map(element => ({ ...element, ref: rename(element.ref) })) } : {}),
    ...(node.kind === 'template' ? { parameters: node.parameters.map(rename) } : {}),
  }) as GraphNode);
  copy.selection = copy.selection.map(root => ({ ...root, declaration: rename(root.declaration), valueRef: root.valueRef && rename(root.valueRef) }));
  return copy;
}

test('compiler IDs, discovery order and union order cannot renumber persisted tuple alternatives', () => {
  const source = fonts(), before = structuredClone(source), registry = capture(source);
  assert.deepEqual(source, before);
  const old = generateRustModules(source, { unionOrder: registry });
  assert.deepEqual(bodies(generateRustModules(permute(source), { unionOrder: registry })), bodies(old));
  assert.match(old['cases.rs'], /Alternative2\(\(/);
  assert.match(old['cases.rs'], /Alternative3\(\(/);
  assert.match(old['cases.rs'], /Alternative4\(\(/);
});

test('explicit ordinals retain gaps rather than using the current member position', () => {
  const source = fonts(), registry = capture(source);
  const entry = Object.values(registry.unions).find(entry => entry.name === 'Fonts')!;
  const arm = entry.members.find(member => member.serdeVariant === 'Alternative3')!;
  arm.ordinal = 13; arm.serdeVariant = 'Alternative13';
  const files = generateRustModules(permute(source), { unionOrder: registry });
  assert.match(files['cases.rs'], /Alternative13\(\(/);
  assert.doesNotMatch(files['cases.rs'], /Alternative3\(\(/);
});

test('missing, extra, duplicate and renamed persisted policy entries fail visibly', () => {
  for (const alter of [
    (registry: UnionOrderRegistry) => { delete registry.unions[Object.keys(registry.unions)[0]]; },
    (registry: UnionOrderRegistry) => { registry.unions['a'.repeat(64)] = { members: [] }; },
    (registry: UnionOrderRegistry) => { Object.values(registry.unions)[0].members.pop(); },
    (registry: UnionOrderRegistry) => { const arms = Object.values(registry.unions)[0].members; arms[1].ordinal = arms[0].ordinal; },
    (registry: UnionOrderRegistry) => { const arms = Object.values(registry.unions)[0].members; arms[1].key = arms[0].key; },
    (registry: UnionOrderRegistry) => { Object.values(registry.unions)[0].members[0].serdeVariant = 'Renamed'; },
  ]) {
    const source = fonts(), registry = capture(source); alter(registry);
    assert.throws(() => generateRustModules(source, { unionOrder: registry }), /union|policy/i);
  }
});

test('supplied falsy metadata cannot select unreviewed fixture mode', () => {
  for (const value of [null, false, 0, '']) {
    assert.throws(() => assertUnionOrderRegistry(value), /registry/);
    assert.throws(() => generateRustModules(fonts(), { unionOrder: value as unknown as UnionOrderRegistry }), /registry/);
    assert.throws(() => generateRustModules({ ...fonts(), unionOrder: value as unknown as UnionOrderRegistry }), /registry/);
  }
});

function collectionInput(nullable: 'none' | 'null' | 'undefined', indexed = false, open = false): GenerationInput {
  const nullish: GraphNode = { id: `primitive:${nullable}`, kind: 'primitive', value: nullable };
  const element = nullable === 'none' ? harm.id : 'maybe';
  const container: GraphNode = indexed
    ? { id: 'container', kind: 'object', fields: [], indexSignatures: [{ key: 'primitive:string', value: element, readonly: false }] }
    : { id: 'container', kind: 'array', element, readonly: false };
  const value = input([string, number, harm, container,
    ...(nullable === 'none' ? [] : [nullish, { id: 'maybe', kind: 'union', members: [harm.id, nullish.id] } as GraphNode]),
    { id: 'root', name: 'Cases', kind: 'union', members: ['container', number.id] }]);
  if (open) value.openTraitArrays = ['container'];
  return value;
}
test('collection nullability and open trait widening are part of semantic identity', () => {
  const closed = capture(collectionInput('none'));
  for (const changed of [collectionInput('null'), collectionInput('undefined'), collectionInput('none', false, true)])
    assert.throws(() => generateRustModules(changed, { unionOrder: closed }), /Unreviewed union/);
  // Undefined array positions persist as null; undefined dictionary values do not.
  assert.deepEqual(capture(collectionInput('null')), capture(collectionInput('undefined')));
  const map = capture(collectionInput('none', true));
  assert.deepEqual(bodies(generateRustModules(collectionInput('undefined', true), { unionOrder: map })),
    bodies(generateRustModules(collectionInput('none', true), { unionOrder: map })));
});

test('recursive and anonymous shapes survive graph reidentification without mutating evidence', () => {
  const field = (name: string, ref: string) => ({ name, ref, optional: false, nullable: false, undefinedAllowed: false, forbidden: false, declaredAt: [] });
  const source = input([string, number, harm,
    { id: 'children', kind: 'array', element: 'root', readonly: false },
    { id: 'node', name: 'Node', kind: 'object', fields: [field('kind', harm.id), field('children', 'children')], indexSignatures: [] },
    { id: 'root', name: 'Cases', kind: 'union', members: ['node', number.id] }]);
  const registry = capture(source);
  assert.deepEqual(bodies(generateRustModules(permute(source), { unionOrder: registry })), bodies(generateRustModules(source, { unionOrder: registry })));
});

test('printer-only generic changes cannot rename a named open variant', () => {
  const source = input([string,
    { id: 'box', name: 'Box', typeArguments: [{ expression: 'Payload', declaredAt: [] }], kind: 'open', domain: 'non-nullish' },
    { id: 'root', name: 'Cases', kind: 'union', members: ['box', string.id] }]);
  const registry = capture(source), changed = structuredClone(source);
  changed.nodes.find(node => node.id === 'box')!.typeArguments![0].expression = 'ns.Payload';
  assert.throws(() => generateRustModules(changed, { unionOrder: registry }), /Persisted union variant drift/);
});

test('canonical vocabulary unions require no duplicate policy data', () => {
  const source = input([harm, heal, { id: 'root', name: 'Cases', kind: 'union', members: [harm.id, heal.id] }]);
  assert.deepEqual(capture(source).unions, {});
  assert.deepEqual(bodies(generateRustModules(source, { unionOrder: capture(source) })), bodies(generateRustModules(source)));
});

test('cyclic fingerprint work has an enforced finite limit', () => {
  const nodes = new Map<string, GraphNode>([
    [harm.id, harm],
    ['root', { id: 'root', kind: 'union', members: ['children', harm.id] }],
    ['children', { id: 'children', kind: 'array', element: 'root', readonly: false }],
  ]);
  const state = () => ({ nullable: false, onlyNullish: false });
  assert.equal(Object.keys(new UnionOrder([...nodes.keys()], ref => nodes.get(ref)!, state, new Set()).registry().unions).length, 1);
  assert.throws(() => new UnionOrder([...nodes.keys()], ref => nodes.get(ref)!, state, new Set(), undefined, 1), /work limit exceeded/);
});
