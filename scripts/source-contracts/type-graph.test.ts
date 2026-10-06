import type { GraphField, GraphNode, TypeGraph } from './contracts.js';
import assert from 'node:assert/strict';
import { cp, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { extractTypeGraph } from './type-graph.js';

const fixture = fileURLToPath(new URL('../fixtures/type-graph/', import.meta.url));
const roots = ['Equipment', 'Backpack', 'Family', 'Pair', 'Formula', 'NumericFormula', 'Instantiations',
  'AnonymousFamily', 'SameNamedArguments', 'SameNamedNullable'].map((name) => ({ file: 'models.ts', name }));
const graph = () => extractTypeGraph(fixture, { roots });
const lookup = (result: TypeGraph, id: string | null) => { const node = result.nodes.find((node) => node.id === id); assert.ok(node); return node; };
const rootNode = (result: TypeGraph, name: string) => { const root = result.roots.find((entry) => entry.name === name); assert.ok(root); return lookup(result, root.ref); };
const field = (node: GraphNode, name: string) => { const found = fieldsOf(node).find((entry) => entry.name === name); assert.ok(found); return found; };

test('default portfolio follows pack kinds, complete family unions and registered schema sources', () => {
  const result = extractTypeGraph(fixture);
  assert.equal(result.complete, true, JSON.stringify(result.diagnostics));
  assert.deepEqual(result.portfolio!.documentKinds, ['Actor', 'Item', 'JournalEntry', 'Macro', 'RollTable']);
  assert.equal(result.roots.length, 7);
  for (const family of result.portfolio!.families) assert.deepEqual(family.discovered, family.registered);
  assert.deepEqual(result.portfolio!.ruleKeys, ['Example', 'Inherited']);
  for (const root of result.roots.filter((entry) => entry.ruleKey)) {
    const node = lookup(result, root.ref);
    assert.deepEqual(fieldsOf(node).map((field) => field.name), ['amount', 'choices']);
    assert.equal(valueOf(lookup(result, field(node, 'amount').ref)), 'number');
    assert.equal(lookup(result, field(node, 'choices').ref).kind, 'array');
  }
  assert.ok(!result.nodes.some((node) => 'fields' in node && node.fields?.some((field) => field.name === 'preparedOnly')));
});

test('portfolio changes reveal new families, document kinds, rules and unsupported registry syntax', async () => {
  const temporary = await mkdtemp(path.join(os.tmpdir(), 'atlas-portfolio-'));
  try {
    await cp(fixture, temporary, { recursive: true });
    const configFile = path.join(temporary, 'src/scripts/config/index.ts');
    const config = await readFile(configFile, 'utf8');
    // Same count, different family: counts alone cannot establish coverage.
    await writeFile(configFile, config.replace('spell: Object', 'newFamily: Object'));
    const mismatch = extractTypeGraph(temporary);
    assert.ok(mismatch.diagnostics.some((entry) => entry.code === 'family-coverage'));
    assert.equal(mismatch.complete, false);
    await writeFile(configFile, config);
    const manifestFile = path.join(temporary, 'static/system.json');
    const manifest = JSON.parse(await readFile(manifestFile, 'utf8'));
    manifest.packs.push({ type: 'NewDocument' });
    await writeFile(manifestFile, JSON.stringify(manifest));
    const missing = extractTypeGraph(temporary);
    assert.ok(missing.roots.some((entry) => entry.documentKind === 'NewDocument' && entry.ref === null));
    assert.ok(missing.diagnostics.some((entry) => entry.code === 'missing-root'));
    const documentsFile = path.join(temporary, 'types/foundry/common/documents/module.d.ts');
    await writeFile(documentsFile, `${await readFile(documentsFile, 'utf8')}\nexport interface NewDocumentSource { value: string }`);
    assert.equal(extractTypeGraph(temporary).complete, true);
    const rulesFile = path.join(temporary, 'src/module/rules/index.ts');
    const rules = await readFile(rulesFile, 'utf8');
    await writeFile(rulesFile, rules.replace('Example: GenericRule', 'Added: OtherRule, Example: GenericRule'));
    const added = extractTypeGraph(temporary);
    assert.equal(added.complete, true);
    assert.deepEqual(added.portfolio!.ruleKeys, ['Added', 'Example', 'Inherited']);
    assert.ok(added.roots.find((entry) => entry.ruleKey === 'Added')!.ref);
    const original = extractTypeGraph(fixture);
    assert.equal(added.roots.find((entry) => entry.ruleKey === 'Example')!.ref,
      original.roots.find((entry) => entry.ruleKey === 'Example')!.ref);
    await writeFile(rulesFile, rules.replace('Example: GenericRule', '...{}, Example: missingConstructor'));
    const unsupported = extractTypeGraph(temporary);
    assert.equal(unsupported.complete, false);
    assert.ok(unsupported.diagnostics.some((entry) => entry.code === 'source-portfolio' && entry.message.includes('Unsupported registry entry')));
    assert.ok(unsupported.diagnostics.some((entry) => entry.message.includes('Cannot resolve registered rule constructor Example')));
  } finally { await rm(temporary, { recursive: true, force: true }); }
});

test('missing schema methods and invalid portfolio inputs cannot report a complete graph', async () => {
  const temporary = await mkdtemp(path.join(os.tmpdir(), 'atlas-portfolio-errors-'));
  try {
    await cp(fixture, temporary, { recursive: true });
    const exampleFile = path.join(temporary, 'src/module/rules/example.ts');
    const example = await readFile(exampleFile, 'utf8');
    await writeFile(exampleFile, example.replace('defineSchema()', 'otherMethod()'));
    const missingMethod = extractTypeGraph(temporary);
    assert.equal(missingMethod.complete, false);
    assert.ok(missingMethod.projectDiagnostics.selected.some((entry) => entry.code === 2339));
    await writeFile(exampleFile, example);
    const configFile = path.join(temporary, 'src/scripts/config/index.ts');
    const config = await readFile(configFile, 'utf8');
    for (const changed of [
      config.replace('unrelated,', 'unrelated, Actor: { documentClasses: { changed: Object } },'),
      config.replace('npc: Object, character: Object }', 'npc: Object, character: Object }, documentClasses: { changed: Object }'),
    ]) {
      await writeFile(configFile, changed);
      const duplicate = extractTypeGraph(temporary);
      assert.equal(duplicate.complete, false);
      assert.ok(duplicate.diagnostics.some((entry) => entry.message.includes('Duplicate property')));
    }
    await writeFile(configFile, config);
    const rulesFile = path.join(temporary, 'src/module/rules/index.ts');
    const rules = await readFile(rulesFile, 'utf8');
    for (const changed of [
      rules.replace('static readonly builtin', 'static readonly builtin = { Inherited: OtherRule };\n  static readonly builtin'),
      `${rules}\nclass RuleElements { static builtin = {} }`,
      rules.replace('static readonly builtin', 'readonly builtin'),
      `${rules}\nRuleElements.builtin.Added = OtherRule;`,
      `${rules}\nObject.assign(RuleElements.builtin, { Added: OtherRule });`,
    ]) {
      await writeFile(rulesFile, changed);
      assert.equal(extractTypeGraph(temporary).complete, false);
    }
    await writeFile(rulesFile, rules);
    await writeFile(configFile, `${config}\nconst PF2ECONFIG = {};`);
    assert.equal(extractTypeGraph(temporary).complete, false);
    await writeFile(configFile, `${config}\nPF2ECONFIG.Actor.documentClasses.npc = Object;`);
    assert.equal(extractTypeGraph(temporary).complete, false);
    await writeFile(configFile, config);
    await writeFile(path.join(temporary, 'static/system.json'), '{"packs":[]}');
    const empty = extractTypeGraph(temporary);
    assert.equal(empty.complete, false);
    assert.ok(empty.diagnostics.some((entry) => entry.message.includes('packs must not be empty')));
    await writeFile(path.join(temporary, 'static/system.json'), '{invalid json');
    assert.equal(extractTypeGraph(temporary).complete, false);
  } finally { await rm(temporary, { recursive: true, force: true }); }
});

test('resolves configured imports, inherited/shared references, refinements and source presence', () => {
  const result = graph();
  assert.equal(result.status, 'complete');
  const equipment = rootNode(result, 'Equipment');
  const backpack = rootNode(result, 'Backpack');
  assert.equal(field(equipment, 'level').ref, field(backpack, 'level').ref);
  assert.equal(lookup(result, field(equipment, 'level').ref).name, 'NumericValue');
  assert.equal(field(equipment, 'contents').ref, field(backpack, 'contents').ref);
  assert.equal(valueOf(lookup(result, field(equipment, 'type').ref)), 'equipment');
  assert.equal(valueOf(lookup(result, field(backpack, 'type').ref)), 'backpack');
  const usage = field(equipment, 'usage');
  assert.equal(usage.optional, true);
  assert.equal(usage.nullable, true);
  assert.equal(usage.undefinedAllowed, true);
  assert.equal(field(backpack, 'subitems').forbidden, true);
  assert.ok(basesOf(equipment).some((ref) => lookup(result, ref).name === 'Common'));
  assert.deepEqual(membersOf(rootNode(result, 'Family')), [equipment.id, backpack.id].sort());
  assert.equal(domainOf(lookup(result, field(equipment, 'explicit').ref)), 'any');
  assert.equal(domainOf(lookup(result, field(equipment, 'legacy').ref)), 'object');
  const flags = lookup(result, field(equipment, 'flags').ref);
  assert.equal(domainOf(lookup(result, indicesOf(flags)[0].value)), 'unknown');
});

test('retains instantiated recursive references and tuple/mapped fields', () => {
  const result = graph();
  const array = lookup(result, field(rootNode(result, 'Equipment'), 'contents').ref);
  const nested = lookup(result, elementOf(array));
  const recursion = lookup(result, field(nested, 'children').ref);
  assert.equal(elementOf(recursion), nested.id);
  assert.equal(lookup(result, field(nested, 'payload').ref).name, 'NumericValue');
  assert.equal(elementsOf(rootNode(result, 'Pair'))[1].optional, true);
  assert.equal(readonlyOf(rootNode(result, 'Pair')), true);
  assert.deepEqual(fieldsOf(rootNode(result, 'Formula')).map((entry) => [entry.name, entry.optional]),
    [['a', true], ['b', true]]);
  const numeric = rootNode(result, 'NumericFormula');
  assert.deepEqual(fieldsOf(numeric).map((entry) => entry.name), ['1', '2']);
  const numericValue = lookup(result, field(numeric, '1').ref);
  assert.equal(numericValue.kind, 'union');
  assert.ok(membersOf(numericValue).some((ref) => lookup(result, ref).name === 'NumericValue'));
});

test('marks unresolved and unsupported selected types explicitly, separating unrelated project errors', () => {
  const result = extractTypeGraph(fixture, { roots: [{ file: 'problems.ts', name: 'Broken' },
    { file: 'problems.ts', name: 'Callable' }] });
  assert.equal(result.status, 'incomplete');
  assert.ok(result.diagnostics.some((entry) => entry.code === 'unresolved-type'));
  assert.ok(result.diagnostics.some((entry) => entry.code === 'callable-source-type'));
  assert.ok(result.projectDiagnostics.selected.some((entry) => entry.code === 2304));
  assert.ok(result.projectDiagnostics.unrelated.some((entry) => entry.code === 2322));
  assert.equal(lookup(result, field(rootNode(result, 'Broken'), 'field').ref).kind, 'unresolved');
  const indirect = extractTypeGraph(fixture, { roots: [{ file: 'problems.ts', name: 'IndirectBroken' }] });
  assert.equal(indirect.complete, false);
  assert.ok(indirect.diagnostics.some((entry) => entry.code === 'unresolved-type'));
  const mixed = extractTypeGraph(fixture, { roots: [...roots, { file: 'problems.ts', name: 'IndirectBroken' }] });
  assert.equal(mixed.complete, false);
  assert.equal(domainOf(lookup(mixed, field(rootNode(mixed, 'Equipment'), 'explicit').ref)), 'any');
  assert.equal(lookup(mixed, field(rootNode(mixed, 'IndirectBroken'), 'field').ref).kind, 'unresolved');
});

test('missing root and closure bounds are visible failures', () => {
  assert.equal(extractTypeGraph(fixture, { roots: [{ file: 'missing.ts', name: 'Missing' }] }).status, 'incomplete');
  const limited = extractTypeGraph(fixture, { roots, maxNodes: 2 });
  assert.equal(limited.status, 'incomplete');
  assert.ok(limited.diagnostics.some((entry) => entry.code === 'closure-limit'));
  assert.ok(limited.nodes.some((entry) => entry.id === 'unsupported:closure-limit'));
});

test('serialized Predicate data and modifier callbacks retain explicit source projections', () => {
  const result = extractTypeGraph(fixture, { roots: [{ file: 'serialization.ts', name: 'SerializedSource' }] });
  assert.equal(result.complete, true, JSON.stringify(result.diagnostics));
  const source = rootNode(result, 'SerializedSource');
  const predicate = lookup(result, field(source, 'predicate').ref);
  assert.equal(predicate.name, 'Predicate');
  assert.equal(predicate.kind, 'array');
  assert.equal(arraySerializationOf(predicate).rawRef, field(source, 'rawPredicate').ref);
  assert.equal(elementOf(predicate), elementOf(lookup(result, arraySerializationOf(predicate).rawRef)));
  assert.equal(arraySerializationOf(predicate).method, 'toObject');
  assert.ok(arraySerializationOf(predicate).declaredAt.some((entry) => entry.file.endsWith('predication.ts')));
  const choiceArray = membersOf(lookup(result, field(source, 'choices').ref)).map((id) => lookup(result, id))
    .find((node) => node.kind === 'array');
  const choicePredicate = field(lookup(result, elementOf(choiceArray)), 'predicate');
  assert.equal(choicePredicate.optional, true);
  assert.equal(choiceSerializationOf(choicePredicate).basis, 'predicate-constructor-input');
  assert.equal(choiceSerializationOf(choicePredicate).declaredRef, predicate.id);
  const input = lookup(result, choicePredicate.ref);
  assert.equal(input.kind, 'union');
  const inputNodes = membersOf(input).map((id) => lookup(result, id));
  assert.ok(inputNodes.some((node) => node.kind === 'array' && elementOf(node) === elementOf(predicate)));
  assert.ok(membersOf(input).includes(elementOf(predicate)));
  const adjustments = lookup(result, field(source, 'adjustments').ref);
  const array = membersOf(adjustments).map((id) => lookup(result, id)).find((node) => node.kind === 'array');
  const adjustment = lookup(result, elementOf(array));
  for (const name of ['test', 'getNewValue', 'getDamageType']) {
    const callback = field(adjustment, name);
    assert.equal(callback.forbidden, true);
    assert.equal(callback.optional, true);
    assert.equal(callback.nullable, false);
    assert.equal(valueOf(lookup(result, callback.ref)), 'never');
    assert.equal(callbackSerializationOf(callback).basis, 'omitted-function-property');
    assert.equal(callbackSerializationOf(callback).declaredOptional, name !== 'test');
    assert.ok(callbackSerializationOf(callback).declaredType.includes('=>'));
  }
  assert.equal(field(adjustment, 'slug').optional, false);
  assert.equal(field(adjustment, 'slug').nullable, true);
  assert.equal(field(adjustment, 'suppress').optional, true);
  assert.deepEqual(JSON.parse(JSON.stringify({ slug: 'example', test: () => true,
    getNewValue: (value: number) => value + 1, getDamageType: () => 'fire', suppress: false })),
    { slug: 'example', suppress: false });
  const unsupported = extractTypeGraph(fixture, { roots: [{ file: 'serialization.ts', name: 'OtherSource' }] });
  assert.equal(unsupported.complete, false);
  assert.ok(unsupported.diagnostics.some((entry) => entry.code === 'class-instance-source-type'));
});

test('serialization projections reject upstream type drift and new callable properties', async () => {
  const temporary = await mkdtemp(path.join(os.tmpdir(), 'atlas-serialization-'));
  const selections = { roots: [{ file: 'serialization.ts', name: 'SerializedSource' }] };
  try {
    await cp(fixture, temporary, { recursive: true });
    const modifierFile = path.join(temporary, 'src/module/actor/modifiers.ts');
    const modifier = await readFile(modifierFile, 'utf8');
    await writeFile(modifierFile, modifier.replace('test: (options: string[]) => boolean', 'test: boolean'));
    const changedCallback = extractTypeGraph(temporary, selections);
    assert.equal(changedCallback.complete, false);
    assert.ok(changedCallback.diagnostics.some((entry) => entry.code === 'source-serialization-drift'));
    await writeFile(modifierFile, modifier.replace('slug: string | null;', 'slug: string | null; added: () => number;'));
    const addedCallback = extractTypeGraph(temporary, selections);
    assert.equal(addedCallback.complete, false);
    assert.ok(addedCallback.diagnostics.some((entry) => entry.code === 'callable-source-type'));
    await writeFile(modifierFile, modifier);
    const predicateFile = path.join(temporary, 'src/module/system/predication.ts');
    const predicate = await readFile(predicateFile, 'utf8');
    for (const changed of [
      predicate.replace('toObject(): RawPredicate { return [...this]; }', 'toObject(): string { return "changed"; }'),
      predicate.replace('toObject(): RawPredicate { return [...this]; }', 'toObject(): number[] { return []; }'),
      predicate.replace('readonly isValid = true;', 'readonly isValid = true; toJSON() { return "changed"; }'),
      predicate.replace('extends Array<PredicateStatement>', '').replace('return [...this]', 'return []'),
      predicate.replace('constructor(...statements: PredicateStatement[] | [PredicateStatement[]])', 'constructor(statement: PredicateStatement)').replace('super(...(Array.isArray(statements[0]) ? statements[0] : statements as PredicateStatement[]))', 'super(statement)'),
    ]) {
      await writeFile(predicateFile, changed);
      const result = extractTypeGraph(temporary, selections);
      assert.equal(result.complete, false);
      assert.ok(result.diagnostics.some((entry) => ['class-instance-source-type', 'source-serialization-drift'].includes(entry.code)));
    }
  } finally { await rm(temporary, { recursive: true, force: true }); }
});

test('authored empty interface is an explicit open domain rather than an empty model', () => {
  const result = extractTypeGraph(fixture, { roots: [{ file: 'models.ts', name: 'ExplicitEmpty' }] });
  assert.equal(result.complete, true);
  assert.equal(rootNode(result, 'ExplicitEmpty').kind, 'open');
  assert.equal(domainOf(rootNode(result, 'ExplicitEmpty')), 'non-nullish');
});

test('anonymous nested generic structures retain each resolved instantiation', () => {
  const result = graph();
  const selected = rootNode(result, 'Instantiations');
  const first = lookup(result, field(selected, 'first').ref);
  const second = lookup(result, field(selected, 'second').ref);
  const firstNested = lookup(result, field(first, 'nested').ref);
  const secondNested = lookup(result, field(second, 'nested').ref);
  assert.notEqual(firstNested.id, secondNested.id);
  assert.equal(valueOf(lookup(result, field(firstNested, 'value').ref)), 'string');
  assert.equal(valueOf(lookup(result, field(secondNested, 'value').ref)), 'number');
});

test('anonymous discriminated union alternatives remain distinct and preserve their fields', () => {
  const result = graph();
  const family = rootNode(result, 'AnonymousFamily');
  assert.equal(new Set(membersOf(family)).size, 2);
  assert.deepEqual(membersOf(family).map((ref) => {
    const member = lookup(result, ref);
    return [valueOf(lookup(result, field(member, 'kind').ref)),
      valueOf(lookup(result, field(member, 'payload').ref))];
  }).sort(), [['a', 'string'], ['b', 'number']]);
});

test('same printed type names retain distinct declaration identity through nested and nullable shapes', () => {
  const result = graph();
  const selected = rootNode(result, 'SameNamedArguments');
  const nested = ['first', 'second'].map((name) => {
    const shared = lookup(result, field(selected, name).ref);
    return lookup(result, field(shared, 'nested').ref);
  });
  assert.notEqual(nested[0].id, nested[1].id);
  const values = nested.map((node) => lookup(result, field(node, 'value').ref));
  assert.deepEqual(values.map((node) => fieldsOf(node).map((entry) => entry.name)), [['a'], ['b']]);
  assert.notEqual(values[0].id, values[1].id);
  const nullable = rootNode(result, 'SameNamedNullable');
  const first = lookup(result, field(nullable, 'first').ref);
  const second = lookup(result, field(nullable, 'second').ref);
  assert.notEqual(first.id, second.id);
  assert.ok(membersOf(first).includes(values[0].id));
  assert.ok(membersOf(second).includes(values[1].id));
});

test('repeat and relocated input produce identical output; an upstream field addition has an understandable diff', async () => {
  const original = graph();
  assert.deepEqual(graph(), original);
  const temporary = await mkdtemp(path.join(os.tmpdir(), 'atlas-type-graph-'));
  try {
    await cp(fixture, temporary, { recursive: true });
    assert.deepEqual(extractTypeGraph(temporary, { roots }), original);
    assert.ok(!JSON.stringify(original).includes(fixture));
    const model = path.join(temporary, 'models.ts');
    const source = await readFile(model, 'utf8');
    await writeFile(model, source.replace("type: 'equipment';", "type: 'equipment';\n  newlyAuthored?: number;"));
    const changed = extractTypeGraph(temporary, { roots });
    const before = rootNode(original, 'Equipment');
    const after = rootNode(changed, 'Equipment');
    assert.equal(after.id, before.id);
    assert.deepEqual(fieldsOf(after).filter((entry) => !fieldsOf(before).some((old) => old.name === entry.name))
      .map((entry) => ({ name: entry.name, optional: entry.optional, ref: entry.ref })),
    [{ name: 'newlyAuthored', optional: true, ref: field(after, 'newlyAuthored').ref }]);
    assert.equal(rootNode(changed, 'Backpack').id, rootNode(original, 'Backpack').id);
  } finally { await rm(temporary, { recursive: true, force: true }); }
});


// Accessors assert the discovered shape before inspecting its payload.
function fieldsOf(node: GraphNode) { assert.ok('fields' in node && node.fields); return node.fields; }
function membersOf(node: GraphNode) { assert.ok(node.kind === 'union' || node.kind === 'intersection'); return node.members; }
function valueOf(node: GraphNode) { assert.ok(node.kind === 'primitive' || node.kind === 'literal'); return node.value; }
function domainOf(node: GraphNode) { assert.equal(node.kind, 'open'); assert.ok(node.kind === 'open'); return node.domain; }
function elementOf(node: GraphNode | undefined) { assert.ok(node?.kind === 'array'); return node.element; }
function elementsOf(node: GraphNode) { assert.ok(node.kind === 'tuple'); return node.elements; }
function readonlyOf(node: GraphNode) { assert.ok(node.kind === 'tuple' || node.kind === 'array'); return node.readonly; }
function basesOf(node: GraphNode) { assert.ok(node.kind === 'object' && node.extends); return node.extends; }
function indicesOf(node: GraphNode) { assert.ok(node.kind === 'object'); return node.indexSignatures; }
function arraySerializationOf(node: GraphNode) { assert.ok(node.kind === 'array' && node.serialization); return node.serialization; }
function choiceSerializationOf(field: GraphField) { assert.ok(field.serialization?.basis === 'predicate-constructor-input'); return field.serialization; }
function callbackSerializationOf(field: GraphField) { assert.ok(field.serialization?.basis === 'omitted-function-property'); return field.serialization; }
