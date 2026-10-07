import { readFile, mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { spawnSync } from 'node:child_process';
import type { ExtractionSummary, TypeGraph } from './contracts.js';
import { generateEquipmentRust, selectEquipmentInput, type EquipmentGenerationInput } from './equipment-generation.js';

export function formatRust(text: string): string {
  const result = spawnSync('rustfmt', ['--edition', '2024'], { input: text, encoding: 'utf8' });
  if (result.error || result.status !== 0) throw new Error(`rustfmt failed: ${result.error?.message ?? result.stderr}`);
  return result.stdout;
}
export async function generateEquipment(args: { input?: string; graph?: string; summary?: string; out: string; check?: boolean }) {
  if (Boolean(args.input) === Boolean(args.graph) || Boolean(args.graph) !== Boolean(args.summary))
    throw new Error('Use either --input PATH or --graph PATH --summary PATH');
  const inputs = [args.input, args.graph, args.summary].filter((file): file is string => Boolean(file)).map(file => path.resolve(file));
  const outputs = [path.resolve(args.out), ...(!args.input ? [path.resolve(args.out.replace(/\.rs$/, '') + '.input.json')] : [])];
  if (outputs.some(file => inputs.includes(file)) || new Set(outputs).size !== outputs.length)
    throw new Error('Output paths must be separate from generation inputs and each other');
  const input: EquipmentGenerationInput = args.input
    ? JSON.parse(await readFile(args.input, 'utf8'))
    : selectEquipmentInput(JSON.parse(await readFile(args.graph!, 'utf8')) as TypeGraph,
      JSON.parse(await readFile(args.summary!, 'utf8')) as ExtractionSummary);
  const rust = formatRust(generateEquipmentRust(input));
  if (args.check) {
    if (await readFile(args.out, 'utf8') !== rust) throw new Error(`Generated Rust is stale: ${args.out}`);
    if (!args.input && await readFile(outputs[1], 'utf8') !== JSON.stringify(input, null, 2) + '\n')
      throw new Error(`Generation input is stale: ${outputs[1]}`);
  } else {
    await mkdir(path.dirname(args.out), { recursive: true });
    await writeFile(args.out, rust);
    if (!args.input) await writeFile(args.out.replace(/\.rs$/, '') + '.input.json', JSON.stringify(input, null, 2) + '\n');
  }
  return { source: input.source, fields: input.selection.map(s => ({ declaration: s.declaration,
    selected: s.fields.map(f => ({ name: f.name, optional: f.optional, nullable: f.nullable })), deferred: s.deferred })),
    selectedNodes: input.nodes.length, generatedLines: rust.split('\n').length - 1 };
}
async function main() {
  const { values } = parseArgs({ options: { input: { type: 'string' }, graph: { type: 'string' },
    summary: { type: 'string' }, out: { type: 'string' }, check: { type: 'boolean' }, help: { type: 'boolean', short: 'h' } } });
  if (values.help) { console.log('Usage: npm --prefix scripts/source-contracts run generate-equipment -- (--input PATH | --graph PATH --summary PATH) --out PATH [--check]'); return; }
  if (!values.out) throw new Error('--out is required');
  const cwd = process.env.INIT_CWD ?? process.cwd();
  const resolve = (value: string | undefined) => value ? path.resolve(cwd, value) : undefined;
  console.log(JSON.stringify(await generateEquipment({ input: resolve(values.input), graph: resolve(values.graph),
    summary: resolve(values.summary), out: resolve(values.out)!, check: values.check }), null, 2));
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { await main(); } catch (error) { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; }
}
