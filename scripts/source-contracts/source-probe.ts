import { spawnSync } from 'node:child_process';
import { closeSync, openSync } from 'node:fs';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import type { TypeGraph } from './contracts.js';
import type { GenerationInput } from './generation-input.js';
import { generateRustModules } from './source-generation.js';

export type ProbeResult = { ok: true; fidelity: string | null } | { ok: false; error: { json_path: string; expected: string; actual: string } };
export interface ProbeRoot { key: string; reference: string; name: string; module: string }

/** Compile selected roots against the real primitives; stream raw packets through Rust. */
export async function sourceProbe(args: { graph: TypeGraph; input: GenerationInput; roots: ProbeRoot[];
  packets: string; out: string; target: string }): Promise<ProbeResult[]> {
  const repo = fileURLToPath(new URL('../../..', import.meta.url));
  const files = generateRustModules(args.input);
  for (const [name, text] of Object.entries(files)) {
    const file = path.join(args.out, 'generated', name);
    await mkdir(path.dirname(file), { recursive: true }); await writeFile(file, text);
  }
  await writeFile(path.join(args.out, 'Cargo.toml'), `[package]\nname="atlas-source-probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[[bin]]\nname="probe"\npath="main.rs"\n[dependencies]\nserde={version="1",features=["derive"]}\nserde_json="1"\nryu-js="1.0.3"\n`);
  const rustPath = (file: string) => JSON.stringify(path.join(repo, file));
  await writeFile(path.join(args.out, 'graph.json'), JSON.stringify(args.graph));
  await writeFile(path.join(args.out, 'main.rs'), `#![allow(dead_code, unused_imports)]
#![recursion_limit="512"]
mod source_model {
${['keyed', 'parse', 'presence', 'union', 'value'].map(module => `#[path=${rustPath(`crates/atlas-ingest/src/source_model/${module}.rs`)}] mod ${module};`).join('\n')}
pub use keyed::SourceMap;
pub mod generated;
#[path=${rustPath('crates/atlas-ingest/examples/support/rule_fidelity.rs')}] mod fidelity;
pub fn run() {
    use std::io::BufRead;
    let graph: serde_json::Value = serde_json::from_str(include_str!("graph.json")).unwrap();
    let fidelity = fidelity::FidelityGraph::new(&graph);
    for line in std::io::stdin().lock().lines() {
        let packet: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let raw = value::parse_source(packet["source"].as_str().unwrap().as_bytes()).unwrap();
        let context = parse::SourceContext::new(packet["context"]["record_key"].as_str().unwrap(), packet["context"]["source_path"].as_str().unwrap(), packet["context"]["json_path"].as_str().unwrap());
        let (reference, result) = match packet["key"].as_str().unwrap() {
${args.roots.map(root => `            ${JSON.stringify(root.key)} => (${JSON.stringify(root.reference)}, generated::parse_${root.name.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase()}(&raw, &context, &context.json_path).map(|value| serde_json::to_value(value).unwrap())),`).join('\n')}
            _ => panic!("Unmodeled source sent to probe"),
        };
        let output = match result {
            Ok(model) => serde_json::json!({"ok":true,"fidelity":fidelity.compare(reference, &raw, &model).err()}),
            Err(error) => serde_json::json!({"ok":false,"error":error}),
        };
        println!("{}", output);
    }
}
}
fn main() { source_model::run(); }
`);
  await mkdir(path.join(args.out, 'source_model'), { recursive: true });
  await writeFile(path.join(args.out, 'source_model/generated.rs'), 'include!("../generated/mod.rs");\n');
  const build = spawnSync('cargo', ['build', '--offline', '--quiet', '--manifest-path', path.join(args.out, 'Cargo.toml'), '--target-dir', args.target], { encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 });
  if (build.error || build.status !== 0) throw new Error(`Probe build failed: ${build.error?.message ?? build.stderr}`);
  const input = openSync(args.packets, 'r');
  try {
    const output = openSync(path.join(args.out, 'results.ndjson'), 'w');
    try {
      const run = spawnSync(path.join(args.target, `debug/probe${process.platform === 'win32' ? '.exe' : ''}`), [], { stdio: [input, output, 'pipe'], encoding: 'utf8' });
      if (run.error || run.status !== 0) throw new Error(`Probe failed: ${run.error?.message ?? run.stderr}`);
    } finally { closeSync(output); }
  } finally { closeSync(input); }
  return (await readFile(path.join(args.out, 'results.ndjson'), 'utf8')).trim().split('\n').filter(Boolean).map(line => JSON.parse(line) as ProbeResult);
}
