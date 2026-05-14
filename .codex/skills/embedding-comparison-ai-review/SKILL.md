---
name: embedding-comparison-ai-review
description: Orchestrate blind AI scoring for embedding comparison runs by reading generated candidate-letter review jobs, delegating or executing scoring without exposing model mappings, validating chunk score JSON, merging scores, and aggregating model results. Use when asked to score embedding comparison review packets or coordinate AI review of search result quality.
---

# Embedding Comparison AI Review

Use this skill to score an embedding comparison run without revealing model identities to scoring agents.

## Hard Rules

- Score only from files under `<run>/ai-review-jobs/`.
- Do not open or quote `review-packets/*.mapping.json` until all scoring is complete.
- Do not inspect model identities in `summary.json` while scoring.
- Do not include model ids, model labels, build metrics, artifact paths, or mapping contents in scoring prompts.
- Write scorer outputs only under `<run>/ai-review-scores/candidate-<letter>/chunk-XXX.json`.
- Run `merge-ai-review-scores.mjs` only after all required chunk outputs exist.
- Run `aggregate-scores.mjs` only after merge succeeds.

## Workflow

1. Confirm jobs exist:

   ```bash
   test -f <run>/ai-review-jobs/manifest.json
   ```

   If missing, generate jobs first:

   ```bash
   node scratch/embedding-comparison/prepare-ai-review-jobs.mjs \
     --run <run> \
     --chunk-size 10 \
     --exclude-query aquatic-low-level-creature
   ```

2. Read `<run>/ai-review-jobs/manifest.json` for job paths and expected outputs.

3. Score jobs.
   - If delegation is explicitly authorized, delegate one candidate letter or one chunk per scoring agent.
   - Each scoring agent receives only one `ai-review-jobs/.../chunk-XXX.md` file.
   - If scoring locally, process one job file at a time and write exactly the output JSON requested in the job.

4. Validate and merge:

   ```bash
   node scratch/embedding-comparison/merge-ai-review-scores.mjs --run <run>
   ```

   If merge reports missing or invalid chunks, rerun only those job files.

5. Aggregate:

   ```bash
   node scratch/embedding-comparison/aggregate-scores.mjs --run <run>
   ```

6. After aggregation, it is safe to inspect mappings and model names in the generated score summary.

## Scoring Guidance

- Use the rubric embedded in each job file.
- Score against the query and review guidance only.
- Penalize irrelevant, overbroad, or relationship-noise results.
- Do not invent exact expected records.
- Empty candidate sections should receive low usefulness scores and notes explaining that no results were returned.
