import { argumentPath, cli } from '../release-data.js';
import { validateAssets } from '../validate-release-assets.js';

cli(import.meta.url, async (args) => {
  if (args.length !== 2) { console.error('usage: npm --prefix dev-tools/release run validate -- <tag> <dist-dir>'); return 2; }
  await validateAssets(argumentPath(args[1]));
  return 0;
});
