import { createHash } from 'node:crypto';
import { createReadStream, readFileSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const repoRoot = fileURLToPath(new URL('../../../../', import.meta.url));
export const targets = {
  'aarch64-apple-darwin': ['macos', 'aarch64'],
  'x86_64-unknown-linux-gnu': ['linux', 'x86_64'],
  'aarch64-unknown-linux-gnu': ['linux', 'aarch64'],
  'x86_64-pc-windows-msvc': ['windows', 'x86_64'],
} as const;
export type Target = keyof typeof targets;
export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
export function object(value: Json | undefined): { [key: string]: Json } {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Expected a JSON object');
  return value;
}
export function readJson(file: string): Json { return JSON.parse(readFileSync(file, 'utf8')) as Json; }
export const compare = (a: string, b: string) => a < b ? -1 : a > b ? 1 : 0;
/** Match the previous manifest serialization: sorted keys, ASCII escapes, LF. */
export function jsonText(value: Json): string {
  function serialize(member: Json, depth: number): string {
    if (member === null || typeof member !== 'object') return JSON.stringify(member);
    const indent = '  '.repeat(depth + 1);
    const array = Array.isArray(member);
    const entries = array ? member.map((entry) => serialize(entry, depth + 1))
      : Object.keys(member).sort(compare).map((key) => `${JSON.stringify(key)}: ${serialize(member[key], depth + 1)}`);
    const [open, close] = array ? ['[', ']'] : ['{', '}'];
    return entries.length ? `${open}\n${indent}${entries.join(`,\n${indent}`)}\n${'  '.repeat(depth)}${close}` : open + close;
  }
  // Object insertion order cannot preserve lexical sorting of numeric keys.
  const text = serialize(value, 0);
  return text.replace(/[\u007f-\uffff]/g, (character) => `\\u${character.charCodeAt(0).toString(16).padStart(4, '0')}`) + '\n';
}
export function writeJson(file: string, value: Json): void { writeFileSync(file, jsonText(value)); }
export function isFile(file: string): boolean {
  try { return statSync(file).isFile(); } catch (error) {
    if (error instanceof Error && 'code' in error && ['ENOENT', 'ENOTDIR'].includes(String(error.code))) return false;
    throw error;
  }
}
export async function sha256(file: string): Promise<string> {
  const digest = createHash('sha256');
  for await (const chunk of createReadStream(file, { highWaterMark: 1024 * 1024 })) digest.update(chunk);
  return digest.digest('hex');
}
export function cli(moduleUrl: string, main: (args: string[]) => number | Promise<number>): void {
  if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(moduleUrl)) {
    Promise.resolve().then(() => main(process.argv.slice(2))).then((code) => { process.exitCode = code; })
      .catch((error: unknown) => { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; });
  }
}
export function argumentPath(argument: string): string {
  return path.resolve(process.env.INIT_CWD ?? process.cwd(), argument);
}
