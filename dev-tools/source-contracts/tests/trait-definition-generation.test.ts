import assert from 'node:assert/strict';
import test from 'node:test';
import type { Catalog, TraitCatalog } from '../src/contracts.js';
import { traitDefinitions, generateTraitDefinitionModule } from '../src/generation/trait-definitions.js';

const source = { file: 'src/traits.ts', line: 12, column: 3 };
const entry = (identifier: string, descriptionKey: string | null = 'PF2E.TraitDescriptionDeadly') => ({
  identifier, labelKey: `PF2E.Trait.${identifier}`, descriptionKey,
  label: 'DO NOT EMIT LABEL', description: 'DO NOT EMIT DESCRIPTION', source,
});
const group = (name: string, entries: Catalog['entries'], namespace: Catalog['namespace'] = 'trait'): Catalog => ({
  name, entries, namespace, complete: true, exposedInConfig: true, exportedFromTraits: true, source, familyBindings: [],
});
const catalog = (catalogs: Catalog[]): TraitCatalog => ({ complete: true, catalogs, descriptions: [], diagnostics: [] });

test('exact variants retain shared mappings and sorted catalog memberships without English', () => {
  const definitions = traitDefinitions(catalog([
    group('weaponTraits', [entry('deadly-d12'), entry('deadly-d8'), entry('amp', null), { ...entry('unknown', null), labelKey: null }]),
    group('npcAttackTraits', [entry('deadly-d8')]),
    group('rarity', [entry('rare')], 'rarity'),
    group('tags', [entry('tag')], 'otherTag'),
  ]));
  assert.deepEqual(definitions.map(item => item.identifier), ['amp', 'deadly-d12', 'deadly-d8', 'unknown']);
  assert.equal(definitions[3].labelKey, null);
  assert.equal(definitions[0].descriptionKey, null);
  assert.equal(definitions[1].descriptionKey, definitions[2].descriptionKey);
  assert.deepEqual(definitions[2].catalogs, ['npcAttackTraits', 'weaponTraits']);
  assert.deepEqual(definitions[2].sourceLocations, [source]);
  const rust = generateTraitDefinitionModule(definitions, '// generated');
  assert.match(rust, /description_key: None/);
  assert.match(rust, /binary_search_by_key/);
  assert(!rust.includes('DO NOT EMIT'));
});

test('conflicting label or definition mappings and incomplete catalogs fail before generation', () => {
  for (const field of ['labelKey', 'descriptionKey'] as const) {
    const changed = { ...entry('agile'), [field]: 'conflict' };
    assert.throws(() => traitDefinitions(catalog([group('one', [entry('agile')]), group('two', [changed])])), /Conflicting/);
    assert.throws(() => traitDefinitions(catalog([group('one', [entry('agile')]), group('two', [{ ...changed, [field]: null }])])), /Conflicting/);
  }
  assert.throws(() => traitDefinitions({ ...catalog([]), complete: false }), /incomplete/);
  assert.throws(() => traitDefinitions(catalog([{ ...group('one', []), complete: false }])), /Incomplete/);
});
