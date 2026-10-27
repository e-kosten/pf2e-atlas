#!/usr/bin/env python3
"""Replay compact recorded results; no artifact, source bodies or model assets.

This checks retained bytes, source/packet/prompt bindings, recorded score joins,
ordinal judgments, canonical relevance metrics and timing summaries. It does not
recompute vectors, retrieve candidates or re-verify omitted source bodies.
"""
import argparse
import collections
import hashlib
import gzip
import importlib.util
import json
import math
from pathlib import Path


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def payload(path):
    path = Path(path)
    if path.exists():
        return gzip.decompress(path.read_bytes()) if path.suffix == '.gz' else path.read_bytes()
    return gzip.decompress(path.with_name(path.name+'.gz').read_bytes())


def load(path):
    return json.loads(payload(path))


def lines(path):
    return [json.loads(row) for row in payload(path).decode().splitlines() if row.strip()]


def require(ok, message):
    if not ok:
        raise ValueError(message)


def analysis_module(dataset):
    spec = importlib.util.spec_from_file_location('similarity_analysis', dataset / 'analysis.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def quality_summary(report, scenario_metadata):
    meta = {s['scenario_id']: s for s in scenario_metadata}
    overlap = collections.defaultdict(list)
    for row in report['method_overlap']:
        if row['left'].split('/')[1] != row['right'].split('/')[1]:
            continue
        s = meta[row['scenario_id']]
        overlap[(s['split'], s['stratum'], row['left'], row['right'])].append(row)
    return {
        'version': 1,
        'metric_scope': report['metric_scope'],
        'consensus_policy': report['consensus_policy'],
        'sensitivity_note': report['sensitivity_note'],
        'resolved_judgments': [{k: row[k] for k in ['scenario_id', 'blind_candidate_id', 'grade', 'disagreements', 'adjudication_required']} for row in report['resolved_judgments']],
        'aggregates': report['aggregates'],
        'pairwise_graded_preferences': [r for r in report['pairwise_graded_preferences'] if r['judgment_view'] == 'agreement_or_adjudication' and r['left'].split('/')[1] == r['right'].split('/')[1]],
        'method_overlap': [{'split': key[0], 'stratum': key[1], 'left': key[2], 'right': key[3], 'scenarios': [r['scenario_id'] for r in rows],
                            'mean_jaccard': sum(r['jaccard'] for r in rows)/len(rows), 'mean_shared_roots': sum(r['shared_roots'] for r in rows)/len(rows)}
                           for key, rows in sorted(overlap.items())],
    }


def quality_cases(report):
    return [{k: value for k, value in row.items() if k != 'sensitivity' or row['unknown_candidates']} for row in report['cases']]


def percentile(values, p):
    return sorted(values)[max(0, math.ceil(p * len(values))-1)] if values else None


def score_map(block, keys):
    require(len(block['scores']) == len(keys), 'score vector and key census differ')
    for value in block['scores']:
        require(value is None or (type(value) in [int, float] and math.isfinite(value)), 'nonfinite recorded score')
    return dict(zip(keys, block['scores']))


def verify(dataset, result_dir):
    manifest = load(result_dir / 'manifest.json')
    require(manifest['state'] == 'complete-recorded-experiment', 'result package incomplete')
    recorded_paths = [row['path'] for row in manifest['files']]
    actual_paths = {str(p.relative_to(result_dir)) for p in result_dir.rglob('*') if p.is_file() and p.name != 'manifest.json'}
    require(len(recorded_paths) == len(set(recorded_paths)) and set(recorded_paths) == actual_paths, 'result file census differs')
    for row in manifest['files']:
        path = Path(row['path'])
        require(not path.is_absolute() and '..' not in path.parts, 'unsafe result path')
        source = result_dir / path
        require(source.stat().st_size == row['bytes'] and sha(source) == row['sha256'], f'result bytes changed: {path}')
        if 'payload_sha256' in row:
            require(hashlib.sha256(payload(source)).hexdigest() == row['payload_sha256'], f'compressed payload changed: {path}')
    a = analysis_module(dataset)
    frozen_manifest, _, _, input_digest = a.inputs(dataset)
    require(input_digest == manifest['input_freeze_sha256'], 'different frozen inputs')
    private = load(result_dir / 'rankings.json')
    frozen = load(result_dir / 'judging.freeze.json')
    pool_frozen = load(result_dir / 'pool.freeze.json')
    require(sha(result_dir/'rankings.json') == frozen['private_rankings_sha256'], 'ranking freeze changed')
    require(sha(result_dir/'pool.json') == frozen['public_pool_sha256'], 'public pool freeze changed')
    require(frozen['input_freeze_sha256'] == input_digest, 'judging/input freeze differs')
    require(all(pool_frozen[k] == frozen[k] for k in ['input_freeze_sha256', 'private_rankings_sha256', 'public_pool_sha256', 'randomization_seed']), 'pool/judging bindings differ')
    candidates = {(r['scenario_id'], c['blind_candidate_id']) for r in private['scenarios'] for c in r['candidates']}
    judges = ['judge-a', 'judge-b']
    opinion_paths = sorted((result_dir/'opinions').glob('*.jsonl'))
    adjudication_paths = sorted((result_dir/'adjudications').glob('*.jsonl'))
    opinions, _ = a.read_opinions(opinion_paths, candidates, frozen['packets'], judges)
    adjudications, _ = a.read_opinions(adjudication_paths, candidates, frozen['packets'], judges, True)
    require(len(candidates) == 916 and len(opinions) == 1832, 'opinion census incomplete')
    require(set(opinions) == {(sid, cid, judge) for sid, cid in candidates for judge in judges}, 'missing independent opinion')
    prompt_hashes = {sha(p) for p in (result_dir/'instructions').glob('*.md')}
    require(all(r['judge_prompt_sha256'] in prompt_hashes for r in [*opinions.values(), *adjudications.values()]), 'unbound opinion instruction')
    replay = a.analyze(private, frozen, opinions, judges, adjudications)
    projected = quality_summary(replay, private['scenarios'])
    observed = load(result_dir/'quality.json')
    for key in ['aggregates', 'pairwise_graded_preferences', 'method_overlap']:
        observed[key] = lines(result_dir/f'quality-{key}.jsonl')
    require(projected == observed, 'quality metric replay differs')
    require(quality_cases(replay) == lines(result_dir/'quality-cases.jsonl'), 'per-case metrics differ')
    # The stripped source-address census preserves original indices and exact
    # pointers, not the omitted packet payload; it cannot reconstruct packet SHA.
    bindings = {(r['scenario_id'], r['blind_candidate_id']): r for r in lines(result_dir/'source-bindings.jsonl')}
    require(len(bindings) == 940 and candidates <= set(bindings), 'source-address census incomplete')
    for opinion in [*opinions.values(), *adjudications.values()]:
        sid, cid = opinion['scenario_id'], opinion['blind_candidate_id']
        candidate, seed = bindings[(sid, cid)], bindings[(sid, 'seed')]
        allowed = set(candidate['source_pointers']) | set(seed['source_pointers'])
        for pointer in opinion['source_pointers']:
            if pointer in allowed:
                continue
            file, sep, anchor = pointer.partition('#')
            require(sep and Path(file).name == frozen['packets'][sid]['file'], 'opinion pointer outside assigned source')
            prefix = f"/candidates/{candidate['packet_candidate_index']}/content/"
            seed_prefix = '/seed/content/'
            selected, ordinal = (candidate, anchor[len(prefix):]) if anchor.startswith(prefix) else (seed, anchor[len(seed_prefix):]) if anchor.startswith(seed_prefix) else (None, '')
            require(selected is not None and ordinal.isdigit() and int(ordinal) < selected['source_entry_count'] and ordinal in selected['entries'], 'invalid packet entry pointer')
    scenario_meta = {r['scenario_id']: r for r in private['scenarios']}
    pools = load(result_dir/'fixed-pools.json')
    fixed_rows = lines(result_dir/'fixed-pool-scores.jsonl')
    fixed = {(r['scenario_id'], r['profile'], r['id']): r for r in fixed_rows}
    expected_cells = {(s['id'], profile, variant['id']) for s in frozen_manifest['scenarios'] for profile in frozen_manifest['profiles'] for variant in a.available_variants(frozen_manifest, s)}
    production = lines(result_dir/'production-scores.jsonl')
    production_cells = [(r['scenario_id'], r['profile'], r['id']) for r in production]
    require(len(fixed_rows) == len(fixed) == len(production_cells) == len(set(production_cells)) == 552 and set(fixed) == set(production_cells) == expected_cells, 'configuration score census incomplete')
    top_count = 0
    for row in production:
        sid, profile, variant = row['scenario_id'], row['profile'], row['id']
        require(row['options'] == fixed[(sid, profile, variant)]['options'], 'fixed/production option identity differs')
        values = score_map(fixed[(sid, profile, variant)], pools[sid]['keys'])
        roots = [v['record_key'] for v in row['returned_top20']]
        require(len(roots) == len(set(roots)) == 20, 'production root census changed')
        require(all(v['score'] == values[v['record_key']] == v['effective_score'] for v in row['returned_top20']), 'production/exact score join differs')
        mapping = {c['blind_candidate_id']: c['record_key'] for c in scenario_meta[sid]['candidates']}
        for source, source_roots in [('production', roots), ('fixed_pool', fixed[(sid, profile, variant)]['ranking_top20'])]:
            ranking = next(r for r in scenario_meta[sid]['rankings'] if r['configuration'] == f'{source}/{profile}/{variant}')
            require(source_roots[:10] == [mapping[cid] for cid in ranking['ranking']], 'frozen ranking and score order differ')
        top_count += len(roots)
    actual_count = 0
    prod_lookup = {(r['scenario_id'], r['profile'], r['id']): {v['record_key']: v['score'] for v in r['returned_top20']} for r in production}
    for row in lines(result_dir/'actual-capture-scores.jsonl'):
        expected = prod_lookup[(row['scenario_id'], row['profile'], row['variant_id'])]
        for value in row['returned_scores']:
            require(value['score'] == value['semantic_similarity'] == expected[value['record_key']], 'actual and recovered scores differ')
            actual_count += 1
    require(top_count == 11040 and actual_count == 4320, 'score verification census differs')
    oracle_cells = admission_available = 0
    for case in lines(result_dir/'oracle.jsonl'):
        keys = case['keys']
        require(len(keys) in [277, 278], 'controlled universe census changed')
        for comparison in case['comparisons']:
            oracle_cells += 1
            values = score_map(case['exact_scores'][comparison['id']], keys)
            scored = [(key, score) for key, score in values.items() if score is not None and key not in case['exact_scores'][comparison['id']]['unavailable']]
            direct = [key for key, _ in sorted(scored, key=lambda v: (-v[1], v[0]))[:10]]
            require(direct == comparison['oracle_top10'], 'direct oracle score order differs')
            m = comparison['metrics']
            require(m['valid'] and m['oracle_top_count'] == len(direct), 'oracle denominator differs')
            require(comparison['bounded_returned_top10'] == direct and m['returned_top10_agreement'] == 1, 'returned top-ten agreement differs')
            admitted = case['admitted_groups'][comparison['capture_group']]
            if admitted is None:
                require(not m['admission_available'] and m['candidate_admission_recall_at10'] is None, 'baseline fabricated admission')
            else:
                admission_available += 1
                misses = [key for key in direct if key not in admitted]
                require(m['admission_available'] and misses == comparison['admission_misses'] and m['candidate_admission_recall_at10'] == (len(direct)-len(misses))/len(direct), 'admission recall differs')
    require(oracle_cells == 552 and admission_available == 504, 'oracle cell census incomplete')
    perf = load(result_dir/'performance.json')
    declared = frozen_manifest['performance']
    expected_performance = {(sid, profile, variant) for sid in declared['scenario_ids'] for profile in declared['profiles'] for variant in declared['variant_ids']}
    expected_performance.update((extra['scenario_id'], profile, extra['variant_id']) for extra in declared['additional_cases'] for profile in extra['profiles'])
    actual_performance = [(r['scenario_id'], r['profile'], r['variant_id']) for r in perf['cells']]
    require(len(perf['cells']) == len(set(actual_performance)) == 32 and set(actual_performance) == expected_performance, 'performance census incomplete')
    for cell in perf['cells']:
        require(cell['cold_status'] == cell['warm_status'] == 'ok', 'recorded successful performance run contains a classified failure')
        values = [sample['elapsed_ms'] for sample in cell['warm_samples']]
        require(cell['satisfies_manifest_process_cold'], 'context-cold relabeled process-cold')
        require(len(values) == 20 and cell['warm_statistics']['samples'] == 20, 'warm repetition count changed')
        require(cell['warm_statistics']['p50_ms'] == percentile(values, .5) and cell['warm_statistics']['p95_ms'] == percentile(values, .95), 'warm percentile replay differs')
        require(cell['process']['peak_process_rss_bytes'] > 0, 'missing external peak RSS')
    inventory = load(result_dir/'inventory.json')
    require(len(inventory['cells']) == 10, 'inventory cell census differs')
    for cell in inventory['cells']:
        samples = cell['samples']; warm = [r['elapsed_ms'] for r in samples[1:]]
        require(len(samples) == 21 and all(r['status'] == 200 for r in samples), 'inventory request census/status differs')
        require(cell['cold_ms'] == samples[0]['elapsed_ms'] and cell['warm_p50_ms'] == percentile(warm,.5) and cell['warm_p95_ms'] == percentile(warm,.95), 'inventory percentiles differ')
        require(len({r['selection_signature_sha256'] for r in samples}) == 1 and len({r['full_response_sha256'] for r in samples}) == 1, 'inventory selection changed between requests')
    print(json.dumps({'status': 'ok', 'opinion_rows': len(opinions), 'adjudications': len(adjudications), 'score_values': top_count, 'actual_scores_replayed': actual_count, 'oracle_cells': oracle_cells, 'performance_cells': len(perf['cells']), 'scope': 'recorded-data replay and bindings; no source-body or vector-math re-verification'}, sort_keys=True))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dataset', type=Path, default=Path(__file__).resolve().parent)
    parser.add_argument('--results', type=Path)
    args = parser.parse_args()
    verify(args.dataset, args.results or args.dataset/'results')
