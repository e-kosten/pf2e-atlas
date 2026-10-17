import { argumentPath, cli } from '../release-data.js';
import { generateManifest } from '../generate-release-manifest.js';

cli(import.meta.url, async (args) => {
  if (args.length !== 2) { console.error('usage: npm --prefix dev-tools/release run manifest -- <tag> <dist-dir>'); return 2; }
  await generateManifest(args[0], argumentPath(args[1]));
  return 0;
});
