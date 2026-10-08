import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

import { generate } from '../generation/generate.js';

async function main() {
  const { values } = parseArgs({ options: { manifest: { type: 'string' }, graph: { type: 'string' }, summary: { type: 'string' },
    'snapshot-dir': { type: 'string' }, 'out-dir': { type: 'string' }, check: { type: 'boolean' }, help: { type: 'boolean', short: 'h' } } });
  if (values.help) { console.log('Usage: npm --prefix dev-tools/source-contracts run generate -- (--manifest PATH | --graph PATH --summary PATH --snapshot-dir PATH) --out-dir PATH [--check]'); return; }
  if (!values['out-dir']) throw new Error('--out-dir is required');
  const cwd = process.env.INIT_CWD ?? process.cwd();
  const resolve = (value: string | undefined) => value ? path.resolve(cwd, value) : undefined;
  console.log(JSON.stringify(await generate({ manifest: resolve(values.manifest), graph: resolve(values.graph), summary: resolve(values.summary),
    snapshotDir: resolve(values['snapshot-dir']), outDir: resolve(values['out-dir'])!, check: values.check }), null, 2));
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { await main(); } catch (error) { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; }
}
