import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import ts from 'typescript';
import { sampleItems, type ItemPacket } from './sample-items.js';

export interface RulePacket { key: string; source: string; context: ItemPacket['context'] }
/** Read direct system.rules for every sampled root/embedded Item, retaining raw tokens. */
export function rulePackets(item: ItemPacket): RulePacket[] {
  JSON.parse(item.source);
  const json = ts.parseJsonText(item.context.source_path, item.source);
  const expression = (json.statements[0] as ts.ExpressionStatement).expression;
  const member = (value: ts.Expression | undefined, name: string): ts.Expression | undefined => {
    if (!value || !ts.isObjectLiteralExpression(value)) throw new Error(`Expected object at ${item.context.json_path}: ${item.context.source_path}`);
    const matches = value.properties.filter((property): property is ts.PropertyAssignment => ts.isPropertyAssignment(property)
      && (ts.isStringLiteral(property.name) || ts.isIdentifier(property.name)) && property.name.text === name);
    if (matches.length > 1) throw new Error(`Duplicate ${name}: ${item.context.source_path}:${item.context.json_path}`);
    return matches[0]?.initializer;
  };
  const rules = member(member(expression, 'system'), 'rules');
  if (!rules) return [];
  if (!ts.isArrayLiteralExpression(rules)) throw new Error(`Expected rules array: ${item.context.source_path}:${item.context.json_path}`);
  return rules.elements.map((rule, index) => {
    const key = member(rule, 'key');
    if (!key || !ts.isStringLiteral(key)) throw new Error(`Expected rule key: ${item.context.source_path}:${item.context.json_path}.system.rules[${index}]`);
    return { key: key.text, source: item.source.slice(rule.getStart(json), rule.end),
      context: { ...item.context, json_path: `${item.context.json_path}.system.rules[${index}]` } };
  });
}
export async function* sampleRules(source: string): AsyncGenerator<RulePacket> {
  for await (const item of sampleItems(source)) yield* rulePackets(item);
}
async function main() {
  const { values } = parseArgs({ options: { source: { type: 'string' }, help: { type: 'boolean', short: 'h' } } });
  if (values.help) { console.log('Usage: npm --prefix scripts/source-contracts run sample-rules -- --source PATH'); return; }
  if (!values.source) throw new Error('--source is required');
  for await (const packet of sampleRules(path.resolve(process.env.INIT_CWD ?? process.cwd(), values.source)))
    if (!process.stdout.write(JSON.stringify(packet) + '\n')) await new Promise<void>(resolve => process.stdout.once('drain', resolve));
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { await main(); } catch (error) { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; }
}
