import { createHash } from 'node:crypto';
import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';
import ts from 'typescript';
import { createCatalogReader, type Member, type Value } from '../discovery/catalog-values.js';
import type { TraitCatalog } from '../contracts.js';

export interface ReferenceDefinition {
  namespace: string; identifier: string; labelKey: string | null; descriptionKey: string | null;
  displayLabel: string | null; recordKey: string | null; expectedFamily: string | null;
  requiredLocalizationKey: string | null; sourceFile: string; sourceLine: number;
  excluded: boolean;
}
export interface ReferenceCatalog {
  digest: string; definitions: ReferenceDefinition[];
  senseRules: { sense: string; fixedAcuity: string | null; unlimitedRange: boolean }[];
}
export interface ReferenceRegistry {
  schema: 'reference-definition-registry/v1'; source_package: 'pf2e';
  sense_mappings: { identifier: string; record_key: string; expected_family: string; baseline_localization_key: string }[];
  standalone_labels: Record<string, string>; pf2e_weapon_groups: string[];
  npc_mappings: { localization_key: string; record_key: string; expected_family: string; source_file: string }[];
  wildsong: { record_key: string; expected_family: string; source_file: string };
}
const sha = (value: string | Uint8Array) => createHash('sha256').update(value).digest('hex');
const compare = (a: string, b: string) => a < b ? -1 : a > b ? 1 : 0;
const object = (value: unknown): Record<string, unknown> => {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Expected reference metadata object');
  return value as Record<string, unknown>;
};
function canonical(value: unknown): string {
  if (Array.isArray(value)) return '[' + value.map(canonical).join(',') + ']';
  if (value && typeof value === 'object') return '{' + Object.entries(value).sort(([a], [b]) => compare(a, b))
    .map(([key, entry]) => JSON.stringify(key) + ':' + canonical(entry)).join(',') + '}';
  return JSON.stringify(value);
}
export const referencePackFolders = ['bestiary-ability-glossary-srd', 'bestiary-family-ability-glossary'];
export const referenceAdditionalPaths = [
  ...referencePackFolders.map(folder => 'packs/pf2e/' + folder),
  'packs/pf2e/class-features/wildsong.json',
  'packs/sf2e/bestiary-ability-glossary-srd/hover.json',
  'packs/sf2e/bestiary-ability-glossary-srd/no-breath.json',
];
export const referenceSourceFiles = ['system.pf2e.json', 'static/lang/en.json',
  'src/scripts/config/index.ts', 'src/scripts/config/traits.ts', 'src/scripts/config/damage.ts', 'src/scripts/config/iwr.ts',
  'src/module/actor/creature/values.ts', 'src/module/actor/creature/types.ts',
  'src/module/item/weapon/values.ts', 'src/module/item/deity/types.ts',
  ...referenceAdditionalPaths.filter(name => name.endsWith('.json'))];

/** A separate source identity: adding reference evidence does not change the DTO codec. */
export async function referenceSourceInputs(source: string) {
  const names = [...referenceSourceFiles];
  const walk = async (directory: string): Promise<void> => {
    const entries = await readdir(path.join(source, directory), { withFileTypes: true });
    if (!entries.length) throw new Error('Empty reference pack input: ' + directory);
    for (const entry of entries) {
      const file = directory + '/' + entry.name;
      if (entry.isDirectory()) await walk(file);
      else if (entry.isFile() && entry.name.endsWith('.json')) names.push(file);
      else throw new Error('Unsupported reference pack input: ' + file);
    }
  };
  for (const folder of referencePackFolders) await walk('packs/pf2e/' + folder);
  const hash = createHash('sha256'), documents = [];
  for (const file of names.sort(compare)) {
    const bytes = await readFile(path.join(source, file));
    hash.update(`${Buffer.byteLength(file)}:${file}:${bytes.length}:`); hash.update(bytes);
    documents.push({ file, bytes });
  }
  return { digest: hash.digest('hex'), documents };
}

