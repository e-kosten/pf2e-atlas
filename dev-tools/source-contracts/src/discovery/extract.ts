import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';

import ts from 'typescript';
import { createHash } from 'node:crypto';
import { sourceIdentity } from './source-identity.js';
import { extractTypeGraph } from './type-graph.js';
import { extractTraitCatalog } from './trait-catalog.js';
import type { ExtractionSummary } from '../contracts.js';

export async function extract(sourceRoot: string, outputRoot: string, { strict = false }: { strict?: boolean } = {}) {
  const identity = await sourceIdentity(sourceRoot);
  const typeGraph = await extractTypeGraph(sourceRoot);
  typeGraph.source = identity;
  const traitCatalog = await extractTraitCatalog(sourceRoot);
  const summary: ExtractionSummary = {
    format: 'atlas-source-extraction/v1',
    source: identity,
    typescript_version: ts.version,
    dependency_lock_digest: createHash('sha256').update(await readFile(new URL('../../../package-lock.json', import.meta.url))).digest('hex'),
    complete: typeGraph.complete && traitCatalog.complete,
    type_graph_complete: typeGraph.complete,
    trait_catalog_complete: traitCatalog.complete,
    products: { type_graph: 'type-graph.json', trait_catalog: 'trait-catalog.json' },
  };
  await mkdir(outputRoot, { recursive: true });
  for (const [name, value] of [['type-graph.json', typeGraph], ['trait-catalog.json', traitCatalog], ['summary.json', summary]] as const) {
    await writeFile(path.join(outputRoot, name), `${JSON.stringify(value, null, 2)}\n`);
  }
  return { summary, exitCode: strict && !summary.complete ? 1 : 0 };
}
