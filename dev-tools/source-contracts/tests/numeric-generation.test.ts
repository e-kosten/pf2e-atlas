import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';
import ts from 'typescript';
import { recursiveFixture } from '../fixtures/recursive-fixture.js';
import { generateRustModules } from '../src/generation/source-generation.js';

test('numeric index owners retain nonnumeric keys, constrain numeric named fields and anchor recursion', () => {
  const rust = generateRustModules(recursiveFixture());
  assert.match(rust['common.rs'], /pub struct NumericKeys/);
  assert.match(rust['common.rs'], /f.number_indexed\(&\[\], &\[\], number\)/);
  assert.match(rust['common.rs'], /f.number_indexed\(&\["label", "2"\], &\["3"\], read_a\)/);
  assert.match(rust['common.rs'], /f.number_remaining\(&\["label", "2"\], &\["3"\]\)/);
  assert.match(rust['common.rs'], /SourceMap<RecursiveNumericKeys>/);
  assert.match(rust['consumer.rs'], /generated::common::\{[^}]*NumericKeys/);
});

test('shared owner identity distinguishes number and string index domains on named structs', () => {
  const fixture=recursiveFixture();
  const numeric=fixture.nodes.find(node=>node.id==='NumericBag');assert.ok(numeric?.kind==='object');
  fixture.nodes.push({...numeric,id:'StringIndexedBag',name:'StringIndexedBag',indexSignatures:[{...numeric.indexSignatures[0],key:'primitive:string'}]});
  fixture.selection.push({name:'StringIndexedBag',declaration:'StringIndexedBag',valueRef:'StringIndexedBag',module:'consumer',fields:[],deferred:[]});
  const rust=generateRustModules(fixture);
  assert.match(rust['common.rs'],/pub struct NumericBag/);
  assert.match(rust['consumer.rs'],/pub struct StringIndexedBag/);
  assert.match(rust['consumer.rs'],/f.indexed\(&\["label", "2"\], &\["3"\], read_a\)/);
});

test('null-only fields and numeric, empty or punctuation enum tokens preserve exact source values', () => {
  const rust = generateRustModules(recursiveFixture())['common.rs'];
  assert.match(rust, /pub only: SourcePresence<\(\)>/);
  assert.match(rust, /pub maybe: SourcePresence<\(\)>/);
  assert.match(rust, /"only", null/);
  assert.match(rust, /rename = ""\)\]\s+Empty/);
  assert.match(rust, /rename = "0"\)\]\s+Value0/);
  assert.match(rust, /rename = "-"\)\]\s+ValueU2D/);
});

test('Rust numeric-key fixtures agree with the pinned TypeScript compiler', async () => {
  const cases = JSON.parse(await readFile(new URL('../../../../crates/atlas-foundry-model/tests/fixtures/numeric-keys.json', import.meta.url), 'utf8')) as [string, boolean][];
  // This exported compiler utility defines which property names use a numeric index.
  const compiler = ts as typeof ts & { isNumericLiteralName(name: string): boolean };
  assert.equal(ts.version, '5.9.3');
  for (const [key, expected] of cases) assert.equal(compiler.isNumericLiteralName(key), expected, key);
});
