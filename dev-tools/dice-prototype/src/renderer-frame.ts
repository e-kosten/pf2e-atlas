// Isolated prototype iframe: removing it disposes the library's workers,
// WebGL context and global listeners without relying on private teardown APIs.
const engine = new URLSearchParams(location.search).get('engine');
const report = (message: object) => parent.postMessage({ channel: 'dice-preview', engine, ...message }, location.origin);
try {
  const url = engine === 'three' ? '/renderers/three.js' : '/renderers/box/dice-box.es.js';
  const { default: DiceBox } = await import(/* @vite-ignore */ new URL(url, location.origin).href);
  const box = engine === 'three'
    ? new DiceBox('#scene', { assetPath: '/renderers/three-assets/', sounds: false, baseScale: 50, strength: 1, theme_colorset: 'white', theme_material: 'plastic' })
    : new DiceBox({ container: '#scene', assetPath: '/renderers/box/assets/', offscreen: false, scale: 7, themeColor: '#376d9d' });
  await (engine === 'three' ? box.initialize() : box.init());
  let busy = false;
  addEventListener('message', async (event) => {
    if (event.origin !== location.origin || event.source !== parent || event.data?.channel !== 'dice-preview' || busy) return;
    const { id, notation } = event.data;
    busy = true;
    try {
      const result = await box.roll(notation);
      report({ kind: 'result', id, result });
    } catch (error) {
      report({ kind: 'error', id, message: String(error) });
    } finally { busy = false; }
  });
  report({ kind: 'ready' });
} catch (error) { report({ kind: 'error', message: String(error) }); }
export {};
