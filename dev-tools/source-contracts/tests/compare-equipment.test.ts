import assert from 'node:assert/strict';
import test from 'node:test';
import { compareEquipment } from '../src/comparison/compare-equipment.js';
import type { EquipmentPacket } from '../src/comparison/sample-equipment.js';

const packet: EquipmentPacket = { source: 'unused raw payload', actor: null, ordinal: null,
  context: { record_key: 'a', source_path: 'a.json', json_path: '$' } };
test('comparison detects lost large integers, presence changes and rejection-path differences', () => {
  const before = { ok: true as const, value: { hp: { value: { max: { value: '9007199254740993' } } }, usage: 'missing' } };
  assert.equal(compareEquipment([packet], [before], [structuredClone(before)]).counts.equalValues, 1);
  for (const [name, value] of [
    ['large integer', { hp: { value: { max: { value: '9007199254740992' } } }, usage: 'missing' }],
    ['presence', { hp: { value: { max: { value: '9007199254740993' } } }, usage: 'null' }],
  ] as const) {
    const changed = { ok: true as const, value };
    const result = compareEquipment([packet], [before], [changed]);
    assert.deepEqual(result.differences, [{ context: packet.context, baseline: before, generated: changed }], name);
    assert.equal(result.counts.equalValues, 0, name);
  }
  assert.equal(compareEquipment([packet], [{ ok: false, path: '$.hp' }], [{ ok: false, path: '$.price' }]).differences.length, 1);
  assert.throws(() => compareEquipment([packet], [], []), /result count/);
});
