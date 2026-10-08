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