/** Exact declarations and reviewed associations, without compiling localized prose. */
export async function referenceDefinitions(source: string, registry: ReferenceRegistry, traits: TraitCatalog,
  inputs?: Awaited<ReturnType<typeof referenceSourceInputs>>): Promise<ReferenceCatalog> {
  inputs ??= await referenceSourceInputs(source);
  if (registry.schema !== 'reference-definition-registry/v1' || registry.source_package !== 'pf2e')
    throw new Error('Unsupported reference registry');
  const documents = new Map(inputs.documents.map(document => [document.file, document.bytes]));
  const json = (file: string) => object(JSON.parse(documents.get(file)!.toString()));
  const manifest = json('system.pf2e.json');
  if (manifest.id !== 'pf2e' || !Array.isArray(manifest.packs)) throw new Error('Expected PF2e reference manifest');
  const en = json('static/lang/en.json');
  const locate = (key: string): unknown => key.split('.').reduce<unknown>((value, part) => object(value)[part], en);
  const localizationSource = ts.parseJsonText('static/lang/en.json', documents.get('static/lang/en.json')!.toString());
  const localizationLines = new Map<string, number>();
  const recordLines = (node: ts.Node, parts: string[] = []): void => {
    if (ts.isObjectLiteralExpression(node)) {
      const names = new Set<string>();
      for (const property of node.properties) {
        if (!ts.isPropertyAssignment(property) || !ts.isStringLiteral(property.name) || names.has(property.name.text))
          throw new Error('Repeated or unsupported localization property');
        names.add(property.name.text);
        const key = [...parts, property.name.text];
        localizationLines.set(key.join('.'), localizationSource.getLineAndCharacterOfPosition(property.initializer.getStart(localizationSource)).line + 1);
        recordLines(property.initializer, key);
      }
    }
  };
  const jsonRoot = localizationSource.statements[0];
  if (!jsonRoot || !ts.isExpressionStatement(jsonRoot)) throw new Error('Invalid localization AST');
  recordLines(jsonRoot.expression);
  const selected = new Map<string, { key: string; family: string; file: string; data: Record<string, unknown>; description: string }>();
  for (const folder of referencePackFolders) {
    const packs = manifest.packs.filter(pack => path.posix.basename(String(object(pack).path)) === folder).map(object);
    if (packs.length !== 1 || packs[0].type !== 'Item' || packs[0].system !== 'pf2e' || typeof packs[0].name !== 'string')
      throw new Error('Reference pack mapping changed: ' + folder);
    for (const [file, bytes] of documents) if (file.startsWith('packs/pf2e/' + folder + '/') && !file.endsWith('/_folders.json')) {
      const data = object(JSON.parse(bytes.toString())), family = 'action';
      if (typeof data._id !== 'string' || !/^[A-Za-z0-9]{16}$/.test(data._id) || data.type !== family)
        throw new Error('Invalid reference record identity: ' + file);
      const key = packs[0].name + ':' + data._id;
      if (selected.has(key)) throw new Error('Duplicate reference record identity: ' + key);
      const description = object(object(data.system).description).value;
      if (typeof description !== 'string') throw new Error('Missing reference record description: ' + file);
      selected.set(key, { key, family, file, data, description });
    }
  }
  for (const pack of manifest.packs.map(object)) if (String(pack.path).includes('glossary')
    && !referencePackFolders.includes(path.posix.basename(String(pack.path)))) throw new Error('Unreviewed glossary pack: ' + pack.path);
  const reader = createCatalogReader(source, { systemId: 'pf2e', labelKeyPolicy: 'ascii_words' }), config = reader.load('src/scripts/config/index.ts');
  const values = reader.load('src/module/actor/creature/values.ts');
  const get = (file: ReturnType<typeof reader.load>, name: string) => reader.variable(name, file, new Map());
  const map = (file: ReturnType<typeof reader.load>, name: string) => reader.object(get(file, name));
  const definitions: ReferenceDefinition[] = [];
  const add = (namespace: string, identifier: string, labelKey: string | null, descriptionKey: string | null,
    file: string, line: number, extra: Partial<ReferenceDefinition> = {}) => {
    if (labelKey !== null && typeof locate(labelKey) !== 'string') throw new Error('Missing reference label: ' + labelKey);
    if (descriptionKey !== null && typeof locate(descriptionKey) !== 'string') throw new Error('Missing reference description: ' + descriptionKey);
    definitions.push({ namespace, identifier, labelKey, descriptionKey, displayLabel: null, recordKey: null,
      expectedFamily: null, requiredLocalizationKey: null, sourceFile: file, sourceLine: line, excluded: false, ...extra });
  };
  const labelMap = (namespace: string, entries: Map<string, Member>) => {
    for (const [identifier, member] of entries) {
      if (typeof member.value !== 'string') throw new Error('Reference label mapping is not a string: ' + identifier);
      const location = reader.location(member.node);
      add(namespace, identifier, member.value || null, null, location.file, location.line);
    }
  };
  for (const rarity of traits.catalogs.filter(catalog => catalog.namespace === 'rarity')) {
    if (!rarity.complete) throw new Error('Incomplete rarity catalog');
    for (const entry of rarity.entries) add('rarity', entry.identifier, entry.labelKey, null, entry.source.file, entry.source.line);
  }
  const senses = reader.strings(get(values, 'SENSE_TYPES'));
  const unlimitedRange = reader.strings(get(values, 'SENSES_WITH_UNLIMITED_RANGE'));
  const fixedAcuities = map(values, 'SENSES_WITH_MANDATORY_ACUITIES');
  const acuities = reader.strings(get(values, 'SENSE_ACUITIES'));
  if ([...unlimitedRange, ...fixedAcuities.keys()].some(sense => !senses.includes(sense)))
    throw new Error('Sense rule refers to an undeclared sense');
  const senseRules = senses.map(sense => {
    const fixedAcuity = fixedAcuities.get(sense)?.value ?? null;
    if (fixedAcuity !== null && (typeof fixedAcuity !== 'string' || !acuities.includes(fixedAcuity)))
      throw new Error('Invalid fixed acuity for ' + sense);
    return { sense, fixedAcuity, unlimitedRange: unlimitedRange.includes(sense) };
  });
  const senseLabels = map(config, 'senses');
  if (senseLabels.size !== senses.length || senses.some(token => !senseLabels.has(token))) throw new Error('Sense catalog mismatch');
  for (const token of senses) {
    const mapping = registry.sense_mappings.filter(mapping => mapping.identifier === token);
    if (mapping.length > 1) throw new Error('Duplicate sense association');
    const selectedMapping = mapping[0], record = selectedMapping && selected.get(selectedMapping.record_key);
    if (selectedMapping && (!record || record.family !== selectedMapping.expected_family
      || !record.description.includes(`@Localize[${selectedMapping.baseline_localization_key}]`))) throw new Error('Reviewed sense target changed: ' + token);
    const member = senseLabels.get(token)!, location = reader.location(member.node);
    if (typeof member.value !== 'string') throw new Error('Invalid sense label');
    add('sense', token, member.value, null, location.file, location.line, selectedMapping ? {
      recordKey: selectedMapping.record_key, expectedFamily: selectedMapping.expected_family,
    } : {});
  }
  if (registry.sense_mappings.some(mapping => !senses.includes(mapping.identifier))) throw new Error('Reviewed sense token removed');
  for (const token of ['vision', 'hearing']) {
    const key = 'PF2E.Actor.Creature.Sense.Type.' + token[0].toUpperCase() + token.slice(1);
    add('sense', token, key, null, 'static/lang/en.json', localizationLines.get(key)!, { excluded: true });
  }
  const byRarity = reader.object(map(values, 'LANGUAGES_BY_RARITY').get('pf2e')?.value);
  const languageRecipe = values.declarations.get('LANGUAGES')?.getText(values.file).replace(/\s+/g, '');
  if (languageRecipe !== '["common",...R.values(LANGUAGES_BY_RARITY[SYSTEM_ID]).flat()]'
    || values.imports.get('R')?.module !== 'remeda' || values.imports.get('R')?.name !== '*')
    throw new Error('PF2e language selection recipe changed');
  const languages = ['common', ...[...byRarity.values()].flatMap(member => reader.strings(member.value))];
  if (new Set(languages).size !== languages.length) throw new Error('Duplicate PF2e language token');
  const languageDeclaration = values.declarations.get('LANGUAGES_BY_RARITY')!;
  for (const token of languages) add('language', token, 'PF2E.Actor.Creature.Language.' + token, null,
    'src/module/actor/creature/values.ts', reader.location(languageDeclaration).line);
  const wildsong = registry.wildsong, feature = json(wildsong.source_file), featureSystem = object(feature.system);
  if (feature.type !== wildsong.expected_family || wildsong.record_key !== 'classfeatures:' + feature._id
    || !Array.isArray(object(object(featureSystem.subfeatures).languages).granted)
    || !(object(object(featureSystem.subfeatures).languages).granted as unknown[]).includes('wildsong'))
    throw new Error('Reviewed Wildsong feature changed');
  const wildsongEntry = definitions.find(definition => definition.namespace === 'language' && definition.identifier === 'wildsong');
  if (!wildsongEntry) throw new Error('Wildsong language missing');
  wildsongEntry.recordKey = wildsong.record_key; wildsongEntry.expectedFamily = wildsong.expected_family;

  const domainTypes = reader.load('src/module/item/deity/types.ts');
  const domainType = domainTypes.file.statements.find(node => ts.isTypeAliasDeclaration(node) && node.name.text === 'DeityDomain');
  const domainPath = (node: ts.TypeNode): string[] | null => {
    if (!ts.isTypeReferenceNode(node) || node.typeName.getText() !== 'Lowercase' || node.typeArguments?.length !== 1) return null;
    const keys = node.typeArguments[0];
    if (!ts.isTypeOperatorNode(keys) || keys.operator !== ts.SyntaxKind.KeyOfKeyword) return null;
    const parts: string[] = []; let target = keys.type;
    while (true) {
      if (ts.isParenthesizedTypeNode(target)) { target = target.type; continue; }
      if (!ts.isIndexedAccessTypeNode(target)) break;
      if (!ts.isLiteralTypeNode(target.indexType) || !ts.isStringLiteral(target.indexType.literal)) return null;
      parts.unshift(target.indexType.literal.text); target = target.objectType;
    }
    if (!ts.isTypeQueryNode(target) || !ts.isIdentifier(target.exprName)) return null;
    const imported = domainTypes.file.statements.some(statement => ts.isImportDeclaration(statement)
      && statement.importClause?.name?.text === target.exprName.getText()
      && ts.isStringLiteral(statement.moduleSpecifier) && statement.moduleSpecifier.text === '../../../../static/lang/en.json');
    return imported ? parts : null;
  };
  if (!domainType || !ts.isTypeAliasDeclaration(domainType)
    || JSON.stringify(domainPath(domainType.type)) !== JSON.stringify(['PF2E', 'Item', 'Deity', 'Domain']))
    throw new Error('Authored deity domain identity changed');
  for (const [key, value] of Object.entries(object(locate('PF2E.Item.Deity.Domain')))) {
    const entry = object(value);
    if (typeof entry.Label !== 'string' || typeof entry.Description !== 'string') throw new Error('Incomplete domain definition');
    add('domain', key.toLowerCase(), `PF2E.Item.Deity.Domain.${key}.Label`, `PF2E.Item.Deity.Domain.${key}.Description`,
      'src/module/item/deity/types.ts', reader.location(domainType).line);
  }
  const prefix = 'PF2E.NPC.Abilities.Glossary.';
  const reviewedNpc = new Map(registry.npc_mappings.map(mapping => [mapping.localization_key, mapping]));
  if (reviewedNpc.size !== registry.npc_mappings.length) throw new Error('Duplicate reviewed NPC mapping');
  for (const mapping of reviewedNpc.values()) {
    const target = selected.get(mapping.record_key);
    if (!target || target.family !== mapping.expected_family || target.file !== mapping.source_file
      || !target.file.startsWith('packs/pf2e/bestiary-family-ability-glossary/')
      || !target.description.includes(`@Localize[${mapping.localization_key}]`)
      || !Object.hasOwn(object(locate('PF2E.NPC.Abilities.Glossary')), mapping.localization_key.slice(prefix.length)))
      throw new Error('Reviewed family NPC mapping changed: ' + mapping.localization_key);
  }
  for (const [suffix, value] of Object.entries(object(locate('PF2E.NPC.Abilities.Glossary')))) {
    if (typeof value !== 'string') throw new Error('Unexpected NPC glossary shape');
    const key = prefix + suffix;
    const wrappers = [...selected.values()].filter(record => record.file.startsWith('packs/pf2e/bestiary-ability-glossary-srd/')
      && record.description.includes(`@Localize[${key}]`));
    const reviewed = reviewedNpc.get(key);
    if (reviewed) wrappers.push(selected.get(reviewed.record_key)!);
    if (wrappers.length > 1) throw new Error('Ambiguous NPC glossary wrapper: ' + key);
    const record = wrappers[0], excluded = ['Hover', 'NoBreath'].includes(suffix);
    if (excluded) {
      const file = suffix === 'Hover' ? 'hover.json' : 'no-breath.json';
      const sf = json('packs/sf2e/bestiary-ability-glossary-srd/' + file);
      const sfSystem = object(sf.system);
      if (!String(object(sfSystem.description).value).includes(`@Localize[${key}]`)
        || object(sfSystem.publication).title !== 'Starfinder Alien Core') throw new Error('SF2e glossary exclusion evidence changed');
    }
    if (!record && !registry.standalone_labels[suffix]) throw new Error('Unreviewed standalone NPC label: ' + suffix);
    if (record && (typeof record.data.name !== 'string' || !record.data.name.trim())) throw new Error('Missing NPC wrapper title');
    add('rule', 'pf2e:' + key, null, key, 'static/lang/en.json', localizationLines.get(key)!, {
      displayLabel: record ? record.data.name as string : registry.standalone_labels[suffix], excluded,
      recordKey: record?.key ?? null, expectedFamily: record?.family ?? null,
      requiredLocalizationKey: record ? key : null,
    });
  }
  for (const [namespace, name] of [
    ['ability', 'abilities'],
    ['size', 'sizeTypes'],
    ['armor_group', 'armorGroups'],
  ] as const) labelMap(namespace, map(config, name));
  const skills = map(config, 'skills');
  const skillLabels = new Map([...skills].map(([key, member]) => [key, {
    value: reader.object(member.value).get('label')?.value, node: member.node,
  }]));
  labelMap('skill', skillLabels);
  labelMap('tradition', map(reader.load('src/scripts/config/traits.ts'), 'magicTraditions'));
  labelMap('damage_type', map(reader.load('src/scripts/config/damage.ts'), 'damageTypes'));
  for (const [namespace, name] of [
    ['immunity_type', 'immunityTypes'], ['weakness_type', 'weaknessTypes'],
    ['resistance_type', 'resistanceTypes'],
  ]) labelMap(namespace, map(reader.load('src/scripts/config/iwr.ts'), name));
  const groups = map(config, 'weaponGroups');
  const declaredGroups = reader.strings(get(reader.load('src/module/item/weapon/values.ts'), 'WEAPON_GROUPS'));
  if (JSON.stringify([...groups.keys()].sort(compare)) !== JSON.stringify([...declaredGroups].sort(compare)))
    throw new Error('Declared weapon groups and labels differ');
  const specializations = object(locate('PF2E.Item.Weapon.CriticalSpecialization'));
  for (const token of registry.pf2e_weapon_groups) if (!groups.has(token)) throw new Error('PF2e weapon group removed: ' + token);
  for (const [token, member] of groups) {
    if (typeof member.value !== 'string') throw new Error('Invalid weapon-group label');
    const location = reader.location(member.node), key = 'PF2E.Item.Weapon.CriticalSpecialization.' + token;
    let description: string | null = null;
    if (registry.pf2e_weapon_groups.includes(token) && typeof specializations[token] === 'string') description = key;
    add('weapon_group', token, member.value, description, location.file, location.line);
  }
  if (reader.diagnostics.some(diagnostic => diagnostic.severity === 'error')) throw new Error('Reference catalog AST extraction is incomplete');
  definitions.sort((a, b) => compare(a.namespace, b.namespace) || compare(a.identifier, b.identifier));
  if (new Set(definitions.map(definition => definition.namespace + ':' + definition.identifier)).size !== definitions.length)
    throw new Error('Duplicate reference definition identity');
  return { digest: sha('reference-catalog/v2:' + inputs.digest + ':' + canonical(registry)
    + ':' + canonical(definitions)), definitions, senseRules };
}
