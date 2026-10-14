# FTS Tokenization and Inflection Handling

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-13

## Problem

Inflection differences such as breath, breathes and breathing can affect lexical
search. Evaluate standard tokenizer/stemmer behavior against the selected lexical
fields and actual PF2e queries.

## Evaluation

Compare the current lexical_fts tokenizer with relevant unicode61 settings and
SQLite's porter wrapper. Check diacritics, stop words, token composition, prefixes,
exact primary names and embedded definition names. Use diverse source-backed
queries and distinguish lexical improvements from semantic relevance.

Indexing and query construction must use compatible normalization. Preserve
strict identity resolution and explicit lexical specificity. A tokenizer change
requires corresponding artifact policy/validation updates and a rebuild. Prefer
library-supported normalization to domain-specific substitutions.

Coordinate with [search quality](rust-search-quality-tuning.md) and
[query spelling](rust-cli-typo-tolerant-discovery.md), which answer different
questions about relevance and typo recovery. Current lexical authority follows
[ADR 0046](../../architecture/decisions/0046-source-backed-reference-artifact.md).
