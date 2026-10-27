# Blind judging and analysis helper

`analysis.py` consumes the maintained `atlas-dev similarity evaluate` JSON reports. It does not query an artifact, embed content, calculate similarity, modify frozen experiment inputs, or select a winning configuration. Use the frozen protocol's rubric and decision gates. Raw reports, source excerpts, packets and opinions belong under ignored `scratch/similarity-evaluation/`; tracked conclusions should contain paraphrases and artifact hashes/pointers.

## Construct and freeze the pool

After the complete quality matrix has finished, construct the union of **top ten production and fixed-pool rankings, across both profiles and every applicable variant**, independently for each scenario:

```sh
python3 docs/research/evaluations/2026-10-similarity/analysis.py pool \
  --report scratch/similarity-evaluation/quality/*.json \
  --seed 20261026 \
  --output-dir scratch/similarity-evaluation/judging-pool
```

The default requires every frozen scenario and configuration report. Use repeated `--scenario ID` for an explicitly limited development smoke run. `--allow-incomplete` creates a diagnostic draft that cannot be bound for grading. Captured failed configurations remain failures; they are not zero relevance. The input's freeze, generation, source revision, profile and variant identities must match. Existing outputs are never overwritten.

`pool.json` contains the task, exact selected section labels, seed identity and randomly ordered candidate identities. `private-rankings.json` contains all methods, models, ranks, scores and provenance. **Only the public packet files from the next step go to judges.** The same scenario/root appears once even when found by many configurations or both ranking sources. Randomization uses the fixed seed and SHA-256 ordering; blind IDs remain stable across order changes. `pool.freeze.json` binds both output hashes before source preparation.

## Prepare source packets

Create a private JSON file with this shape; use every scenario and candidate ID from `pool.json`:

```json
{
  "scenarios": [{
    "scenario_id": "dev-h01",
    "seed": {
      "identity": {"record_key": "pack:seed", "title": "Seed title", "kind": "hazard"},
      "availability": "available",
      "content": [{"label": "Description", "text": "Canonical source excerpt", "address": {"document_id": "seed", "field": "description"}}],
      "source_pointers": ["source-file.json#/system/description/value"],
      "source_hashes": ["0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"]
    },
    "candidates": [{
      "blind_candidate_id": "c-from-pool",
      "identity": {"record_key": "pack:candidate", "title": "Candidate title", "kind": "creature"},
      "availability": "bounded",
      "reason": "Selected excerpt plus nearby ownership context; complete document retained at source pointer.",
      "content": [{"label": "Owned ability", "text": "Canonical source excerpt", "address": {"document_id": "ability", "field": "description"}}],
      "source_pointers": ["source-file.json#/items/0/system/description/value"],
      "source_hashes": ["0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"]
    }]
  }]
}
```

Use `trait_key` in place of `record_key` for a trait seed. Identity allows only `record_key`, `trait_key`, `title`, `kind`. Source pieces allow only `identity`, `availability`, `reason`, `content`, `source_pointers`, `source_hashes`. Each content object allows `label`, `text`, `html`, `address`, `source_sha256`, `availability`, `reason`. Addresses can retain canonical typed source navigation. Do not copy retrieval scores, method names, model names or ranks into these fields. Authored game statistics such as a spell's rank may remain in the source text.

Availability is `available`, `bounded`, `label_only`, or `unavailable`. Preserve missing context explicitly: an unavailable piece requires a reason; other pieces require source content, pointers and SHA-256 hashes. A label-only trait should carry the actual catalog label and its source pointer/hash, without inventing a definition. Capture exact selected seed sections and enough candidate context to distinguish intrinsic properties, owned capabilities, mere associations and incidental mentions. Include original context when an embedded item or spell's ownership matters. This helper validates binding and packet shape; source fidelity remains the preparer's responsibility.

Bind the source packet input **before either judge sees it**:

```sh
python3 docs/research/evaluations/2026-10-similarity/analysis.py bind-packets \
  --pool-dir scratch/similarity-evaluation/judging-pool \
  --source-packets scratch/similarity-evaluation/source-packets.json \
  --output-dir scratch/similarity-evaluation/blind-packets
```

