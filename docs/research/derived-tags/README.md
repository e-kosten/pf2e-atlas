# Derived tags: preserved research and deferred design

Status: deferred. Updated: 2026-10-19.

Derived tags were explored as authored, predictable filters for questions that
native PF2e/Foundry facts and semantic search could not answer well enough. The
user now considers the improved semantic search likely to make this workstream
unnecessary. Preserve the knowledge, use the product, and revisit only when
concrete unmet needs justify the maintenance and classification cost.

This is the durable entry point for the earlier Node implementation, canonical
concept experiments, Rust seed corpus and October ontology/assessment discussion.
It is intended to stand on its own after local plans and worktrees are removed.
The snapshots preserve research, including its mistakes and unfinished ideas;
they are not an approved taxonomy, reviewed assignment set or implementation plan.

## Read by question

| Question | Document |
| --- | --- |
| What was tried, what worked, and what should we avoid repeating? | [History and lessons](history-and-lessons.md) |
| If a real gap appears, what design discussion should resume? | [Design if revisited](design-if-revisited.md) |
| What research informed annotation and reviewer choices? | [Research and evaluation](research-and-evaluation.md) |
| Where are the exact sources, and what would cleanup discard? | [Sources and cleanup](sources-and-cleanup.md) |
| What are the preserved local-only bytes? | [Snapshot manifest](sources/manifest.md) |

The [deferred backlog item](../../backlog/items/rust-derived-tag-redesign.md)
tracks the reopening condition. [ADR0054](../../architecture/decisions/0054-derived-tag-deferral.md)
records this deferral. [Tagging architecture](../../architecture/tagging.md)
distinguishes the existing file validator from unimplemented product tagging.

## Reopen from a user problem

Capture a small, concrete retrieval-gap packet during ordinary use:

- The user's question, actual query, explicit filters, search profile/ranking
  and reference generation or source version.
- Expected useful records and observed results, including source evidence and
  why missed results or unwanted matches matter.
- Whether the need is occasional discovery, repeatable inclusion/exclusion,
  composing several concepts, or browsing a stable conceptual collection.
- A comparison with native filters, better field discovery/named filter presets,
  semantic/hybrid search, Similar and graph navigation where relevant.

Then identify the cause: missing source/content admission, eligibility, native
filter coverage, candidate-window loss, ranking, or a genuinely interpretive
concept. A label corpus carries its own omissions and drift; it is not a general
repair for all retrieval failures. A friendly name for an existing native filter
may solve the problem without an AI-maintained assignment corpus.

The [search research synthesis](../search-evaluation-synthesis.md) preserves
measured search improvements and remaining limitations. Neither those studies
nor this deferral prove tags can never add value. They do remove the assumption
that a comprehensive ontology and whole-corpus classification should be built
before a demonstrated need exists.

If a gap survives that comparison, start with the smallest useful concept and
human-approved examples. Recheck the [open choices](design-if-revisited.md#open-choices)
against the then-current architecture before approving implementation. Do not
resume the old heuristic matcher or launch a full-corpus labeling job merely
because the historical artifacts exist.
