import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import ts from 'typescript';

export interface EquipmentPacket {
  source: string; actor: string | null; ordinal: number | null;
  context: { record_key: string; source_path: string; json_path: string };
}
/** AST spans preserve authored number tokens and repeated members in payloads. */
export function equipmentPackets(text: string, sourcePath: string, pack: string): EquipmentPacket[] {
  JSON.parse(text); // Syntax only; this discarded object never supplies payload values.
  const json = ts.parseJsonText(sourcePath, text);
  const statement = json.statements[0];
  if (!statement || !ts.isExpressionStatement(statement) || !ts.isObjectLiteralExpression(statement.expression))
    throw new Error(`Expected source document object: ${sourcePath}`);
  const member = (object: ts.ObjectLiteralExpression, key: string): ts.Expression | undefined => {
    const properties = object.properties.filter((property): property is ts.PropertyAssignment =>
      ts.isPropertyAssignment(property) && (ts.isStringLiteral(property.name) || ts.isIdentifier(property.name))
      && property.name.text === key);
    if (properties.length > 1) throw new Error(`Duplicate context member ${key}: ${sourcePath}`);
    return properties[0]?.initializer;
  };
  const string = (value: ts.Expression | undefined) => value && ts.isStringLiteral(value) ? value.text : null;
  const document = statement.expression;
  const items = member(document, 'items');
  const actor = items ? string(member(document, 'type')) : null;
  if (items && !ts.isArrayLiteralExpression(items)) throw new Error(`Actor items must be an array: ${sourcePath}`);
  const candidates = items && ts.isArrayLiteralExpression(items) ? [...items.elements] : [document];
  return candidates.flatMap((item, index) => {
    if (!ts.isObjectLiteralExpression(item)) throw new Error(`Expected embedded Item object: ${sourcePath}`);
    if (string(member(item, 'type')) !== 'equipment') return [];
    return [{ source: text.slice(item.getStart(json), item.end), actor, ordinal: items ? index : null,
      context: { record_key: `${pack}:${string(member(item, '_id')) ?? sourcePath}`,
        source_path: sourcePath, json_path: items ? `$.items[${index}]` : '$' } }];
  });
}

export async function* sampleEquipment(source: string): AsyncGenerator<EquipmentPacket> {
  const manifest = JSON.parse(await readFile(path.join(source, 'static/system.json'), 'utf8')) as {
    packs: { type: string; path: string; name: string }[];
  };
  async function files(directory: string): Promise<string[]> {
    const entries = await readdir(directory, { withFileTypes: true });
    const result: string[] = [];
    for (const entry of entries.sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0)) {
      const target = path.join(directory, entry.name);
      if (entry.isDirectory()) result.push(...await files(target));
      else if (entry.isFile() && entry.name.endsWith('.json') && entry.name !== '_folders.json') result.push(target);
    }
    return result;
  }
  for (const pack of manifest.packs.filter(pack => ['Item', 'Actor'].includes(pack.type))) {
    for (const file of await files(path.join(source, pack.path))) {
      yield* equipmentPackets(await readFile(file, 'utf8'), path.relative(source, file).replaceAll('\\', '/'), pack.name);
    }
  }
}

async function main() {
  const { values } = parseArgs({ options: { source: { type: 'string' }, help: { type: 'boolean', short: 'h' } } });
  if (values.help) { console.log('Usage: npm --prefix scripts/source-contracts run sample-equipment -- --source PATH'); return; }
  if (!values.source) throw new Error('--source is required');
  const source = path.resolve(process.env.INIT_CWD ?? process.cwd(), values.source);
  for await (const packet of sampleEquipment(source)) {
    if (!process.stdout.write(JSON.stringify(packet) + '\n')) await new Promise<void>(resolve => process.stdout.once('drain', resolve));
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { await main(); } catch (error) { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; }
}
