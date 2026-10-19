import { existsSync } from 'node:fs';
import path from 'node:path';

/** PF2e manifests name distribution paths; the source monorepo namespaces packs. */
export function sourcePackPath(source: string, declared: string): string {
  const direct = path.join(source, declared);
  if (existsSync(direct)) return direct;
  const namespaced = path.join(source, 'packs', 'pf2e', path.basename(declared));
  return existsSync(namespaced) ? namespaced : direct;
}
