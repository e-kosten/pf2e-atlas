import type { GraphField, GraphNode, TypeGraph } from './contracts.js';

/** Small constructor-shape regressions; the corpus comparison uses upstream declarations. */
export function authoredRuleFixture(): TypeGraph {
  const choiceFile = 'src/module/rules/rule-element/choice-set/data.ts';
  const damageFile = 'src/module/actor/modifiers.ts';
  const field = (name: string, ref: string, optional = false, file = choiceFile): GraphField => ({
    name, ref, optional, nullable: false, undefinedAllowed: optional, forbidden: false,
    declaredAt: [{ file, line: 1, column: 1 }],
  });
  const forbidden = (names: string[]) => names.map(name => ({ ...field(name, 'primitive:undefined', true), forbidden: true }));
  const object = (name: string, fields: GraphField[]): GraphNode => ({ id: name, name, kind: 'object', fields, indexSignatures: [] });
  const predicate = () => field('predicate', 'RawPredicate');
  const nodes: GraphNode[] = [
    ...(['string', 'number', 'boolean', 'undefined'] as const).map(value => ({ id: `primitive:${value}`, kind: 'primitive' as const, value })),
    { id: 'FixtureStrings', kind: 'array', element: 'primitive:string', readonly: false },
    { id: 'RawPredicate', name: 'RawPredicate', kind: 'array', element: 'primitive:string', readonly: false },
    object('ChoiceSetOwnedItems', [field('ownedItems', 'primitive:boolean'), field('types', 'FixtureStrings'), predicate(), ...forbidden(['config', 'attacks', 'unarmedAttacks'])]),
    object('ChoiceSetAttacks', [field('attacks', 'primitive:boolean', true), field('unarmedAttacks', 'primitive:boolean', true), predicate(), ...forbidden(['config', 'ownedItems'])]),
    object('ChoiceSetConfig', [field('config', 'primitive:string'), predicate(), ...forbidden(['ownedItems', 'attacks', 'unarmedAttacks'])]),
    object('ChoiceSetPackQuery', [field('filter', 'RawPredicate'), ...forbidden(['config', 'ownedItems', 'attacks', 'unarmedAttacks'])]),
    object('FixturePickableChoice', [field('value', 'primitive:string'), field('predicate', 'RawPredicate', true)]),
    { id: 'FixtureChoiceArray', kind: 'array', element: 'FixturePickableChoice', readonly: false },
    { id: 'FixtureChoices', kind: 'union', members: ['primitive:string', 'FixtureChoiceArray', 'ChoiceSetOwnedItems', 'ChoiceSetAttacks', 'ChoiceSetConfig', 'ChoiceSetPackQuery'] },
    object('ChoiceRule', [field('choices', 'FixtureChoices')]),
    { id: 'FixtureFire', kind: 'literal', value: 'fire' }, { id: 'FixtureD6', kind: 'literal', value: 'd6' },
    { id: 'FixtureDamageType', kind: 'union', members: ['FixtureFire', 'primitive:undefined'] },
    { id: 'FixtureDieSize', kind: 'union', members: ['FixtureD6', 'primitive:undefined'] },
    { id: 'FixtureDiceNumber', kind: 'union', members: ['primitive:number', 'primitive:undefined'] },
    object('DamageDiceOverride', [field('damageType', 'FixtureDamageType', true, damageFile), field('dieSize', 'FixtureDieSize', true, damageFile),
      field('diceNumber', 'FixtureDiceNumber', true, damageFile), field('upgrade', 'primitive:boolean', true, damageFile)]),
    object('DamageRule', [field('override', 'DamageDiceOverride')]),
    { id: 'FixtureTrait', kind: 'literal', value: 'agile' },
    { id: 'FixtureTraits', kind: 'array', element: 'FixtureTrait', readonly: false },
    object('StrikeRule', [field('traits', 'FixtureTraits')]),
  ];
  return { format: 'atlas-source-type-graph/v1', typescript: 'fixture', complete: true, status: 'complete',
    diagnostics: [], projectDiagnostics: { selected: [], unrelated: [] }, nodes, roots: [
      { file: choiceFile, name: 'ChoiceRule', ruleKey: 'ChoiceSet', ref: 'ChoiceRule', arrayInputs: [] },
      { file: damageFile, name: 'DamageRule', ruleKey: 'DamageDice', ref: 'DamageRule', arrayInputs: [] },
      { file: 'fixture', name: 'StrikeRule', ruleKey: 'Strike', ref: 'StrikeRule', arrayInputs: [
        { field: 'traits', arrayRef: 'FixtureTraits', elementRef: 'FixtureTrait', fieldClass: 'ArrayField', declaredAt: [] }] },
    ] };
}
