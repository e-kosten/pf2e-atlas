import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import type { ExtractionSummary, TypeGraph } from './contracts.js';
import { sourceIdentity } from './source-identity.js';
import { generateRustModules } from './source-generation.js';
import { loadGenerationInput } from './generation-input.js';
import { authoredRuleInputs } from './rule-inputs.js';
import { sampleRules, type RulePacket } from './sample-rules.js';

export type ProbeResult = { ok: true; fidelity: string | null } | { ok: false; error: { json_path: string; expected: string; actual: string } };
/** Every probed occurrence gets an acceptance and fidelity outcome. */
export function compareRuleResults(packets: RulePacket[], schema: ProbeResult[], authored: ProbeResult[]) {
  if (schema.length !== packets.length || authored.length !== packets.length) throw new Error('Rule probe result count mismatch');
  const counts = { rules: packets.length, schemaAccepted: 0, authoredAccepted: 0, recovered: 0, regressed: 0, schemaFidelityFailures: 0, fidelityFailures: 0 };
  const families: Record<string, { rules: number; schemaAccepted: number; authoredAccepted: number }> = Object.create(null);
  const failures: { context: RulePacket['context']; rule: string; category: 'unresolved' | 'parser-defect'; result: ProbeResult }[] = [];
  packets.forEach((packet, index) => {
    const before = schema[index], after = authored[index];
    families[packet.key] ??= { rules: 0, schemaAccepted: 0, authoredAccepted: 0 };
    families[packet.key].rules++;
    if (before.ok) { counts.schemaAccepted++; families[packet.key].schemaAccepted++; }
    if (after.ok) { counts.authoredAccepted++; families[packet.key].authoredAccepted++; }
    if (!before.ok && after.ok) counts.recovered++;
    if (before.ok && !after.ok) counts.regressed++;
    if (before.ok && before.fidelity !== null) counts.schemaFidelityFailures++;
    if (after.ok && after.fidelity !== null) counts.fidelityFailures++;
    if (!after.ok || after.fidelity !== null) failures.push({ context: packet.context, rule: packet.key,
      category: after.ok ? 'parser-defect' : 'unresolved', result: after });
  });
  return { counts, families, failures };
}
const contains = (parent: string, child: string) => {
  const relative = path.relative(parent, child);
  return !relative || relative !== '..' && !relative.startsWith('..' + path.sep) && !path.isAbsolute(relative);
};
export async function compareRules(args: { source: string; graph: string; summary: string; out: string; policyManifest?: string }) {
  const repo = fileURLToPath(new URL('../../..', import.meta.url));
  if ([args.source, args.graph, args.summary, ...(args.policyManifest ? [args.policyManifest] : []), path.join(repo, 'crates'), path.join(repo, 'scripts')]
    .some(input => contains(args.out, input) || contains(input, args.out))) throw new Error('Comparison output must be separate from source/code inputs');
  const graph = JSON.parse(await readFile(args.graph, 'utf8')) as TypeGraph;
  const summary = JSON.parse(await readFile(args.summary, 'utf8')) as ExtractionSummary;
  const identity = await sourceIdentity(args.source);
  if (graph.format !== 'atlas-source-type-graph/v1' || summary.format !== 'atlas-source-extraction/v1'
    || !graph.complete || !summary.complete || graph.typescript !== summary.typescript_version
    || identity.source_digest !== summary.source.source_digest || graph.source?.source_digest !== identity.source_digest)
    throw new Error('Comparison requires complete extraction of the same source bytes');
  const policy = args.policyManifest ? await loadGenerationInput(args.policyManifest) : undefined;
  if (policy && policy.source.source_digest !== identity.source_digest) throw new Error('Policy manifest must describe the same source bytes');
  const openTraitArrays = policy?.openTraitArrays ?? [];
  const packets: RulePacket[] = [];
  for await (const packet of sampleRules(args.source)) packets.push(packet);
  const keys = new Set(graph.roots.flatMap(root => root.ruleKey ? [root.ruleKey] : []));
  const modeled = packets.filter(packet => keys.has(packet.key));
  const unmodeled = packets.filter(packet => !keys.has(packet.key));
  const authored = authoredRuleInputs(graph, openTraitArrays);
  await mkdir(args.out, { recursive: true });
  const json = async (name: string, value: unknown) => writeFile(path.join(args.out, name), JSON.stringify(value, null, 2) + '\n');
  const run = async (profile: string, graph: TypeGraph): Promise<ProbeResult[]> => {
    const directory = path.join(args.out, profile);
    const roots = graph.roots.filter(root => root.ruleKey);
    const selection = roots.map((root, index) => ({ name: `Rule${index}`, declaration: root.ref!, valueRef: root.ref!,
      module: `rules/rule${index}`, fields: [], deferred: [] }));
    const files = generateRustModules({ source: identity, nodes: graph.nodes, selection, openTraitArrays });
    for (const [name, text] of Object.entries(files)) {
      const file = path.join(directory, 'generated', name);
      await mkdir(path.dirname(file), { recursive: true }); await writeFile(file, text);
    }
    await writeFile(path.join(directory, 'Cargo.toml'), `[package]\nname="atlas-rule-input-probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[[bin]]\nname="probe"\npath="main.rs"\n[dependencies]\nserde={version="1",features=["derive"]}\nserde_json="1"\n`);
    const rustPath = (file: string) => JSON.stringify(path.join(repo, file));
    await writeFile(path.join(directory, 'graph.json'), JSON.stringify(graph));
    await writeFile(path.join(directory, 'main.rs'), `#![allow(dead_code, unused_imports)]
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
${roots.map((root, index) => `            ${JSON.stringify(root.ruleKey)} => (${JSON.stringify(root.ref)}, generated::parse_rule${index}(&raw, &context, &context.json_path).map(|value| serde_json::to_value(value).unwrap())),`).join('\n')}
            _ => panic!("Unmodeled rule sent to probe"),
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
    // A single scratch executable compiles the entire rule portfolio against real primitives.
    // Paths are absolute, so caller working directory cannot change code ownership.
    await mkdir(path.join(directory, 'source_model'), { recursive: true });
    // Inline module paths resolve generated beside the inline module's directory.
    await writeFile(path.join(directory, 'source_model/generated.rs'), `include!("../generated/mod.rs");\n`);
    const build = spawnSync('cargo', ['build', '--offline', '--quiet', '--manifest-path', path.join(directory, 'Cargo.toml'), '--target-dir', path.join(args.out, 'target')], { encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 });
    if (build.error || build.status !== 0) throw new Error(`${profile} probe build failed: ${build.error?.message ?? build.stderr}`);
    const result = spawnSync(path.join(args.out, `target/debug/probe${process.platform === 'win32' ? '.exe' : ''}`), [], { input: modeled.map(packet => JSON.stringify(packet) + '\n').join(''), encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
    if (result.error || result.status !== 0) throw new Error(`${profile} probe failed: ${result.error?.message ?? result.stderr}`);
    const results = result.stdout.trim().split('\n').filter(Boolean).map(line => JSON.parse(line) as ProbeResult);
    await json(`${profile}.json`, results);
    return results;
  };
  const before = await run('schema', graph), after = await run('authored', authored.graph);
  const comparison = compareRuleResults(modeled, before, after);
  const report = { source: identity, corpusDigest: createHash('sha256').update(packets.map(packet => JSON.stringify(packet)).join('\n')).digest('hex'),
    occurrences: packets.length, runtimeAdmission: 'not-executed', openTraitArrays, changes: authored.changes,
    sharedIwrChanges: authored.sharedIwrChanges, valueChanges: authored.valueChanges,
    ...comparison, unmodeled };
  await json('comparison.json', report);
  return { report, exitCode: comparison.failures.length || comparison.counts.schemaFidelityFailures || comparison.counts.regressed || unmodeled.length ? 1 : 0 };
}
async function main() {
  const { values } = parseArgs({ options: { source: { type: 'string' }, graph: { type: 'string' }, summary: { type: 'string' }, out: { type: 'string' }, 'policy-manifest': { type: 'string' }, help: { type: 'boolean', short: 'h' } } });
  if (values.help) { console.log('Usage: npm --prefix scripts/source-contracts run compare-rules -- --source PATH --graph PATH --summary PATH --out PATH [--policy-manifest PATH]'); return; }
  if (!values.source || !values.graph || !values.summary || !values.out) throw new Error('--source, --graph, --summary and --out are required');
  const cwd = process.env.INIT_CWD ?? process.cwd();
  const { report, exitCode } = await compareRules({ source: path.resolve(cwd, values.source), graph: path.resolve(cwd, values.graph), summary: path.resolve(cwd, values.summary), out: path.resolve(cwd, values.out),
    policyManifest: values['policy-manifest'] ? path.resolve(cwd, values['policy-manifest']) : undefined });
  console.log(JSON.stringify({ occurrences: report.occurrences, ...report.counts, families: report.families, unmodeled: report.unmodeled.length, runtimeAdmission: report.runtimeAdmission }, null, 2));
  process.exitCode = exitCode;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { await main(); } catch (error) { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; }
}
