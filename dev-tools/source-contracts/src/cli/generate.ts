import path from 'node:path';
import { parseArgs } from 'node:util';
import { generate } from '../generation/generate.js';

try {
  const { values } = parseArgs({ options: { source: { type: 'string' }, 'cache-dir': { type: 'string' },
    'out-dir': { type: 'string' }, check: { type: 'boolean' }, help: { type: 'boolean', short: 'h' } } });
  if (values.help) console.log('Usage: npm --prefix dev-tools/source-contracts run generate -- [--check] [--source PATH] [--cache-dir PATH] [--out-dir PATH]\nFetches the pinned source when absent; extracts afresh and checks or writes the Rust portfolio.');
  else {
    const cwd = process.env.INIT_CWD ?? process.cwd();
    const resolve = (value: string | undefined) => value ? path.resolve(cwd, value) : undefined;
    console.log(JSON.stringify(await generate({ source: resolve(values.source), cacheDir: resolve(values['cache-dir']),
      outDir: resolve(values['out-dir']), check: values.check }), null, 2));
  }
} catch (error) { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; }
