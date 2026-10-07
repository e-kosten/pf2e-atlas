/** Synthetic regression graph, compiled using the real Rust source primitives. */
import type {GraphField,GraphNode} from './contracts.js';
import type {GenerationInput} from './generation-input.js';

export function recursiveFixture():GenerationInput {
  const field=(name:string,ref:string,optional=false,nullable=false):GraphField=>({name,ref,optional,nullable,undefinedAllowed:false,forbidden:false,declaredAt:[]});
  const object=(id:string,fields:GraphField[]):GraphNode=>({id,name:id,kind:'object',fields,indexSignatures:[]});
  const nodes:GraphNode[]=[{id:'primitive:string',kind:'primitive',value:'string'},
    {id:'primitive:number',kind:'primitive',value:'number'},{id:'primitive:null',kind:'primitive',value:'null'},
    {id:'primitive:never',kind:'primitive',value:'never'},
    {id:'Yes',name:'Yes',kind:'literal',value:true},{id:'One',name:'One',kind:'literal',value:1},
    {id:'A',name:'A',kind:'literal',value:'a'},{id:'B',name:'B',kind:'literal',value:'b'},
    {id:'NullableA',kind:'union',members:['A','primitive:null']},
    object('Node',[field('next','Node',true),field('expression','Expr',true)]),
    {id:'Expr',name:'Expr',kind:'union',members:['primitive:string','Node']},
    {id:'Empty',kind:'tuple',elements:[],readonly:false},
    {id:'Single',kind:'tuple',elements:[{ref:'primitive:string',optional:false,rest:false}],readonly:false},
    object('Left',[field('type','NullableA',false,true),field('if','Yes',true)]),object('Right',[field('type','B')]),
    {id:'Discriminated',name:'Discriminated',kind:'union',members:['Left','Right']},
    object('Optional',[field('value','primitive:string',true)]),object('Required',[field('value','primitive:string')]),object('Other',[field('other','primitive:string')]),
    {id:'Strict',name:'Strict',kind:'union',members:['Required','Other']},
    {id:'Loose',name:'Loose',kind:'union',members:['Optional','Other']},
    object('BoolConsumer',[field('enabled','Yes')]),
    {id:'Harm',name:'Harm',kind:'literal',value:'harm'},{id:'Heal',name:'Heal',kind:'literal',value:'heal'},
    {id:'NoFonts',kind:'array',element:'primitive:never',readonly:false},
    {id:'HarmFont',kind:'tuple',elements:[{ref:'Harm',optional:false,rest:false}],readonly:false},
    {id:'HealFont',kind:'tuple',elements:[{ref:'Heal',optional:false,rest:false}],readonly:false},
    {id:'BothFonts',kind:'tuple',elements:[{ref:'Harm',optional:false,rest:false},{ref:'Heal',optional:false,rest:false}],readonly:false},
    {id:'DivineFonts',name:'DivineFonts',kind:'union',members:['NoFonts','HarmFont','HealFont','BothFonts']},
    {id:'TupleExpr',name:'TupleExpr',kind:'union',members:['primitive:string','NestedTuple']},
    {id:'NestedTuple',kind:'tuple',elements:[{ref:'TupleExpr',optional:false,rest:false}],readonly:false}];
  return {source:{system_version:'fixture',source_digest:'fixture',input_file_count:1,git_commit:null,git_clean:null},nodes,
    selection:[...['Node','Expr','Empty','Single','Discriminated','Strict','Loose','Yes','One','DivineFonts','TupleExpr'].map(name=>({name,declaration:name,valueRef:name,module:'common',fields:[],deferred:[]})),
      {name:'BoolConsumer',declaration:'BoolConsumer',valueRef:'BoolConsumer',module:'consumer',fields:[],deferred:[]}]};
}
