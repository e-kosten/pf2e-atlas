import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

import { sampleEquipment } from '../comparison/sample-equipment.js';

async function main() {
  const { values } = parseArgs({ options: { source: { type: 'string' }, help: { type: 'boolean', short: 'h' } } });
  if (values.help) { console.log('Usage: npm --prefix dev-tools/source-contracts run sample-equipment -- --source PATH'); return; }
  if (!values.source) throw new Error('--source is required');
  const source = path.resolve(process.env.INIT_CWD ?? process.cwd(), values.source);
  for await (const packet of sampleEquipment(source)) {
    if (!process.stdout.write(JSON.stringify(packet) + '\n')) await new Promise<void>(resolve => process.stdout.once('drain', resolve));
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { await main(); } catch (error) { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; }
}
