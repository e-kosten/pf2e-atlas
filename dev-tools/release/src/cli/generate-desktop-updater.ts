import { argumentPath, cli } from '../release-data.js';
import { generateDesktopUpdater } from '../desktop-updater.js';

cli(import.meta.url, (args) => {
  if (args.length !== 2) throw new Error('Expected the release tag and asset directory');
  generateDesktopUpdater(args[0], argumentPath(args[1]));
  return 0;
});
