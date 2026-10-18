import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';

/** Check the delivered UI bytes, not just an HTTP 200 or the CLI help path. */
export async function checkWebAssets(base: string, bundle: string): Promise<void> {
  const expected = await readFile(path.join(bundle, 'index.html'));
  const html = expected.toString('utf8');
  const assets = [...html.matchAll(/(?:src|href)=["'](\/assets\/[^"']+\.(?:js|css))["']/g)].map((match) => match[1]);
  assert.ok(assets.some((asset) => asset.endsWith('.js')), 'UI entry JavaScript is missing');
  assert.ok(assets.some((asset) => asset.endsWith('.css')), 'UI stylesheet is missing');
  for (const asset of ['/', ...new Set(assets)]) {
    const response = await fetch(new URL(asset, base), { signal: AbortSignal.timeout(5000) });
    assert.equal(response.status, 200, `${asset}: HTTP status`);
    const mime = asset === '/' ? 'text/html' : asset.endsWith('.js') ? 'javascript' : 'text/css';
    assert.ok(response.headers.get('content-type')?.includes(mime), `${asset}: content type`);
    const bytes = asset === '/' ? expected : await readFile(path.join(bundle, asset.slice(1)));
    assert.deepEqual(Buffer.from(await response.arrayBuffer()), bytes, `${asset}: packaged content differs from built bundle`);
  }
}

export async function smokePackagedWeb(binary: string, bundle: string,
  options: { prefixArgs?: string[]; timeoutMs?: number } = {}): Promise<void> {
  const directory = await mkdtemp(path.join(tmpdir(), 'atlas-packaged-web-'));
  const child = spawn(path.resolve(binary), [...(options.prefixArgs ?? []), 'web', '--port', '0', '--path-mode', 'global',
    '--source', path.join(directory, 'source'), '--index', path.join(directory, 'index.sqlite'),
    '--embedding-cache-path', path.join(directory, 'models')], { cwd: directory, stdio: ['ignore', 'pipe', 'pipe'] });
  let output = '';
  let timer: NodeJS.Timeout | undefined;
  const closed = new Promise<void>((resolve) => child.once('close', () => resolve()));
  const stopped = new Promise<never>((_, reject) => {
    child.once('error', reject);
    child.once('exit', (code, signal) => reject(new Error(`Packaged web server exited (${code ?? signal})`)));
  });
  const ready = new Promise<string>((resolve) => {
    child.stdout.on('data', (chunk: Buffer) => {
      output = (output + chunk.toString()).slice(-16384);
      const match = /Atlas web service listening at (http:\/\/127\.0\.0\.1:\d+)\r?\n/.exec(output);
      if (match) resolve(match[1]);
    });
    child.stderr.on('data', (chunk: Buffer) => { output = (output + chunk.toString()).slice(-16384); });
  });
  try {
    await Promise.race([
      ready.then((base) => checkWebAssets(base, bundle)),
      stopped,
      new Promise<never>((_, reject) => {
        timer = setTimeout(() => reject(new Error('Packaged web smoke test timed out')), options.timeoutMs ?? 30000);
      }),
    ]);
  } catch (error) {
    throw new Error(`${error instanceof Error ? error.message : error}\nServer output:\n${output}`);
  } finally {
    clearTimeout(timer);
    if (child.exitCode === null && child.signalCode === null) child.kill('SIGKILL');
    await closed;
    await rm(directory, { recursive: true, force: true });
  }
}
