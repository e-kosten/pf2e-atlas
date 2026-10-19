import assert from 'node:assert/strict';
import { access, mkdtemp, mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { test } from 'node:test';
import { smokePackagedWeb } from '../src/smoke-packaged-web.js';

// A real child process and HTTP server exercise startup, HTTP failures and cleanup.
// Native packaging jobs additionally run the same helper against the real atlas.
for (const mode of ['ok', 'missing', 'wrong-bytes', 'wrong-type', 'exit', 'hang']) {
  test(`packaged web smoke: ${mode}`, async (t) => {
    const root = await mkdtemp(path.join(tmpdir(), 'atlas-smoke-fixture-'));
    t.after(() => rm(root, { recursive: true, force: true }));
    const bundle = path.join(root, 'bundle');
    await mkdir(path.join(bundle, 'assets'), { recursive: true });
    const html = '<html><script src="/assets/app.js"></script><link href="/assets/app.css" rel="stylesheet"></html>';
    await writeFile(path.join(bundle, 'index.html'), html);
    await writeFile(path.join(bundle, 'assets/app.js'), 'console.log("fixture");');
    await writeFile(path.join(bundle, 'assets/app.css'), 'body { color: red; }');
    const fixture = path.join(root, 'server.mjs');
    await writeFile(fixture, `
      import { createServer } from 'node:http';
      import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
      import path from 'node:path';
      const mode = ${JSON.stringify(mode)};
      if (readdirSync(process.cwd()).length !== 0) process.exit(9);
      writeFileSync(${JSON.stringify(path.join(root, 'working-directory.txt'))}, process.cwd());
      if (mode === 'exit') process.exit(7);
      if (mode === 'hang') { setInterval(() => {}, 1000); }
      else {
        const server = createServer((request, response) => {
          const asset = request.url;
          if (mode === 'missing' && asset.endsWith('.js')) { response.writeHead(404); response.end(); return; }
          const mime = asset === '/' ? 'text/html' : asset.endsWith('.js') ? 'text/javascript' : 'text/css';
          response.setHeader('content-type', mode === 'wrong-type' && asset.endsWith('.js') ? 'text/plain' : mime);
          const bytes = readFileSync(path.join(${JSON.stringify(bundle)}, asset === '/' ? 'index.html' : asset.slice(1)));
          response.end(mode === 'wrong-bytes' && asset.endsWith('.css') ? 'stale bundle' : bytes);
        });
        server.listen(0, '127.0.0.1', () => console.log('Atlas web service listening at http://127.0.0.1:' + server.address().port));
      }
    `);
    const run = () => smokePackagedWeb(process.execPath, bundle, { prefixArgs: [fixture], timeoutMs: mode === 'hang' ? 2000 : 10000 });
    if (mode === 'ok') await run();
    else {
      const expected = { missing: /HTTP status/, 'wrong-bytes': /content differs/, 'wrong-type': /content type/, exit: /exited \(7\)/, hang: /timed out/ }[mode];
      await assert.rejects(run, expected!);
    }
    const workingDirectory = await readFile(path.join(root, 'working-directory.txt'), 'utf8');
    await assert.rejects(access(workingDirectory), { code: 'ENOENT' });
  });
}
