import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { equipmentPackets, sampleEquipment } from './sample-equipment.js';

test('corpus packets retain raw numeric tokens and repeated payload keys', () => {
  const item = '{"_id":"a","type":"equipment","system":{"hp":{"value":9007199254740993,"max":1.0},"extra":1,"extra":2}}';
  assert.equal(equipmentPackets(item, 'packs/a.json', 'equipment')[0].source, item);
  const packets = equipmentPackets(`{"type":"npc","items":[{"type":"weapon"},${item}]}`, 'packs/npc.json', 'bestiary');
  assert.equal(packets.length, 1);
  assert.equal(packets[0].source, item);
  assert.equal(packets[0].actor, 'npc');
  assert.equal(packets[0].ordinal, 1);
  assert.equal(packets[0].context.json_path, '$.items[1]');
  assert.equal(packets[0].context.record_key, 'bestiary:a');
  assert.throws(() => equipmentPackets('{"type":"equipment","type":"weapon"}', 'a', 'a'), /Duplicate context/);
  assert.throws(() => equipmentPackets(item + ' trailing', 'a', 'a'), SyntaxError);
});

test('manifest sampling includes root and actor items, skips folder metadata, and fails on missing packs', async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'atlas-equipment-corpus-'));
  try {
    await mkdir(path.join(directory, 'static'));
    await mkdir(path.join(directory, 'packs/equipment'), { recursive: true });
    await mkdir(path.join(directory, 'packs/npc'));
    await writeFile(path.join(directory, 'static/system.json'), JSON.stringify({ packs: [
      { name: 'equipment', type: 'Item', path: 'packs/equipment' },
      { name: 'bestiary', type: 'Actor', path: 'packs/npc' },
    ] }));
    await writeFile(path.join(directory, 'packs/equipment/a.json'), '{"type":"equipment","system":{}}');
    await writeFile(path.join(directory, 'packs/equipment/_folders.json'), '[]');
    await writeFile(path.join(directory, 'packs/npc/a.json'), '{"type":"npc","items":[{"type":"equipment","system":{}}]}');
    const packets = [];
    for await (const packet of sampleEquipment(directory)) packets.push(packet);
    assert.equal(packets.length, 2);
    assert.equal(packets[0].ordinal, null);
    assert.equal(packets[1].ordinal, 0);
    await writeFile(path.join(directory, 'static/system.json'), '{"packs":[{"name":"missing","type":"Item","path":"packs/missing"}]}');
    await assert.rejects(async () => { for await (const _packet of sampleEquipment(directory)) { /* exhaust */ } }, /ENOENT/);
  } finally { await rm(directory, { recursive: true, force: true }); }
});
