import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';

import ts from 'typescript';

export interface PredicatePacket {
  source:string; shape:'array'|'input'; field:string; declaration:'current'|'legacy-definition';
  context:{record_key:string;source_path:string;json_path:string};
}
/** Discover selected predicate values, preserving raw spans and array-only contexts. */
export function predicatePackets(text:string,sourcePath:string,pack:string):PredicatePacket[] {
  const parsed=JSON.parse(text) as {_id?:string};
  const json=ts.parseJsonText(sourcePath,text);const statement=json.statements[0];
  if(!statement || !ts.isExpressionStatement(statement) || !ts.isObjectLiteralExpression(statement.expression))throw new Error(`Expected source document: ${sourcePath}`);
  const packets:PredicatePacket[]=[];
  const name=(property:ts.PropertyAssignment)=>ts.isStringLiteral(property.name)||ts.isIdentifier(property.name)?property.name.text:undefined;
  const walk=(node:ts.Expression,jsonPath:string,rule?:{key:string;path:string})=>{
    if(ts.isObjectLiteralExpression(node)) {
      const props=node.properties.filter(ts.isPropertyAssignment);
      const key=props.find(property=>name(property)==='key')?.initializer;
      if(key && ts.isStringLiteral(key) && /\.rules\[\d+\]$/.test(jsonPath))rule={key:key.text,path:jsonPath};
      for(const property of props) {
        const key=name(property);if(key===undefined)continue;
        const fieldPath=jsonPath+(/^[A-Za-z_][A-Za-z0-9_]*$/.test(key)?'.'+key:'['+JSON.stringify(key)+']');
        const relative=rule?fieldPath.slice(rule.path.length):'';
        let field:string|undefined,shape:'array'|'input'='array';
        if(key==='predicate') {field='predicate';if(rule?.key==='ChoiceSet' && /^\.choices\[\d+\]\.predicate$/.test(relative))shape='input';}
        else if(key==='definition' && rule)field='definition';
        else if(rule?.key==='ChoiceSet' && relative==='.choices.filter')field='choices.filter';
        else if(rule?.key==='CraftingAbility' && relative==='.craftableItems')field='craftableItems';
        else if(rule?.key==='RollOption' && relative==='.disabledIf')field='disabledIf';
        else if(['FlatModifier','SubstituteRoll'].includes(rule?.key??'') && relative==='.removeAfterRoll'
          && ![ts.SyntaxKind.TrueKeyword,ts.SyntaxKind.FalseKeyword].includes(property.initializer.kind)
          && !ts.isStringLiteral(property.initializer))field='removeAfterRoll';
        if(field)packets.push({source:text.slice(property.initializer.getStart(json),property.initializer.end),shape,field,
          declaration:field==='definition' && rule?.key==='ItemAlteration'?'legacy-definition':'current',
          context:{record_key:`${pack}:${parsed._id??path.basename(sourcePath,'.json')}`,source_path:sourcePath,json_path:fieldPath}});
        walk(property.initializer,fieldPath,rule);
      }
    }else if(ts.isArrayLiteralExpression(node))node.elements.forEach((child,index)=>walk(child,`${jsonPath}[${index}]`,rule));
  };
  walk(statement.expression,'$');return packets;
}
export async function* samplePredicates(source:string):AsyncGenerator<PredicatePacket> {
  const manifest=JSON.parse(await readFile(path.join(source,'static/system.json'),'utf8')) as {packs:{name:string;path:string}[]};
  const files=async(directory:string):Promise<string[]>=>{
    const result:string[]=[];
    for(const entry of (await readdir(directory,{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))){
      const file=path.join(directory,entry.name);
      if(entry.isDirectory())result.push(...await files(file));
      else if(entry.isFile() && entry.name.endsWith('.json') && entry.name!=='_folders.json')result.push(file);
    }return result;
  };
  for(const pack of manifest.packs)for(const file of await files(path.join(source,pack.path)))
    yield* predicatePackets(await readFile(file,'utf8'),path.relative(source,file).replaceAll('\\','/'),pack.name);
}
