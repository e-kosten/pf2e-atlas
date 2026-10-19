import { mkdirSync, symlinkSync, existsSync, lstatSync, unlinkSync } from 'node:fs';
mkdirSync('public/renderers', { recursive: true });
for (const [name, target] of [
  ['box', '../../renderer-deps/node_modules/@3d-dice/dice-box/dist'],
  ['three-assets', '../../renderer-deps/node_modules/@3d-dice/dice-box-threejs/public'],
  ['three.js', '../../renderer-deps/node_modules/@3d-dice/dice-box-threejs/dist/dice-box-threejs.es.js'],
]) {
  const link = `public/renderers/${name}`;
  // Links resolve relative to public/renderers.
  if (!existsSync(link)) {
    if (lstatSync(link, { throwIfNoEntry: false })?.isSymbolicLink()) unlinkSync(link);
    symlinkSync(target, link);
  }
}
