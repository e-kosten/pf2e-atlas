import type { ExtractionSummary, TypeGraph } from './contracts.js';
import { selectItemInput } from './item-selection.js';
import { nodeReferences, type GenerationInput } from './generation-input.js';

/** Add complete persisted predicates using the same emitter and shared ownership. */
export function selectSourceInput(graph: TypeGraph, summary: ExtractionSummary): GenerationInput {
  const input = selectItemInput(graph, summary);
  const nodes = new Map(graph.nodes.map(node => [node.id,node]));
  const statement = 'src/module/system/predication.ts#PredicateStatement';
  const array = graph.nodes.find(node => node.kind==='array' && node.element===statement && node.serialization?.basis==='array-subclass');
  const pickable=graph.nodes.filter(node=>node.kind==='object' && node.name==='PickableThing'
    && node.fields.some(field=>field.name==='predicate' && field.serialization?.basis==='predicate-constructor-input'));
  if(!nodes.has(statement) || !array || pickable.length!==1) throw new Error('Missing or ambiguous persisted predicate declarations');
  const choice=pickable[0];if(choice.kind!=='object')throw new Error('Expected ChoiceSet choice declaration');
  const constructor=nodes.get(choice.fields.find(field=>field.name==='predicate')!.ref);
  if(!constructor)throw new Error('Missing predicate constructor input');
  input.selection.push(...[{name:'PredicateStatement',ref:statement},{name:'PredicateStatements',ref:array.id},{name:'PredicateInput',ref:constructor.id,sourceRef:choice.id}].map(({name,ref,sourceRef})=>({
    name,declaration:ref,valueRef:ref,module:'rules/predicate',fields:[],deferred:[],...(sourceRef?{sourceRef}:{}) })));
  const selected = new Map(input.nodes.map(node=>[node.id,node]));
  const visit=(ref:string)=>{if(selected.has(ref))return;const node=nodes.get(ref);if(!node)throw new Error(`Missing predicate node: ${ref}`);
    selected.set(ref,node);nodeReferences(node).forEach(visit);};
  visit(statement);visit(array.id);visit(constructor.id);visit(choice.id);
  input.nodes=[...selected.values()].sort((a,b)=>a.id.localeCompare(b.id));
  return input;
}
