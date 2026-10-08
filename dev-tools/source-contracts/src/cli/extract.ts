import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

import { extract } from '../discovery/extract.js';

async function main() {
  const { values } = parseArgs({ options: { source: { type: 'string' }, out: { type: 'string' }, strict: { type: 'boolean' }, help: { type: 'boolean', short: 'h' } } });
  if (values.help) {
    console.log('Usage: npm --prefix dev-tools/source-contracts run extract -- --source PATH --out PATH [--strict]');
    console.log('Writes source type discovery and trait metadata; --strict rejects incomplete extraction.');
    return 0;
  }
  if (!values.source || !values.out) throw new Error('--source and --out are required');
  // npm runs package scripts from the package directory. Keep CLI paths relative
  // to the caller, matching direct Node execution from that same directory.
  const cwd = process.env.INIT_CWD ?? process.cwd();
  const { summary, exitCode } = await extract(path.resolve(cwd, values.source), path.resolve(cwd, values.out), { strict: values.strict });
  console.log(JSON.stringify(summary, null, 2));
  return exitCode;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { process.exitCode = await main(); }
  catch (error) { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; }
}
