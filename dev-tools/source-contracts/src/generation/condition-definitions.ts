import { createHash } from 'node:crypto';
import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';
import ts from 'typescript';
import { conditionValue, conditionPredicate, type ConditionValue, type ConditionPredicate } from './condition-expressions.js';
import { traitImmunities, type TraitImmunity, type ImmunityPredicate } from './trait-immunities.js';

export interface ConditionCoverage {
  adapters: { slug: string; ordinal: number; key: string; digest: string; owner: string }[];
  persistentGrants: { slug: string; ordinal: number; digest: string; owner: string }[];
  overrides: { from: string; to: string; owner: string }[];
}
export interface NativeModifier { ordinal: number; slug: string; selectors: string[]; type: string; value: ConditionValue; predicate: ConditionPredicate }
export interface NativeGrant { ordinal: number; target: string; inMemoryOnly?: boolean; allowDuplicate?: boolean; granter?: string; grantee?: string }
export interface NativeCondition {
  slug: string; id: string; name: string; valued: boolean; defaultValue: number | null;
  modifiers: NativeModifier[]; grants: NativeGrant[]; overrides: string[];
  hpLoss?: { value: ConditionValue; reevaluateOnUpdate: boolean };
  file: string; digest: string;
}
export interface ConditionCatalog { digest: string; conditions: NativeCondition[]; immunities: TraitImmunity[] }
const sha = (value: string | Uint8Array) => createHash('sha256').update(value).digest('hex');
const selectors = new Set(['perception','all','all-speeds','ac','reflex','cha-based','int-based','wis-based',
  'attack-roll','skill-check','con-based','hp','dex-based','saving-throw','str-based','str-damage']);
