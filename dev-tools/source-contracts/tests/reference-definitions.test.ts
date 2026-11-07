import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { createCatalogReader } from '../src/discovery/catalog-values.js';
import { generateReferenceModule } from '../src/generation/reference-module.js';

test('reference metadata output retains real record pointers without copying prose', () => {
  const output = generateReferenceModule({ digest: 'a'.repeat(64),
    definitions: [{ namespace: 'sense', identifier: 'darkvision', labelKey: 'PF2E.Darkvision', descriptionKey: null,
      displayLabel: null, recordKey: 'general:aaaaaaaaaaaaaaaa', expectedFamily: 'action', requiredLocalizationKey: null,
      sourceFile: 'src/values.ts', sourceLine: 7, excluded: false, filterFields: ['senses'] }] },
    '// fixture', { repository: 'https://example.invalid/source.git', commit: 'b'.repeat(40) });
  assert.match(output, /record_key: Some\("general:aaaaaaaaaaaaaaaa"\)/);
  assert.match(output, /description_key: None/);
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
