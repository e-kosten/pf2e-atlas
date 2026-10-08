import type { GraphField, TypeGraph } from '../contracts.js';
import { projectFields, type FieldReplacements } from './authored-projection.js';
import { objectPatch } from './object-patch.js';
import { authoredRuleInputs } from './rule-inputs.js';

/** Bounded authored forms supported by the pinned document implementations. */
export function authoredDocumentInputs(schema: TypeGraph, openTraitArrays: string[] = []) {
  if (!schema.complete) throw new Error('Authored document inputs require complete schema extraction');
  // Generate the same authored rule roots as the standalone rule comparison.
  // Item.rules retains upstream's generic RuleElementSource; compare rules separately.
  const rules = authoredRuleInputs(schema, openTraitArrays);
  const graph = rules.graph;
  const nodes = new Map(graph.nodes.map(node => [node.id, node]));
  const replacements: FieldReplacements = new Map();
  const changes: { ownerRef: string; field: string; schemaRef: string; authoredRef: string; basis: string; declaredAt: GraphField['declaredAt'] }[] = [];
  const owner = (name: string, file: string) => {
    const found = schema.nodes.filter(node => node.name === name && node.declaredAt?.some(location => location.file === file));
    if (found.length !== 1 || !('fields' in found[0]) || !found[0].fields) throw new Error(`Authored document declaration drift: ${name}`);
    return found[0] as typeof found[0] & { fields: GraphField[] };
  };
  const replace = (ownerRef: string, field: GraphField, authoredRef: string, basis: string) => {
    if (!field.declaredAt.length || field.forbidden) throw new Error(`Authored document field drift: ${ownerRef}.${field.name}`);
    if (!replacements.has(ownerRef)) replacements.set(ownerRef, new Map());
    replacements.get(ownerRef)!.set(field.name, { ref: authoredRef });
    changes.push({ ownerRef, field: field.name, schemaRef: field.ref, authoredRef, basis, declaredAt: field.declaredAt });
  };
  const empty = 'authored-document:empty-string';
  graph.nodes.push({ id: empty, kind: 'literal', value: '' });
  const addEmpty = (ownerRef: string, field: GraphField | undefined, file: string, basis: string, nullable = true, domain: 'vocabulary' | 'number' = 'vocabulary') => {
    if (!field || field.optional || field.nullable !== nullable || field.declaredAt.some(location => location.file !== file))
      throw new Error(`Authored empty-string field drift: ${ownerRef}`);
    const node = nodes.get(field.ref);
    const members = node?.kind === 'union' ? node.members : [field.ref];
    const vocabulary = members.some(ref => nodes.get(ref)?.kind === 'literal') && !members.some(ref => {
      const member = nodes.get(ref);
      return !member || !(member.kind === 'literal' && typeof member.value === 'string' && member.value.length > 0
        || member.kind === 'primitive' && member.value === 'null');
    });
    if (domain === 'number' ? node?.kind !== 'primitive' || node.value !== 'number' : !vocabulary)
      throw new Error(`Authored empty-string ${domain} drift: ${ownerRef}.${field.name}`);
    const authoredRef = `${field.ref}#authored-empty-string`;
    if (!graph.nodes.some(node => node.id === authoredRef)) graph.nodes.push({ id: authoredRef, kind: 'union', members: [...members, empty] });
    replace(ownerRef, field, authoredRef, basis);
  };
  const weaponFile = 'src/module/item/weapon/data.ts';
  const weapon = owner('WeaponSystemSource', weaponFile);
  const reload = nodes.get(weapon.fields.find(field => field.name === 'reload')?.ref ?? '');
  if (!reload || !('fields' in reload) || !reload.fields) throw new Error('Weapon reload declaration drift');
  addEmpty(reload.id, reload.fields.find(field => field.name === 'value'), weaponFile, 'weapon/document.ts: prepareBaseData reload.value ||= null');
  const damage = owner('WeaponDamage', weaponFile);
  addEmpty(damage.id, damage.fields.find(field => field.name === 'die'), weaponFile, 'weapon/document.ts: prepareBaseData damage.die ||= null');
  for (const name of ['bonusDamage', 'splashDamage']) {
    const value = nodes.get(weapon.fields.find(field => field.name === name)?.ref ?? '');
    if (value?.kind !== 'object') throw new Error(`Weapon damage value declaration drift: ${name}`);
    addEmpty(value.id, value.fields.find(field => field.name === 'value'), weaponFile,
      `weapon/document.ts: prepareBaseData ${name}.value ||= 0`, false, 'number');
  }

  const spellFile = 'src/module/item/spell/data.ts';
  const spell = owner('SpellSystemSource', spellFile);
  const overlay = owner('SpellOverlayOverride', spellFile);
  const spellDamage = owner('SpellDamageSource', spellFile);
  addEmpty(spellDamage.id, spellDamage.fields.find(field => field.name === 'category'), spellFile,
    'spell/document.ts: _preUpdate clears empty damage.category in base, overlay and fixed heightening updates');
  const area = owner('SpellArea', spellFile);
  const areaValue = area.fields.find(field => field.name === 'value');
  if (!areaValue || areaValue.ref !== 'primitive:number' || areaValue.optional || areaValue.nullable
    || areaValue.declaredAt.some(location => location.file !== spellFile)) throw new Error('Spell area value declaration drift');
  const areaValueRef = 'authored-document:spell-area-number-or-string';
  graph.nodes.push({ id: areaValueRef, kind: 'union', members: ['primitive:string', 'primitive:number'] });
  replace(area.id, areaValue, areaValueRef, 'spell/document.ts: prepareBaseData uses area.value truthiness and numeric division before storing a number');
  addEmpty(area.id, area.fields.find(field => field.name === 'type'), spellFile,
    'spell/document.ts: prepareBaseData supplies area.type ||= burst for a present area', false);
  const defense = owner('SpellDefenseSource', spellFile);
  for (const name of ['passive', 'save']) {
    const field = defense.fields.find(field => field.name === name);
    const value = field && nodes.get(field.ref);
    const refs = value?.kind === 'union' ? value.members : [field?.ref];
    const objects = refs.map(ref => ref ? nodes.get(ref) : undefined).filter(node => node?.kind === 'object');
    if (!field || !field.nullable || field.optional || objects.length !== 1 || objects[0]?.kind !== 'object')
      throw new Error(`Spell defense declaration drift: ${name}`);
    addEmpty(objects[0].id, objects[0].fields.find(field => field.name === 'statistic'), spellFile,
      'spell/document.ts: _preUpdate clears a passive/save defense when statistic is an empty string', false);
  }
  const initiativeFile = 'src/module/actor/creature/data.ts';
  const initiative = owner('CreatureInitiativeSource', initiativeFile);
  const statistic = initiative.fields.find(field => field.name === 'statistic');
  const statisticNode = statistic && nodes.get(statistic.ref);
  if (!statistic || statistic.optional || statistic.nullable || statistic.declaredAt.some(location => location.file !== initiativeFile)
    || statisticNode?.kind !== 'union' || statisticNode.members.some(ref => {
      const node = nodes.get(ref); return node?.kind !== 'literal' || typeof node.value !== 'string';
    })) throw new Error('Creature initiative statistic declaration drift');
  replace(initiative.id, statistic, 'primitive:string', 'actor/initiative.ts: ActorInitiative resolves a string slug through actor.getStatistic, including lore and synthetic statistics');
  const npcFile = 'src/module/actor/npc/data.ts';
  if (schema.nodes.some(node => node.name === 'NPCSystemSource')) {
    const npc = owner('NPCSystemSource', npcFile);
    const perception = nodes.get(npc.fields.find(field => field.name === 'perception')?.ref ?? '');
    const senses = perception && 'fields' in perception ? perception.fields?.find(field => field.name === 'senses') : undefined;
    const array = senses && nodes.get(senses.ref);
    const input = schema.constructorInputs?.find(input => input.file === 'src/module/actor/creature/sense.ts' && input.name === 'Sense');
    const shape = input && nodes.get(input.ref);
    if (!perception || !senses || array?.kind !== 'array' || !input?.declaredAt.length
      || !shape || !('fields' in shape) || !shape.fields
      || !shape.fields.some(field => field.name === 'type' && !field.optional)
      || !['acuity', 'range'].every(name => shape.fields!.some(field => field.name === name && field.optional)))
      throw new Error('NPC sense constructor-input drift; re-extract and inspect the Sense constructor');
    const authoredRef = `${array.id}#authored-sense-constructor`;
    graph.nodes.push({ ...structuredClone(array), id: authoredRef, element: input.ref });
    replace(perception.id, senses, authoredRef,
      'system/statistic/perception.ts: prepares NPC senses with new Sense(data); creature/sense.ts: constructor input before schema defaults');
  }
  // Build patch shapes from the already projected value graph, including sentinels.
  const sentinels = projectFields(graph, replacements, 'authored-document');
  replacements.clear();
  const system = overlay.fields.find(field => field.name === 'system');
  if (!system || !system.optional || system.nullable || system.declaredAt.some(location => location.file !== spellFile))
    throw new Error('Spell overlay system declaration drift');
  const partial = nodes.get(system.ref);
  const partialMembers = partial?.kind === 'union' ? partial.members.map(ref => nodes.get(ref)) : [partial];
  const patchFields = (node: typeof partial) => node?.kind === 'object'
    && node.fields.every(field => field.optional)
    && JSON.stringify(node.fields.map(field => field.name).sort()) === JSON.stringify(spell.fields.map(field => field.name).sort());
  if (!partialMembers.some(node => patchFields(node) && node?.name === 'DeepPartial'
    && node.declaredAt?.some(location => location.file === 'types/foundry/util.d.ts'))
    || partialMembers.some(node => !patchFields(node) && !(node?.kind === 'primitive' && node.value === 'undefined')))
    throw new Error('Spell overlay partial declaration drift');
  // Start from the rule-projected spell owner so rules in a patch keep authored shapes.
  const ruleSpell = graph.nodes.find(node => node.id.startsWith(`${spell.id}#authored-input`)) ?? spell;
  const ruleOverlay = graph.nodes.find(node => node.id.startsWith(`${overlay.id}#authored-input`)) ?? overlay;
  const authoredSpell = graph.nodes.find(node => node.id === sentinels(ruleSpell.id))!;
  const authoredOverlay = graph.nodes.find(node => node.id === sentinels(ruleOverlay.id))!;
  const patch = objectPatch(graph, authoredSpell.id);
  replace(authoredOverlay.id, system, patch, 'spell/overlay.ts: updateOverride saves diffObject; spell/document.ts: loadVariant merges system');
  const fixed = owner('SpellHeighteningFixed', spellFile);
  const levels = fixed.fields.find(field => field.name === 'levels');
  const mapped = levels && nodes.get(levels.ref);
  if (!levels || levels.optional || levels.nullable || levels.declaredAt.some(location => location.file !== spellFile)
    || mapped?.kind !== 'object' || mapped.indexSignatures.length || mapped.fields.length !== 10
    || mapped.fields.some(field => !field.optional || field.nullable || field.forbidden || !/^(?:[1-9]|10)$/.test(field.name))
    || new Set(mapped.fields.map(field => field.ref)).size !== 1)
    throw new Error('Fixed spell heightening levels declaration drift');
  const layer = nodes.get(mapped.fields[0].ref);
  const layerMembers = layer?.kind === 'union' ? layer.members.map(ref => nodes.get(ref)) : [layer];
  if (!layerMembers.some(patchFields) || layerMembers.some(node => !patchFields(node) && !(node?.kind === 'primitive' && node.value === 'undefined')))
    throw new Error('Fixed spell heightening layer declaration drift');
  const authoredMapped = graph.nodes.find(node => node.id === sentinels(mapped.id))!;
  const authoredLevels = { ...structuredClone(authoredMapped), id: `${mapped.id}#authored-spell-levels`,
    fields: mapped.fields.map(field => ({ ...structuredClone(field), ref: patch })) };
  graph.nodes.push(authoredLevels);
  const ruleFixed = graph.nodes.find(node => node.id.startsWith(`${fixed.id}#authored-input`)) ?? fixed;
  replace(sentinels(ruleFixed.id), levels, authoredLevels.id, 'spell/document.ts: loadVariant merges fixed heighten layer system into base spell');
  projectFields(graph, replacements, 'authored-document-patch');
  return { graph, changes, ruleChanges: { changes: rules.changes, sharedIwrChanges: rules.sharedIwrChanges, valueChanges: rules.valueChanges } };
}
