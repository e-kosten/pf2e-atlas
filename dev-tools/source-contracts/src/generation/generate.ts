import { spawnSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import type { TypeGraph } from '../contracts.js';
import { extract } from '../discovery/extract.js';
import { sourceIdentity } from '../discovery/source-identity.js';
import { selectPortfolioInput } from './portfolio-selection.js';
import type { GenerationInput } from './generation-input.js';
import { prepareGeneratedFiles } from './generated-files.js';
import { generateRustModules, generatedHeader } from './source-generation.js';
import { assertSourcePin, contains, defaultCache, preparePinnedSource, readSourcePin, repositoryRoot } from './pinned-source.js';

export function formatRust(text: string): string {
  const result = spawnSync('rustfmt', ['--edition', '2024', '--config', 'skip_children=true'], {
    input: text, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024, timeout: 60_000,
  });
  if (result.error || result.status !== 0) throw new Error('rustfmt failed: ' + (result.error?.message ?? result.stderr));
  return result.stdout;
}
/** Fresh extraction is deliberate: cached graphs never authorize generated output. */
export async function preparePortfolio(args: { source?: string; cacheDir?: string } = {}) {
  const pin = await readSourcePin(), cacheDir = args.cacheDir ?? defaultCache;
  const source = await preparePinnedSource(pin, cacheDir, args.source);
  const extraction = path.join(cacheDir, pin.commit, 'extraction');
  const { summary } = await extract(source, extraction, { strict: true });
  assertSourcePin(summary.source, pin);
  assertSourcePin(await sourceIdentity(source), pin);
  if (!summary.complete) throw new Error('Pinned source extraction is incomplete; inspect ' + extraction);
  const graph = JSON.parse(await readFile(path.join(extraction, 'type-graph.json'), 'utf8')) as TypeGraph;
  const input = selectPortfolioInput(graph, summary);
  if (JSON.stringify(input.portfolio!.schemaRoots.flatMap(root => root.ruleKey ? [root.ruleKey] : []).sort())
    !== JSON.stringify([...pin.ruleKeys].sort())) throw new Error('Extracted rule keys do not match source-pin.json');
  input.source = { ...input.source, git_commit: null, git_clean: null };
  await writeFile(path.join(extraction, 'generation-input.json'), JSON.stringify(input, null, 2) + '\n');
  return { input, graph, summary, source, extraction };
}
export async function generateInput(input: GenerationInput, outDir: string, check = false) {
  const rust = Object.fromEntries(Object.entries(generateRustModules(input)).map(([file, text]) => [file, formatRust(text)]));
  const write = await prepareGeneratedFiles(outDir, rust, check, text => text.startsWith(generatedHeader + '\n'));
  await write();
  return { source: input.source, selectedNodes: input.nodes.length, selectedRoots: input.selection.length, schemaRoots: input.portfolio?.schemaRoots.length ?? 0,
    modules: Object.keys(rust).sort(), generatedLines: Object.values(rust).reduce((sum, text) => sum + text.split('\n').length - 1, 0) };
}
export async function generate(args: { source?: string; cacheDir?: string; outDir?: string; check?: boolean } = {}) {
  const outDir = args.outDir ?? path.join(repositoryRoot, 'crates/atlas-ingest/src/source_model/generated');
  const cacheDir = args.cacheDir ?? defaultCache;
  if (contains(outDir, cacheDir) || contains(cacheDir, outDir)
    || args.source && (contains(outDir, args.source) || contains(args.source, outDir)))
    throw new Error('Generated output must be separate from source/cache inputs');
  const prepared = await preparePortfolio({ source: args.source, cacheDir });
  return { ...await generateInput(prepared.input, outDir, args.check), extraction: prepared.extraction };
}
