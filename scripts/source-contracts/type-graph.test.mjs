import assert from 'node:assert/strict';
import { cp, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { extractTypeGraph } from './type-graph.mjs';

const fixture = fileURLToPath(new URL('./fixtures/type-graph/', import.meta.url));
const roots = ['Equipment', 'Backpack', 'Family', 'Pair', 'Formula', 'NumericFormula', 'Instantiations',
  'AnonymousFamily', 'SameNamedArguments', 'SameNamedNullable'].map((name) => ({ file: 'models.ts', name }));
const graph = () => extractTypeGraph(fixture, { roots });
const lookup = (result, id) => result.nodes.find((node) => node.id === id);
const rootNode = (result, name) => lookup(result, result.roots.find((entry) => entry.name === name).ref);
const field = (node, name) => node.fields.find((entry) => entry.name === name);

test('default portfolio follows pack kinds, complete family unions and registered schema sources', () => {
  const result = extractTypeGraph(fixture);
  assert.equal(result.complete, true, JSON.stringify(result.diagnostics));
  assert.deepEqual(result.portfolio.documentKinds, ['Actor', 'Item', 'JournalEntry', 'Macro', 'RollTable']);
  assert.equal(result.roots.length, 7);
  for (const family of result.portfolio.families) assert.deepEqual(family.discovered, family.registered);
  assert.deepEqual(result.portfolio.ruleKeys, ['Example', 'Inherited']);
  for (const root of result.roots.filter((entry) => entry.ruleKey)) {
    const node = lookup(result, root.ref);
    assert.deepEqual(node.fields.map((field) => field.name), ['amount', 'choices']);
    assert.equal(lookup(result, field(node, 'amount').ref).value, 'number');
    assert.equal(lookup(result, field(node, 'choices').ref).kind, 'array');
  }
  assert.ok(!result.nodes.some((node) => node.fields?.some((field) => field.name === 'preparedOnly')));
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
    assert.deepEqual(added.portfolio.ruleKeys, ['Added', 'Example', 'Inherited']);
    assert.ok(added.roots.find((entry) => entry.ruleKey === 'Added').ref);
    const original = extractTypeGraph(fixture);
    assert.equal(added.roots.find((entry) => entry.ruleKey === 'Example').ref,
      original.roots.find((entry) => entry.ruleKey === 'Example').ref);
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
  assert.equal(lookup(result, field(equipment, 'type').ref).value, 'equipment');
  assert.equal(lookup(result, field(backpack, 'type').ref).value, 'backpack');
  const usage = field(equipment, 'usage');
  assert.equal(usage.optional, true);
  assert.equal(usage.nullable, true);
  assert.equal(usage.undefinedAllowed, true);
  assert.equal(field(backpack, 'subitems').forbidden, true);
  assert.ok(equipment.extends.some((ref) => lookup(result, ref).name === 'Common'));
  assert.deepEqual(rootNode(result, 'Family').members, [equipment.id, backpack.id].sort());
  assert.equal(lookup(result, field(equipment, 'explicit').ref).domain, 'any');
  assert.equal(lookup(result, field(equipment, 'legacy').ref).domain, 'object');
  const flags = lookup(result, field(equipment, 'flags').ref);
  assert.equal(lookup(result, flags.indexSignatures[0].value).domain, 'unknown');
});

test('retains instantiated recursive references and tuple/mapped fields', () => {
  const result = graph();
  const array = lookup(result, field(rootNode(result, 'Equipment'), 'contents').ref);
  const nested = lookup(result, array.element);
  const recursion = lookup(result, field(nested, 'children').ref);
  assert.equal(recursion.element, nested.id);
  assert.equal(lookup(result, field(nested, 'payload').ref).name, 'NumericValue');
  assert.equal(rootNode(result, 'Pair').elements[1].optional, true);
  assert.equal(rootNode(result, 'Pair').readonly, true);
  assert.deepEqual(rootNode(result, 'Formula').fields.map((entry) => [entry.name, entry.optional]),
    [['a', true], ['b', true]]);
  const numeric = rootNode(result, 'NumericFormula');
  assert.deepEqual(numeric.fields.map((entry) => entry.name), ['1', '2']);
  const numericValue = lookup(result, field(numeric, '1').ref);
  assert.equal(numericValue.kind, 'union');
  assert.ok(numericValue.members.some((ref) => lookup(result, ref).name === 'NumericValue'));
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
  assert.equal(lookup(mixed, field(rootNode(mixed, 'Equipment'), 'explicit').ref).domain, 'any');
  assert.equal(lookup(mixed, field(rootNode(mixed, 'IndirectBroken'), 'field').ref).kind, 'unresolved');
});

test('missing root and closure bounds are visible failures', () => {
  assert.equal(extractTypeGraph(fixture, { roots: [{ file: 'missing.ts', name: 'Missing' }] }).status, 'incomplete');
  const limited = extractTypeGraph(fixture, { roots, maxNodes: 2 });
  assert.equal(limited.status, 'incomplete');
  assert.ok(limited.diagnostics.some((entry) => entry.code === 'closure-limit'));
  assert.ok(limited.nodes.some((entry) => entry.id === 'unsupported:closure-limit'));
});

test('authored empty interface is an explicit open domain rather than an empty model', () => {
  const result = extractTypeGraph(fixture, { roots: [{ file: 'models.ts', name: 'ExplicitEmpty' }] });
  assert.equal(result.complete, true);
  assert.equal(rootNode(result, 'ExplicitEmpty').kind, 'open');
  assert.equal(rootNode(result, 'ExplicitEmpty').domain, 'non-nullish');
});

test('anonymous nested generic structures retain each resolved instantiation', () => {
  const result = graph();
  const selected = rootNode(result, 'Instantiations');
  const first = lookup(result, field(selected, 'first').ref);
  const second = lookup(result, field(selected, 'second').ref);
  const firstNested = lookup(result, field(first, 'nested').ref);
  const secondNested = lookup(result, field(second, 'nested').ref);
  assert.notEqual(firstNested.id, secondNested.id);
  assert.equal(lookup(result, field(firstNested, 'value').ref).value, 'string');
  assert.equal(lookup(result, field(secondNested, 'value').ref).value, 'number');
});

test('anonymous discriminated union alternatives remain distinct and preserve their fields', () => {
  const result = graph();
  const family = rootNode(result, 'AnonymousFamily');
  assert.equal(new Set(family.members).size, 2);
  assert.deepEqual(family.members.map((ref) => {
    const member = lookup(result, ref);
    return [lookup(result, field(member, 'kind').ref).value,
      lookup(result, field(member, 'payload').ref).value];
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
  assert.deepEqual(values.map((node) => node.fields.map((entry) => entry.name)), [['a'], ['b']]);
  assert.notEqual(values[0].id, values[1].id);
  const nullable = rootNode(result, 'SameNamedNullable');
  const first = lookup(result, field(nullable, 'first').ref);
  const second = lookup(result, field(nullable, 'second').ref);
  assert.notEqual(first.id, second.id);
  assert.ok(first.members.includes(values[0].id));
  assert.ok(second.members.includes(values[1].id));
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
    assert.deepEqual(after.fields.filter((entry) => !before.fields.some((old) => old.name === entry.name))
      .map((entry) => ({ name: entry.name, optional: entry.optional, ref: entry.ref })),
    [{ name: 'newlyAuthored', optional: true, ref: field(after, 'newlyAuthored').ref }]);
    assert.equal(rootNode(changed, 'Backpack').id, rootNode(original, 'Backpack').id);
  } finally { await rm(temporary, { recursive: true, force: true }); }
});