const object = (value: unknown): Record<string, unknown> => {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Expected condition object');
  return value as Record<string, unknown>;
};
function keys(value: Record<string, unknown>, allowed: string[]): void {
  if (Object.keys(value).some(key => !allowed.includes(key))) throw new Error('Unreviewed condition fields: ' + Object.keys(value).join(','));
}
function strings(value: unknown): string[] {
  if (!Array.isArray(value) || !value.every(v => typeof v === 'string')) throw new Error('Expected condition string array');
  return value;
}
function optionalBoolean(value: unknown): boolean | undefined {
  if (value !== undefined && typeof value !== 'boolean') throw new Error('Expected condition boolean');
  return value as boolean | undefined;
}
export async function conditionSourceInputs(source: string) {
  const directory = 'packs/pf2e/conditions';
  const files = await readdir(path.join(source, directory), { withFileTypes: true });
  if (!files.length || files.some(file => !file.isFile() || !file.name.endsWith('.json'))) throw new Error('Unsupported condition pack inventory');
  const names = ['system.pf2e.json', ...files.map(file => directory + '/' + file.name)].sort();
  const hash = createHash('sha256'), documents = [];
  for (const file of names) {
    const bytes = await readFile(path.join(source, file));
    hash.update(`${Buffer.byteLength(file)}:${file}:${bytes.length}:`); hash.update(bytes);
    documents.push({ file, bytes });
  }
  return { digest: hash.digest('hex'), documents };
}
function schemaInitial(text: string): string {
  const source = ts.createSourceFile('flat-modifier.ts', text, ts.ScriptTarget.Latest, true);
  const owner = source.statements.find(node => ts.isClassDeclaration(node) && node.name?.text === 'FlatModifierRuleElement');
  if (!owner || !ts.isClassDeclaration(owner)) throw new Error('FlatModifier class missing');
  const method = owner.members.find(node => ts.isMethodDeclaration(node) && node.name?.getText(source) === 'defineSchema');
  if (!method || !ts.isMethodDeclaration(method) || !method.body) throw new Error('FlatModifier schema missing');
  const result = method.body.statements.find(ts.isReturnStatement)?.expression;
  if (!result || !ts.isObjectLiteralExpression(result)) throw new Error('Unsupported FlatModifier schema');
  const field = result.properties.find(node => ts.isPropertyAssignment(node) && node.name.getText(source) === 'type');
  if (!field || !ts.isPropertyAssignment(field) || !ts.isNewExpression(field.initializer) || field.initializer.expression.getText(source) !== 'fields.StringField') throw new Error('FlatModifier type default missing');
  const options = field.initializer.arguments?.[0];
  if (!options || !ts.isObjectLiteralExpression(options)) throw new Error('FlatModifier type options missing');
  const initial = options.properties.find(node => ts.isPropertyAssignment(node) && node.name.getText(source) === 'initial');
  if (!initial || !ts.isPropertyAssignment(initial) || !ts.isStringLiteral(initial.initializer)) throw new Error('Unknown FlatModifier default');
  return initial.initializer.text;
}
export async function conditionDefinitions(source: string, knownTraits: ReadonlySet<string>, coverage: ConditionCoverage,
  inputs?: Awaited<ReturnType<typeof conditionSourceInputs>>): Promise<ConditionCatalog> {
  inputs ??= await conditionSourceInputs(source);
  const manifest = object(JSON.parse(inputs.documents.find(doc => doc.file === 'system.pf2e.json')!.bytes.toString()));
  const packs = manifest.packs;
  if (!Array.isArray(packs) || packs.filter(pack => pack.name === 'conditionitems' && pack.path === 'packs/conditions' && pack.type === 'Item').length !== 1)
    throw new Error('Condition pack mapping changed');
  const flatSource = await readFile(path.join(source, 'src/module/rules/rule-element/flat-modifier.ts'), 'utf8');
  const helperSource = await readFile(path.join(source, 'src/module/actor/creature/helpers.ts'), 'utf8');
  const defaultType = schemaInitial(flatSource);
  const raw = inputs.documents.filter(doc => doc.file !== 'system.pf2e.json').map(doc => ({ ...doc, data: object(JSON.parse(doc.bytes.toString())) }));
  const names = new Map<string, string>();
  for (const {data,file} of raw) {
    if (data.type !== 'condition' || typeof data._id !== 'string' || typeof data.name !== 'string') throw new Error('Invalid condition identity: ' + file);
    const slug = path.basename(file,'.json');
    if (names.has(data._id) || names.has(data.name)) throw new Error('Duplicate condition identity');
    names.set(data._id,slug); names.set(data.name,slug);
  }
  const seenAdapters = new Set<string>(), seenOverrides = new Set<string>(), seenPersistent = new Set<string>();
  const conditions: NativeCondition[] = raw.map(({data,file,bytes}) => {
    const slug = path.basename(file,'.json'), system = object(data.system), value = object(system.value);
    keys(value,['immutable','isValued','value']);
    if (typeof value.isValued !== 'boolean' || value.value !== null && !Number.isSafeInteger(value.value)) throw new Error('Unknown condition value metadata');
    const condition: NativeCondition = { slug, id: data._id as string, name: data.name as string,
      valued: value.isValued, defaultValue: value.value as number | null, modifiers: [], grants: [],
      overrides: strings(system.overrides), file, digest: sha(bytes) };
    for (const target of condition.overrides) {
      const identity = slug + ':' + target;
      if (!coverage.overrides.some(entry => entry.from === slug && entry.to === target && entry.owner)) throw new Error('Unreviewed condition override: ' + identity);
      seenOverrides.add(identity);
    }
    if (!Array.isArray(system.rules)) throw new Error('Unknown condition rule collection');
    system.rules.forEach((input: unknown, ordinal: number) => {
      const rule = object(input), kind = rule.key;
      if (kind === 'FlatModifier') {
        keys(rule,['key','selector','slug','type','value','predicate']);
        const domains = typeof rule.selector === 'string' ? [rule.selector] : strings(rule.selector);
        const type = rule.type ?? defaultType;
        if (!domains.length || domains.some(selector => !selectors.has(selector)) || !['status','circumstance','untyped'].includes(String(type)) || typeof rule.slug !== 'string')
          throw new Error('Unsupported condition modifier');
        condition.modifiers.push({ordinal, slug: rule.slug, selectors: domains, type: String(type), value: conditionValue(rule.value), predicate: conditionPredicate(rule.predicate ?? [])});
      } else if (kind === 'GrantItem') {
        keys(rule,['key','uuid','inMemoryOnly','allowDuplicate','onDeleteActions']);
        if (typeof rule.uuid !== 'string' || !rule.uuid.startsWith('Compendium.pf2e.conditionitems.Item.')) throw new Error('Unknown condition grant UUID');
        const target = names.get(rule.uuid.slice('Compendium.pf2e.conditionitems.Item.'.length));
        if (!target) throw new Error('Condition grant destination missing');
        const deletion = object(rule.onDeleteActions ?? {}); keys(deletion,['granter','grantee']);
        for (const action of Object.values(deletion)) if (!['restrict','detach','cascade'].includes(String(action))) throw new Error('Unknown grant deletion policy');
        if (rule.inMemoryOnly !== true) {
          const reviewed = coverage.persistentGrants.find(entry => entry.slug === slug && entry.ordinal === ordinal);
          if (!reviewed || !reviewed.owner || reviewed.digest !== sha(JSON.stringify(rule))) throw new Error('Unreviewed persistent grant lifetime: ' + slug + ':' + ordinal);
          seenPersistent.add(slug+':'+ordinal);
        }
        condition.grants.push({ordinal,target,inMemoryOnly:optionalBoolean(rule.inMemoryOnly),allowDuplicate:optionalBoolean(rule.allowDuplicate),
          granter:deletion.granter as string | undefined,grantee:deletion.grantee as string | undefined});
      } else if (kind === 'LoseHitPoints') {
        keys(rule,['key','reevaluateOnUpdate','value']);
        // The encounter event planner deliberately supports Drained, not arbitrary HP events.
        if (slug !== 'drained' || condition.hpLoss || typeof rule.reevaluateOnUpdate !== 'boolean') throw new Error('Unsupported condition HP loss');
        condition.hpLoss={value:conditionValue(rule.value),reevaluateOnUpdate:rule.reevaluateOnUpdate};
      } else {
        const reviewed = coverage.adapters.find(entry => entry.slug === slug && entry.ordinal === ordinal);
        if (!reviewed || reviewed.key !== kind || reviewed.digest !== sha(JSON.stringify(rule)) || !reviewed.owner)
          throw new Error('Unreviewed condition adapter: ' + slug + ':' + ordinal + ':' + kind);
        seenAdapters.add(slug+':'+ordinal);
      }
    });
    return condition;
  }).sort((a,b)=>a.slug < b.slug ? -1 : a.slug > b.slug ? 1 : 0);
  if (seenAdapters.size !== coverage.adapters.length || seenOverrides.size !== coverage.overrides.length || seenPersistent.size !== coverage.persistentGrants.length) throw new Error('Stale condition coverage entries');
  const bySlug = new Map(conditions.map(condition => [condition.slug,condition]));
  const visit = (slug: string, active: Set<string>) => {
    if (active.has(slug)) throw new Error('Cyclic condition grant');
    const condition = bySlug.get(slug); if (!condition) throw new Error('Unknown condition target: ' + slug);
    const next = new Set(active).add(slug);
    for (const grant of condition.grants) visit(grant.target,next);
    for (const target of condition.overrides) if (!bySlug.has(target)) throw new Error('Unknown override destination');
  };
  for (const condition of conditions) visit(condition.slug,new Set());
  const immunities = traitImmunities(helperSource,knownTraits);
  return { digest: sha(JSON.stringify({input:inputs.digest, flat:sha(flatSource), helper:sha(helperSource),coverage})),conditions,immunities };
}