This emits one public `SCENARIO.json` file and `judging.freeze.json`, binding packet bytes, source-input digest, randomization seed, frozen experiment and private ranking digest. Give each independent model judge the same packet and rubric, without the private mapping, report files, other opinions or ordering metadata. Preserve the exact prompt text separately and hash its bytes. Record a model identifier/version; these are model judgments, not human labels. Candidate order is fixed within a packet; do not reorder it using a method's ranking.

## Record independent opinions and adjudicate disputes

Each judge emits one JSONL row per candidate (unknown is an explicit grade):

```json
{"scenario_id":"dev-h01","blind_candidate_id":"c-from-pool","judge_id":"judge-a","judge_model":"provider/model-version","judge_prompt_sha256":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef","packet_sha256":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef","grade":2,"evidence_support":"supported","connection_class":"owned-capability","uncertainty":"low","source_pointers":["source-file.json#/items/0/system/description/value"],"rationale":"Source-grounded usefulness explanation."}
```

Grades are integers 0–3 or the string `unknown`. Evidence support is `supported`, `partial`, `unsupported`, `unknown`. Connection class is `intrinsic`, `owned-capability`, `associated-only`, `incidental`, `none`, `unknown`. Uncertainty is `low`, `medium`, `high`, `unknown`. Numeric grades require source pointers. Duplicate opinions fail rather than overwrite prior judgments; keep revisions in new analysis runs. The original parsed opinions and input byte hashes are preserved in the analysis output.

The primary combined view accepts exact grade agreement only. Every grade disagreement, any unknown opinion, and material evidence-support or ownership dispute remains unknown unless an independent adjudicator resolves it. Material disputes include supported versus unsupported and associated-only versus intrinsic/owned-capability even when usefulness grades agree. The adjudicator uses another ID/model and the same frozen packet, blind to retrieval metadata. Preserve both original opinions and the adjudication. Adjudication cannot replace a missing initial independent opinion. An unknown adjudication remains unknown.

```sh
python3 docs/research/evaluations/2026-10-similarity/analysis.py metrics \
  --pool-dir scratch/similarity-evaluation/judging-pool \
  --judging-freeze scratch/similarity-evaluation/blind-packets/judging.freeze.json \
  --judge judge-a --judge judge-b \
  --judgments scratch/similarity-evaluation/judge-a.jsonl \
  --judgments scratch/similarity-evaluation/judge-b.jsonl \
  --adjudications scratch/similarity-evaluation/adjudications.jsonl \
  --output scratch/similarity-evaluation/quality-analysis.json
```

Omit `--adjudications` for the first pass. Its unresolved entries identify disputes; do not inspect methods or use a method's score to settle them. The command verifies packet bytes and private-ranking digest again before reading grades.

## Interpretation and verification

- nDCG@10 uses gains 0, 1, 3, 7 and an ideal ranking over the **entire judged union pool**, not the full corpus. A zero-gain ideal makes nDCG undefined (`null`). Useful@5 counts grades at least 2.
- Unknowns remain unknown. Primary metrics require the entire scenario pool judged because the ideal ranking also depends on that pool. Compared configurations share exactly the same complete scenarios within ranking source, split, stratum and judgment view. Failed cells remove that scenario uniformly from compared aggregate results, with failures still visible.
- The primary whole-record summary first averages tasks within each kind/theme cluster, then clusters within a kind, then equally weights hazard, creature and spell. Missing required kinds produce an undefined kind macro. Traits and section-focused tasks remain separate; baseline's full-seed result is context-only for explicit section tasks.
- Both individual judge views and the conservative agreement/adjudication view are retained. Uniform assignments of every unknown to grade 0 or every unknown to grade 3 are sensitivity scenarios, **not mathematical bounds or confidence intervals**. They do not silently enter primary metrics.
- Derived pairwise wins/ties are paired differences in graded nDCG on the same shared complete cases; they are not independently elicited pairwise preferences. Method overlap is identity-set overlap, not relevance. Candidate admission recall and returned-top-ten agreement are measured by the runner and are not recomputed here.

Run synthetic regression tests and confirm frozen inputs remain unchanged:

```sh
python3 -m unittest discover -s docs/research/evaluations/2026-10-similarity -p test_analysis.py
python3 docs/research/evaluations/2026-10-similarity/verify.py
```
