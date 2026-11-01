import path from 'node:path';
import { argumentPath, cli } from '../release-data.js';
import { smokePackagedCli } from '../smoke-packaged-cli.js';
cli(import.meta.url, (args) => {
  if (args.length !== 3 || !['full', 'cli-only'].includes(args[2])) {
    console.error('usage: smoke-packaged-cli <binary> <version> <full|cli-only>'); return 2;
  }
  smokePackagedCli(path.resolve(argumentPath(args[0])), args[1], args[2] as 'full' | 'cli-only');
  return 0;
});
