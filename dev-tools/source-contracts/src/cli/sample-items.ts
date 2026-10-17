import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

import { sampleItems } from '../comparison/sample-items.js';

async function main() {
  const {values} = parseArgs({options:{source:{type:'string'},help:{type:'boolean',short:'h'}}});
  if(values.help) {console.log('Usage: npm --prefix dev-tools/source-contracts run sample-items -- --source PATH');return;}
  if(!values.source) throw new Error('--source is required');
  for await (const packet of sampleItems(path.resolve(process.env.INIT_CWD ?? process.cwd(),values.source)))
    if(!process.stdout.write(JSON.stringify(packet)+'\n')) await new Promise<void>(resolve=>process.stdout.once('drain',resolve));
}
if(process.argv[1] && path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  try{await main();}catch(error){console.error(error instanceof Error ? error.message : String(error));process.exitCode=1;}
}
