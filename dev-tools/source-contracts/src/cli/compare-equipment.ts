import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import type { EquipmentPacket } from '../comparison/sample-equipment.js';
import { compareEquipment, type Result } from '../comparison/compare-equipment.js';

async function main() {
  const { values } = parseArgs({ options: { packets: { type: 'string' }, baseline: { type: 'string' },
    generated: { type: 'string' }, help: { type: 'boolean', short: 'h' } } });
  if (values.help) { console.log('Usage: npm --prefix dev-tools/source-contracts run compare-equipment -- --packets PATH --baseline PATH --generated PATH'); return; }
  if (!values.packets || !values.baseline || !values.generated) throw new Error('--packets, --baseline and --generated are required');
  const read = async <T>(file: string): Promise<T[]> => (await readFile(path.resolve(process.env.INIT_CWD ?? process.cwd(), file), 'utf8'))
    .split('\n').filter(Boolean).map(line => JSON.parse(line) as T);
  const result = compareEquipment(await read<EquipmentPacket>(values.packets), await read<Result>(values.baseline), await read<Result>(values.generated));
  console.log(JSON.stringify(result, null, 2));
  if (result.differences.length) process.exitCode = 1;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { await main(); } catch (error) { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; }
}
