import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';
import { itemPackets, type ItemPacket } from './sample-items.js';

export interface DocumentPacket extends ItemPacket { key: 'Actor' | 'Item' | 'JournalEntry' | 'Macro' | 'RollTable' }

/** Root Actors and every Item occurrence, retaining exact authored JSON bytes. */
export function documentPackets(text: string, sourcePath: string, pack: string, kind: 'Actor' | 'Item'): DocumentPacket[] {
  const root = JSON.parse(text) as { _id?: unknown; type?: unknown; system?: unknown };
  const items = itemPackets(text, sourcePath, pack, kind).map(packet => ({ ...packet, key: 'Item' as const }));
  if (kind === 'Item') return items;
  if (typeof root._id !== 'string' || typeof root.type !== 'string' || !root.system || typeof root.system !== 'object')
    throw new Error(`Expected root Actor source with _id/type/system: ${sourcePath}`);
  // Context is decoded here, but the probe parses the untouched document text.
  return [{ key: 'Actor', source: text, family: root.type,
    context: { record_key: `${pack}:${root._id}`, source_path: sourcePath, json_path: '$' } }, ...items];
}
export async function* sampleDocuments(source: string, includeOtherKinds = false): AsyncGenerator<DocumentPacket> {
  const manifest = JSON.parse(await readFile(path.join(source, 'static/system.json'), 'utf8')) as { packs: { type: string; path: string; name: string }[] };
  const files = async (directory: string): Promise<string[]> => {
    const result: string[] = [];
    for (const entry of (await readdir(directory, { withFileTypes: true })).sort((a,b) => a.name.localeCompare(b.name))) {
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) result.push(...await files(file));
      else if (entry.isFile() && entry.name.endsWith('.json') && entry.name !== '_folders.json') result.push(file);
    }
    return result;
  };
  for (const pack of manifest.packs.filter(pack => ['Actor','Item'].includes(pack.type) || includeOtherKinds)) {
    if (!['Actor', 'Item', 'JournalEntry', 'Macro', 'RollTable'].includes(pack.type)) throw new Error(`Unmodeled document kind: ${pack.type}`);
    for (const file of await files(path.join(source, pack.path))) {
      const text = await readFile(file, 'utf8'), sourcePath = path.relative(source, file).replaceAll('\\', '/');
      if (['Actor', 'Item'].includes(pack.type)) yield* documentPackets(text, sourcePath, pack.name, pack.type as 'Actor' | 'Item');
      else {
        const root = JSON.parse(text) as { _id?: unknown };
        if (typeof root._id !== 'string') throw new Error(`Expected document _id: ${sourcePath}`);
        yield { key: pack.type as DocumentPacket['key'], family: pack.type, source: text,
          context: { record_key: `${pack.name}:${root._id}`, source_path: sourcePath, json_path: '$' } };
      }
    }
  }
}