const pascal = (text: string) => text.split(/[^a-zA-Z0-9]+/).map(part=>part[0].toUpperCase()+part.slice(1)).join('');
const quote = (value: unknown): string => JSON.stringify(value);
function expression(value: ConditionValue): string {
  return value.kind === 'fixed' ? `ConditionValue::Fixed(${value.value})` : value.kind === 'magnitude' ? `ConditionValue::Magnitude(${value.coefficient})`
    : `ConditionValue::LevelMagnitude { coefficient: ${value.coefficient}, minimum_level: ${value.minimumLevel} }`;
}
function predicate(value: ConditionPredicate): string {
  return value.kind === 'atom' ? `ConditionPredicate::Atom(${quote(value.value)})` : value.kind === 'not' ? `ConditionPredicate::Not(&${predicate(value.term)})`
    : `ConditionPredicate::${value.kind === 'all' ? 'All' : 'Any'}(&[${value.terms.map(predicate).join(',')}])`;
}
function immunityPredicate(value: ImmunityPredicate): string {
  if (value.kind === 'not') return `ImmunityPredicate::Not(&${immunityPredicate(value.term)})`;
  if (value.kind === 'all' || value.kind === 'any') return `ImmunityPredicate::${value.kind === 'all' ? 'All' : 'Any'}(&[${value.terms.map(immunityPredicate).join(',')}])`;
  return `ImmunityPredicate::${value.kind === 'trait' ? 'Trait' : value.kind === 'actorKind' ? 'ActorKind' : 'Authored'}(${quote((value as { value: string }).value)})`;
}
export function generateConditionModule(catalog: ConditionCatalog, header: string, source: {repository: string; commit: string}): string {
  const optional = (value: unknown) => value === undefined || value === null ? 'None' : `Some(${typeof value === 'string' ? quote(value) : value})`;
  const id = (slug: string) => 'ConditionId::'+pascal(slug);
  return `${header}
// Derived factual metadata from ${source.repository.replace(/\.git$/,'')}/tree/${source.commit}/packs/pf2e/conditions.
// Engine behavior: src/module/actor/creature/helpers.ts.
// Upstream system code: Copyright 2019 Hooking and contributors; Apache-2.0.
// License: ${source.repository.replace(/\.git$/,'')}/blob/${source.commit}/LICENSE.
// Compendium facts retain their upstream provenance; rule descriptions are not copied.
pub const CONDITION_CATALOG_ID: &str = ${quote(catalog.digest)};
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConditionId { ${catalog.conditions.map(c=>pascal(c.slug)).join(',')} }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionValue { Fixed(i64), Magnitude(i64), LevelMagnitude { coefficient: i64, minimum_level: i64 } }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionModifierType { Status, Circumstance, Untyped }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionPredicate { Atom(&'static str), All(&'static [Self]), Any(&'static [Self]), Not(&'static Self) }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConditionModifier { pub ordinal: usize, pub slug: &'static str, pub selectors: &'static [&'static str], pub modifier_type: ConditionModifierType, pub value: ConditionValue, pub predicate: ConditionPredicate }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConditionGrant { pub ordinal: usize, pub target: ConditionId, pub in_memory_only: Option<bool>, pub allow_duplicate: Option<bool>, pub on_delete_granter: Option<&'static str>, pub on_delete_grantee: Option<&'static str> }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConditionHpLoss { pub value: ConditionValue, pub reevaluate_on_update: bool }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConditionDefinition { pub id: ConditionId, pub record_key: &'static str, pub slug: &'static str, pub name: &'static str, pub has_value: bool, pub default_value: Option<i64>, pub modifiers: &'static [ConditionModifier], pub grants: &'static [ConditionGrant], pub overrides: &'static [ConditionId], pub hp_loss: Option<ConditionHpLoss>, pub source_file: &'static str, pub source_digest: &'static str }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImmunityPredicate { Trait(&'static str), ActorKind(&'static str), Authored(&'static str), All(&'static [Self]), Any(&'static [Self]), Not(&'static Self) }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraitImmunity { pub immunity: &'static str, pub predicate: ImmunityPredicate, pub source_label_key: &'static str, pub source_line: usize }
pub static CONDITION_DEFINITIONS: &[ConditionDefinition] = &[
${catalog.conditions.map(c=>`ConditionDefinition { id: ${id(c.slug)}, record_key: ${quote('conditionitems:'+c.id)}, slug: ${quote(c.slug)}, name: ${quote(c.name)}, has_value: ${c.valued}, default_value: ${optional(c.defaultValue)},
modifiers: &[${c.modifiers.map(m=>`ConditionModifier { ordinal:${m.ordinal}, slug:${quote(m.slug)}, selectors:&[${m.selectors.map(quote).join(',')}], modifier_type:ConditionModifierType::${pascal(m.type)}, value:${expression(m.value)}, predicate:${predicate(m.predicate)} }`).join(',')}],
grants: &[${c.grants.map(g=>`ConditionGrant { ordinal:${g.ordinal}, target:${id(g.target)}, in_memory_only:${optional(g.inMemoryOnly)}, allow_duplicate:${optional(g.allowDuplicate)}, on_delete_granter:${optional(g.granter)}, on_delete_grantee:${optional(g.grantee)} }`).join(',')}],
overrides: &[${c.overrides.map(id).join(',')}], hp_loss:${c.hpLoss ? `Some(ConditionHpLoss {value:${expression(c.hpLoss.value)},reevaluate_on_update:${c.hpLoss.reevaluateOnUpdate}})` : 'None'},source_file:${quote(c.file)},source_digest:${quote(c.digest)} }`).join(',\n')}
];
impl ConditionId {
    pub fn definition(self) -> &'static ConditionDefinition { match self { ${catalog.conditions.map((c,i)=>`Self::${pascal(c.slug)} => &CONDITION_DEFINITIONS[${i}]`).join(',')} } }
    pub fn from_record_key(key: &str) -> Option<Self> { CONDITION_DEFINITIONS.iter().find(|entry| entry.record_key == key).map(|entry| entry.id) }
}
pub static TRAIT_IMMUNITIES: &[TraitImmunity] = &[
${catalog.immunities.map(rule=>`TraitImmunity { immunity:${quote(rule.type)},predicate:${immunityPredicate(rule.predicate)},source_label_key:${quote(rule.labelKey)},source_line:${rule.line} }`).join(',\n')}
];
`;
}
