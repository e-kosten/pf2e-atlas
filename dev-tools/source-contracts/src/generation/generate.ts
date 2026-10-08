import { readFile } from 'node:fs/promises';
import path from 'node:path';

import { spawnSync } from 'node:child_process';
import type { ExtractionSummary, TypeGraph } from '../contracts.js';
import { selectSourceInput } from './predicate-selection.js';
import { loadGenerationInput, snapshotFiles } from './generation-input.js';
import { prepareGeneratedFiles } from './generated-files.js';
import { generateRustModules, generatedHeader } from './source-generation.js';

export function formatRust(text: string): string {
  const result = spawnSync('rustfmt', ['--edition', '2024', '--config', 'skip_children=true'], { input: text, encoding: 'utf8' });
  if (result.error || result.status !== 0) throw new Error(`rustfmt failed: ${result.error?.message ?? result.stderr}`);
  return result.stdout;
}
const contains = (directory: string, file: string) => {
  const relative = path.relative(path.resolve(directory), path.resolve(file));
  return !relative || (!relative.startsWith('..' + path.sep) && relative !== '..' && !path.isAbsolute(relative));
};
const snapshotOwned = (text: string) => {
  try { return ['atlas-source-generation/v1', 'atlas-source-generation-module/v1'].includes((JSON.parse(text) as { format: string }).format); }
  catch { return false; }
};
export async function generate(args: { manifest?: string; graph?: string; summary?: string; snapshotDir?: string; outDir: string; check?: boolean }) {
  if (Boolean(args.manifest) === Boolean(args.graph) || Boolean(args.graph) !== Boolean(args.summary)
    || Boolean(args.graph) !== Boolean(args.snapshotDir)) throw new Error('Use either --manifest PATH or --graph PATH --summary PATH --snapshot-dir PATH');
  const inputs = [args.manifest, args.graph, args.summary].filter((file): file is string => Boolean(file));
  if (inputs.some(file => contains(args.outDir, file)) || (args.manifest && contains(path.dirname(args.manifest), args.outDir))
    || (args.snapshotDir && (contains(args.snapshotDir, args.outDir) || contains(args.outDir, args.snapshotDir)
      || inputs.some(file => contains(args.snapshotDir!, file)))))
    throw new Error('Output directories must be separate from inputs and each other');
  const input = args.manifest ? await loadGenerationInput(args.manifest)
    : selectSourceInput(JSON.parse(await readFile(args.graph!, 'utf8')) as TypeGraph,
      JSON.parse(await readFile(args.summary!, 'utf8')) as ExtractionSummary);
  // Resolve all roots and format all Rust before writing either artifact set.
  const rust = Object.fromEntries(Object.entries(generateRustModules(input)).map(([file, text]) => [file, formatRust(text)]));
  const snapshots = snapshotFiles(input);
  if (args.manifest) {
    snapshots[path.basename(args.manifest)] = snapshots['manifest.json'];
    if (path.basename(args.manifest) !== 'manifest.json') delete snapshots['manifest.json'];
  }
  const writeSnapshots = await prepareGeneratedFiles(args.snapshotDir ?? path.dirname(args.manifest!), snapshots,
    args.manifest ? true : Boolean(args.check), snapshotOwned);
  const writeRust = await prepareGeneratedFiles(args.outDir, rust, Boolean(args.check), text => text.startsWith(generatedHeader + '\n'));
  await writeSnapshots();
  await writeRust();
  return { source: input.source, selectedNodes: input.nodes.length,
    fields: input.selection.map(root => ({ declaration: root.declaration, module: root.module,
      selected: root.fields.map(field => ({ name: field.name, optional: field.optional, nullable: field.nullable })), deferred: root.deferred })),
    modules: Object.keys(rust).sort(), generatedLines: Object.values(rust).reduce((sum, text) => sum + text.split('\n').length - 1, 0) };
}
