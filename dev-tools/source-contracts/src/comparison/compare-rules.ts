import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import type { ExtractionSummary, TypeGraph } from '../contracts.js';
import { sourceIdentity } from '../discovery/source-identity.js';
import { documentTraitArrays } from '../generation/document-traits.js';
import { authoredRuleInputs } from '../generation/rule-inputs.js';
import { sampleRules, type RulePacket } from './sample-rules.js';

import { sourceProbe, type ProbeResult } from './source-probe.js';
export type { ProbeResult } from './source-probe.js';
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
export async function compareRules(args: { source: string; graph: string; summary: string; out: string }) {
  const repo = fileURLToPath(new URL('../../../../..', import.meta.url));
  if ([args.source, args.graph, args.summary, path.join(repo, 'crates'), path.join(repo, 'scripts')]
    .some(input => contains(args.out, input) || contains(input, args.out))) throw new Error('Comparison output must be separate from source/code inputs');
  const graph = JSON.parse(await readFile(args.graph, 'utf8')) as TypeGraph;
  const summary = JSON.parse(await readFile(args.summary, 'utf8')) as ExtractionSummary;
  const identity = await sourceIdentity(args.source);
  if (graph.format !== 'atlas-source-type-graph/v1' || summary.format !== 'atlas-source-extraction/v1'
    || !graph.complete || !summary.complete || graph.typescript !== summary.typescript_version
    || identity.source_digest !== summary.source.source_digest || graph.source?.source_digest !== identity.source_digest)
    throw new Error('Comparison requires complete extraction of the same source bytes');
  const openTraitArrays = documentTraitArrays(graph);
  const packets: RulePacket[] = [];
  for await (const packet of sampleRules(args.source)) packets.push(packet);
  const keys = new Set(graph.roots.flatMap(root => root.ruleKey ? [root.ruleKey] : []));
  const modeled = packets.filter(packet => keys.has(packet.key));
  const unmodeled = packets.filter(packet => !keys.has(packet.key));
  const authored = authoredRuleInputs(graph, openTraitArrays);
  await mkdir(args.out, { recursive: true });
  const json = async (name: string, value: unknown) => writeFile(path.join(args.out, name), JSON.stringify(value, null, 2) + '\n');
  const packetFile = path.join(args.out, "packets.ndjson");
  await writeFile(packetFile, modeled.map(packet => JSON.stringify(packet) + "\n").join(""));
  const run = async (profile: string, graph: TypeGraph): Promise<ProbeResult[]> => {
    const roots = graph.roots.filter(root => root.ruleKey).map((root, index) => ({key: root.ruleKey!, reference: root.ref!, name: `Rule${index}`, module: `rules/rule${index}`}));
    const selection = roots.map(root => ({name: root.name, declaration: root.reference, valueRef: root.reference, module: root.module, fields: [], deferred: []}));
    const results = await sourceProbe({graph, input: {source: identity, nodes: graph.nodes, selection, openTraitArrays}, roots, packets: packetFile, out: path.join(args.out, profile), target: path.join(args.out, "target")});
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
