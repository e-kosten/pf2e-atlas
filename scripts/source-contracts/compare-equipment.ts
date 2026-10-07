import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import type { EquipmentPacket } from './sample-equipment.js';

type Result = { ok: true; value: Record<string, unknown> } | { ok: false; path: string };
/** Probe values already carry numbers as strings; raw source packets are never reserialized. */
export function compareEquipment(packets: EquipmentPacket[], baseline: Result[], generated: Result[]) {
  assert.equal(baseline.length, packets.length, 'baseline result count');
  assert.equal(generated.length, packets.length, 'generated result count');
  const counts = { records: packets.length, root: 0, embedded: 0, baselineAccepted: 0, generatedAccepted: 0,
    equalValues: 0, equalRejections: 0 };
  const presence: Record<string, Record<string, number>> = {};
  const differences: { context: EquipmentPacket['context']; baseline: Result; generated: Result }[] = [];
  packets.forEach((packet, index) => {
    counts[packet.ordinal === null ? 'root' : 'embedded']++;
    const before = baseline[index], after = generated[index];
    if (before.ok) counts.baselineAccepted++;
    if (after.ok) {
      counts.generatedAccepted++;
      for (const [field, value] of Object.entries(after.value)) {
        const state = typeof value === 'string' ? value : 'value';
        presence[field] ??= { missing: 0, null: 0, value: 0 };
        presence[field][state] = (presence[field][state] ?? 0) + 1;
      }
    }
    try {
      assert.deepEqual(after, before);
      if (after.ok) counts.equalValues++; else counts.equalRejections++;
    } catch { differences.push({ context: packet.context, baseline: before, generated: after }); }
  });
  return { counts, presence, differences };
}
async function main() {
  const { values } = parseArgs({ options: { packets: { type: 'string' }, baseline: { type: 'string' },
    generated: { type: 'string' }, help: { type: 'boolean', short: 'h' } } });
  if (values.help) { console.log('Usage: npm --prefix scripts/source-contracts run compare-equipment -- --packets PATH --baseline PATH --generated PATH'); return; }
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
