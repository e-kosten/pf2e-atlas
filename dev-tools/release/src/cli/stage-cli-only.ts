import { argumentPath, cli } from '../release-data.js';
import { stageCliOnly } from '../stage-cli-only.js';
cli(import.meta.url, async (args) => {
  if (args.length !== 3) { console.error('usage: stage-cli-only <target> <dist> <cargo-dist-manifest>'); return 2; }
  await stageCliOnly(args[0], argumentPath(args[1]), argumentPath(args[2]));
  return 0;
});
