import assert from 'node:assert/strict';

import path from 'node:path';

import type { EquipmentPacket } from './sample-equipment.js';

export type Result = { ok: true; value: Record<string, unknown> } | { ok: false; path: string };
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
