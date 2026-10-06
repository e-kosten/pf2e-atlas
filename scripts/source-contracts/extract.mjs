import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import ts from 'typescript';
import { createHash } from 'node:crypto';
import { sourceIdentity } from './source-identity.mjs';
import { extractTypeGraph } from './type-graph.mjs';
import { extractTraitCatalog } from './trait-catalog.mjs';

export async function extract(sourceRoot, outputRoot, { strict = false } = {}) {
  const identity = await sourceIdentity(sourceRoot);
  const typeGraph = await extractTypeGraph(sourceRoot);
  const traitCatalog = await extractTraitCatalog(sourceRoot);
  const summary = {
    format: 'atlas-source-extraction/v1',
    source: identity,
    typescript_version: ts.version,
    dependency_lock_digest: createHash('sha256').update(await readFile(new URL('./package-lock.json', import.meta.url))).digest('hex'),
    complete: typeGraph.complete && traitCatalog.complete,
    type_graph_complete: typeGraph.complete,
    trait_catalog_complete: traitCatalog.complete,
    products: { type_graph: 'type-graph.json', trait_catalog: 'trait-catalog.json' },
  };
  await mkdir(outputRoot, { recursive: true });
  for (const [name, value] of [['type-graph.json', typeGraph], ['trait-catalog.json', traitCatalog], ['summary.json', summary]]) {
    await writeFile(path.join(outputRoot, name), `${JSON.stringify(value, null, 2)}\n`);
  }
  return { summary, exitCode: strict && !summary.complete ? 1 : 0 };
}

async function main() {
  const { values } = parseArgs({ options: { source: { type: 'string' }, out: { type: 'string' }, strict: { type: 'boolean' }, help: { type: 'boolean', short: 'h' } } });
  if (values.help) {
    console.log('Usage: node scripts/source-contracts/extract.mjs --source PATH --out PATH [--strict]');
    console.log('Writes source type discovery and trait metadata; --strict rejects incomplete extraction.');
    return 0;
  }
  if (!values.source || !values.out) throw new Error('--source and --out are required');
  const { summary, exitCode } = await extract(path.resolve(values.source), path.resolve(values.out), { strict: values.strict });
  console.log(JSON.stringify(summary, null, 2));
  return exitCode;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { process.exitCode = await main(); }
  catch (error) { console.error(error.message); process.exitCode = 1; }
}
