import { createHash } from 'node:crypto';
import type { RulePacket } from './sample-rules.js';

export type AdmissionProbeResult = {
  ok: true; retained: boolean; modeled: boolean; fidelity: string | null;
  diagnostics: { json_path: string; expected: string; actual: string }[];
} | { ok: false; error: { json_path: string; expected: string; actual: string } };

export interface AdmissionBaseline {
  sourceDigest: string; corpusDigest: string; outcomeDigest: string;
  counts: ReturnType<typeof admissionResults>['counts'];
}
export function admissionBaselineMatches(actual: AdmissionBaseline, expected: AdmissionBaseline): boolean {
  return actual.sourceDigest === expected.sourceDigest && actual.corpusDigest === expected.corpusDigest
    && actual.outcomeDigest === expected.outcomeDigest
    && (Object.keys(actual.counts) as (keyof AdmissionBaseline['counts'])[])
      .every(key => actual.counts[key] === expected.counts?.[key]);
}

/** Raw retention, partial typing and rule exclusion are distinct measured outcomes. */
export function admissionResults(contexts: Omit<RulePacket, 'source'>[], results: AdmissionProbeResult[]) {
  if (contexts.length !== results.length) throw new Error('Admission probe result count mismatch');
  const empty = () => ({ occurrences: 0, retained: 0, modeled: 0, fullyTyped: 0,
    partial: 0, rawOnly: 0, rejected: 0, fidelityFailures: 0, diagnostics: 0 });
  const counts = empty(), byRoot: Record<string, ReturnType<typeof empty>> = Object.create(null);
  const outcomes: { packet: Omit<RulePacket, 'source'>; result: AdmissionProbeResult }[] = [];
  results.forEach((result, index) => {
    const packet = contexts[index];
    byRoot[packet.key] ??= empty();
    for (const scope of [counts, byRoot[packet.key]]) {
      scope.occurrences++;
      if (!result.ok) { scope.rejected++; continue; }
      scope.retained += Number(result.retained);
      scope.modeled += Number(result.modeled);
      if (!result.modeled) scope.rawOnly++;
      else if (result.diagnostics.length) scope.partial++;
      else scope.fullyTyped++;
      scope.diagnostics += result.diagnostics.length;
      scope.fidelityFailures += Number(!result.retained || result.fidelity !== null);
    }
    if (!result.ok || !result.modeled || result.diagnostics.length || !result.retained || result.fidelity !== null)
      outcomes.push({ packet, result });
  });
  return { counts, byRoot, outcomes,
    outcomeDigest: createHash('sha256').update(JSON.stringify(outcomes)).digest('hex') };
}
