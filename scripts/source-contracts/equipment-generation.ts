import type { ExtractionSummary, GraphField, GraphNode, TypeGraph } from './contracts.js';

const selected = ['equipped', 'hp', 'price', 'usage'];
const physical = 'src/module/item/physical/data.ts#PhysicalSystemSource';
const equipment = 'src/module/item/equipment/data.ts#EquipmentSystemSource';

/** Bounded experiment input, not a complete equipment schema. */
export interface EquipmentGenerationInput {
  format: 'atlas-equipment-generation/v1';
  source: ExtractionSummary['source'];
  selection: { name: string; declaration: string; fields: GraphField[]; deferred: string[] }[];
  nodes: GraphNode[];
}

export function selectEquipmentInput(graph: TypeGraph, summary: ExtractionSummary): EquipmentGenerationInput {
  if (!graph.complete || !summary.complete) throw new Error('Complete declaration extraction is required');
  const nodes = new Map(graph.nodes.map(node => [node.id, node]));
  if (nodes.size !== graph.nodes.length) throw new Error('Duplicate graph node identities');
  const visited = new Set<string>();
  const visit = (ref: string) => {
    if (visited.has(ref)) return;
    visited.add(ref);
    const node = nodes.get(ref);
    if (!node) throw new Error(`Missing selected node: ${ref}`);
    if ('fields' in node) node.fields?.forEach(field => visit(field.ref));
    if ('members' in node) node.members.forEach(visit);
    if (node.kind === 'array') visit(node.element);
    if (node.kind === 'tuple') node.elements.forEach(element => visit(element.ref));
    if ('indexSignatures' in node) node.indexSignatures?.forEach(index => visit(index.value));
  };
  const selection = [physical, equipment].map((declaration, index) => {
    const node = nodes.get(declaration);
    if (!node || node.kind !== 'object') throw new Error(`Missing object declaration: ${declaration}`);
    const fields = selected.map(name => {
      const field = node.fields.find(field => field.name === name);
      if (!field) throw new Error(`Missing selected field: ${declaration}.${name}`);
      visit(field.ref);
      return field;
    });
    return { name: index === 0 ? 'PhysicalEquipmentFields' : 'EquipmentFields', declaration, fields,
      deferred: node.fields.filter(field => !selected.includes(field.name)).map(field => field.name) };
  });
  return { format: 'atlas-equipment-generation/v1', source: summary.source, selection,
    nodes: graph.nodes.filter(node => visited.has(node.id)).sort((a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : 0) };
}

const pascal = (name: string) => name.replace(/[^A-Za-z0-9]+/g, ' ').split(/\s+/)
  .filter(Boolean).map(word => word[0].toUpperCase() + word.slice(1)).join('');
const snake = (name: string) => name.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase();
const rustString = (text: string) => JSON.stringify(text).replace(/\\u([0-9a-f]{4})/gi, '\\u{$1}');

/** Emit only supported, reachable value shapes. SourcePresence retains pre-default states. */
export function generateEquipmentRust(input: EquipmentGenerationInput): string {
  if (input.format !== 'atlas-equipment-generation/v1') throw new Error('Unknown generation input format');
  const nodes = new Map(input.nodes.map(node => [node.id, node]));
  if (nodes.size !== input.nodes.length) throw new Error('Duplicate generation node identities');
  const declarations: string[] = [];
  const owners = new Map<string, { type: string; parser: string }>();
  const usedNames = new Set<string>();
  const lookup = (id: string): GraphNode => {
    const node = nodes.get(id);
    if (!node) throw new Error(`Missing generation node: ${id}`);
    return node;
  };
  const normalized = (id: string): GraphNode => {
    const node = lookup(id);
    if (node.kind !== 'union') return node;
    const members = node.members.filter(id => !['primitive:null', 'primitive:undefined'].includes(id));
    if (members.length === 1) return normalized(members[0]);
    if (members.length === 2 && members.every(id => lookup(id).kind === 'literal'
      && typeof (lookup(id) as GraphNode & { value: unknown }).value === 'boolean')) {
      return { id: 'primitive:boolean', kind: 'primitive', value: 'boolean' };
    }
    return { ...node, members };
  };
  const signature = (node: GraphNode, ancestors = new Set<string>()): string => {
    if (ancestors.has(node.id)) throw new Error(`Recursive selected shape is outside this trial: ${node.id}`);
    const next = new Set([...ancestors, node.id]);
    const child = (ref: string) => signature(normalized(ref), next);
    if (node.kind === 'object' || node.kind === 'intersection') {
      if (node.kind === 'object' && node.indexSignatures.length) throw new Error(`Selected index signature is unsupported: ${node.id}`);
      if (node.kind === 'intersection') for (const member of node.members) {
        const constituent = normalized(member);
        if (constituent.kind !== 'object' && constituent.kind !== 'intersection')
          throw new Error(`Unsupported selected intersection member: ${member}`);
        child(member); // Resolved fields do not encode index-signature constraints.
      }
      return JSON.stringify(node.fields.map(field => [field.name, field.forbidden, child(field.ref)]));
    }
    if (node.kind === 'union') return JSON.stringify(node.members.map(child).sort());
    if (node.kind === 'array') return `array:${child(node.element)}`;
    if (node.kind === 'primitive' || node.kind === 'literal') return JSON.stringify([node.kind, node.value]);
    throw new Error(`Unsupported selected construct: ${node.id} (${node.kind})`);
  };
  const allocate = (hint: string) => {
    const name = pascal(hint);
    if (!/^[A-Z][A-Za-z0-9]*$/.test(name) || usedNames.has(name)) throw new Error(`Rust name collision or invalid name: ${name}`);
    usedNames.add(name);
    return name;
  };
  const emit = (ref: string, hint: string): { type: string; parser: string } => {
    const node = normalized(ref);
    const key = signature(node);
    const existing = owners.get(key);
    if (existing) return existing;
    if (node.kind === 'primitive') {
      const primitives: Record<string, { type: string; parser: string }> = { string: { type: 'String', parser: 'string' }, number: { type: 'Number', parser: 'number' },
        boolean: { type: 'bool', parser: 'boolean' } };
      const value = primitives[node.value];
      if (!value) throw new Error(`Unsupported primitive: ${node.value}`);
      return value;
    }
    if (node.kind === 'array') throw new Error(`Selected arrays are outside this trial: ${node.id}`);
    const name = allocate(node.name ?? hint);
    const value = { type: name, parser: `read_${snake(name)}` };
    owners.set(key, value);
    if (node.kind === 'object' || node.kind === 'intersection') {
      const fields = node.fields.map(field => {
        if (field.forbidden) throw new Error(`Forbidden selected field is outside this trial: ${node.id}.${field.name}`);
        const child = emit(field.ref, name + pascal(field.name));
        return { field, child, rust: snake(field.name) };
      });
      const names = fields.map(field => field.rust);
      if (new Set(names).size !== names.length || names.includes('additional_fields')
        || names.some(name => !/^[a-z][a-z0-9_]*$/.test(name)
          || ['type', 'self', 'mod', 'use', 'ref', 'match', 'struct', 'enum', 'pub', 'fn', 'const', 'static', 'let', 'loop', 'move', 'crate', 'super', 'where', 'trait', 'impl', 'async', 'await', 'dyn', 'in', 'as', 'if', 'else', 'return', 'break', 'continue', 'mut', 'unsafe', 'extern', 'for', 'while'].includes(name)))
        throw new Error(`Invalid or unsupported Rust field names: ${node.id}`);
      declarations.push(`// Source declaration: ${node.id}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ${name} {
${fields.map(({ field, child, rust }) => `    // Declared optional=${field.optional}, nullable=${field.nullable}; retained before defaults.
    pub ${rust}: SourcePresence<${child.type}>,`).join('\n')}
    pub additional_fields: SourceObject,
}
fn ${value.parser}(v: &SourceValue, c: &SourceContext, p: &str) -> ParseResult<${name}> {
    let f = Fields::new(v, c, p)?;
    Ok(${name} {
${fields.map(({ field, child, rust }) => `        ${rust}: f.presence(${rustString(field.name)}, ${child.parser})?,`).join('\n')}
        additional_fields: f.remaining(&[${fields.map(({ field }) => rustString(field.name)).join(', ')}]),
    })
}`);
      return value;
    }
    const members = node.kind === 'union' ? node.members.map(lookup) : [node];
    if (!members.length) throw new Error(`Selected union has no supported value alternatives: ${node.id}`);
    if (members.every(member => member.kind === 'literal' && typeof member.value === 'string')) {
      const tokens = members.map(member => (member as GraphNode & { value: string }).value).sort();
      const variants = tokens.map(pascal);
      if (new Set(variants).size !== variants.length || variants.some(variant => !/^[A-Z][A-Za-z0-9]*$/.test(variant)))
        throw new Error(`Unrepresentable literal variants: ${node.id}`);
      declarations.push(`#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ${name} {
${tokens.map((token, index) => `    #[serde(rename = ${rustString(token)})]
    ${variants[index]},`).join('\n')}
}
fn ${value.parser}(v: &SourceValue, c: &SourceContext, p: &str) -> ParseResult<${name}> {
    match string(v, c, p)?.as_str() {
${tokens.map((token, index) => `        ${rustString(token)} => Ok(${name}::${variants[index]}),`).join('\n')}
        _ => Err(c.error(p, ${rustString(tokens.join(' | '))}, v)),
    }
}`);
      return value;
    }
    if (members.every(member => member.kind === 'literal' && typeof member.value === 'number')) {
      const numbers = members.map(member => (member as GraphNode & { value: number }).value);
      if (numbers.some(value => !Number.isSafeInteger(value))) throw new Error(`Unsafe numeric literal: ${node.id}`);
      value.type = 'Number';
      declarations.push(`fn ${value.parser}(v: &SourceValue, c: &SourceContext, p: &str) -> ParseResult<Number> {
    let value = number(v, c, p)?;
    if value.as_f64().is_some_and(|n| [${numbers.map(n => n.toFixed(1)).join(', ')}].contains(&n)) {
        Ok(value)
    } else {
        Err(c.error(p, ${rustString(numbers.join(' | '))}, v))
    }
}`);
      return value;
    }
    throw new Error(`Unsupported selected union: ${node.id}`);
  };
  const roots = input.selection.map(selection => {
    const id = `selection:${selection.name}`;
    nodes.set(id, { id, name: selection.name, kind: 'object', fields: selection.fields, indexSignatures: [] });
    const value = emit(id, selection.name);
    const alias = value.type === selection.name ? '' : `pub type ${selection.name} = ${value.type};\n`;
    return `// ${selection.declaration}; deferred fields: ${selection.deferred.join(', ')}
${alias}pub(super) fn parse_${snake(selection.name)}(v: &SourceValue, c: &SourceContext, p: &str) -> ParseResult<${selection.name}> {
    ${value.parser}(v, c, p)
}`;
  });
  return `// Generated by scripts/source-contracts/generate-equipment.ts; do not edit.
// Partial source model for PF2e ${input.source.system_version}; source digest ${input.source.source_digest}.
// SourcePresence intentionally preserves missing/null before Foundry defaults.
use serde::Serialize;
use serde_json::Number;
use super::presence::SourcePresence;
use super::value::{SourceObject, SourceValue};
use super::parse::{Fields, ParseResult, SourceContext, boolean, number, string};

${declarations.join('\n\n')}

${roots.join('\n\n')}
`;
}
