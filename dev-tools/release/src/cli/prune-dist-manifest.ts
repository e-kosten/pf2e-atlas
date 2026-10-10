import { argumentPath, cli } from '../release-data.js';
import { pruneManifest } from '../prune-dist-manifest.js';

cli(import.meta.url, (args) => {
  if (args.length !== 2) { console.error('usage: npm --prefix dev-tools/release run prune -- <manifest> <dist-dir>'); return 2; }
  pruneManifest(argumentPath(args[0]), argumentPath(args[1]));
  return 0;
});
