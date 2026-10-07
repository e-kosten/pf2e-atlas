import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import ts from 'typescript';

export interface ItemPacket {
  source: string; family: string;
  context: { record_key: string; source_path: string; json_path: string };
}
/** Walk Item/Actor packs recursively; raw AST spans preserve numbers and repeated payload keys. */
export function itemPackets(text: string, sourcePath: string, pack: string, documentKind: 'Item' | 'Actor'): ItemPacket[] {
  JSON.parse(text);
  const json = ts.parseJsonText(sourcePath,text);
  const statement = json.statements[0];
  if (!statement || !ts.isExpressionStatement(statement) || !ts.isObjectLiteralExpression(statement.expression)) throw new Error(`Expected document object: ${sourcePath}`);
  const packets: ItemPacket[] = [];
  const member = (object: ts.ObjectLiteralExpression, key: string, strict = false) => {
    const properties = object.properties.filter((property): property is ts.PropertyAssignment => ts.isPropertyAssignment(property)
      && (ts.isStringLiteral(property.name) || ts.isIdentifier(property.name)) && property.name.text===key);
    if (strict && properties.length>1) throw new Error(`Duplicate Item context member ${key}: ${sourcePath}`);
    return properties[0]?.initializer;
  };
  const walk = (node: ts.Expression, jsonPath: string, root = false) => {
    if (ts.isObjectLiteralExpression(node)) {
      const id = member(node,'_id'), type = member(node,'type'), system = member(node,'system');
      if ((!root || documentKind==='Item') && id && ts.isStringLiteral(id) && type && ts.isStringLiteral(type) && system) {
        ['_id','type','system'].forEach(key=>member(node,key,true));
        packets.push({ source:text.slice(node.getStart(json),node.end),family:type.text,
          context:{record_key:`${pack}:${id.text}`,source_path:sourcePath,json_path:jsonPath} });
      } else if (root && documentKind==='Item') {
        throw new Error(`Expected root Item source with _id/type/system: ${sourcePath}`);
      }
      for (const property of node.properties) if (ts.isPropertyAssignment(property) && (ts.isStringLiteral(property.name) || ts.isIdentifier(property.name))) {
        const key = property.name.text;
        walk(property.initializer, `${jsonPath}${/^[A-Za-z_][A-Za-z0-9_]*$/.test(key) ? '.'+key : '['+JSON.stringify(key)+']'}`);
      }
    } else if (ts.isArrayLiteralExpression(node)) node.elements.forEach((child,index)=>walk(child,`${jsonPath}[${index}]`));
  };
  walk(statement.expression,'$',true);
  return packets;
}
export async function* sampleItems(source: string): AsyncGenerator<ItemPacket> {
  const manifest = JSON.parse(await readFile(path.join(source,'static/system.json'),'utf8')) as {packs:{type:string;path:string;name:string}[]};
  const files = async (directory: string): Promise<string[]> => {
    const result: string[] = [];
    for (const entry of (await readdir(directory,{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))) {
      const file = path.join(directory,entry.name);
      if(entry.isDirectory()) result.push(...await files(file));
      else if(entry.isFile() && entry.name.endsWith('.json') && entry.name!=='_folders.json') result.push(file);
    }
    return result;
  };
  for (const pack of manifest.packs.filter(pack=>['Item','Actor'].includes(pack.type))) for (const file of await files(path.join(source,pack.path)))
    yield* itemPackets(await readFile(file,'utf8'),path.relative(source,file).replaceAll('\\','/'),pack.name,pack.type as 'Item'|'Actor');
}
async function main() {
  const {values} = parseArgs({options:{source:{type:'string'},help:{type:'boolean',short:'h'}}});
  if(values.help) {console.log('Usage: npm --prefix scripts/source-contracts run sample-items -- --source PATH');return;}
  if(!values.source) throw new Error('--source is required');
  for await (const packet of sampleItems(path.resolve(process.env.INIT_CWD ?? process.cwd(),values.source)))
    if(!process.stdout.write(JSON.stringify(packet)+'\n')) await new Promise<void>(resolve=>process.stdout.once('drain',resolve));
}
if(process.argv[1] && path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  try{await main();}catch(error){console.error(error instanceof Error ? error.message : String(error));process.exitCode=1;}
}
