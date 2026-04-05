---
name: pf2e-derived-tag-overfit-audit
description: Audit PF2E derived tags in the Pathfinder MCP repo for false positives surfaced by unusually dense tagging. Use when Codex needs to inspect records with many derived tags, suspicious same-family stacks, habitat or motif explosions, or broad weak-anchor matches that may signal overfit rules before a refinement pass.
---

# PF2E Derived Tag Overfit Audit

Use this skill when the first sign of trouble is entity-level tag density rather than one clearly broken tag.

Dense tagging is not proof of a bug. It is a review queue for finding:
- overly broad weak anchors
- same-family tag explosions
- incidental source phrases that trigger setting or motif tags
- records whose tag set no longer matches the source text

The goal is to prevent the same class from recurring. A good audit produces regression-ready noisy records, not just a note that a tag looks wrong.

## Workflow

1. Use the current local index.
   If derived-tag rules changed recently, rebuild first with `npm run refresh-index -- --reuse-embeddings`.

2. Generate a dense-tag review queue.
   Use the helper script:

   ```bash
   node --import tsx/esm .codex/skills/pf2e-derived-tag-overfit-audit/scripts/dense-tag-report.ts --limit 30
   ```

   Useful variants:

   ```bash
   node --import tsx/esm .codex/skills/pf2e-derived-tag-overfit-audit/scripts/dense-tag-report.ts --category creature --show-tags --limit 20
   node --import tsx/esm .codex/skills/pf2e-derived-tag-overfit-audit/scripts/dense-tag-report.ts --category hazard --min-tags 4 --show-tags
   node --import tsx/esm .codex/skills/pf2e-derived-tag-overfit-audit/scripts/dense-tag-report.ts --name "Sakugami" --show-tags
   ```

3. Treat the output as triage, not verdict.
   Prioritize records with:
   - 5+ tags overall
   - unusually high counts for their category
   - multiple tags from the same family, especially `setting`, `motif`, or other thematically broad families
   - tag combinations that imply very different habitats or roles

4. Read the source record before touching rules.
   Pull the canonical record from the SQLite index or source JSON and ask:
   - Which tags are clearly justified?
   - Which tags look incidental or stretched?
   - Is one phrase, trait, or repeated weak token doing too much work?

5. Turn suspicious records into false-positive classes.
   Do not fix a single named record in isolation.
   Instead, identify the reusable class, such as:
   - `Inner Sea` firing `aquatic_setting`
   - generic guardian language inflating habitat tags
   - one vague art term creating `living_artwork`
   - a broad condition phrase causing too many support tags
   Prefer naming at least one real record that should become a regression test before the refinement pass starts.
   Strong seed records when they fit the issue:
   - `Crushing Ground`
   - `Imprisonment`
   - `Artevil Suspension`
   - `Blindpepper Bomb`
   - `Mycological Malady`
   - one troll lore paragraph for creature-setting noise

6. Inspect the rule slice that produced the suspect tags.
   Use `rg` and read:
   - `src/tags/rules/*.ts`
   - `src/tags/shared.ts`
   - direct derivation tests in `tests/tags/derived-tags-*.test.ts`
   - service behavior in `tests/service/search-and-lookup.test.ts`
   Then run the deterministic discovery tools on the suspect tag or slice:
   - `npm run analyze-derived-tag-evidence -- --category <category> --tag <tag> ...`
   - `npm run discover-ruleable-cohorts -- --category <category> --tag <tag> ...`
   Use these to see whether the suspect tag is being driven by one weak anchor, multiple separable cohorts, or a native-trait alias.

7. Prefer evidence redesign over special casing.
   Good fixes:
   - strengthen anchors
   - add blockers
   - require contextual proximity
   - split one overly broad rule into stronger and weaker paths with thresholds

   Bad fixes:
   - adding one named exception for a single record
   - masking a broad weak anchor without addressing why it overfires

8. Hand off to `$pf2e-derived-tag-refinement` for implementation.
   Use this skill to find the suspicious class and explain the likely cause.
   Use refinement mode to propose the calibration slice, get approval if needed, add the regression cases in the relevant `tests/tags/derived-tags-*.test.ts` file, and then make the rule/test changes.

## Audit Heuristics

- `creature` and `spell` can legitimately stack many tags. Review them, but expect some honest dense cases.
- `hazard` and `affliction` are much sparser. Four or five tags there is already unusual.
- Many tags from one family are often more suspicious than many tags spread across different families.
- Habitat stacks deserve special scrutiny. One mention of a region, sea, spring, or village can accidentally inflate multiple environment tags.
- Motif stacks deserve scrutiny when the record is only aesthetically adjacent to the concept.
- A good audit note names the candidate false-positive class, the likely anchor, and the likely safer boundary.
- Prefer handing refinement a brief that already includes evidence-report anchors, top cohort signatures, and one or two contrast records.

## Category Starting Thresholds

Use these as defaults when deciding what to review first:
- `creature`: start at `6+`
- `spell`: start at `6+`
- `equipment`: start at `5+`
- `affliction`: start at `4+`
- `hazard`: start at `4+`

Lower the threshold when chasing a known noisy family. Raise it when you only want the strongest outliers.

## Helper Script

`dense-tag-report.ts` reports:
- category baseline stats for tagged records
- dense canonical records above a threshold
- per-record tag count
- family breakdown for the matched tags
- optional full tag lists

Use `--json` when another tool or agent needs structured output.

## Expected Output Shape

The useful end state from this skill is not a patch. It is a short refinement brief:
- suspicious records
- suspect tags
- likely anchor or blocker problem
- generalized false-positive class
- named real records to pin as regressions
- recommended refinement slice

Then switch to `$pf2e-derived-tag-refinement` to implement and validate the fix.
