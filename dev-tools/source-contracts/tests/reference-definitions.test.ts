import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { createCatalogReader } from '../src/discovery/catalog-values.js';
import { validateReferenceTemplate, type ReferenceRegistry } from '../src/generation/reference-definitions.js';
import { generateReferenceModule } from '../src/generation/reference-module.js';

const hash = (value: string) => createHash('sha256').update(value).digest('hex');
function templateFixture() {
  const body = '<p>0 feet. @Check[will|dc:]</p><p>@Localize[PF2E.Example]</p>';
  const localized = '@Localize[PF2E.Example]', parameter = '@Check[will|dc:]';
  const registry: ReferenceRegistry['general_templates'][number] = {
    record_key: 'general:aaaaaaaaaaaaaaaa', expected_family: 'action', source_package: 'pf2e',
    source_field: '/system/description/value', definition_localization_key: 'PF2E.Example',
    signature: { authored_description_sha256: hash(body), rules_sha256: hash('[{"key":"Aura","radius":0}]') },
    localized_selection: { source_span: [body.indexOf(localized), body.indexOf(localized) + localized.length], localization_key: 'PF2E.Example' },
    parameter_roles: [
      { role: 'check_dc', source_pointer: '/system/description/value', source_span: [body.indexOf(parameter), body.indexOf(parameter) + parameter.length], expected_fragment_sha256: hash(parameter) },
      { role: 'aura_radius', source_pointer: '/system/rules/0/radius', expected_value: 0 },
    ],
  };
  return { registry, record: { type: 'action', system: { description: { value: body }, rules: [{ radius: 0, key: 'Aura' }] } } };
}

test('template evidence verifies complete signatures, byte spans and rule-only roles', () => {
  const { registry, record } = templateFixture();
  const template = validateReferenceTemplate(registry, record);
  assert.equal(template.parameterRoles[0].fragmentSha256, registry.parameter_roles[0].expected_fragment_sha256);
  assert.equal(template.parameterRoles[1].start, null);
  assert.equal(template.parameterRoles[1].fragmentSha256, null);
  assert.equal(template.parameterRoles[1].sourcePointer, '/system/rules/0/radius');
  assert.equal(template.parameterRoles[1].expectedValueJson, '0');
  for (const change of [
    { ...record, type: 'feat' },
    { ...record, system: { ...record.system, description: { value: record.system.description.value + '<p>customized</p>' } } },
    { ...record, system: { ...record.system, rules: [{ key: 'Aura', radius: 5 }] } },
  ]) assert.throws(() => validateReferenceTemplate(registry, change), /signature changed/);
  const moved = structuredClone(registry); moved.parameter_roles[0].source_span![0]++;
  assert.throws(() => validateReferenceTemplate(moved, record), /parameter span changed/);
  const unknown = structuredClone(registry); unknown.parameter_roles[0].role = 'arbitrary_layout';
  assert.throws(() => validateReferenceTemplate(unknown, record), /Unreviewed/);
});

test('reference metadata output contains pointers and signatures without authoring a second body', () => {
  const { registry, record } = templateFixture();
  const output = generateReferenceModule({ digest: 'a'.repeat(64), records: [{ key: registry.record_key, family: 'action' }],
    templates: [validateReferenceTemplate(registry, record)], definitions: [{ namespace: 'sense', identifier: 'darkvision',
      labelKey: 'PF2E.Darkvision', descriptionKey: null, displayLabel: null, recordKey: registry.record_key,
      expectedFamily: 'action', requiredLocalizationKey: null, sourceFile: 'src/values.ts', sourceLine: 7,
      excluded: false, filterFields: ['senses'] }] }, '// fixture', { repository: 'https://example.invalid/source.git', commit: 'b'.repeat(40) });
  assert.match(output, /record_key: Some\("general:aaaaaaaaaaaaaaaa"\)/);
  assert.match(output, /description_key: None/);
  assert.match(output, /expected_value_json: Some\("0"\)/);
  assert.doesNotMatch(output, /<p>|@Check\[|@Localize\[/);
});

test('shared AST reader selects explicit PF2e branches and preserves exact vocabulary keys', async t => {
  const root = await mkdtemp(path.join(os.tmpdir(), 'atlas-reference-ast-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(path.join(root, 'catalog.ts'), `import * as R from 'remeda'; import { sluggify } from '@util';
    const senses = new Set(['greater-darkvision','scent'] as const);
    const labels = R.mapToObj(Array.from(senses), t => [t, \`PF2E.\${sluggify(t,{camel:'bactrian'})}\`]);
    const named = R.mapValues({fire:'old'}, (_v,k) => \`New.\${k}\`);
    const skills = {arcana:'Arcana', ...(SYSTEM_ID === 'sf2e' ? {piloting:'Piloting'} : {})};
    const unsupported = dangerousRuntimeCall();`);
  const reader = createCatalogReader(root, { systemId: 'pf2e', labelKeyPolicy: 'ascii_words' });
  const file = reader.load('catalog.ts');
  const labels = reader.object(reader.variable('labels', file, new Map()));
  assert.deepEqual([...labels.keys()], ['greater-darkvision', 'scent']);
  assert.equal(labels.get('greater-darkvision')!.value, 'PF2E.GreaterDarkvision');
  assert.equal(reader.object(reader.variable('named', file, new Map())).get('fire')!.value, 'New.fire');
  assert.deepEqual([...reader.object(reader.variable('skills', file, new Map())).keys()], ['arcana']);
  assert.throws(() => reader.variable('unsupported', file, new Map()), /Unsupported computed/);
  const unknownSystem = createCatalogReader(root);
  assert.throws(() => unknownSystem.variable('skills', unknownSystem.load('catalog.ts'), new Map()), /ConditionalExpression/);
  assert.match(await readFile(path.join(root, 'catalog.ts'), 'utf8'), /dangerousRuntimeCall/);
});

test('template spans are UTF-8 bytes and never split multibyte source text', () => {
  const { registry, record } = templateFixture();
  const body = record.system.description.value.replace('0 feet', '雪 feet');
  record.system.description.value = body;
  registry.signature.authored_description_sha256 = hash(body);
  const locate = (fragment: string): [number, number] => {
    const start = Buffer.byteLength(body.slice(0, body.indexOf(fragment)));
    return [start, start + Buffer.byteLength(fragment)];
  };
  registry.localized_selection.source_span = locate('@Localize[PF2E.Example]');
  registry.parameter_roles[0].source_span = locate('@Check[will|dc:]');
  assert.equal(validateReferenceTemplate(registry, record).localizedStart, registry.localized_selection.source_span[0]);
  const stale = structuredClone(registry); stale.localized_selection.source_span[0] -= 2;
  assert.throws(() => validateReferenceTemplate(stale, record), /localization span changed/);
  const split = structuredClone(registry); split.parameter_roles[0].source_span = [3, 4];
  split.parameter_roles[0].expected_fragment_sha256 = createHash('sha256').update(Buffer.from(body).subarray(3, 4)).digest('hex');
  assert.throws(() => validateReferenceTemplate(split, record), /Invalid UTF-8/);
  for (const rules of [undefined, null]) {
    assert.throws(() => validateReferenceTemplate(registry, { ...record, system: { ...record.system, rules } }), /signature changed/);
  }
});
