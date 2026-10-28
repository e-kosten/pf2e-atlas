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
  assert.deepEqual(definitions.slice(1, 3).map(item => item.descriptionKey),
    ['PF2E.TraitDescriptionDeadly', 'PF2E.TraitDescriptionDeadly']);
  assert.deepEqual(definitions[2].catalogs, ['npcAttackTraits', 'weaponTraits']);
  assert.deepEqual(definitions[2].sourceLocations, [source]);
  const rust = generateTraitDefinitionModule(definitions, '// generated', { repository: 'https://github.com/foundryvtt/pf2e.git', commit: 'a'.repeat(40) });
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


test('generated trait metadata retains pinned source attribution and transformation notice', () => {
  const commit = 'b'.repeat(40);
  const rust = generateTraitDefinitionModule([], '// generated', { repository: 'https://github.com/foundryvtt/pf2e.git', commit });
  assert(rust.includes(`https://github.com/foundryvtt/pf2e/blob/${commit}/src/scripts/config/traits.ts`));
  assert(rust.includes(`Apache-2.0 (https://github.com/foundryvtt/pf2e/blob/${commit}/LICENSE)`));
  assert(rust.includes('Copyright 2019 Hooking'));
  assert(rust.includes('Generated transformation: exact trait identifiers, localization-key mappings,'));
  assert(rust.includes('Localized prose is not included.'));
});
